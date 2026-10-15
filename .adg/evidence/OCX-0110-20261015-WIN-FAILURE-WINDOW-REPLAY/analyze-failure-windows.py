"""Offline replay of fixed historical failures; never run a test or attach to a PID."""
import collections
from decimal import Decimal
import gzip
import hashlib
import json
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parent
repo = root.parents[2]
evidence = root.parent
control = repo / (
    "docs/04-项目资料/03-项目协作资料/01-协作包/"
    "2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/"
    "20261009-概览升级入口与备份通知策略/验证/20261014-控制Node低CPU失败证据"
)
inputs = []


def consume(path):
    data = path.read_bytes()
    inputs.append({"path": path.relative_to(repo).as_posix(), "bytes": len(data),
                   "sha256": hashlib.sha256(data).hexdigest()})
    return data


def json_file(path):
    return json.loads(consume(path).decode("utf-8-sig"))


def marker(value):
    if value.startswith('Ok("'):
        return json.loads(json.loads(value[3:-1]))
    assert value.startswith("Err(Os { code: 2, kind: NotFound,"), value
    return None


def duration_ms(value):
    number, unit = re.fullmatch(r"([0-9.]+)(ns|µs|ms|s)", value).groups()
    return Decimal(number) * {"ns": Decimal(".000001"), "µs": Decimal(".001"),
                              "ms": Decimal(1), "s": Decimal(1000)}[unit]


def analyze_log(run, path, expected_raw_hash):
    raw = gzip.decompress(consume(path))
    assert hashlib.sha256(raw).hexdigest() == expected_raw_hash
    text = raw.decode("utf-8-sig")
    fixtures = collections.defaultdict(lambda: {"snapshots": {}, "controlMarkers": {}, "phases": {}})
    for scope, mode, message in re.findall(r"query fixture (version|metadata)/(\w+): ([^\r\n]+)", text):
        item = fixtures[f"{scope}/{mode}"]
        if message.startswith("control-snapshot="):
            snapshot = json.loads(message.removeprefix("control-snapshot="))
            assert snapshot["point"] not in item["snapshots"]
            item["snapshots"][snapshot["point"]] = snapshot
        elif message.startswith("control-marker-at="):
            point, value = message.removeprefix("control-marker-at=").split(", evidence=", 1)
            assert point not in item["controlMarkers"]
            item["controlMarkers"][point] = marker(value)
        elif message.startswith("phase="):
            phase, value = message.removeprefix("phase=").split(", evidence=", 1)
            assert phase not in item["phases"]
            item["phases"][phase] = marker(value)
        elif message.startswith("started-unix-ms="):
            matched = re.fullmatch(
                r"started-unix-ms=(\d+), total-elapsed=([^,]+), node=(.+), control-pid=(\d+)", message)
            assert matched and "fixtureStartedUnixMs" not in item
            item["fixtureStartedUnixMs"] = int(matched[1])
            item["queryReturnedTotalElapsedMs"] = float(duration_ms(matched[2]))
            item["node"] = matched[3]
            item["controlPid"] = int(matched[4])
        elif message.startswith("elapsed="):
            elapsed, outcome = re.fullmatch(r"elapsed=([^,]+), result=(\w+)", message).groups()
            item["queryElapsedMs"] = float(duration_ms(elapsed))
            item["queryOutcome"] = outcome
    assert len(fixtures) == 7
    expected_points = {"before-query", "query-returned", "observer-returned"}
    for name, item in fixtures.items():
        assert set(item["snapshots"]) == set(item["controlMarkers"]) == expected_points
        snapshots = item["snapshots"]
        creations = {row["process_times"]["creation_filetime_100ns"] for row in snapshots.values()}
        assert len(creations) == 1
        assert {row["process_id"] for row in snapshots.values()} == {item["controlPid"]}
        assert all(row["wait_status"] == 258 and row["exit_code"] == 259
                   and row["exit_error"] is row["times_error"] is None
                   and row["process_times"]["exit_filetime_100ns"] == 0 for row in snapshots.values())
        created_ticks = creations.pop()
        created_ms = Decimal(created_ticks - 116444736000000000) / 10000
        item["controlCreationFiletime100ns"] = created_ticks
        item["controlCreationUnixMsDecimal"] = str(created_ms)
        item["controlCreationAfterFixtureStartMs"] = float(created_ms - item["fixtureStartedUnixMs"])
        assert item["controlCreationAfterFixtureStartMs"] >= -1
        terminal = snapshots["observer-returned"]
        item["observerElapsedMs"] = terminal["elapsed_ms"]
        item["observerControlCpuMs"] = (
            terminal["process_times"]["kernel_cpu_100ns"] +
            terminal["process_times"]["user_cpu_100ns"]) / 10000
        item["controlStillLiveAtObserver"] = True
        item["controlMarkerMissingAtQueryReturn"] = item["controlMarkers"]["query-returned"] is None
        item["controlMarkerMissingAtObserverReturn"] = item["controlMarkers"]["observer-returned"] is None
        observed = [value for value in item["controlMarkers"].values() if value is not None]
        if observed:
            assert len({(value["pid"], value["atUnixMs"]) for value in observed}) == 1
            assert observed[0]["pid"] == item["controlPid"]
            item["markerPublicationAfterCreationMs"] = float(Decimal(observed[0]["atUnixMs"]) - created_ms)
            item["markerPublicationAfterFixtureStartMs"] = observed[0]["atUnixMs"] - item["fixtureStartedUnixMs"]
        else:
            item["markerPublicationAfterCreationMs"] = None
            item["markerPublicationAfterFixtureStartMs"] = None
        # Rust libtest buffers captured output. Log-line prefix is its delivery
        # time, not the embedded fixture execution time.
        item["githubLogPrefixUsedAsExecutionClock"] = False
        item["phaseMarkersObservedAtQueryReturn"] = sorted(
            phase for phase, value in item["phases"].items() if value is not None)
    counts = re.findall(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
    assert counts
    return {
        "run": run, "rawLogSha256": expected_raw_hash,
        "library": {"status": counts[0][0], "passed": int(counts[0][1]),
                    "failed": int(counts[0][2]), "ignored": int(counts[0][3])},
        "fixtures": dict(sorted(fixtures.items())),
        "diagnosticOnly": True, "historicalRunnerPidsAreNotLocalTargets": True,
        "schedulerOrIoFailureTracePresent": False, "rootCauseDetermined": False,
    }


pr_summary = json_file(control / "control-pr-terminal-summary.json")
push_summary = json_file(control / "control-push-final-summary.json")


def selected_hash(summary, basename):
    matches = [row for row in summary["selected_logs"] if row["file"] == basename]
    assert len(matches) == 1
    return matches[0]["sha256"]


cohorts = [
    analyze_log(38537166560, control / "control-pr-terminal-backend-windows.log.gz",
                selected_hash(pr_summary, "control-pr-terminal-backend-windows.log")),
    analyze_log(38537161445, control / "control-push-final-backend-windows.log.gz",
                selected_hash(push_summary, "control-push-final-backend-windows.log")),
]
for directory, run in (
    ("OCX-0110-20261014-WIN-BD2-VOLUME-CI", 38572890687),
    ("OCX-0110-20261014-WIN-BD2-BASENAME-CI", 38583689702),
    ("OCX-0110-20261014-WIN-BD2-BASENAME-CI", 38583679031),
):
    folder = evidence / directory
    summary = json_file(folder / f"ci-log-summary-{run}.json")
    original = next(row for row in summary["selectedOriginalLogs"]
                    if row["savedAs"] == f"ci-{run}-backend-windows-tests.log.gz")
    cohorts.append(analyze_log(run, folder / original["savedAs"], original["sha256"]))

original_failure, original_success, later_failure, e6_failure, e6_success = cohorts
assert original_failure["library"]["failed"] == 7
assert all(row["controlMarkerMissingAtObserverReturn"] for row in original_failure["fixtures"].values())
assert original_success["library"]["failed"] == e6_success["library"]["failed"] == 0
assert all(not row["controlMarkerMissingAtQueryReturn"]
           for run in (original_success, e6_success) for row in run["fixtures"].values())
for cohort in (later_failure, e6_failure):
    assert cohort["library"]["failed"] == 3
    for name, item in cohort["fixtures"].items():
        if name.startswith("version/"):
            assert item["queryOutcome"] == "Timeout"
            assert item["controlMarkerMissingAtQueryReturn"] and not item["controlMarkerMissingAtObserverReturn"]
            assert 3000 < item["markerPublicationAfterFixtureStartMs"] < item["observerElapsedMs"]
            assert not item["phaseMarkersObservedAtQueryReturn"]
        else:
            assert not item["controlMarkerMissingAtQueryReturn"]

event_path = evidence / (
    "OCX-0110-20261015-WIN-CI-ETW/diagnostic-original/runtime/"
    "ci-bd2-query-etw/capture/owned-etw.events.jsonl")
event_bytes = consume(event_path)
assert hashlib.sha256(event_bytes).hexdigest() == "a4f4260a3a58d812a1e17a436364c2616e1a9e8fb8c9c40086cf116c4f76b1d8"
events = [json.loads(line) for line in event_bytes.splitlines()]
outgoing = [row for row in events if row["kind"] == "owned-thread-scheduled-out"]
assert len(outgoing) == 3175
histograms = {key: dict(sorted(collections.Counter(str(row[key]) for row in outgoing).items()))
              for key in ("waitMode", "waitReason", "state")}
unnamed_modes = sum(row["waitMode"] not in {"NonSwap", "Swappable"} for row in outgoing)
assert unnamed_modes == 402
assert all(str(int(row["waitMode"])) == row["waitMode"]
           for row in outgoing if row["waitMode"] not in {"NonSwap", "Swappable"})
assert all(row["original"]["version"] == 5 for row in outgoing)
control_marker_creates = [row for row in events if row["kind"] == "owned-file-create"
                          and (row["file"] or "").endswith("control-started.json.pending")]
control_marker_writes = [row for row in events if row["kind"] == "owned-file-read-write"
                         and (row["file"] or "").endswith("control-started.json.pending")]
assert len({row["pid"] for row in control_marker_writes}) == 7
assert all(row["original"]["eventName"] == "FileIO/Write" for row in control_marker_writes)
decoder_review = {
    "run": 38662786466, "scope": "successful comparison only",
    "scheduledOutRecords": len(outgoing), "eventVersion": 5,
    "decodedFieldHistograms": histograms,
    "waitModeValuesWithoutPinnedEnumNames": unnamed_modes,
    "unnamedWaitModeFraction": unnamed_modes / len(outgoing),
    "unknownValuesPreservedWithoutMasking": True,
    "waitMode96IsNotReinterpretedAsKernelMode": True,
    "version5WaitFieldSemanticsVerified": False,
    "enumNameIsNotAWaitChainOrRootCause": True,
    "controlMarkerCreateEvents": len(control_marker_creates),
    "controlMarkerWriteEvents": len(control_marker_writes),
    "uniqueControlPidsWithMarkerWrites": 7,
    "markerFileWriteProvesSuccessfulComparisonProgressOnly": True,
}
report = {
    "method": "Offline recomputation from hash-checked original logs and recorded events",
    "cohorts": cohorts, "decoderReview": decoder_review,
    "originalBd2FailureStillOpen": True, "actualFailureTimeWindowEtwObtained": False,
    "noNewExecution": True, "desktopStarted": False, "nodeStarted": False,
    "etwSessionStarted": False, "originalDeadlinesOrAssertionsChanged": False,
    "rootCauseClaimed": False, "fullVersionGatePassed": False,
    "nextCaptureRequirements": [
        "Retain exact failed-artifact and official-Node identity and all original budgets.",
        "Record original process/thread/image starts before first owned Node launch.",
        "Keep capture through the original five-second observer, not only the query return.",
        "Differentiate control startup, query-worker startup and marker I/O by original time and process generation.",
        "Retain numeric decoded wait fields; do not infer version-five wait semantics from a version-two table.",
        "Capture a naturally failing first library run; do not repeat successful replays or prewarm Node.",
    ],
}
inputs_path, report_path = root / "input-manifest.json", root / "analysis.json"
if "--build" in sys.argv:
    assert not inputs_path.exists() and not report_path.exists()
    inputs_path.write_text(json.dumps(inputs, indent=2) + "\n", encoding="utf-8")
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert json.loads(inputs_path.read_text(encoding="utf-8")) == inputs
    assert json.loads(report_path.read_text(encoding="utf-8")) == report
print(json.dumps({
    "historicalLibrariesReplayedOffline": len(cohorts),
    "fixtureTimelinesVerified": sum(len(cohort["fixtures"]) for cohort in cohorts),
    "originalFailureControlMarkersMissingAtObserver": 7,
    "laterFailureVersionMarkersArriveAfterQueryButBeforeObserver": 6,
    "successfulReplayWaitModesWithoutPinnedEnumNames": unnamed_modes,
    "originalFailureNotCleared": True, "actualFailureEtwObtained": False,
    "newExecution": False,
}))

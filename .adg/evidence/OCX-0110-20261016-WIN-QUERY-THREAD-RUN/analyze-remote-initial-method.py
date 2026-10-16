"""Verify the richer single success trace without clearing the historical failure."""
import collections
import hashlib
import json
import re
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CAPTURE = ROOT / "remote-original/runtime/ci-bd2-query-etw/capture"
OUT = ROOT / "remote-original/out/windows-query-etw"
SOURCE = "7704a17902c98ceed047cc3dec2a361937b5e1f7"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


terminal = read(ROOT / "remote-terminal.original.json")
assert terminal["run"]["id"] == 38829382543 and terminal["run"]["run_attempt"] == 1
assert terminal["run"]["head_sha"] == SOURCE and terminal["run"]["conclusion"] == "success"
download = read(ROOT / "remote-download-verification.json")
for entry in download["entries"]:
    data = (ROOT / "remote-original" / entry["path"]).read_bytes()
    assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"]
admission = read(OUT / "admission.json")
assert admission["collectorSource"] == SOURCE and admission["collectorCiRun"] == 38826901050
assert admission["originalTestSource"] == "bd2f6eb10d58115235e35d575507bb69f44666da"
assert admission["testSha256"] == "497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3"
assert admission["nodeSha256"] == "ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32"
assert admission["nodeVersion"] == "24.21.0" and admission["originalFailedRun"] == 38537166560
assert admission["diagnosticPath"] == "isolated-replay" and not admission["exactHistoricalEnvironmentReproduced"]
assert admission["originalLibraryRunsAllowed"] == 1 and admission["originalLibraryCommandArguments"] == []
assert admission["originalConcurrencyAndDeadlinesPreserved"] and not admission["prewarm"]
capture = read(CAPTURE / "capture-result.json")
assert capture["boundedOriginalRunCount"] == 1 and capture["defaultConcurrencyAndCapturePreserved"]
assert capture["testExitCode"] == 0 and capture["originalLibraryCompleted"] and capture["ownedRootExited"]
assert capture["ownedSessionStopSucceeded"] and capture["processTraceJoined"] and capture["writerJoined"]
assert capture["maximumSerializedBytes"] == 64 * 1024 * 1024
assert capture["maximumQueueRecords"] == capture["maximumTrackedIrps"] == 32768
assert capture["droppedRecords"] == capture["kernelEventsLost"] == capture["decoderCallbackErrors"] == 0
assert not capture["knownImageCoverageGap"] and not capture["rootQueryThreadCoverageGap"]
assert capture["namedQueryThreadEvents"] == capture["expectedNamedQueryFixtures"] == 7
assert not capture["queryNameHistoryOverflowed"] and capture["maximumQueryNameHistoryRecords"] == 512
assert capture["rootQueryHotEventsBeginAtNameEvent"] and not capture["versionSpecificWaitSemanticsVerified"]
assert not capture["nodePrewarmed"] and not capture["forcedOuterCleanup"]
assert not capture["systemWideEtlWritten"] and not capture["unrelatedEventPayloadsSerialized"]
assert not capture["traceEventGenericStopInvoked"] and not capture["globalCpuSampleIntervalWritten"]
assert capture["captureError"] is None and capture["writerError"] is None and capture["runError"] is None
assert not capture["cleanupErrors"] and capture["stdoutSaved"] and capture["stderrSaved"]
raw = (CAPTURE / "owned-etw.events.jsonl").read_bytes()
assert len(raw) == capture["actualSerializedBytes"] == capture["queuedSerializedBytes"] <= 64 * 1024 * 1024
assert raw.endswith(b"\n") and b"\r\n" not in raw and not raw.startswith(b"\xef\xbb\xbf")
events = [json.loads(line) for line in raw.splitlines()]
assert len(events) == capture["recordedEvents"]
starts = [e for e in events if e["kind"] == "process-start"]
stops = [e for e in events if e["kind"] == "process-stop"]
assert collections.Counter(e["pid"] for e in starts) == collections.Counter(e["pid"] for e in stops)
binding, = [e for e in events if e["kind"] == "root-handle-binding"]
assert binding["pid"] == capture["testPid"] and binding["createdUtc"] == capture["rootCreationUtc"]
assert binding["actualToken"]["elevated"] and binding["fullLibraryArgumentCount"] == 0
assert binding["defaultConcurrency"] and binding["noPrewarm"] and binding["logicalProcessors"] == 4
names = [e for e in events if e["kind"] == "root-query-thread-name"]
assert {e["fixture"] for e in names} == set(capture["namedQueryFixtures"])
assert all(e["pid"] == capture["testPid"] and e["nameSource"] == "ThreadSetName payload" for e in names)
assert len({e["tid"] for e in names}) == 7
stdout = (CAPTURE / "test-stdout.original.log").read_text(encoding="utf-8-sig")
total = re.search(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", stdout)
assert total and tuple(map(int, total.groups()[1:])) == (530, 0, 4)
queries = {name: outcome for name, outcome in re.findall(r"test (\S+) \.\.\. (ok|FAILED)", stdout)
           if name in capture["namedQueryFixtures"]}
assert len(queries) == 7 and set(queries.values()) == {"ok"}
fixtures = []
for name in names:
    fixture = name["fixture"]
    selected = [e for e in events if e.get("fixture") == fixture]
    assert all(e["pid"] == capture["testPid"] and e["tid"] == name["tid"]
               and e["original"]["qpc"] >= name["original"]["qpc"] for e in selected)
    end, = [e for e in events if e["kind"] == "thread-stop"
            and e["pid"] == name["pid"] and e["tid"] == name["tid"]
            and e["original"]["qpc"] > name["original"]["qpc"]]
    assert all(e["original"]["qpc"] <= end["original"]["qpc"] for e in selected)
    counts = dict(collections.Counter(e["kind"] for e in selected))
    assert counts.get("owned-thread-scheduled-in", 0) > 0
    assert counts.get("owned-thread-scheduled-out", 0) > 0
    assert counts.get("owned-file-create", 0) > 0
    span = (datetime.fromisoformat(end["original"]["atUtc"].replace("Z", "+00:00")) -
            datetime.fromisoformat(name["original"]["atUtc"].replace("Z", "+00:00"))).total_seconds() * 1000
    fixtures.append({"fixture": fixture, "result": queries[fixture], "pid": name["pid"],
                     "tid": name["tid"], "nameQpc": name["original"]["qpc"],
                     "stopQpc": end["original"]["qpc"], "observedNameToStopMs": span,
                     "counts": counts, "testDurationBeforeNameKnown": False,
                     "waitReasonSemanticsUsedForDiagnosis": False})
result = {
    "collectorSource": SOURCE, "runId": 38829382543, "originalFailedRun": 38537166560,
    "originalLibraryRuns": 1, "libraryExitCode": 0,
    "tests": {"passed": 530, "failed": 0, "ignored": 4},
    "actualThreadSetNameEvents": 7, "namedQueryFixtures": fixtures,
    "eventCount": len(events), "eventBytes": len(raw), "eventSha256": hashlib.sha256(raw).hexdigest(),
    "eventKinds": dict(collections.Counter(e["kind"] for e in events)),
    "processStarts": len(starts), "processStops": len(stops),
    "rootActualToken": binding["actualToken"], "rootCreationUtc": capture["rootCreationUtc"],
    "captureDrops": 0, "knownDescendantGap": False, "namedQueryThreadGap": False,
    "fullRootHotEventCoverage": False,
    "rootSchedulingEventsOmitted": capture["rootSchedulingEventsIntentionallyOmitted"],
    "rootFileIoInitiationsOmitted": capture["rootFileIoInitiationsIntentionallyOmitted"],
    "targetQueryFailureReproduced": False, "actualFailureWindowObtained": False,
    "rootCauseDetermined": False, "historicalFailureCleared": False,
    "versionSpecificWaitSemanticsVerified": False,
    "exactHistoricalEnvironmentReproduced": False, "oneRevisedCaptureCompleted": True,
    "anotherOriginalRunRequested": False, "hostDesktopOrUacUsed": False,
    "fullPerformanceGatePassed": False, "fullVersionGatePassed": False,
}
path = ROOT / "analysis.json"
if path.exists():
    assert read(path) == result
else:
    path.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps({k: v for k, v in result.items() if k not in {"namedQueryFixtures", "eventKinds"}}))

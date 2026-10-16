"""Recompute one future complete 019/b4 pair. Never substitute prior samples."""
import hashlib
import json
import statistics
import sys
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parent
ALLOWED = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop/test/runtime").resolve()
assert len(sys.argv) == 2, "An explicit task-owned run directory is required"
RUN = Path(sys.argv[1]).resolve()
assert RUN.is_dir() and RUN != ALLOWED and RUN.is_relative_to(ALLOWED)
assert not (RUN / "comparison.json").exists(), "Preserve the original comparison"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def stamp(value):
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


def filetime(value):
    # Preserve the seventh fractional digit rather than round through JS/double.
    dt = stamp(value)
    epoch = datetime(1601, 1, 1, tzinfo=dt.tzinfo)
    delta = dt - epoch
    fraction = value.split(".", 1)[1].split("Z")[0].split("+")[0] if "." in value else ""
    extra = int((fraction + "0000000")[:7][-1])
    return (delta.days * 86400 + delta.seconds) * 10_000_000 + delta.microseconds * 10 + extra


contract = read(ROOT / "contract.json")
assert contract["candidateSource"] == "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
assert contract["order"] == ["baseline019", "candidate0110"]
assert contract["budgetsUnchanged"] == {"cpuIncreasePercentagePoints": 0.5, "rssIncreaseMiB": 10}
assert contract["minimumCompletedCounterSpanSeconds"] == 600 and contract["settleSecondsPerVersion"] == 60
human = read(RUN / "human-window.json")
assert human["explicitHumanStart"] and not human["automaticContinuation"] and human["userMessage"]
assert human["scope"] == contract["humanWindowScope"]
window_start, window_end = stamp(human["authorizedAtUtc"]), stamp(human["expiresAtUtc"])
assert 0 < (window_end - window_start).total_seconds() <= 1800
results = {}
for version, prefix in (("baseline019", "baseline"), ("candidate0110", "candidate")):
    launch_path = RUN / ("idle-" + version + "-launch.json")
    ready_path = RUN / (version + "-sky-settled-ready.json")
    launch, ready = read(launch_path), read(ready_path)
    output = RUN / version
    identity, terminal = read(output / "identity.json"), read(output / "terminal.json")
    stop = read(RUN / (version + "-exit.json"))
    assert launch["source"] == identity["source"] == contract[prefix + "Source"]
    assert launch["exeSha256"] == identity["exeSha256"] == contract[prefix + "ExeSha256"]
    assert launch["pid"] == identity["pid"] == stop["pid"]
    assert launch["creationTimeUtc"] == identity["creationTimeUtc"]
    assert not launch["sandboxUsed"] and not launch["cdpUsed"]
    token = launch["applicationToken"]
    assert token["sidSha256"] == "373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310"
    assert not token["elevated"] and token["elevationType"] == 3
    assert not identity["actualToken"]["elevated"] and identity["actualToken"]["elevationType"] == 3
    assert identity["launchInputSha256"] == digest(launch_path)
    assert identity["readinessInputSha256"] == digest(ready_path)
    assert identity["moduleSha256"] == contract["methodWorkingFileSha256"]["test/smoke/windows-idle-family.ps1"]
    assert identity["methodSha256"] == contract["methodWorkingFileSha256"]["test/smoke/windows-idle-observe.ps1"]
    assert (identity["clientWidth"], identity["clientHeight"]) == (2360, 1520)
    assert "尚未接入 OpenCodex" in ready["accessibility"]["tree"]
    assert (stamp(ready["observedAtUtc"]) - stamp(launch["creationTimeUtc"])).total_seconds() >= 60
    assert terminal["measurementCompleted"] and not terminal["observationInvalid"]
    assert terminal["readOrFrameFailures"] == 0 and terminal["counterSpanSeconds"] >= 600
    assert stop["exitObserved"] and stop["exitCode"] == 0 and stop["retainedHandle"]
    assert not stop["forcedTermination"]
    kernel = stop["kernelTimes"]
    assert kernel["pid"] == launch["pid"] and kernel["exited"] and kernel["retainedHandle"]
    assert int(kernel["creationFileTime"]) == filetime(launch["creationTimeUtc"])
    assert int(kernel["exitFileTime"]) >= int(kernel["creationFileTime"])
    assert kernel["sourceMethodSha256"] == contract["kernelTimesMethodSha256"]
    path = output / "samples.original.jsonl"
    rows = [json.loads(line) for line in path.read_text(encoding="utf-8-sig").splitlines()]
    assert len(rows) == terminal["samples"] > 1
    span = rows[-1]["elapsedSeconds"] - rows[0]["elapsedSeconds"]
    assert span == terminal["counterSpanSeconds"] >= 600
    known = set()
    for row in rows:
        assert row["foreground"] and row["visible"] and not row["minimized"] and row["nativeCount"] == 1
        assert window_start <= stamp(row["atUtc"]) <= window_end
        members = {(m["pid"], m["creationUtc"]) for m in row["members"]}
        assert known <= members
        known = members
        assert row["cpuSeconds"] == sum(m["cpuSeconds"] for m in row["members"])
        assert row["workingSetBytes"] == sum(m["workingSetBytes"] for m in row["members"])
    assert all(b["elapsedSeconds"] > a["elapsedSeconds"] and b["cpuSeconds"] >= a["cpuSeconds"]
               for a, b in zip(rows, rows[1:]))
    results[version] = {
        "source": launch["source"], "exeSha256": launch["exeSha256"], "samples": len(rows),
        "counterSpanSeconds": span, "rawSha256": digest(path),
        "cpuOneLogicalProcessorPercent": 100 * (rows[-1]["cpuSeconds"] - rows[0]["cpuSeconds"]) / span,
        "meanRssMiB": statistics.mean(row["workingSetBytes"] for row in rows) / 1048576,
        "creationFileTime": kernel["creationFileTime"], "exitFileTime": kernel["exitFileTime"],
        "collectorCpuSeconds": terminal["collectorCpuSeconds"],
    }
baseline, candidate = results["baseline019"], results["candidate0110"]
assert int(candidate["creationFileTime"]) >= int(baseline["exitFileTime"])
cpu = candidate["cpuOneLogicalProcessorPercent"] - baseline["cpuOneLogicalProcessorPercent"]
rss = candidate["meanRssMiB"] - baseline["meanRssMiB"]
comparison = {
    "versions": results, "validPairForRecordedSubset": True, "priorSamplesUsed": False,
    "cpuIncreasePercentagePoints": cpu, "meanRssIncreaseMiB": rss,
    "cpuBudgetPassed": cpu <= 0.5, "meanRssBudgetPassed": rss <= 10,
    "budgets": contract["budgetsUnchanged"], "sharedResidentPagesDeduplicated": False,
    "shortLivedChildrenFullyCovered": False, "fullPerformanceGatePassed": False,
    "historicalFailuresCleared": False, "releaseAccepted": False,
}
with (RUN / "comparison.json").open("x", encoding="utf-8") as stream:
    json.dump(comparison, stream, indent=2)
print(json.dumps({k: v for k, v in comparison.items() if k != "versions"}))

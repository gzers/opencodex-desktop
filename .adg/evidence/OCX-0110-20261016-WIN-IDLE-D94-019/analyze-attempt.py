"""Audit rejected observations without inventing a complete performance pair."""
import hashlib
import json
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


contract = read("contract.json")
assert contract["candidateSource"] == "d94c215d869c0560717d79dc02881ce8858afe88"
assert contract["budgetsUnchanged"] == {"cpuIncreasePercentagePoints": 0.5, "rssIncreaseMiB": 10}
versions = {}
for version, prefix in (("baseline019", "baseline"), ("candidate0110", "candidate")):
    launch = read("idle-" + version + "-launch.json")
    terminal = read(version + "-terminal.json")
    close = read(version + "-exit.json")
    ready = read(version + "-readiness-binding.json")
    source_view = read(version + "-sky-settled-ready.json")
    assert launch["source"] == contract[prefix + "Source"]
    assert launch["exeSha256"] == contract[prefix + "ExeSha256"]
    assert launch["applicationUser"]["elevated"] is False and launch["applicationUser"]["elevationType"] == 3
    assert launch["applicationUser"]["sidSha256"] == "373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310"
    assert not launch["sandboxUsed"] and not launch["cdpUsed"]
    assert close["pid"] == launch["pid"] and close["exitObserved"] and close["exitCode"] == 0
    assert close["retainedHandle"] and not close["forcedTermination"]
    assert ready["settledSeconds"] >= 60 and ready["ownerPid"] == launch["pid"]
    assert (ready["clientWidth"], ready["clientHeight"]) == (2360, 1520)
    assert "尚未接入 OpenCodex" in source_view["accessibility"]["tree"]
    path = ROOT / (version + "-samples.original.jsonl")
    raw = path.read_bytes()
    rows = [json.loads(line) for line in raw.splitlines()]
    assert len(rows) > 1
    assert all(b["elapsedSeconds"] > a["elapsedSeconds"] and b["cpuSeconds"] >= a["cpuSeconds"]
               for a, b in zip(rows, rows[1:]))
    for row in rows:
        assert row["cpuSeconds"] == sum(m["cpuSeconds"] for m in row["members"])
        assert row["workingSetBytes"] == sum(m["workingSetBytes"] for m in row["members"])
        assert row["nativeCount"] == 1 and row["visible"] and not row["minimized"]
    roles = read(version + "-role-snapshot.json")
    assert 120 <= roles["sampleElapsedSeconds"] <= 300 and roles["source"] == launch["source"]
    span = rows[-1]["elapsedSeconds"] - rows[0]["elapsedSeconds"]
    if version == "baseline019":
        metadata = read(version + "-measurement.json")
        assert terminal["measurementCompleted"] and terminal["completed"] == [version]
        assert span == metadata["observedCounterSpanSeconds"] >= 600
        assert metadata["childDiscoveryReadFailures"] == 1
        assert len(rows) == metadata["samples"] == 598
        assert all(row["foreground"] for row in rows)
        read_failures = 1
        rejection = "One child discovery read failed; missed process generation/counters unknown."
    else:
        assert not terminal["measurementCompleted"] and terminal["completed"] == []
        assert span < 600 and len(rows) == 148
        assert not rows[-1]["foreground"] and all(row["foreground"] for row in rows[:-1])
        assert not (ROOT / (version + "-measurement.json")).exists()
        assert "Foreground was interrupted" in (ROOT / (version + "-failure.original.txt")).read_text(encoding="utf-8-sig")
        read_failures = None
        rejection = "Foreground interrupted at the last recorded sample before 600 seconds."
    versions[version] = {
        "source": launch["source"], "exeSha256": launch["exeSha256"],
        "pid": launch["pid"], "createdAt": launch["creationTimeUtc"],
        "samples": len(rows), "counterSpanSeconds": span,
        "firstObservedAt": rows[0]["atUtc"], "lastObservedAt": rows[-1]["atUtc"],
        "foregroundLostSamples": sum(not row["foreground"] for row in rows),
        "nativeInstanceViolations": 0, "childDiscoveryReadFailures": read_failures,
        "collectorCompletedMeasurement": terminal["measurementCompleted"],
        "validForPerformancePair": False, "rejection": rejection,
        "rawSha256": hashlib.sha256(raw).hexdigest(), "normalExit": True,
    }
assert datetime.fromisoformat(versions["candidate0110"]["createdAt"].replace("Z", "+00:00")) > \
       datetime.fromisoformat(read("baseline019-exit.json")["atUtc"].replace("Z", "+00:00"))
assert not (ROOT / "comparison.json").exists()
result = {
    "source": contract["candidateSource"], "versions": versions,
    "validPairForRecordedSubset": False, "cpuBudgetPassed": False,
    "meanRssBudgetPassed": False, "newCandidateMemoryBudgetDetermined": False,
    "missingChildWasAssumedNegligible": False,
    "foregroundInterruptionCauseEstablished": False,
    "originalBudgetAndZeroReadFailureCriterionPreserved": True,
    "oldBaselineOrInterruptedSamplesSubstituted": False,
    "automaticRerunRequested": False, "bothNormalExits": True,
    "fullPerformanceGatePassed": False, "historicalFailuresErased": False,
    "macHistoricalFailureCleared": False, "taskClosed": False, "releaseAccepted": False,
}
path = ROOT / "attempt-result.json"
if path.exists():
    assert read(path.name) == result
else:
    path.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps({key: value for key, value in result.items() if key != "versions"}))

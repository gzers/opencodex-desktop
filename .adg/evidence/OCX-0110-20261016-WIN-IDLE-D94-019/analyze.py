"""Recompute this single-window pair without using any prior measurement."""
import hashlib
import json
import statistics
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


contract = read("contract.json")
method = read("method-admission.json")
assert contract["candidateSource"] == method["source"] == "d94c215d869c0560717d79dc02881ce8858afe88"
assert contract["candidateExeSha256"] == method["candidateExeSha256"]
assert contract["order"] == ["baseline019", "candidate0110"]
assert contract["budgetsUnchanged"] == method["budgets"] == {
    "cpuIncreasePercentagePoints": 0.5, "rssIncreaseMiB": 10}
assert not method["priorSamplesUsedInNewComparison"]
results = {}
for version in contract["order"]:
    terminal = read(version + "-terminal.json")
    assert terminal["completed"] == [version] and terminal["measurementCompleted"]
    launch = read("idle-" + version + "-launch.json")
    stop = read(version + "-exit.json")
    binding = read(version + "-readiness-binding.json")
    ready = read(version + "-sky-settled-ready.json")
    metadata = read(version + "-measurement.json")
    prefix = "baseline" if version == "baseline019" else "candidate"
    assert launch["source"] == contract[prefix + "Source"]
    assert launch["exeSha256"] == contract[prefix + "ExeSha256"]
    assert launch["applicationUser"]["sidSha256"] == "373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310"
    assert not launch["applicationUser"]["elevated"] and launch["applicationUser"]["elevationType"] == 3
    assert not launch["sandboxUsed"] and not launch["cdpUsed"]
    assert stop["pid"] == launch["pid"] and stop["exitObserved"] and stop["exitCode"] == 0
    assert not stop["forcedTermination"] and stop["retainedHandle"]
    assert binding["ownerPid"] == launch["pid"]
    assert (binding["clientWidth"], binding["clientHeight"]) == (2360, 1520)
    assert binding["settledSeconds"] >= 60 and "尚未接入 OpenCodex" in ready["accessibility"]["tree"]
    path = ROOT / (version + "-samples.original.jsonl")
    rows = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]
    assert len(rows) == metadata["samples"] > 1
    first, last = rows[0], rows[-1]
    span = last["elapsedSeconds"] - first["elapsedSeconds"]
    assert span == metadata["observedCounterSpanSeconds"] >= 600
    complete_family_reads = metadata["childDiscoveryReadFailures"] == 0
    assert all(row["foreground"] and row["visible"] and not row["minimized"]
               and row["nativeCount"] == 1 for row in rows)
    assert all(b["elapsedSeconds"] > a["elapsedSeconds"] and b["cpuSeconds"] >= a["cpuSeconds"]
               for a, b in zip(rows, rows[1:]))
    known = set()
    for row in rows:
        members = {(m["pid"], m["creationUtc"]) for m in row["members"]}
        assert known <= members
        known = members
        assert row["cpuSeconds"] == sum(m["cpuSeconds"] for m in row["members"])
        assert row["workingSetBytes"] == sum(m["workingSetBytes"] for m in row["members"])
    roles = read(version + "-role-snapshot.json")
    assert roles["source"] == launch["source"] and 120 <= roles["sampleElapsedSeconds"] <= 300
    assert not roles["inputInjected"] and not roles["counterSamplingChanged"]
    by_role = {}
    observed = {(r["pid"], r["creationUtc"]): r["role"] for r in roles["members"]}
    for role in sorted(set(observed.values())):
        by_role[role] = statistics.mean(
            sum(m["workingSetBytes"] for m in row["members"]
                if observed.get((m["pid"], m["creationUtc"])) == role) for row in rows) / 1048576
    main_rss = statistics.mean(sum(m["workingSetBytes"] for m in r["members"]
                                  if m["pid"] == launch["pid"]) for r in rows) / 1048576
    results[version] = {
        "source": launch["source"], "exeSha256": launch["exeSha256"],
        "pid": launch["pid"], "creationTimeUtc": launch["creationTimeUtc"],
        "firstObservedAt": first["atUtc"], "lastObservedAt": last["atUtc"],
        "samples": len(rows), "observedSpanSeconds": span,
        "singleLogicalProcessorPercent": 100 * (last["cpuSeconds"] - first["cpuSeconds"]) / span,
        "meanWorkingSetMiB": statistics.mean(row["workingSetBytes"] for row in rows) / 1048576,
        "startWorkingSetMiB": first["workingSetBytes"] / 1048576,
        "endWorkingSetMiB": last["workingSetBytes"] / 1048576,
        "peakWorkingSetMiB": max(row["workingSetBytes"] for row in rows) / 1048576,
        "mainProcessMeanWorkingSetMiB": main_rss,
        "meanWorkingSetByObservedRoleMiB": by_role,
        "rawSha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "foregroundLostSamples": 0, "nativeInstanceViolations": 0,
        "minimumFamilyMembers": min(len(row["members"]) for row in rows),
        "maximumFamilyMembers": max(len(row["members"]) for row in rows),
        "childDiscoveryReadFailures": metadata["childDiscoveryReadFailures"],
        "collectorCpuSeconds": metadata["collectorCpuSeconds"],
        "validForRecordedForegroundIdleSubset": complete_family_reads,
        "incompleteFamilyReadReason": None if complete_family_reads else
            "Child discovery failed at least once; the missed generation/counters were not retained.",
    }
baseline, candidate = results["baseline019"], results["candidate0110"]
baseline_stop = read("baseline019-exit.json")
assert datetime.fromisoformat(candidate["creationTimeUtc"].replace("Z", "+00:00")) > datetime.fromisoformat(
    baseline_stop["atUtc"].replace("Z", "+00:00"))
cpu = candidate["singleLogicalProcessorPercent"] - baseline["singleLogicalProcessorPercent"]
rss = candidate["meanWorkingSetMiB"] - baseline["meanWorkingSetMiB"]
valid_pair = (baseline["validForRecordedForegroundIdleSubset"]
              and candidate["validForRecordedForegroundIdleSubset"])
comparison = {
    "versions": results, "budgets": contract["budgetsUnchanged"],
    "cpuIncreasePercentagePoints": cpu, "meanWorkingSetIncreaseMiB": rss,
    "endWorkingSetIncreaseMiB": candidate["endWorkingSetMiB"] - baseline["endWorkingSetMiB"],
    "peakWorkingSetIncreaseMiB": candidate["peakWorkingSetMiB"] - baseline["peakWorkingSetMiB"],
    "mainProcessMeanWorkingSetIncreaseMiB":
        candidate["mainProcessMeanWorkingSetMiB"] - baseline["mainProcessMeanWorkingSetMiB"],
    "validPairForRecordedSubset": valid_pair, "cpuBudgetPassed": valid_pair and cpu <= 0.5,
    "meanRssBudgetPassed": valid_pair and rss <= 10,
    "diagnosticObservedCpuDeltaWithinLimit": cpu <= 0.5,
    "diagnosticObservedMeanRssDeltaWithinLimit": rss <= 10,
    "rawDeltasAreAcceptanceEvidence": valid_pair,
    "bothFreshProcessesInSameHumanWindow": True,
    "bothSameReadinessObserver": True, "priorMeasurementsUsed": False,
    "sharedResidentPagesDeduplicated": False, "shortLivedChildrenFullyCovered": False,
    "fullPerformanceGatePassed": False, "historicalFailuresErased": False,
    "macHistoricalFailureCleared": False, "releaseAccepted": False,
}
(ROOT / "comparison.json").write_text(json.dumps(comparison, indent=2) + "\n", encoding="utf-8")
print(json.dumps({k: v for k, v in comparison.items() if k != "versions"}))

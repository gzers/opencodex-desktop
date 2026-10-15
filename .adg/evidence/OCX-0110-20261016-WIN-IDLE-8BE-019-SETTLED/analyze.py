import hashlib
import json
import statistics
from pathlib import Path

ROOT = Path(__file__).resolve().parent

def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))

contract = read("contract.json")
terminal = read("terminal.json")
assert terminal["measurementCompleted"] and terminal["completed"] == contract["order"]
results = {}
for version in contract["order"]:
    path = ROOT / f"{version}-samples.original.jsonl"
    rows = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]
    assert len(rows) > 1
    first, last = rows[0], rows[-1]
    span = last["elapsedSeconds"] - first["elapsedSeconds"]
    assert span > 0 and all(b["elapsedSeconds"] > a["elapsedSeconds"]
                            for a, b in zip(rows, rows[1:]))
    assert all(row["cpuSeconds"] == sum(member["cpuSeconds"] for member in row["members"])
               for row in rows)
    assert all(row["workingSetBytes"] == sum(member["workingSetBytes"] for member in row["members"])
               for row in rows)
    metadata = read(f"{version}-measurement.json")
    exit_record = read(f"{version}-exit.json")
    launch = read(f"idle-{version}-launch.json")
    assert launch["applicationUser"]["elevated"] is False
    assert launch["applicationUser"]["sidSha256"] == "373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310"
    assert exit_record["exitObserved"] and exit_record["exitCode"] == 0 and not exit_record["forcedTermination"]
    valid = (span >= 600 and all(row["foreground"] and row["visible"] and not row["minimized"]
                               and row["nativeCount"] == 1 for row in rows)
             and metadata["childDiscoveryReadFailures"] == 0
             and min(len(row["members"]) for row in rows) > 1)
    results[version] = {
        "source": launch["source"], "exeSha256": launch["exeSha256"],
        "samples": len(rows), "observedSpanSeconds": span,
        "foregroundLostSamples": sum(not row["foreground"] for row in rows),
        "nativeInstanceViolations": sum(row["nativeCount"] != 1 for row in rows),
        "minimumFamilyMembers": min(len(row["members"]) for row in rows),
        "maximumFamilyMembers": max(len(row["members"]) for row in rows),
        "childDiscoveryReadFailures": metadata["childDiscoveryReadFailures"],
        "cpuSecondsDelta": last["cpuSeconds"] - first["cpuSeconds"],
        "singleLogicalProcessorPercent": 100 * (last["cpuSeconds"] - first["cpuSeconds"]) / span,
        "meanWorkingSetMiB": statistics.mean(row["workingSetBytes"] for row in rows) / 1048576,
        "startWorkingSetMiB": first["workingSetBytes"] / 1048576,
        "endWorkingSetMiB": last["workingSetBytes"] / 1048576,
        "peakWorkingSetMiB": max(row["workingSetBytes"] for row in rows) / 1048576,
        "validForRecordedForegroundIdleSubset": valid,
        "rawSha256": hashlib.sha256(path.read_bytes()).hexdigest(),
    }
candidate, baseline = results["candidate0110"], results["baseline019"]
cpu = candidate["singleLogicalProcessorPercent"] - baseline["singleLogicalProcessorPercent"]
rss = candidate["meanWorkingSetMiB"] - baseline["meanWorkingSetMiB"]
end_rss = candidate["endWorkingSetMiB"] - baseline["endWorkingSetMiB"]
peak_rss = candidate["peakWorkingSetMiB"] - baseline["peakWorkingSetMiB"]
valid_pair = candidate["validForRecordedForegroundIdleSubset"] and baseline["validForRecordedForegroundIdleSubset"]
result = {
    "versions": results,
    "cpuIncreasePercentagePoints": cpu, "meanWorkingSetIncreaseMiB": rss,
    "endWorkingSetIncreaseMiB": end_rss, "peakWorkingSetIncreaseMiB": peak_rss,
    "budgets": contract["budgetsUnchanged"], "validPairForRecordedSubset": valid_pair,
    "cpuBudgetPassed": valid_pair and cpu <= 0.5,
    "meanRssBudgetPassed": valid_pair and rss <= 10,
    "candidateWasReattachedAfterInterruption": False,
    "baselineWasReattachedAfterReadinessToolFailure": False,
    "oldInterruptedSamplesUsed": False,
    "sharedResidentPagesDeduplicated": False,
    "shortLivedChildrenFullyCovered": False,
    "fullPerformanceGatePassed": False,
    "macHistoricalFailureCleared": False,
}
(ROOT / "comparison.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result))

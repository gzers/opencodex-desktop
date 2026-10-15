"""Compare a completed fresh candidate with the bound completed baseline."""
import hashlib
import json
import statistics
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def read(root, name):
    return json.loads((root / name).read_text(encoding="utf-8-sig"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


contract = read(ROOT, "contract.json")
assert contract["order"] == ["candidate0110"]
assert contract["budgetsUnchanged"] == {
    "cpuIncreasePercentagePoints": 0.5, "rssIncreaseMiB": 10}
BASELINE = ROOT.parent / contract["reusedBaselineEvidence"]
for name, binding in read(ROOT, "reused-evidence-bindings.json").items():
    path = BASELINE / name
    assert path.stat().st_size == binding["bytes"]
    assert digest(path) == binding["sha256"]
assert read(ROOT, "terminal.json")["measurementCompleted"]
assert read(ROOT, "terminal.json")["completed"] == ["candidate0110"]
assert read(BASELINE, "terminal.json")["completed"] == ["baseline019"]
assert not read(BASELINE, "terminal.json")["measurementCompleted"]
assert digest(ROOT / "preferences-before.original.json") == read(
    BASELINE, "preparation.original.json")["fixturePreferencesSha256"]
readiness = read(ROOT, "readiness-binding.json")
assert readiness["settledSeconds"] >= 60
assert (readiness["clientWidth"], readiness["clientHeight"]) == (2360, 1520)
assert "尚未接入 OpenCodex" in read(ROOT, "sky-settled-ready.json")["accessibility"]["tree"]

results = {}
for version, folder, expected in (
    ("baseline019", BASELINE, "baseline"),
    ("candidate0110", ROOT, "candidate"),
):
    raw = folder / f"{version}-samples.original.jsonl"
    rows = [json.loads(line) for line in raw.read_text(encoding="utf-8").splitlines()]
    metadata = read(folder, f"{version}-measurement.json")
    launch = read(folder, f"idle-{version}-launch.json")
    stop = read(folder, f"{version}-exit.json")
    assert launch["source"] == contract[f"{expected}Source"]
    assert launch["exeSha256"] == contract[f"{expected}ExeSha256"]
    assert not launch["sandboxUsed"] and not launch["cdpUsed"]
    user = launch["applicationUser"]
    assert not user["elevated"] and user["elevationType"] == 3
    assert user["sidSha256"] == "373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310"
    assert stop["exitObserved"] and stop["exitCode"] == 0
    assert not stop["forcedTermination"]
    assert len(rows) == metadata["samples"] > 1
    first, last = rows[0], rows[-1]
    span = last["elapsedSeconds"] - first["elapsedSeconds"]
    assert span == metadata["observedCounterSpanSeconds"] >= 600
    assert metadata["childDiscoveryReadFailures"] == 0
    assert all(row["foreground"] and row["visible"] and not row["minimized"]
               and row["nativeCount"] == 1 for row in rows)
    assert all(b["elapsedSeconds"] > a["elapsedSeconds"]
               and b["cpuSeconds"] >= a["cpuSeconds"]
               for a, b in zip(rows, rows[1:]))
    retained = set()
    for row in rows:
        keys = {(m["pid"], m["creationUtc"]) for m in row["members"]}
        assert retained <= keys
        retained = keys
        assert row["cpuSeconds"] == sum(m["cpuSeconds"] for m in row["members"])
        assert row["workingSetBytes"] == sum(m["workingSetBytes"] for m in row["members"])
    role_path = folder / f"{version}-role-snapshot.json"
    roles = read(folder, role_path.name) if role_path.exists() else None
    per_process = []
    initial = {(m["pid"], m["creationUtc"]): m for m in first["members"]}
    for member in last["members"]:
        key = member["pid"], member["creationUtc"]
        old = initial.get(key)
        role = next((r["role"] for r in roles["members"]
                     if r["pid"] == member["pid"] and r["creationUtc"] == member["creationUtc"]),
                    "unobserved") if roles else "unobserved"
        per_process.append({
            "pid": member["pid"], "creationUtc": member["creationUtc"], "role": role,
            "presentAtBothEndpoints": old is not None,
            "cpuSecondsDelta": member["cpuSeconds"] - (old["cpuSeconds"] if old else 0),
            "endWorkingSetMiB": member["workingSetBytes"] / 1048576,
            "exited": member["exited"],
        })
    results[version] = {
        "source": launch["source"], "exeSha256": launch["exeSha256"],
        "evidenceDirectory": folder.name, "rawSha256": digest(raw),
        "samples": len(rows), "observedSpanSeconds": span,
        "firstObservedAt": first["atUtc"], "lastObservedAt": last["atUtc"],
        "singleLogicalProcessorPercent": 100 * (last["cpuSeconds"] - first["cpuSeconds"]) / span,
        "meanWorkingSetMiB": statistics.mean(row["workingSetBytes"] for row in rows) / 1048576,
        "startWorkingSetMiB": first["workingSetBytes"] / 1048576,
        "endWorkingSetMiB": last["workingSetBytes"] / 1048576,
        "peakWorkingSetMiB": max(row["workingSetBytes"] for row in rows) / 1048576,
        "foregroundLostSamples": sum(not r["foreground"] for r in rows),
        "nativeInstanceViolations": sum(r["nativeCount"] != 1 for r in rows),
        "minimumFamilyMembers": min(len(r["members"]) for r in rows),
        "maximumFamilyMembers": max(len(r["members"]) for r in rows),
        "childDiscoveryReadFailures": metadata["childDiscoveryReadFailures"],
        "collectorCpuSeconds": metadata["collectorCpuSeconds"],
        "validForRecordedForegroundIdleSubset": True,
        "perObservedProcess": per_process,
    }
candidate, baseline = results["candidate0110"], results["baseline019"]
cpu = candidate["singleLogicalProcessorPercent"] - baseline["singleLogicalProcessorPercent"]
rss = candidate["meanWorkingSetMiB"] - baseline["meanWorkingSetMiB"]
result = {
    "versions": results, "cpuIncreasePercentagePoints": cpu,
    "meanWorkingSetIncreaseMiB": rss,
    "endWorkingSetIncreaseMiB": candidate["endWorkingSetMiB"] - baseline["endWorkingSetMiB"],
    "peakWorkingSetIncreaseMiB": candidate["peakWorkingSetMiB"] - baseline["peakWorkingSetMiB"],
    "budgets": contract["budgetsUnchanged"],
    "validPairForRecordedSubset": True, "cpuBudgetPassed": cpu <= 0.5,
    "meanRssBudgetPassed": rss <= 10,
    "baselineReusedFromExplicitlyBoundCompletedObservation": True,
    "candidateResampledAfterForegroundInterruption": True,
    "interruptedCandidateSamplesUsed": False,
    "readinessObserverDiffersFromBaseline": True,
    "pairedVersionsObservedWithoutInterveningInterruption": False,
    "sharedResidentPagesDeduplicated": False,
    "shortLivedChildrenFullyCovered": False,
    "causalAttributionToDepthBufferEstablished": False,
    "fullPerformanceGatePassed": False, "macHistoricalFailureCleared": False,
}
(ROOT / "comparison.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps({k: v for k, v in result.items() if k != "versions"}))

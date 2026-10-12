"""Validate a negative completion audit and its source boundaries, never close TASK."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
docs = root.parents[2]
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
audit = read(root / "requirement-audit.json")
old = read(docs / ".adg/evidence/OCX-0110-20261012-WIN-FINAL-CI-AUDIT/requirement-audit.json")
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
running = "afff490a430a81f8c233d44928aa3681a23e91e0"
assert audit["sourceCommit"] == source and audit["runningSourceCommit"] == running
assert audit["objectiveScopeUnchanged"] and audit["auditOnlyNotFormalTaskChecks"]
assert audit["originalRequirementCount"] == len(audit["requirements"]) == 14
assert [row["requirement"] for row in audit["requirements"]] == [
    row["requirement"] for row in old["requirements"]
]
assert audit["overallResult"] == "not_achieved"
assert not any(audit[name] for name in [
    "formalTaskReceipt", "formalCertificate", "performanceFailureCleared",
    "coreAuthorityPromoted", "releaseAccepted",
])
assert any(not row["completeOriginalRequirement"] for row in audit["requirements"])
assert audit["requirements"][0]["status"] == "proved_current_source"
assert audit["requirements"][1]["status"] == "incomplete_missing_original_authority"
assert audit["requirements"][11]["status"] == "not_measured_original_failure_retained"
assert audit["requirements"][12]["status"] == "waiting_real_time_not_accepted"
budget = audit["originalPerformanceBudget"]
assert budget == {
    "coldStartsMinimum": 20, "hotStartsMinimum": 20,
    "p95IncrementMaxMilliseconds": 100, "p95IncrementMaxPercent": 5,
    "idleMinutes": 10, "idleCpuIncrementMaxPercentagePoints": 0.5,
    "rssIncrementMaxMiB": 10, "mainThreadTaskMaxMilliseconds": 50,
    "cacheRoutes": 100, "cacheRestarts": 10, "unexpiredCacheAutoNetworkOrQuerySubprocessMax": 0,
}
live = read(root / "current-source-and-lifecycle.json")
assert live["mainSource"] == live["originMainSource"] == source
assert live["requestedCodeBaselineAncestor"] and live["requestedDocsBaselineAncestor"]
assert live["codeBranch"] == "main" and live["docsBranch"] == "docs/governance-main"
assert live["codeWorktreeClean"] and live["onlyOneDesktopApplication"]
assert live["retainedApplicationPid"] == 38924 and live["retainedApplicationSource"] == running
assert live["monitorPid"] == 18944 and live["monitorExecSession"] == 93170
assert live["retainedApplicationAndMonitorLiveHandlesVerified"] and live["realCycleScheduleUnchanged"]
assert live["actualScheduleSha256"] == live["baselineScheduleSha256"]
assert live["dailyExeSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert all(row["identical"] and row["codeBlob"] == row["docsBlob"] for row in live["sharedFiles"])
assert len(live["schedulerBlobs"]) == 4
assert all(row["identical"] and row["runningSourceBlob"] == row["currentMainBlob"] for row in live["schedulerBlobs"])
assert not any(live[name] for name in [
    "task182Present", "newMainDesktopLaunched", "localInstallerExecuted",
    "scriptsSecondConfirmationReceived", "scriptsExecuted", "formalTaskReceipt",
    "goalCompletionAccepted", "realCycleCompleted", "originalPerformanceFailureCleared",
    "realDataMigrated", "releaseAccepted",
])
native = read(root / "current-native-read.json")
assert native["sourceCommit"] == running and native["applicationPid"] == 38924
assert native["result"]["runtime"]["kind"] == "unresolved"
assert native["result"]["preferences"] == {
    "theme": "system", "automaticCheck": True, "channel": "stable",
    "autoPanel": False, "launchWithCodex": False,
}
assert native["result"]["backupPolicy"]["mode"] == "manual"
recorded_session = read(root / "retained-session-at-audit.json")
assert recorded_session["status"] == "running"
assert recorded_session["applicationPid"] == 38924 and recorded_session["applicationToken"]["elevated"] is False
assert recorded_session["applicationToken"]["elevationType"] == 3
monitor = read(root / "live-monitor-at-audit.json")
assert monitor["status"] == "observing-real-deadlines"
assert not monitor["fullCycleAccepted"] and monitor["failedObservations"] == 0
prefix = (root / "complete-live-observation-prefix.ndjson").read_bytes()
assert prefix.endswith(b"\n")
samples = [json.loads(line) for line in prefix.decode("utf-8-sig").splitlines() if line.strip()]
timing = read(root / "independent-cycle-prefix-recalculation.json")
assert timing["completeRecords"] == len(samples) and samples
assert hashlib.sha256(prefix).hexdigest() == timing["observationPrefixSha256"]
assert not timing["newMainCycleAccepted"] and not timing["realCycleCompleted"]
assert timing["elapsedFromInitialEnableToLastSampleMs"] < 86_400_000
for previous, current in zip(samples, samples[1:]):
    assert previous["native"]["result"]["atMs"] <= current["native"]["result"]["atMs"]
schedule = read(root / "schedule-at-audit.json")
assert hashlib.sha256((root / "schedule-at-audit.json").read_bytes()).hexdigest() == live["actualScheduleSha256"]
for item in samples:
    assert item["originalLifecycleLive"]
    assert item["native"]["sourceCommit"] == running and item["native"]["applicationPid"] == 38924
    assert item["schedule"] == schedule
for target in ["manager-stable", "panel-latest"]:
    assert timing["targets"][target]["intervalSeconds"] == 86400
    assert timing["targets"][target]["observedChangesFromBaseline"] == 0
    assert timing["targets"][target]["secondsUntilDueAtAudit"] > 0
    assert timing["targets"][target]["nextDueEpoch"] == schedule["entries"][target]["next_due"]
assert not (docs / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json").exists()
support = read(root / "supporting-evidence-hashes.json")["entries"]
for item in support:
    path = docs / item["path"]
    assert path.resolve().is_relative_to(docs.resolve()) and path.is_file()
    data = path.read_bytes()
    assert len(data) == item["bytes"] and hashlib.sha256(data).hexdigest() == item["sha256"], item["path"]
for row in audit["requirements"]:
    assert row["evidence"] and row["sourceApplicability"] and row["reason"]
    assert all((docs / name).is_file() for name in row["evidence"])
excluded = {"manifest.json", "archive-verification.json"}
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
assert all(path.suffix.lower() not in {".exe", ".msi", ".zip", ".dll"} for path in files)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read(root / "manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "scope": "original 14-requirement incomplete audit, preserved budgets, referenced evidence identities and old-source live cycle prefix",
    "overallGoalAchieved": False, "originalTaskPresent": False,
    "currentMainNativeAccepted": False, "pairedPerformanceAccepted": False,
    "realCyclesAccepted": False, "formalTaskReceipt": False, "formalCertificate": False,
    "originalPerformanceFailureCleared": False, "releaseAccepted": False,
    "filesVerified": len(entries), "supportedRequirements": 14,
})
print(json.dumps({"archiveIntegrity": "passed", "overallGoal": "not achieved", "requirements": 14,
                  "recordedCycleSamples": len(samples), "files": len(entries)}))

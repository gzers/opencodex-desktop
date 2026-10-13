"""Verify interrupted lifecycle and planned persistent restart without accepting the cycle."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
snapshot = root / "interruption-snapshot"
record = read(snapshot / "interruption-and-restart-disposition.json")
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
candidate = "be7395acfce0ec1e36b83300b576db408f8c35bb"
assert record["status"] == "old-owner-lifecycle-unavailable-before-original-deadlines"
assert record["originalSourceCommit"] == source and record["candidateSourceCommit"] == candidate
assert record["originalObservationsPreserved"] == 201 and record["deadlineStateByteUnchanged"]
assert record["candidateExecutableSha256"] == "3a26098e813ddd56070a0092cbcb640473cb73e626aee45198c4b335ec2d8bd1"
assert all(item["unchanged"] and item["oldSourceBlob"] == item["candidateBlob"] for item in record["scheduleSourceBlobWitnesses"])
assert len(record["scheduleSourceBlobWitnesses"]) == 3
for key in ["exactExitTimeKnown", "productCrashCauseProven", "staleRunningStatusIsLiveProof",
            "oldContinuousLifecycleCompleted", "ownerCleanupReceiptExists",
            "continuousOldProcessObservationMayBeClaimed", "candidateNativeBehaviorProvenByOldRecords",
            "rawNoDuplicateNetworkOrQueryCountsProven", "currentTurnForegroundInputIssued",
            "candidateStarted", "clocksOrTimersChanged", "originalTaskRecovered",
            "formalTaskReceipt", "mainIntegrated", "realFullCyclesAccepted",
            "pairedPerformanceAccepted", "fullNativeGateAccepted", "releaseAccepted"]:
    assert record[key] is False
actual = read(snapshot / "actual-processes-and-product-crash-events.json")
assert actual["desktopCount"] == 0 and not actual["desktopProcesses"]
assert {item["pid"] for item in actual["originalProcesses"]} == {37276, 24300, 3920}
assert all(not item["present"] for item in actual["originalProcesses"])
assert not actual["crashCauseProven"]
monitor = read(snapshot / "old-monitor-state.json")
session = read(snapshot / "old-current-session.json")
assert monitor["sourceCommit"] == session["sourceCommit"] == source
assert monitor["status"] == "observing-real-deadlines" and session["status"] == "running"
assert not monitor.get("finishedAtUtc") and not session.get("finishedAtUtc")
assert monitor["lastRetainedHandleCheckUtc"] == "2026-10-13T07:39:58.6399260Z"
assert monitor["lastNativeObservationUtc"] == "2026-10-13T07:37:48.380Z"
observations = [json.loads(line) for line in (snapshot / "complete-old-observations-prefix.ndjson").read_bytes().splitlines() if line.strip()]
assert len(observations) == monitor["observations"] == 201
assert all(item["originalLifecycleLive"] and item["native"]["sourceCommit"] == source for item in observations)
assert all(b["elapsedMonitorMs"] > a["elapsedMonitorMs"] for a, b in zip(observations, observations[1:]))
assert (snapshot / "actual-schedule.json").read_bytes() == (snapshot / "original-schedule-baseline.json").read_bytes()
schedule = read(snapshot / "actual-schedule.json")
for target, due in [("manager-stable", 1791903339), ("panel-latest", 1791903340)]:
    assert schedule["entries"][target]["next_due"] == due
    assert schedule["entries"][target]["next_due"] - schedule["entries"][target]["last_success"] == 86400
    assert schedule["entries"][target]["failures"] == 0
    assert record["originalPersistentDeadlinePairs"][target]["nextDueEpoch"] == due
plan = read(root / "restart_plan.json")
assert plan["status"] == "prepared_not_started" and plan["originalSourceCommit"] == source
assert plan["candidateSourceCommit"] == candidate and plan["oldLifecycleClaims"]["retainedPrefixSamples"] == 201
assert not plan["continuous24hProcessUptimeRequiredByThatClause"]
assert not plan["existingTimerRecordAloneSatisfiesRealDueBehavior"]
assert plan["originalTargets"]["manager-stable"]["nextDueEpoch"] == 1791903339
assert plan["originalTargets"]["panel-latest"]["nextDueEpoch"] == 1791903340
for key in ["candidateStarted", "foregroundInputIssued", "manualUpdateCheckIssued", "deadlineOrClockChanged",
            "newContinuousCycleStarted", "originalMonitorRestarted", "fullCyclesAccepted",
            "pairedPerformanceAccepted", "fullNativeGateAccepted", "mainIntegrated",
            "formalTaskReceipt", "releaseAccepted"]:
    assert plan[key] is False
method = (root / "methods/observe-restarted.js").read_text(encoding="utf-8")
assert "currentProcessElapsedMs: performance.now()" in method
assert "originalContinuousLifecycleProven: false" in method
assert not any(item in method for item in ["invoke('check_for_update'", "save_preferences",
                                         "update_schedule_plan", ".focus(", "loadRemoteLatest(", ".go("])
assert read(root / "observation-method-syntax.json")["nodeSyntaxCheck"] == "passed"
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in {"manifest.json", "archive-verification.json"})
assert all(path.suffix.lower() not in {".exe", ".dll", ".zip", ".msi", ".pyc"} for path in files)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, value):
    (root / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read(root / "manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "filesVerified": len(entries), "oldObservationsPreserved": 201,
    "scope": "stopped old owners, preserved original deadlines/data, unchanged schedule source, restart plan and read-only observer syntax",
    "exactExitTimeOrCauseProven": False, "oldContinuousLifecycleCompleted": False,
    "originalOwnerCleanupAccepted": False, "candidateStarted": False,
    "candidateNativeCycleAccepted": False, "realFullCyclesAccepted": False,
    "pairedPerformanceAccepted": False, "fullNativeGateAccepted": False,
    "mainIntegrated": False, "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "oldObservationsPreserved": 201,
                  "originalDeadlineStatePreserved": True, "candidateStarted": False, "fullCyclesAccepted": False}))

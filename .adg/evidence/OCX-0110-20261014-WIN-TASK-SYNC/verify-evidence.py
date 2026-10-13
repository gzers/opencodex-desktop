"""Verify original TASK arrival and truthful pending goal edit without claiming async success."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
task = read("task-windows-readmission.json")
provenance = read("mac-original-task-provenance.json")
assert task["providerValidation"] == "passed" and task["originalTaskRecoveredOnWindows"]
assert task["taskRawSha256"] == provenance["raw_sha256"] == \
    "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert task["taskBytes"] == provenance["raw_bytes"] == 3878 and task["taskRevision"] == 2
assert task["taskState"] == "in_progress" and task["taskResult"] == "pending"
assert task["taskContentDigest"] == provenance["task_content_digest"] == \
    "23908e0d312409d81886b6f0f570c39581d8f9fc413ac67b0d043102eecfc29c"
assert task["taskSummaryDigest"] == provenance["task_summary_digest"]
assert task["baselineDigest"] == provenance["baseline_digest"]
assert len(task["requiredChecks"]) == 8 and task["requiredChecks"] == provenance["required_checks"]
assert task["acceptanceBindings"] == 9 and len(task["matchedExistingReceipts"]) == 2
assert all(item["taskContentSummaryAndBaselineMatch"] for item in task["matchedExistingReceipts"])
assert task["sameBytesAsTrackedMacSyncCommit"] and task["sourceDocsCommit"] == "ae6cd0801965236093c9b42c4c24a07365405ac1"
for key in ["taskReconstructedOrModified", "existingPerformanceFailedCleared", "taskStatusPromoted",
            "formalNewReceiptOrCertificateCreated", "taskClosed"]:
    assert task[key] is False
attempt = read("goal-user-edit-result.json")
assert attempt["threadId"] == "01a12e0e-c38f-7df3-b42f-6e80f629545e"
assert attempt["status"] == "error-requires-readback-before-retry" and attempt["goalMutationIssued"]
assert "already has an active writer" in attempt["error"]
assert attempt["before"]["status"] == "paused" and attempt["before"]["createdAt"] == 1791774103
assert attempt["before"]["tokensUsed"] >= 18453728
assert not attempt["newGoalCreated"] and not attempt["goalMarkedComplete"] and not attempt["tokenBudgetChanged"]
schema = read("official-generated-ThreadGoalSetParams.json")
assert {"threadId", "objective", "status", "origin"} <= set(schema["properties"])
assert "user" in schema["definitions"]["ThreadGoalMutationOrigin"]["enum"]
objective = (root / "goal-objective.txt").read_text(encoding="utf-8")
for value in ["原契约", "原预算", "完整版本目标", "八项required_checks", "100/125/150", "30分钟", "0.1.9",
              "全局只运行一个", "Mac依赖", "不降低门禁"]:
    assert value in objective
wait = read("goal-writer-wait-start-snapshot.json")
assert wait["status"] == "waiting_for_current_writer" and wait["helperPid"] == 42320
assert wait["requestedObjective"] == objective.strip() and wait["maximumElapsedSeconds"] == 600
assert wait["retryIntervalSeconds"] == 45 and not wait["goalMutationSucceeded"]
assert wait["attempts"] and all("already has an active writer" in item["error"]["message"] for item in wait["attempts"])
for key in ["newGoalCreated", "goalMarkedComplete", "tokenBudgetChanged", "writerLockOverridden",
            "threadOrDesktopRestarted", "nativeAppStarted", "foregroundInputIssued"]:
    assert wait[key] is False
live = read("goal-writer-wait-live-handle.json")
assert live["helperPid"] == 42320 and live["actualLiveHandle"] and live["execSession"] == 47019
assert live["maximumElapsedSeconds"] == 600 and live["retrySeconds"] == 45
assert not live["nativeInstanceStarted"] and not live["writerLockOverrideIssued"]
followup = read("caption-hit-contract-followup.json")
assert followup["sourceCommit"] == "be7395acfce0ec1e36b83300b576db408f8c35bb"
assert followup["existingMinHitAllowedValues"] == [2, 8] and followup["currentReadonlyObservedHits"] == [2, 9, 20]
assert followup["normalReadOnlyHitPatternWithinExistingContract"]
assert followup["actualMinimizeClickRequiredAndNotExecutedThisTurn"]
assert not followup["testContractOrAssertionModified"] and not followup["readonlyMin2IsProductDefect"]
method = (root / "apply-goal-after-current-writer.py").read_text(encoding="utf-8")
assert 'time.sleep(45)' in method and 'time.monotonic() - clock < 600' in method
assert '"origin": "user"' in method and 'goal["status"] != "paused"' in method
assert '"thread/goal/set"' in method and '"thread/goal/get"' in method
assert "writerLockOverridden" in method and "tokenBudgetChanged" in method
assert not any(value in method for value in ["sqlite3", "thread/goal/clear", "thread/stop", "thread/resume", "complete\"}"])
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
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "filesVerified": len(entries),
    "scope": "original TASK byte/provider readmission, existing caption query contract, user-authorized goal edit request and bounded writer wait snapshot",
    "originalTaskRecoveredOnWindows": True, "taskRequiredChecksChanged": False,
    "windowsGoalEditRequested": True, "goalEditSucceededInFrozenSnapshot": False,
    "goalWriterWaitLiveAtSnapshot": True, "asyncGoalOutcomeRequiresReadback": True,
    "writerLockOverridden": False, "newGoalOrUsageAccountingReset": False,
    "readonlyMin2ClassifiedAsProductDefect": False, "captionActualClickAccepted": False,
    "existingPerformanceFailedCleared": False, "fullNativeGateAccepted": False,
    "formalTaskReceiptCreated": False, "taskClosed": False, "mainIntegrated": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "originalTaskRecoveredOnWindows": True,
                  "goalEditSucceededAtSnapshot": False, "goalWriterWaitVerifiedLive": True,
                  "fullNativeAccepted": False}))

"""Verify the goal remains paused without replacing its contract or accounting."""
import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
resume = read("status-only-resume.json")
assert resume["mutationParameters"] == {
    "threadId": "01a12e0e-c38f-7df3-b42f-6e80f629545e", "status": "active", "origin": "user",
}
assert resume["status"] == "not-resumed-readback-verified" and not resume["resumeVerified"]
assert resume["originalAccountingPreserved"] and resume["before"] == resume["readback"]
assert resume["readback"]["status"] == "paused" and resume["readback"]["createdAt"] == 1791774103
assert resume["readback"]["tokensUsed"] == 18453728 and resume["readback"]["timeUsedSeconds"] == 63871
assert "already has an active writer" in resume["setResponse"]["error"]["message"]
for key in ["objectiveReplacementIssued", "budgetChangeIssued", "writerLockOverrideIssued",
            "retryLoopStarted", "goalClearedOrMarkedComplete", "nativeInputIssued"]:
    assert resume[key] is False
prior = read("prior-waiter-final-status.json")
assert prior["status"] == "bounded-writer-wait-ended-without-goal-change"
assert prior["maximumElapsedSeconds"] == 600 and 630 <= prior["elapsedSeconds"] < 631
assert not prior["goalMutationSucceeded"] and prior["finishedAtUtc"]
assert prior["attempts"] and all("active writer" in item["error"]["message"] for item in prior["attempts"])
findings = read("protocol-findings.json")
assert findings["priorDeferredHelperEnded"] and findings["technicalHostControlStillRequired"]
assert findings["objectiveReplacementResetsUsageAccounting"] and findings["statusOnlyUpdatesRetainExistingObjectiveAndUsage"]
assert not findings["userAuthorizationMissing"] and not findings["newDeferredRetryLoopStarted"]
assert not findings["preparedInstructionsAppliedToAppGoal"] and not findings["versionTaskClosed"]
text = (root / "windows-goal-instructions.txt").read_text(encoding="utf-8")
for value in ["原契约", "原预算", "Mac", "固定", "前台", "0.1.9", "30分钟", "不降低门禁", "不自动合并"]:
    # Fixed-SHA acceptance is stated in the text as "每个真实 SHA".
    assert value in text or value == "固定" and "每个真实 SHA" in text
files = sorted(p for p in root.rglob("*") if p.is_file() and p.name not in {"manifest.json", "archive-verification.json"})
manifest = [{"path": p.relative_to(root).as_posix(), "bytes": p.stat().st_size,
             "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in files]
if "--build" in sys.argv:
    assert not (root / "manifest.json").exists()
    (root / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
else:
    assert read("manifest.json") == manifest, "Frozen evidence bytes changed"
report = {"archiveIntegrity": "passed", "files": len(files), "goalStatus": "paused",
          "goalResumed": False, "originalAccountingPreserved": True,
          "preparedInstructionsApplied": False, "newDeferredRetryLoopStarted": False}
if "--build" in sys.argv:
    (root / "archive-verification.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))

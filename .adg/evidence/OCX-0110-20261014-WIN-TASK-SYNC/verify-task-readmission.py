"""Validate the synchronized original TASK through the installed governance provider."""
import hashlib
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
root = Path(__file__).resolve().parent
workspace = root.parents[2]
skill = Path(r"C:\Users\15119\.codex\skills\project-governance")
sys.path.insert(0, str(skill / "scripts/lib"))
from runtime.kernel import calculate_content_digest, calculate_summary_digest, validate_object

task_path = workspace / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
raw = task_path.read_bytes()
task = json.loads(raw.decode("utf-8"))
assert len(raw) == 3878
assert hashlib.sha256(raw).hexdigest() == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
validate_object(task)
assert calculate_content_digest(task) == task["content_digest"] == \
    "23908e0d312409d81886b6f0f570c39581d8f9fc413ac67b0d043102eecfc29c"
assert calculate_summary_digest(task) == task["summary_digest"] == \
    "299e06b9c8cbfae8af09310811a42a90aaab151617f929e58199754a479a33b1"
assert task["state"] == "in_progress" and task["result"] == "pending" and task["revision"] == 2
assert task["certificate_digest"] is None and task["imported_receipt_digest"] is None
assert task["required_checks"] == [
    "review:backup-tree-0110", "review:event-coverage-0110", "review:installer-0110",
    "review:native-0110", "review:scheduler-performance-0110", "review:scope-0110",
    "review:storage-0110", "review:updates-0110",
]
receipts = []
for name in ["RCP-OPENCODEX-DESKTOP-182-PERF-20261012.json", "RCP-OPENCODEX-DESKTOP-182-SCHEDULE-ERROR-BACKOFF-20261012.json"]:
    path = workspace / ".adg/work/objects/rcp" / name
    receipt = json.loads(path.read_text(encoding="utf-8-sig"))
    validate_object(receipt)
    assert receipt["task_digest"] == task["content_digest"]
    assert receipt["task_summary_digest"] == task["summary_digest"]
    assert receipt["baseline_digest"] == task["baseline_digest"]
    receipts.append({"path": str(path.relative_to(workspace)), "result": receipt["result"], "checks": receipt["checks"],
                     "taskContentSummaryAndBaselineMatch": True})
task_git = subprocess.check_output(["git", "-C", str(workspace), "show",
    "ae6cd0801965236093c9b42c4c24a07365405ac1:.adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"])
assert task_git == raw
assert task_path.read_bytes() == raw
record = {
    "atUtc": datetime.now(timezone.utc).isoformat(),
    "sourceDocsCommit": "ae6cd0801965236093c9b42c4c24a07365405ac1",
    "taskPath": str(task_path.relative_to(workspace)), "taskRawSha256": hashlib.sha256(raw).hexdigest(),
    "taskBytes": len(raw), "taskRevision": 2, "taskState": task["state"], "taskResult": task["result"],
    "taskContentDigest": task["content_digest"], "taskSummaryDigest": task["summary_digest"],
    "baselineDigest": task["baseline_digest"], "requiredChecks": task["required_checks"],
    "acceptanceBindings": len(task["acceptance_bindings"]),
    "providerValidation": "passed", "providerRoot": str(skill),
    "providerFunction": "scripts/lib/runtime/kernel.py::validate_object",
    "providerSourceSha256": hashlib.sha256((skill / "scripts/lib/runtime/kernel.py").read_bytes()).hexdigest(),
    "sameBytesAsTrackedMacSyncCommit": True, "matchedExistingReceipts": receipts,
    "originalTaskRecoveredOnWindows": True, "taskReconstructedOrModified": False,
    "existingPerformanceFailedCleared": False, "taskStatusPromoted": False,
    "formalNewReceiptOrCertificateCreated": False, "taskClosed": False,
    "firstProviderImportWrapperError": "scripts path/lib.runtime import did not satisfy provider's runtime absolute imports; corrected to installed scripts/lib import root, no object or Skill source changed",
    "scope": "raw original TASK byte sync plus provider and prior receipt identity verification; not full acceptance",
}
(root / "task-windows-readmission.json").write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"originalTaskRecoveredOnWindows": True, "providerValidation": "passed",
                  "requiredChecks": len(task["required_checks"]), "state": task["state"],
                  "performanceFailedCleared": False, "taskClosed": False}))

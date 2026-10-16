"""Seal the offline role-timeline receipt without changing TASK or prior evidence."""
import hashlib
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, "C:/Users/15119/.codex/skills/project-governance/scripts/lib")
from runtime.kernel import seal_object, validate_object

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
TASK = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
IMP = PROJECT / "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
RECEIPT = PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-RSS-ROLE-TIMELINE-20261016.json"
assert not RECEIPT.exists() and not (ROOT / "manifest.json").exists()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(path, value):
    with path.open("xb") as out:
        out.write((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


assert sha(TASK.read_bytes()) == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
task = json.loads(TASK.read_text(encoding="utf-8-sig"))
validate_object(task)
assert task["revision"] == 2 and task["state"] == "in_progress" and task["result"] == "pending"
verification = subprocess.run([sys.executable, str(ROOT / "verify.py")], cwd=PROJECT,
                              capture_output=True, check=True)
assert not verification.stderr
write(ROOT / "verification.json", json.loads(verification.stdout))
relative = ROOT.relative_to(PROJECT).as_posix()
prior = ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES"
receipt = seal_object({
    "kind": "RCP", "id": RECEIPT.stem, "task_id": task["id"],
    "task_digest": task["content_digest"], "task_summary_digest": task["summary_digest"],
    "baseline_digest": task["baseline_digest"], "result": "failed",
    "changed_paths": [relative, RECEIPT.relative_to(PROJECT).as_posix(), IMP.relative_to(PROJECT).as_posix()],
    "checks": [
        {"name": "local:windows-rss-role-timeline", "status": "passed",
         "evidence": relative + "/verification.json"},
        {"name": "local:windows-b4-rss-budget", "status": "failed",
         "evidence": prior + "/original/counter-role-analysis.json"},
        {"name": "local:windows-test-profile-restoration", "status": "failed",
         "evidence": prior + "/original/cleanup-final-readback.json"},
        {"name": "review:native-0110", "status": "failed",
         "evidence": prior + "/original/candidate0110-exit.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": prior + "/original/counter-role-analysis.json"},
    ],
    "acceptance_results": [],
    "fact_impacts": [{
        "target": "imp-20", "path": IMP.relative_to(PROJECT).as_posix(),
        "after_digest": sha(IMP.read_bytes()),
        "summary": "GPU增量在计数初段已存在且末段未持续上升；仅收窄为启动/资源构成方向，不归因具体分配，RSS门禁继续失败。",
    }],
    "completed_at": datetime.now(timezone.utc).isoformat(),
    "summary": "离线重算冻结597+597样本角色时间线，GPU均值差+24.086376MiB、初段+24.895378MiB、末段+21.276172MiB；不解除RSS或完整门禁。",
    "revision": 1,
})
write(RECEIPT, receipt)
files = []
for path in sorted(ROOT.rglob("*")):
    if path.is_file() and path.name != "manifest.json":
        data = path.read_bytes()
        files.append({"path": path.relative_to(ROOT).as_posix(), "bytes": len(data), "sha256": sha(data)})
write(ROOT / "manifest.json", {"files": files})
subprocess.run([sys.executable, str(ROOT / "verify.py")], cwd=PROJECT, check=True)
assert sha(TASK.read_bytes()) == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
print(json.dumps({"receipt": receipt["id"], "result": receipt["result"],
                  "manifestFiles": len(files), "taskUnchanged": True}))

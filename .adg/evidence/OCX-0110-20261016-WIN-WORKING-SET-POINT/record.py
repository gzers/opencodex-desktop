"""Seal this diagnostic receipt without changing TASK or old failure evidence."""
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
TASK_PATH = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
IMP_PATH = PROJECT / "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"
RCP_PATH = PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-WORKING-SET-POINT-20261016.json"
assert not RCP_PATH.exists() and not (ROOT / "manifest.json").exists()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(path, value):
    with path.open("xb") as out:
        out.write((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


assert sha(TASK_PATH.read_bytes()) == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
task = json.loads(TASK_PATH.read_text(encoding="utf-8-sig"))
validate_object(task)
assert task["revision"] == 2 and task["state"] == "in_progress" and task["result"] == "pending"
assert len(task["required_checks"]) == 8
verification = subprocess.run([sys.executable, str(ROOT / "verify.py")], cwd=PROJECT,
                              capture_output=True, check=True)
assert not verification.stderr
write(ROOT / "verification.json", json.loads(verification.stdout))
relative = ROOT.relative_to(PROJECT).as_posix()
prior = ".adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES"
receipt = seal_object({
    "kind": "RCP", "id": RCP_PATH.stem, "task_id": task["id"],
    "task_digest": task["content_digest"], "task_summary_digest": task["summary_digest"],
    "baseline_digest": task["baseline_digest"], "result": "failed",
    "changed_paths": [relative, RCP_PATH.relative_to(PROJECT).as_posix(),
                      IMP_PATH.relative_to(PROJECT).as_posix()],
    "checks": [
        {"name": "local:windows-working-set-controlled-method", "status": "passed",
         "evidence": relative + "/original/method-admission.json"},
        {"name": "local:windows-working-set-same-gpu-generation-point", "status": "passed",
         "evidence": relative + "/verification.json"},
        {"name": "local:windows-test-profile-restoration", "status": "failed",
         "evidence": relative + "/original/cleanup-followup.original.json"},
        {"name": "review:native-0110", "status": "failed",
         "evidence": prior + "/original/candidate0110-exit.json"},
        {"name": "review:scheduler-performance-0110", "status": "failed",
         "evidence": prior + "/original/counter-role-analysis.json"},
    ],
    "acceptance_results": [],
    "fact_impacts": [{
        "target": "imp-20", "path": IMP_PATH.relative_to(PROJECT).as_posix(),
        "after_digest": sha(IMP_PATH.read_bytes()),
        "summary": "34项方法及同代际GPU晚期61,230页分类通过；不回填原计数或归因RSS差值，原RSS/退出失败及配置待还原保持。",
    }],
    "completed_at": datetime.now(timezone.utc).isoformat(),
    "summary": "仅新增同代际GPU非原子页分类与同源Windows方法CI证据；原RSS+20.040155MiB超预算、退出超时和配置未还原保持，完整门禁不通过。",
    "revision": 1,
})
write(RCP_PATH, receipt)
items = []
for path in sorted(ROOT.rglob("*")):
    if path.is_file() and path.name != "manifest.json":
        data = path.read_bytes()
        items.append({"path": path.relative_to(ROOT).as_posix(),
                      "bytes": len(data), "sha256": sha(data)})
write(ROOT / "manifest.json", {"files": items})
subprocess.run([sys.executable, str(ROOT / "verify.py")], cwd=PROJECT, check=True)
assert sha(TASK_PATH.read_bytes()) == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
print(json.dumps({"receipt": receipt["id"], "result": receipt["result"],
                  "filesInManifest": len(items), "taskUnchanged": True}))

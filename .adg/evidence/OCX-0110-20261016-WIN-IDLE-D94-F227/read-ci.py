"""Read the existing collector run only; keep later observations separate."""
import importlib.util
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
assert not (ROOT / "manifest.json").exists()
spec = importlib.util.spec_from_file_location(
    "idle_f227_reader",
    ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py",
)
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, run = api.request("actions/runs/38834762492", token)
_, jobs = api.request("actions/runs/38834762492/jobs?per_page=100", token)
assert run["head_sha"] == "f2271f361b9da562b62c71afcdbeb15e61d183e2"
assert run["run_attempt"] == 1 and run["event"] == "push"
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
(ROOT / f"ci-{stamp}.original.json").write_text(
    json.dumps({"run": run, "jobs": jobs}, indent=2) + "\n", encoding="utf-8")
summary = {
    "observedAtUtc": datetime.now(timezone.utc).isoformat(),
    "runId": run["id"], "source": run["head_sha"],
    "status": run["status"], "conclusion": run["conclusion"],
    "jobs": [{k: j[k] for k in ("id", "name", "status", "conclusion")} for j in jobs["jobs"]],
    "rerunRequested": False,
}
(ROOT / "ci-status.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))

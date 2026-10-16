"""Follow the existing admitted candidate; never start or rerun a workflow."""
import importlib.util
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
handle = json.loads((ROOT / "candidate-handle.json").read_text(encoding="utf-8"))
spec = importlib.util.spec_from_file_location(
    "candidate_gh_reader", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
identifier = handle["runId"]
_, run = api.request(f"actions/runs/{identifier}", token)
_, jobs = api.request(f"actions/runs/{identifier}/jobs?per_page=100", token)
assert run["head_sha"] == handle["source"] and run["run_attempt"] == 1
summary = {"atUtc": datetime.now(timezone.utc).isoformat(), "source": handle["source"],
           "runId": identifier, "status": run["status"], "conclusion": run["conclusion"],
           "jobs": [{key: j[key] for key in ("name", "status", "conclusion")} for j in jobs["jobs"]],
           "workflowRerunRequested": False}
(ROOT / "candidate-progress.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
if run["status"] == "completed":
    _, artifacts = api.request(f"actions/runs/{identifier}/artifacts?per_page=100", token)
    terminal = ROOT / "candidate-terminal.json"
    if not terminal.exists():
        terminal.write_text(json.dumps({"run": run, "jobs": jobs, "artifacts": artifacts},
                                       indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))

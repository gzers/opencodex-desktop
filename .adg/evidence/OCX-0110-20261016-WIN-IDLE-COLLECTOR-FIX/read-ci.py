"""Read the one exact collector source run; never launch a product or retry CI."""
import importlib.util
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
assert not (ROOT / "manifest.json").exists(), "Frozen checkpoint; write later CI observations to a new evidence directory"
SOURCE = "f2271f361b9da562b62c71afcdbeb15e61d183e2"
spec = importlib.util.spec_from_file_location(
    "idle_collector_ci_reader", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, listing = api.request(f"actions/workflows/ci.yml/runs?head_sha={SOURCE}&per_page=20", token)
runs = [r for r in listing["workflow_runs"] if r["head_sha"] == SOURCE]
assert len(runs) == 1 and runs[0]["run_attempt"] == 1 and runs[0]["event"] == "push"
run = runs[0]
_, jobs = api.request(f"actions/runs/{run['id']}/jobs?per_page=100", token)
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
(ROOT / f"ci-{stamp}.original.json").write_text(
    json.dumps({"run": run, "jobs": jobs}, indent=2) + "\n", encoding="utf-8")
summary = {"atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE,
           "runId": run["id"], "status": run["status"], "conclusion": run["conclusion"],
           "workflowAccepted": True,
           "jobs": [{k: j[k] for k in ("name", "status", "conclusion")} for j in jobs["jobs"]],
           "workflowRerunRequested": False, "productCandidateRebuilt": False,
           "fullPerformanceGatePassed": False}
(ROOT / "ci-progress.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))

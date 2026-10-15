"""Read the exact new source's CI and PR status without rerunning any workflow."""
import importlib.util
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
SOURCE = "d94c215d869c0560717d79dc02881ce8858afe88"
spec = importlib.util.spec_from_file_location(
    "bounded_gh_reader", ROOT.parent / "OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/github-reader.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, runs = api.request(f"actions/workflows/ci.yml/runs?head_sha={SOURCE}&per_page=20", token)
_, pr = api.request("pulls/5", token)
assert pr["head"]["sha"] == SOURCE and pr["state"] == "open" and pr["draft"]
assert pr["merged_at"] is None and pr["auto_merge"] is None
summary = []
for run in runs["workflow_runs"]:
    assert run["head_sha"] == SOURCE and run["run_attempt"] == 1
    if run["status"] == "completed":
        path = ROOT / f"ci-{run['id']}-terminal.original.json"
        if not path.exists():
            path.write_text(json.dumps(run, indent=2) + "\n", encoding="utf-8")
    summary.append({key: run[key] for key in ("id", "event", "status", "conclusion", "head_sha", "run_attempt")})
stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
snapshot = {"atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE, "runs": summary,
            "pr": {"number": 5, "head": pr["head"]["sha"], "draft": pr["draft"],
                   "merged": False, "autoMerge": False}, "workflowRerunRequested": False}
(ROOT / f"ci-{stamp}.original.json").write_text(
    json.dumps({"runs": runs, "pr": pr}, indent=2) + "\n", encoding="utf-8")
(ROOT / "ci-progress.json").write_text(json.dumps(snapshot, indent=2) + "\n", encoding="utf-8")
if len(summary) == 2 and {r["event"] for r in summary} == {"push", "pull_request"} and all(r["status"] == "completed" for r in summary):
    (ROOT / "ci-terminal.json").write_text(json.dumps(snapshot, indent=2) + "\n", encoding="utf-8")
print(json.dumps(snapshot))

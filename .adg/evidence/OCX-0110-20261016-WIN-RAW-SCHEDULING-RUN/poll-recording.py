"""Poll only the accepted recording handle; never dispatch from a read."""
import importlib.util
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent
handle = json.loads((ROOT / "remote-handle.json").read_text(encoding="utf-8"))
assert handle["runId"] == 38876971001 and handle["source"] == "54927d2cb184a38d5dd2e1f7c00ed4697b2e0a56"
spec = importlib.util.spec_from_file_location("recording_proxy_reader", ROOT / "github-reader-via-proxy.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
_, run = api.request(f"actions/runs/{handle['runId']}", token)
_, jobs = api.request(f"actions/runs/{handle['runId']}/jobs?per_page=100", token)
assert run["id"] == handle["runId"] and run["head_sha"] == handle["source"] and run["run_attempt"] == 1
assert run["event"] == "workflow_dispatch"
summary = {"source": handle["source"], "runId": run["id"], "status": run["status"], "conclusion": run["conclusion"],
    "jobs": [{"id": j["id"], "name": j["name"], "status": j["status"], "conclusion": j["conclusion"],
              "activeSteps": [s["name"] for s in j.get("steps", []) if s["status"] == "in_progress"]}
             for j in jobs["jobs"]]}
latest = ROOT / "recording-status.json"
previous = json.loads(latest.read_text(encoding="utf-8")) if latest.exists() else None
if summary != previous:
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    with (ROOT / f"recording-state-{stamp}.original.json").open("xb") as output:
        output.write((json.dumps({"run": run, "jobs": jobs}, indent=2) + "\n").encode("utf-8"))
    latest.write_bytes((json.dumps(summary, indent=2) + "\n").encode("utf-8"))
print(json.dumps({"observedAtUtc": datetime.now(timezone.utc).isoformat(), **summary}))

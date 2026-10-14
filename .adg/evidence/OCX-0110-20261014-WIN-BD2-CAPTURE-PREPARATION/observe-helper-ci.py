"""Observe automatic helper-branch CI only; never rerun or dispatch a job."""
import json
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
source = "5fa36a60a6867124cf13f27e1e1856ae495ef29b"
url = "https://api.github.com/repos/gzers/opencodex-desktop/"
def get(path):
    with urllib.request.urlopen(urllib.request.Request(
        url + path, headers={"Accept": "application/vnd.github+json",
                            "User-Agent": "ocx-bd2-helper-readonly"}), timeout=25) as response:
        return json.load(response)
record = {"atUtc": datetime.now(timezone.utc).isoformat(), "source": source,
          "mutationsIssued": False, "collectorRecordExecuted": False}
try:
    runs = get(f"actions/runs?head_sha={source}&per_page=30")
    record["runs"] = []
    for run in runs["workflow_runs"]:
        assert run["head_sha"] == source
        record["runs"].append({"run": run, "jobs": get(f"actions/runs/{run['id']}/jobs?per_page=100")})
    record["status"] = "observed"
except Exception as error:
    record.update(status="observation-unavailable", errorType=type(error).__name__)
(root / "helper-ci-observation.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"status": record["status"], "runs": [
    {"id": item["run"]["id"], "status": item["run"]["status"], "conclusion": item["run"]["conclusion"]}
    for item in record.get("runs", [])], "recordingExecuted": False}))

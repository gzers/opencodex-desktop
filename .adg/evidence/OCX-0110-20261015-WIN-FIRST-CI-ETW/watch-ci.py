"""Observe the same naturally triggered CI handles; never rerun or dispatch here."""
from datetime import datetime, timezone
import importlib.util
import json
from pathlib import Path
import sys
import time

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("github_read", root / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
token = api.credential()
duration = int(sys.argv[1]) if len(sys.argv) > 1 else 40
assert 1 <= duration <= 900
deadline = time.monotonic() + duration
while True:
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    observation = {"atUtc": datetime.now(timezone.utc).isoformat(), "source": api.SOURCE,
                   "readOnly": True, "runs": [], "errors": []}
    try:
        _, listing = api.request("actions/runs?head_sha=" + api.SOURCE + "&per_page=30", token)
        runs = [row for row in listing["workflow_runs"] if row["path"] == ".github/workflows/ci.yml"
                and row["event"] in {"push", "pull_request"}]
        for run in runs:
            _, jobs = api.request("actions/runs/" + str(run["id"]) + "/jobs?per_page=100", token)
            observation["runs"].append({"run": run, "jobs": jobs})
    except Exception as error:
        observation["errors"].append({"type": type(error).__name__, "httpStatus": getattr(error, "code", None)})
    (root / ("ci-observation-" + stamp + ".json")).write_text(
        json.dumps(observation, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"atUtc": observation["atUtc"], "runs": [
        {"id": row["run"]["id"], "event": row["run"]["event"], "status": row["run"]["status"],
         "conclusion": row["run"]["conclusion"], "jobs": [
             {"name": job["name"], "status": job["status"], "conclusion": job["conclusion"]}
             for job in row["jobs"]["jobs"]]} for row in observation["runs"]],
        "errors": observation["errors"]}), flush=True)
    complete = (not observation["errors"] and len(observation["runs"]) == 2
                and {row["run"]["event"] for row in observation["runs"]} == {"push", "pull_request"}
                and all(row["run"]["status"] == "completed" for row in observation["runs"]))
    if complete or time.monotonic() >= deadline:
        print(json.dumps({"observationEnded": True, "bothRemoteRunsTerminal": complete,
                          "remoteJobsStoppedByObserver": False}), flush=True)
        break
    time.sleep(min(30, max(0, deadline - time.monotonic())))

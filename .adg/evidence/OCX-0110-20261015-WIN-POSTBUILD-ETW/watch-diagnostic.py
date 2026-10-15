"""Retain one dispatched run handle and observe it without ever resubmitting."""
from datetime import datetime, timedelta, timezone
import importlib.util
import json
from pathlib import Path
import sys
import time

root = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("github_read", root / "github-read.py")
api = importlib.util.module_from_spec(spec)
spec.loader.exec_module(api)
attempt = json.loads((root / "dispatch-attempt.json").read_text())
threshold = datetime.fromisoformat(attempt["atUtc"]) - timedelta(seconds=2)
handle_path = root / "diagnostic-handle.json"
run_id = json.loads(handle_path.read_text())["runId"] if handle_path.exists() else None
token = api.credential()
duration = int(sys.argv[1]) if len(sys.argv) > 1 else 600
assert 1 <= duration <= 900
deadline = time.monotonic() + duration
while True:
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    snapshot = {"atUtc": datetime.now(timezone.utc).isoformat(), "readOnly": True, "runId": run_id}
    terminal = False
    try:
        if run_id is None:
            _, listing = api.request("actions/runs?head_sha=" + api.SOURCE + "&per_page=30", token)
            matches = [run for run in listing["workflow_runs"] if run["event"] == "workflow_dispatch"
                       and run["path"] == ".github/workflows/windows-candidate.yml"
                       and run["head_branch"] == "feature/0.1.10-windows-query-etw"
                       and datetime.fromisoformat(run["created_at"].replace("Z", "+00:00")) >= threshold]
            assert len(matches) <= 1, "More than one dispatch candidate; inspect instead of guessing."
            if matches:
                run_id = matches[0]["id"]
                assert not handle_path.exists()
                handle_path.write_text(json.dumps({"runId": run_id, "source": api.SOURCE,
                                                   "request": "dispatch-attempt.json"}, indent=2) + "\n",
                                       encoding="utf-8")
        if run_id is not None:
            _, run = api.request("actions/runs/" + str(run_id), token)
            _, jobs = api.request("actions/runs/" + str(run_id) + "/jobs?per_page=100", token)
            assert run["head_sha"] == api.SOURCE and run["event"] == "workflow_dispatch"
            assert run["run_attempt"] == 1
            snapshot.update(runId=run_id, run=run, jobs=jobs)
            terminal = run["status"] == "completed"
    except Exception as error:
        snapshot["error"] = {"type": type(error).__name__, "httpStatus": getattr(error, "code", None)}
    (root / ("diagnostic-observation-" + stamp + ".json")).write_text(
        json.dumps(snapshot, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({
        "atUtc": snapshot["atUtc"], "runId": run_id,
        "status": snapshot.get("run", {}).get("status"),
        "conclusion": snapshot.get("run", {}).get("conclusion"),
        "runningSteps": [step["name"] for job in snapshot.get("jobs", {}).get("jobs", [])
                         for step in job["steps"] if step["status"] == "in_progress"],
        "error": snapshot.get("error"),
    }), flush=True)
    if terminal or time.monotonic() >= deadline:
        result = {"atUtc": datetime.now(timezone.utc).isoformat(), "runId": run_id,
                  "remoteRunTerminal": terminal, "remoteRunStoppedOrRerunByObserver": False}
        (root / ("diagnostic-watch-result-" + stamp + ".json")).write_text(
            json.dumps(result, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(result), flush=True)
        break
    time.sleep(min(30, max(0, deadline - time.monotonic())))

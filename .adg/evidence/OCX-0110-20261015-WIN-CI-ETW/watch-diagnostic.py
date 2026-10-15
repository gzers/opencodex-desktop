"""Observe the one dispatched diagnostic run, never resubmit it."""
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
result_path = root / "diagnostic-watch-result.json"
assert not result_path.exists()
dispatch = json.loads((root / "diagnostic-dispatch-result.json").read_text(encoding="utf-8"))
assert dispatch["httpStatus"] == "204" and not dispatch["automaticRetryAllowed"]
started = time.monotonic()
last = None
observations = 0
run = None
while time.monotonic() - started < 900:
    response = subprocess.run(
        [sys.executable, str(root / "github-read.py"), "38662786466"],
        capture_output=True, text=True, encoding="utf-8", timeout=60)
    observations += 1
    snapshot = json.loads(max(root.glob("observation-*.json")).read_text(encoding="utf-8"))
    read_run = snapshot["reads"].get("run-38662786466", {})
    if read_run.get("id") == 38662786466:
        assert read_run["head_sha"] == "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
        assert read_run["event"] == "workflow_dispatch" and read_run["run_attempt"] == 1
        run = read_run
        jobs = snapshot["reads"].get("jobs-38662786466", {})
        if not jobs.get("readError"):
            package = next(row for row in jobs["jobs"] if row["name"] == "package")
            assert package["conclusion"] == "skipped", "Diagnostic must not package a candidate."
        if run["status"] == "completed" and not jobs.get("readError"):
            (root / "diagnostic-terminal.json").write_text(
                json.dumps({"atUtc": snapshot["atUtc"], "run": run, "jobs": jobs}, indent=2) + "\n", encoding="utf-8")
            print(response.stdout.strip(), flush=True)
            break
    if response.stdout != last:
        print(response.stdout.strip(), flush=True)
        last = response.stdout
    time.sleep(20)
report = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "runId": 38662786466, "observations": observations,
    "elapsedSeconds": time.monotonic() - started,
    "remoteStatus": None if run is None else run["status"],
    "remoteConclusion": None if run is None else run["conclusion"],
    "remoteTerminalObserved": bool(run and run["status"] == "completed"),
    "resubmissions": 0, "localDesktopStarted": False,
}
result_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report), flush=True)

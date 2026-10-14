"""Bounded read-only wait for two already-live CI runs; no tests or recording."""
import json
import os
import subprocess
import time
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
source = "5fa36a60a6867124cf13f27e1e1856ae495ef29b"
identifiers = [38555890377, 38556292712]
base = "https://api.github.com/repos/gzers/opencodex-desktop/"
record = {
    "startedAtUtc": datetime.now(timezone.utc).isoformat(), "helperPid": os.getpid(),
    "source": source, "runIds": identifiers, "maximumWaitSeconds": 600,
    "intervalSeconds": 30, "status": "observing-existing-runs",
    "manuallyDispatchedOrRerun": False, "testsOrEtwStarted": False,
    "recordingApprovalInferredFromGoalContinuation": False, "observations": [],
}
out = root / "ci-wait-status.json"
assert not out.exists()
clock = time.monotonic()


def save():
    pending = root / "ci-wait-status.pending.json"
    pending.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    pending.replace(out)


def read(path):
    request = urllib.request.Request(base + path, headers={
        "Accept": "application/vnd.github+json", "User-Agent": "ocx-bd2-ci-readonly-wait"})
    try:
        with urllib.request.urlopen(request, timeout=15) as response:
            return json.load(response)
    except Exception:
        result = subprocess.run([
            r"C:\Windows\System32\curl.exe", "-4", "--http1.1", "--connect-timeout", "5",
            "--max-time", "15", "--fail", "--silent", "--show-error", base + path],
            capture_output=True, timeout=20)
        if result.returncode:
            raise RuntimeError("Read observation failed; do not infer remote termination")
        return json.loads(result.stdout)


last = None
save()
try:
    while time.monotonic() - clock < 600:
        observation = {"atUtc": datetime.now(timezone.utc).isoformat(), "runs": []}
        for run_id in identifiers:
            try:
                run = read(f"actions/runs/{run_id}")
                assert run["head_sha"] == source
                observation["runs"].append({
                    "id": run_id, "status": run["status"], "conclusion": run["conclusion"]})
                if run["status"] == "completed" and not (root / f"ci-terminal-{run_id}.json").exists():
                    jobs = read(f"actions/runs/{run_id}/jobs?per_page=100")
                    artifacts = read(f"actions/runs/{run_id}/artifacts?per_page=100")
                    with (root / f"ci-terminal-{run_id}.json").open("x", encoding="utf-8") as stream:
                        json.dump({"observedAtUtc": observation["atUtc"], "run": run,
                                   "jobs": jobs, "artifacts": artifacts},
                                  stream, ensure_ascii=False, indent=2)
                        stream.write("\n")
            except Exception as error:
                observation["runs"].append({"id": run_id, "readErrorType": type(error).__name__,
                                            "terminalStateInferred": False})
        record["observations"].append(observation)
        key = json.dumps(observation["runs"], sort_keys=True)
        if key != last:
            print(json.dumps(observation["runs"]), flush=True)
            last = key
        if all((root / f"ci-terminal-{run_id}.json").exists() for run_id in identifiers):
            record["status"] = "both-existing-runs-terminal-verified"
            break
        save()
        remaining = 600 - (time.monotonic() - clock)
        if remaining <= 0:
            break
        time.sleep(min(30, remaining))
    else:
        record["status"] = "bounded-wait-ended-remote-state-not-guessed"
    if record["status"] == "observing-existing-runs":
        record["status"] = "bounded-wait-ended-remote-state-not-guessed"
finally:
    record["finishedAtUtc"] = datetime.now(timezone.utc).isoformat()
    record["elapsedSeconds"] = time.monotonic() - clock
    save()
print(json.dumps({"status": record["status"], "testsOrEtwStarted": False}), flush=True)

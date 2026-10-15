"""Continue the same packaging handle after the earlier local observer timed out."""
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
prior = json.loads((root / "candidate-watch-result.json").read_text())
assert prior["lastObservation"]["runId"] == 38663871103 and not prior["remoteTerminalObserved"]
result_path = root / "candidate-followup-watch-result.json"
assert not result_path.exists()
started, count, last, printed = time.monotonic(), 0, prior["lastObservation"], None
while time.monotonic() - started < 900:
    response = subprocess.run(
        [sys.executable, str(root / "candidate-api.py"), "observe"],
        capture_output=True, text=True, encoding="utf-8", timeout=90)
    count += 1
    if response.returncode == 0:
        last = json.loads(response.stdout)
        assert last.get("runId") == 38663871103
        if response.stdout != printed:
            print(response.stdout.strip(), flush=True)
            printed = response.stdout
        if last["status"] == "completed":
            break
    else:
        stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
        (root / f"candidate-followup-read-error-{stamp}.json").write_text(json.dumps({
            "atUtc": datetime.now(timezone.utc).isoformat(), "runId": 38663871103,
            "readFailed": True, "remoteTerminalInferred": False, "remoteRestartRequested": False,
        }, indent=2) + "\n", encoding="utf-8")
    time.sleep(30)
report = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "runId": 38663871103,
    "sameRemoteHandle": True, "observations": count, "elapsedSeconds": time.monotonic() - started,
    "lastObservation": last, "remoteTerminalObserved": last.get("status") == "completed",
    "remoteRestarts": 0, "hostDesktopStarted": False,
}
result_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report), flush=True)

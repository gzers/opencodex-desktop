"""Bounded read-only observation of two existing CI handles; never dispatch or restart."""
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
result_path = root / "ci-watch-result.json"
assert not result_path.exists(), "Inspect the original handles rather than restarting this watcher."
ids = [38660621496, 38660625647]
started = time.monotonic()
last_output = None
last_states = {}
observations = 0
while time.monotonic() - started < 900:
    observed = subprocess.run(
        [sys.executable, str(root / "github-read.py"), *map(str, ids)],
        capture_output=True, text=True, encoding="utf-8", timeout=60,
    )
    observations += 1
    if observed.returncode:
        print(json.dumps({"observation": observations, "readFailed": True,
                          "remoteTerminalClaimed": False}), flush=True)
    else:
        latest = max(root.glob("observation-*.json"))
        snapshot = json.loads(latest.read_text(encoding="utf-8"))
        for number in ids:
            row = snapshot["reads"].get("run-" + str(number), {})
            if not row.get("readError") and row.get("id") == number:
                assert row["head_sha"] == "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
                last_states[str(number)] = {key: row.get(key) for key in ("id", "event", "status", "conclusion")}
        if observed.stdout != last_output:
            print(observed.stdout.strip(), flush=True)
            last_output = observed.stdout
        if len(last_states) == 2 and all(row["status"] == "completed" for row in last_states.values()):
            break
    time.sleep(30)
report = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "observations": observations,
    "elapsedSeconds": time.monotonic() - started, "handles": last_states,
    "allRemoteTerminalObserved": len(last_states) == 2 and all(row["status"] == "completed" for row in last_states.values()),
    "dispatches": 0, "restarts": 0, "localDesktopStarted": False,
}
result_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report), flush=True)

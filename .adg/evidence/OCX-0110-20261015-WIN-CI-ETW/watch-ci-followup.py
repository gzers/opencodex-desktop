"""Continue reading the same two remote handles after the earlier observer's timebox."""
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
prior = json.loads((root / "ci-watch-result.json").read_text(encoding="utf-8"))
assert not prior["allRemoteTerminalObserved"]
assert set(prior["handles"]) == {"38660621496", "38660625647"}
result_path = root / "ci-watch-followup-result.json"
assert not result_path.exists()
started = time.monotonic()
observations = 0
states = prior["handles"]
last_print = None
while time.monotonic() - started < 900:
    result = subprocess.run(
        [sys.executable, str(root / "github-read.py"), "38660621496", "38660625647"],
        capture_output=True, text=True, encoding="utf-8", timeout=60)
    observations += 1
    if result.returncode == 0:
        snapshot = json.loads(max(root.glob("observation-*.json")).read_text(encoding="utf-8"))
        for number in states:
            row = snapshot["reads"].get("run-" + number, {})
            if not row.get("readError") and row.get("id") == int(number):
                assert row["head_sha"] == "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
                states[number] = {key: row.get(key) for key in ("id", "event", "status", "conclusion")}
        if result.stdout != last_print:
            print(result.stdout.strip(), flush=True)
            last_print = result.stdout
        if all(row["status"] == "completed" for row in states.values()):
            break
    else:
        print(json.dumps({"readFailed": True, "remoteTerminalClaimed": False}), flush=True)
    time.sleep(30)
report = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "observations": observations,
    "elapsedSeconds": time.monotonic() - started, "handles": states,
    "allRemoteTerminalObserved": all(row["status"] == "completed" for row in states.values()),
    "sameRemoteHandlesAsPreviousObserver": True, "dispatches": 0, "remoteRestarts": 0,
}
result_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report), flush=True)

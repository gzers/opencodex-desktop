"""Capture read-only production readbacks without console transcoding."""
import json
import subprocess
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[1]
label = sys.argv[1]
assert label.replace("-", "").isalnum()
method = sys.argv[2] if len(sys.argv) > 2 else "read-ui.js"
assert method in {"read-ui.js", "read-alignment.js"}
destination = root / (label + ".json")
assert not destination.exists(), "Preserve prior observation"
with destination.open("xb") as output, (root / (label + ".stderr.txt")).open("xb") as errors:
    result = subprocess.run([
        r"C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe",
        r"C:\Users\15119\.codex\tmp\ocx-0110-cdp-query.mjs",
        str(root / "methods" / method),
    ], stdout=output, stderr=errors, timeout=50, creationflags=subprocess.CREATE_NO_WINDOW)
assert result.returncode == 0
value = json.loads(destination.read_text(encoding="utf-8-sig"))
assert value["sourceCommit"] == "eae85322ebca3b755c034db97cbebf2ea5bf71fc" and value["applicationPid"] == 37276
print(json.dumps({"readback": label, "source": value["sourceCommit"], "pid": value["applicationPid"]}))

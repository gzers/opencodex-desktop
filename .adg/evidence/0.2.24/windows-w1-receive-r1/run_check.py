"""Receive-side checks only; never executes a Windows binary or installer."""
from pathlib import Path
import json
import os
import subprocess
import sys
import time

evidence = Path(__file__).resolve().parent
name, code_root, *command = sys.argv[1:]
assert name.replace("-", "").isalnum()
output = evidence / (name + ".log")
record = evidence / (name + ".json")
assert not output.exists() and not record.exists(), "immutable check names"
environment = os.environ.copy()
environment["PYTHONDONTWRITEBYTECODE"] = "1"
started = time.monotonic()
with output.open("xb") as stream:
    result = subprocess.run(command, cwd=code_root, env=environment, stdout=stream, stderr=subprocess.STDOUT)
document = {"command": command, "cwd": code_root, "exitCode": result.returncode,
            "elapsedSeconds": round(time.monotonic() - started, 3),
            "windowsRuntimeExecutedHere": False}
record.write_text(json.dumps(document, ensure_ascii=False, indent=2) + "\n")
print(json.dumps(document, ensure_ascii=False))
print(output.read_text(errors="replace")[-3500:])
sys.exit(result.returncode)

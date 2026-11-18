"""Exercise the offline reviewer against archived V2 bytes and forged receipt links."""

import copy
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).absolute().parent
ROWS = ("v2-record-000001.json", "v2-record-000002.json")
PREFIX = "original recovery retained/unknown; worker DATA="
CMD_SHA = "85f56dd180ee907b70e68ed531d90a4e5a480cb104264bdad4890053e5da60f2"
LAUNCHER_SHA = "6b553204560d59ba7a292f64c322404028eceb1a0899661f5f9dc9d76953e990"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def encode(data):
    return json.dumps(data, indent=2).encode("utf-8")


def run(archive):
    blobs = [(archive / name).read_bytes() for name in ROWS]
    action = json.loads(blobs[0])["recoveryActionId"]
    snapshot = {"actionId": action, "preparationPending": {
        "intentDigest": digest(blobs[0]), "pendingDigest": digest(blobs[1]),
        "attachmentAttempted": True, "durable": True, "startRawError": 31,
        "attachmentRawError": None, "attachmentErrorKind": None,
    }}
    explicit = encode({"mode": "Explicit", "passed": True, "recoveryActionId": action,
                       "manager": {"result": {"response": {"error": {
                           "detail": PREFIX + json.dumps(snapshot)
                       }}}}})
    after = {"mode": "DiagnosticAfter", "passed": True,
             "manager": {"result": {"response": {"facts": {"snapshot": snapshot}}}},
             "attachment": [{"name": f"DiagnosticAfter-{name}", "bytes": len(blob), "sha256": digest(blob)}
                            for name, blob in zip(ROWS, blobs)]}
    binding = {"sid": "S-1-5-21-587408064-856152790-2583328096-1003", "session": 7,
               "launcherSha256": CMD_SHA, "launcherScriptSha256": LAUNCHER_SHA,
               "explicitResultSha256": digest(explicit), "recoveryActionId": action}
    # One tick below the deadline. The displayed milliseconds round up to 5000.
    timing = {"binding": binding, "startedQpc": "70000000", "finishedQpc": "119999999",
              "qpcFrequency": "10000000", "elapsedUpperBoundMs": 5000, "originalBudgetMs": 5000,
              "strictlyWithinBudget": True, "launcherRaw": 0, "launcherError": None,
              "resultSha256": digest(encode(after)), "accepted": True, "firstError": None,
              "launcherFirstError": None}
    expected = {
        "one-tick-under": None,
        "at-cutoff": "Original five-second readback bound unproven.",
        "forged-timing-display": "Derived timing fields differ from original ticks.",
        "mixed-pair": "Attempt or timing binding differs.",
        "wrong-explicit-hash": "Explicit receipt differs.",
        "wrong-action-with-recomputed-hash": "DiagnosticAfter snapshot action differs.",
        "changed-export": "Independent attachment 2 bytes differ.",
    }
    results = []
    with tempfile.TemporaryDirectory(prefix="ocx-r7-post-review-") as temp:
        for name, error in expected.items():
            root = Path(temp) / name
            root.mkdir()
            actual_timing = copy.deepcopy(timing)
            actual_after = copy.deepcopy(after)
            actual_blobs = blobs.copy()
            explicit_pin = digest(explicit)
            if name == "at-cutoff":
                actual_timing["finishedQpc"] = "120000000"
            elif name == "forged-timing-display":
                actual_timing["elapsedUpperBoundMs"] = 5
            elif name == "mixed-pair":
                actual_timing["binding"]["launcherScriptSha256"] = "0" * 64
            elif name == "wrong-explicit-hash":
                explicit_pin = "0" * 64
            elif name == "wrong-action-with-recomputed-hash":
                actual_after["manager"]["result"]["response"]["facts"]["snapshot"]["actionId"] = (
                    "44444444-4444-4444-8444-444444444444"
                )
                actual_timing["resultSha256"] = digest(encode(actual_after))
            elif name == "changed-export":
                actual_blobs[1] += b" "
            files = {"Explicit-result.json": explicit,
                     "DiagnosticAfter-independent-timing-attempt.json": encode(binding),
                     "DiagnosticAfter-independent-timing.json": encode(actual_timing),
                     "DiagnosticAfter-result.json": encode(actual_after)}
            files.update({f"DiagnosticAfter-{name}": blob for name, blob in zip(ROWS, actual_blobs)})
            for file, data in files.items():
                (root / file).write_bytes(data)
            result = subprocess.run([sys.executable, HERE / "review-readback.py", root,
                                     "--explicit-sha256", explicit_pin, "--cmd-sha256", CMD_SHA,
                                     "--launcher-sha256", LAUNCHER_SHA], capture_output=True, text=True)
            actual = json.loads(result.stdout)
            assert not result.stderr, (name, result.stderr)
            assert result.returncode == (0 if error is None else 1), (name, actual)
            assert actual.get("firstError") == error, (name, actual)
            results.append({"case": name, "raw": result.returncode, "result": actual})
    print(json.dumps({"scope": "synthetic receipts with archived V2 bytes", "cases": results,
                      "cleanupComplete": not Path(temp).exists(), "scratch": temp}, indent=2))


if __name__ == "__main__":
    run(Path(sys.argv[1]))

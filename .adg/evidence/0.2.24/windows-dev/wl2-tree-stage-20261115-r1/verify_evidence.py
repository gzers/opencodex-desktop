"""Offline evidence-byte replay, not production or Closed acceptance."""
from pathlib import Path
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parent


def load(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


inventory = load(ROOT / "payload-manifest.json")
assert inventory["productionAcceptance"] is False
for row in inventory["files"]:
    path = ROOT / row["path"]
    assert path.resolve().is_relative_to(ROOT.resolve())
    raw = path.read_bytes()
    assert len(raw) == row["bytes"] and sha(raw) == row["sha256"], row["path"]

expected = {"library01": (101, 9, 1), "library02": (0, 10, 0),
            "current01": (0, 2, 0), "native01": (0, 5, 0),
            "locks01": (101, None, None), "locks02": (0, 3, 0),
            "library03": (0, 23, 0), "library04": (0, 23, 0),
            "product01": (0, None, None)}
for name, (code, passed, failed) in expected.items():
    run = ROOT / name
    result = load(run / "result.json")
    assert result["exitCode"] == code and result["accepted"] is False, name
    manifest = load(run / "source-manifest.json")
    for row in manifest:
        path = run / "executed-source" / row["path"]
        assert path.resolve().is_relative_to((run / "executed-source").resolve())
        raw = path.read_bytes()
        assert len(raw) == row["bytes"] and sha(raw) == row["sha256"], str(path)
    if passed is not None:
        text = (run / "stdout.log").read_text(encoding="utf-8-sig")
        match = re.search(r"test result: .*?(\d+) passed; (\d+) failed;", text)
        assert match and (int(match[1]), int(match[2])) == (passed, failed), name

status = load(ROOT / "final-status.json")
assert status["allAccepted"] is False and status["productionExecuted"] is False
assert status["actualGUI"] is False and status["startOpen"] is False
assert status["cleanup"]["historicalRejectedDeletionRetried"] is False
assert status["cleanup"]["overallCleanupComplete"] is False
print(json.dumps({"payloads": len(inventory["files"]), "runs": list(expected),
                  "integrity": "verified", "productionAcceptance": False}))

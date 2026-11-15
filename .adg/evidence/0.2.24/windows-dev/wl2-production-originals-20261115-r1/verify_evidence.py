"""Offline immutable-byte replay; does not grant native production authority."""
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
status = load(ROOT / "final-status.json")
for name, expected in status["runs"].items():
    run = ROOT / name
    result = load(run / "result.json")
    assert result["exitCode"] == expected["rawExit"] and result["accepted"] is False
    for row in load(run / "source-manifest.json"):
        path = run / "executed-source" / row["path"]
        assert path.resolve().is_relative_to((run / "executed-source").resolve())
        raw = path.read_bytes()
        assert len(raw) == row["bytes"] and sha(raw) == row["sha256"]
    if expected["passed"] is not None:
        stdout = (run / "stdout.log").read_text(encoding="utf-8-sig")
        match = re.search(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", stdout)
        assert match and tuple(map(int, match.groups())) == (expected["passed"], 0, expected["ignored"])
assert status["allAccepted"] is False and status["productionExecuted"] is False
assert status["actualGUI"] is False and status["startOpen"] is False
assert status["cleanup"]["historicalRejectedDeletionRetried"] is False
assert status["cleanup"]["overallCleanupComplete"] is False
assert status["missingArtifactHash"]["run"] == "library02"
print(json.dumps(dict(payloads=len(inventory["files"]), runs=list(status["runs"]), integrity="verified", productionAcceptance=False)))

"""Offline original byte consistency only; never grants native authority."""
from pathlib import Path
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parent
load = lambda p: json.loads(p.read_text(encoding="utf-8-sig"))
sha = lambda b: hashlib.sha256(b).hexdigest()
inventory = load(ROOT / "payload-manifest.json")
assert inventory["productionAcceptance"] is False
for row in inventory["files"]:
    p = ROOT / row["path"]
    assert p.resolve().is_relative_to(ROOT.resolve())
    raw = p.read_bytes()
    assert len(raw) == row["bytes"] and sha(raw) == row["sha256"], row["path"]
status = load(ROOT / "final-status.json")
for name, expected in status["runs"].items():
    run = ROOT / name
    result = load(run / "result.json")
    assert result["exitCode"] == expected["rawExit"] and result["accepted"] is False
    for row in load(run / "source-manifest.json"):
        p = run / "executed-source" / row["path"]
        assert p.resolve().is_relative_to((run / "executed-source").resolve())
        raw = p.read_bytes()
        assert len(raw) == row["bytes"] and sha(raw) == row["sha256"]
    if expected["passed"] is not None:
        text = (run / "stdout.log").read_text(encoding="utf-8-sig")
        m = re.search(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", text)
        assert m and tuple(map(int, m.groups())) == (expected["passed"], 0, expected["ignored"])
assert status["allAccepted"] is False and status["productionExecuted"] is False
assert status["actualGUI"] is False and status["startOpen"] is False
assert status["cleanup"]["historicalRejectedDeletionRetried"] is False
assert status["cleanup"]["overallCleanupComplete"] is False
assert status["cleanup"]["deletionAttempted"] is False
print(json.dumps(dict(payloads=len(inventory["files"]), integrity="verified", productionAcceptance=False)))

"""Offline raw-byte replay only, never official/native lifecycle acceptance."""
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
for name, passed in {"library01": 27, "library02": 28, "library03": 29, "product01": None}.items():
    run = ROOT / name
    result = load(run / "result.json")
    assert result["exitCode"] == 0 and result["accepted"] is False
    for row in load(run / "source-manifest.json"):
        p = run / "executed-source" / row["path"]
        assert p.resolve().is_relative_to((run / "executed-source").resolve())
        raw = p.read_bytes()
        assert len(raw) == row["bytes"] and sha(raw) == row["sha256"]
    if passed is not None:
        text = (run / "stdout.log").read_text(encoding="utf-8-sig")
        m = re.search(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", text)
        assert m and tuple(map(int, m.groups())) == (passed, 0, 0)
status = load(ROOT / "final-status.json")
assert status["allAccepted"] is False and status["productionExecuted"] is False
assert status["actualGUI"] is False and status["startOpen"] is False
assert status["cleanup"]["historicalRejectedDeletionRetried"] is False
assert status["cleanup"]["overallCleanupComplete"] is False
request = load(ROOT / "shared-minimum-request.json")
assert request["allAccepted"] is False
print(json.dumps(dict(payloads=len(inventory["files"]), integrity="verified", productionAcceptance=False)))

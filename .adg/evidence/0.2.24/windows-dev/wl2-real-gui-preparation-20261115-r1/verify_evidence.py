"""Offline preparation evidence only; does not execute GUI or issue a proof."""
from pathlib import Path
import hashlib
import json
import zipfile

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
for run in ["library01", "product01"]:
    d = ROOT / run
    assert load(d / "result.json")["exitCode"] == 0
    assert load(d / "result.json")["accepted"] is False
    for row in load(d / "source-manifest.json"):
        p = d / "executed-source" / row["path"]
        assert p.resolve().is_relative_to((d / "executed-source").resolve())
        raw = p.read_bytes()
        assert len(raw) == row["bytes"] and sha(raw) == row["sha256"]
text = (ROOT / "library01/stdout.log").read_text(encoding="utf-8-sig")
assert "6 passed; 0 failed; 3 ignored" in text
with zipfile.ZipFile(ROOT / "frontend-dist.zip") as z:
    for row in load(ROOT / "frontend-dist-manifest.json"):
        raw = z.read(row["path"])
        assert len(raw) == row["bytes"] and sha(raw) == row["sha256"]
status = load(ROOT / "final-status.json")
assert status["allAccepted"] is False and status["actualGUIExecuted"] is False
assert status["officialOwnerCreated"] is False and status["productionExecuted"] is False
assert status["startOpen"] is False and status["cleanup"]["overallCleanupComplete"] is False
assert status["cleanup"]["historicalRejectedDeletionRetried"] is False
assert status["prepared"]["inputHashesVerified"] is True
print(json.dumps(dict(payloads=len(inventory["files"]), preparationBytes="verified", actualGUIExecuted=False, productionAcceptance=False)))

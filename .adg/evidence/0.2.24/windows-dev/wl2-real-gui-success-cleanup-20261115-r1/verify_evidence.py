from pathlib import Path
import hashlib
import json

root = Path(__file__).resolve().parent
load = lambda p: json.loads(p.read_text(encoding="utf-8-sig"))
sha = lambda raw: hashlib.sha256(raw).hexdigest()
inventory = load(root / "payload-manifest.json")
assert inventory["productionAcceptance"] is False
for row in inventory["files"]:
    path = root / row["path"]
    assert path.resolve().is_relative_to(root.resolve())
    raw = path.read_bytes()
    assert len(raw) == row["bytes"] and sha(raw) == row["sha256"]
status = load(root / "final-status.json")
assert status["firstError"]["rawExit"] == 1 and status["firstError"]["deletionAttempted"] is False
assert status["archivePrimitiveResult"]["rawExit"] == 0
assert status["archivePrimitiveResult"]["sourceChangeRefused"] is True
assert status["archivePrimitiveResult"]["extraDirectoryRefused"] is True
assert status["allAccepted"] is False and status["productionExecuted"] is False
assert status["startOpen"] is False and status["cleanup"]["overallCleanupComplete"] is False
assert status["cleanup"]["historicalRejectedDeletionRetried"] is False
assert load(root / "final-parse-and-function-equivalence.json")["allExecutedArchiveFunctionsEqualPrepared"] is True
for row in load(root / "prepared-location.json")["files"]:
    if row["path"].endswith(".cmd") or row["path"].endswith(".ps1"):
        raw = (root / "prepared-source" / row["path"]).read_bytes()
        assert len(raw) == row["bytes"] and sha(raw) == row["sha256"]
print(json.dumps(dict(payloads=len(inventory["files"]), integrity="verified", productionAcceptance=False)))

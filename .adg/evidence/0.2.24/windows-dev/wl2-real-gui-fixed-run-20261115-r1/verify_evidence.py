from pathlib import Path
import base64
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
export = load(root / "same-sid-export01.json")
assert export["complete"] is True and not export["fileErrors"] and export["session"] == 7
for row in export["files"]:
    raw = base64.b64decode(row["base64"], validate=True)
    assert len(raw) == row["bytes"] and sha(raw) == row["sha256"]
    assert raw == (root / "received" / row["relative"]).read_bytes()
status = load(root / "final-status.json")
assert status["desktop"]["rawTestExit"] == 0 and status["actualGUIComponentPassed"] is True
assert status["brokerRawExit"] is None and status["officialOwnerCreated"] is False
assert status["productionExecuted"] is False and status["startOpen"] is False
assert status["cleanup"]["overallCleanupComplete"] is False
assert status["cleanup"]["historicalRejectedDeletionRetried"] is False
assert [m["generation"] for m in status["modes"]] == [1, 2, 3, 4]
assert [m["managerRawExit"] for m in status["modes"]] == [0, 1, 0, 0]
print(json.dumps(dict(payloads=len(inventory["files"]), received=len(export["files"]), integrity="verified", productionAcceptance=False)))

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
assert export["complete"] is True and not export["fileErrors"]
for row in export["files"]:
    raw = base64.b64decode(row["base64"], validate=True)
    assert len(raw) == row["bytes"] and sha(raw) == row["sha256"]
    assert raw == (root / "received" / row["relative"]).read_bytes()
result = load(root / "receive-result.json")
assert result["result"]["rawTestExit"] == 101 and result["actualGUIComponentPassed"] is False
assert result["productionExecuted"] is False and result["deletionAttempted"] is False
print(json.dumps(dict(payloads=len(inventory["files"]), received=len(export["files"]), integrity="verified", productionAcceptance=False)))

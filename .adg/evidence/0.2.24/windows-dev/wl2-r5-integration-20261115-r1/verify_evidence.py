from pathlib import Path
import hashlib,json,re
root=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text(encoding="utf-8-sig"))
sha=lambda b:hashlib.sha256(b).hexdigest()
manifest=load(root/"payload-manifest.json")
assert manifest["productionAcceptance"] is False
for row in manifest["files"]:
    p=root/row["path"];assert p.resolve().is_relative_to(root.resolve())
    raw=p.read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"],row["path"]
status=load(root/"final-status.json")
for name,result in status["runs"].items():
    assert load(root/name/"result.json")["exitCode"]==result["rawExit"]==0
    for row in load(root/name/"source-manifest.json"):
        raw=(root/name/"executed-source"/row["path"]).read_bytes()
        assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
    if name.startswith("library"):
        assert "test result: ok. 54 passed; 0 failed; 0 ignored" in (root/name/"stdout.log").read_text(encoding="utf-8-sig")
assert load(root/"merge-resolution.json")["fullRecoveryReceiverPreserved"] is True
assert status["startOpen"] is False and status["productionExecuted"] is False
assert status["allAccepted"] is False and status["firstError"] is None
assert status["cleanup"]["overallCleanupComplete"] is False and status["cleanup"]["historicalRejectedDeletionRetried"] is False
print(json.dumps(dict(payloads=len(manifest["files"]),integrity="verified",productionAcceptance=False)))

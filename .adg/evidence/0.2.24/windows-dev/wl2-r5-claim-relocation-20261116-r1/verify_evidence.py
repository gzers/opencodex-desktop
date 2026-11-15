from pathlib import Path
import hashlib,json,re
root=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text(encoding="utf-8-sig"))
sha=lambda b:hashlib.sha256(b).hexdigest()
manifest=load(root/"payload-manifest.json");assert manifest["productionAcceptance"] is False
for row in manifest["files"]:
    p=root/row["path"];assert p.resolve().is_relative_to(root.resolve())
    raw=p.read_bytes();assert len(raw)==row["bytes"] and sha(raw)==row["sha256"]
status=load(root/"final-status.json")
for name,row in status["runs"].items():
    assert load(root/name/"result.json")["exitCode"]==row["rawExit"]
    for f in load(root/name/"source-manifest.json"):
        raw=(root/name/"executed-source"/f["path"]).read_bytes();assert len(raw)==f["bytes"] and sha(raw)==f["sha256"]
    if row["passed"] is not None:
        text=(root/name/"stdout.log").read_text(encoding="utf-8-sig");m=re.search(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored",text)
        assert m and tuple(map(int,m.groups()))==(row["passed"],row["failed"],row["ignored"])
assert [r["moved"] for r in status["sharingMatrix"]]==[True,True,False,False,False]
assert status["continuousOpenClaimPositivePassed"] is False
assert status["allAccepted"] is False and status["productionExecuted"] is False and status["startOpen"] is False
assert status["blockingError"]["rawWin32"]==5 and status["blockingError"]["originalClaimReleaseImplemented"] is False
assert status["cleanup"]["overallCleanupComplete"] is False and status["cleanup"]["historicalRejectedDeletionRetried"] is False
print(json.dumps(dict(payloads=len(manifest["files"]),integrity="verified",productionAcceptance=False)))

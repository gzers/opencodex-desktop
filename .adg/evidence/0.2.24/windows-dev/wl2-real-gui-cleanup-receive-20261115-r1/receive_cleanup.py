"""Receive exact same-SID cleanup receipt without adopting native authority."""
from pathlib import Path
from datetime import datetime
import hashlib
import json
import uuid

ROOT = Path(__file__).resolve().parent
PREP = ROOT.parent / "wl2-real-gui-success-cleanup-20261115-r1"
PUBLIC = Path(r"C:/Users/Public/ocx-h024-gui-archive-20261115-r1")
sha = lambda raw: hashlib.sha256(raw).hexdigest()
load = lambda p: json.loads(p.read_text(encoding="utf-8-sig"))


def copy(source, relative):
    data = source.read_bytes()
    target = ROOT / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("xb") as out:
        out.write(data)
    return data


for name in ["archive-h024-fixed-success.ps1", "archive-h024-fixed-success.cmd"]:
    data = copy(PUBLIC / name, "executed-input/" + name)
    assert data == (PREP / "prepared-source" / name).read_bytes(), name
plan_raw = copy(PUBLIC / "archive-plan.json", "executed-input/archive-plan.json")
assert plan_raw == (PREP / "archive-plan.json").read_bytes()
intent_raw = copy(PUBLIC / "out01/archive-cleanup-intent.json", "received/archive-cleanup-intent.json")
raw = copy(PUBLIC / "out01/archive-cleanup-result.json", "received/archive-cleanup-result.json")
intent, result = json.loads(intent_raw), json.loads(raw)
assert intent["sid"] == result["sid"] == "S-1-5-21-587408064-856152790-2583328096-1003"
assert intent["session"] == result["session"] == 7 and intent["oneAttempt"] is True
assert result["complete"] is True and result["removedSuccessfulProfileChild"] is True
assert result["deletionAttempted"] is True and result["brokerRawExit"] is None
for name in ["forcedProcessCleanup", "oldRejectedPathsRetried", "originalFailureRootChanged",
             "accountOrProfileDeleted", "dailyPolicyOrPathChanged", "nativeMetadataCloned"]:
    assert result[name] is False, name
assert result["publicSourceRetained"] is True
assert len(result["targets"]) == 2
profile, public = result["targets"]
assert profile["path"] == r"C:\Users\marenben\ocx024-h-1fa5e23f-12c9-4289-84f3-bcc901642b35"
assert public["path"] == r"C:\Users\Public\ocx-h024-actual-gui-fixed-20261115-r1"
assert profile["verified"] is True and profile["sourceRetained"] is False
assert public["verified"] is True and public["sourceRetained"] is True
assert (profile["files"], profile["directories"], profile["bytes"]) == (685, 291, 119851895)
assert (public["files"], public["directories"], public["bytes"]) == (20, 2, 178669957)
archive = result["archiveDirectory"]
prefix = "C:\\Users\\marenben\\ocx024-gui-archive-"
assert archive.startswith(prefix)
uuid.UUID(archive[len(prefix):])
for row, leaf in [(profile, "successful-profile-tree.zip"), (public, "successful-public-inputs-logs.zip")]:
    assert row["archive"] == archive + "\\" + leaf
    assert len(row["archiveSha256"]) == 64
status = dict(time=datetime.now().astimezone().isoformat(), sourceHead="0549a660ee3f77ea243e798ec55b01f52545e492",
    productSourceHead="6e312f02270f247d2f440ec369385e523a4a8043", inputBytesMatchPrepared=True,
    intentSha256=sha(intent_raw), receiptSha256=sha(raw), successfulTaskSameSidCleanupPassed=True,
    removedProfileChild=dict(path=profile["path"], files=685, directories=291, bytes=119851895),
    archivedPublic=dict(path=public["path"], files=20, directories=2, bytes=178669957, retained=True),
    archiveDirectory=archive, archiveZipHashes=[row["archiveSha256"] for row in result["targets"]],
    privateManifestSha256=result["privateManifestSha256"], verifiedUnderOriginalSid=True,
    privateArchivesIndependentlyReadByController=False, nativeMetadataCloned=False,
    oldFailedBrokerExitUnconfirmed=True, historicalRejectedDeletionRetried=False,
    accountProfileOrDailyPolicyChanged=False, overallCleanupComplete=False,
    firstError=None, productionExecuted=False, officialOwnerCreated=False, startOpen=False, allAccepted=False)
with (ROOT / "receive-result.json").open("x", encoding="utf-8") as out:
    json.dump(status, out, ensure_ascii=False, indent=2)
print(json.dumps(dict(inputBytesMatchPrepared=True, successfulProfileChildRemoved=True,
    files=685, bytes=119851895, publicRetained=True, overallCleanupComplete=False)))

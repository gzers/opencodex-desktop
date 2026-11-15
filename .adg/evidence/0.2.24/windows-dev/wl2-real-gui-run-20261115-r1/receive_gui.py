"""Receive immutable same-SID export; decode only task logs, never native proof."""
from pathlib import Path
from datetime import datetime
import base64
import hashlib
import json

ROOT = Path(__file__).resolve().parent
PUBLIC = Path(r"C:/Users/Public/ocx-h024-actual-gui-20261115-r1")
sha = lambda raw: hashlib.sha256(raw).hexdigest()
raw = (PUBLIC / "out01/same-sid-export01.json").read_bytes()
with (ROOT / "same-sid-export01.json").open("xb") as out:
    out.write(raw)
export = json.loads(raw)
assert export["complete"] is True and not export["fileErrors"]
assert export["sid"] == "S-1-5-21-587408064-856152790-2583328096-1003" and export["session"] == 7
seen = set()
for row in export["files"]:
    rel = row["relative"]
    assert rel not in seen and not rel.startswith("/") and ".." not in Path(rel).parts
    seen.add(rel)
    assert rel.startswith(("out/", "D/"))
    data = base64.b64decode(row["base64"], validate=True)
    assert len(data) == row["bytes"] and sha(data) == row["sha256"] and row["stableLength"] is True
    dest = ROOT / "received" / rel
    assert dest.resolve().is_relative_to((ROOT / "received").resolve())
    dest.parent.mkdir(parents=True, exist_ok=True)
    with dest.open("xb") as out:
        out.write(data)
for name in ["launch-h024-actual-gui.cmd", "launch-h024-actual-gui.ps1", "collect-h024-actual-gui.ps1", "input-manifest.json"]:
    dest = ROOT / "launch-input" / name
    dest.parent.mkdir(parents=True, exist_ok=True)
    with dest.open("xb") as out:
        out.write((PUBLIC / name).read_bytes())
load = lambda rel: json.loads((ROOT / "received" / rel).read_text(encoding="utf-8-sig"))
result = load("out/launch-result.json")
bootstrap = load("out/bootstrap-admission.json")
assert result["rawTestExit"] == 101 and result["actualGUIComponentPassed"] is False
assert bootstrap["noJob"] is True and bootstrap["identity"]["elevated"] is False
status = dict(time=datetime.now().astimezone().isoformat(), sourceHead="0c618cdfde4da37da0ca90e1b0913d7d81f63a29",
    result=result, bootstrap=bootstrap, exportSha256=sha(raw), exportedFiles=len(seen),
    firstError="observer missing D/gui-normal.json; GUI admission refused raw=None",
    originalTaskBase=export["base"], originalTaskSceneRetained=True,
    nativeMetadataCloned=False, tokenOrPolicyChanged=False, deletionAttempted=False,
    liveBrokerExitUnconfirmed=True, actualGUIComponentPassed=False, productionExecuted=False, allAccepted=False)
with (ROOT / "receive-result.json").open("x", encoding="utf-8") as out:
    json.dump(status, out, ensure_ascii=False, indent=2)
print(json.dumps(dict(files=len(seen), sameSidComplete=True, rawExit=101, productionAcceptance=False)))

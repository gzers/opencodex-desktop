"""Capture new same-SID desktop result once, preserving every raw outcome."""
from pathlib import Path
import base64
import hashlib
import json
import sys

PREP = Path(__file__).resolve().parent
# A later desktop response must create a new evidence round, never append to
# the already-sealed preparation inventory.
assert len(sys.argv) == 2, "pass a fresh evidence round path"
ROOT = Path(sys.argv[1]).resolve()
assert ROOT.parent == PREP.parent and ROOT.name.startswith("wl2-real-gui-fixed-run-")
assert not ROOT.exists(), "immutable receive round already exists"
ROOT.mkdir()
with (ROOT / ".gitattributes").open("x", encoding="utf-8") as out:
    out.write("* -text\n")
PUBLIC = Path(r"C:/Users/Public/ocx-h024-actual-gui-fixed-20261115-r1")
load = lambda p: json.loads(p.read_text(encoding="utf-8-sig"))
sha = lambda raw: hashlib.sha256(raw).hexdigest()
raw = (PUBLIC / "out01/same-sid-export01.json").read_bytes()
with (ROOT / "same-sid-export01.json").open("xb") as out:
    out.write(raw)
export = json.loads(raw)
assert export["sid"] == "S-1-5-21-587408064-856152790-2583328096-1003" and export["session"] == 7
seen = set()
for row in export["files"]:
    rel = row["relative"]
    assert rel not in seen and rel.startswith(("out/", "D/")) and ".." not in Path(rel).parts
    seen.add(rel)
    data = base64.b64decode(row["base64"], validate=True)
    assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    dest = ROOT / "received" / rel
    assert dest.resolve().is_relative_to((ROOT / "received").resolve())
    dest.parent.mkdir(parents=True, exist_ok=True)
    with dest.open("xb") as out:
        out.write(data)
result = load(ROOT / "received/out/launch-result.json")
bootstrap = load(ROOT / "received/out/bootstrap-admission.json")
assert result["elevated"] is False and bootstrap["identity"]["elevated"] is False
assert bootstrap["noJob"] is True
assert export["complete"] is True and not export["fileErrors"]
passed = result["rawTestExit"] == 0 and result["actualGUIComponentPassed"] is True
checks = []
if passed:
    component = load(ROOT / "received/out/result.json")
    assert component["actualGUI"] is True and component["fixtureManager"] is False
    assert component["officialOwnerCreated"] is False and component["lifecycleAccepted"] is False
    assert component["sameOriginalBrokerAcrossActualGUIs"] is True
    assert component["originalCreationHandleRetained"] is True
    assert component["brokerExitedExplicitly"] is True
    # The original fixture does not expose the broker's raw exit code. Never
    # relabel the observed exit or host event as raw0.
    assert component["brokerRawExit"] is None
    previous_generation = 0
    for mode in ["normal", "hold", "after-failure", "finish"]:
        after = load(ROOT / f"received/out/after-{mode}.json")
        receipt = load(ROOT / f"received/D/gui-{mode}.json")
        assert after["actualGUI"] is True and after["nativeWindowVerifiedByObserver"] is True
        assert after["originalCreationHandleRetained"] is True
        assert after["broker"] == component["originalBroker"]
        assert receipt["mainPageFinished"] is True and receipt["fullNormalSetup"] is True
        assert receipt["fixtureManager"] is False and receipt["officialOwnerCreated"] is False
        assert receipt["reply"]["oldConnectionReplayRefused"] is True
        generation = int(after["managerGeneration"])
        assert generation > previous_generation
        previous_generation = generation
        if mode != "hold":
            assert after["managerExit"] == 0 and after["forcedManagerOnly"] is False
        else:
            assert after["forcedManagerOnly"] is True
        checks.append(dict(mode=mode, managerRawExit=after["managerExit"], generation=generation,
            originalBrokerMatched=True, nativeWindowVerifiedByObserver=True))
status = dict(source=load(PREP / "final-source.json"), result=result, exportSha256=sha(raw),
    exportedFiles=len(seen), sameSidExportComplete=True, actualGUIComponentPassed=passed,
    modes=checks, originalTaskBase=export["base"], originalTaskSceneRetained=True,
    brokerRawExit=None, nativeMetadataCloned=False, deletionAttempted=False,
    officialOwnerCreated=False, productionExecuted=False, allAccepted=False)
with (ROOT / "desktop-result.json").open("x", encoding="utf-8") as out:
    json.dump(status, out, ensure_ascii=False, indent=2)
print(json.dumps(dict(files=len(seen), rawExit=result["rawTestExit"], componentPassed=passed,
    modes=checks, brokerRawExit=None, productionAcceptance=False)))

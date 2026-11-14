"""Offline evidence replay only. No Windows process, token, ACL, PATH or runtime actions."""
from pathlib import Path, PurePosixPath
import base64, hashlib, json
ROOT = Path(__file__).resolve().parent

def load(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))

def digest(data):
    return hashlib.sha256(data).hexdigest()

def safe(name):
    p = PurePosixPath(name)
    assert name and "\\" not in name and ":" not in name
    assert not p.is_absolute() and all(x not in ("", ".", "..") for x in p.parts)
    assert p.as_posix() == name
    return p

def payload(path, record):
    data = path.read_bytes()
    assert len(data) == record["bytes"], str(path)
    assert digest(data) == record["sha256"], str(path)
    return data

def manifest():
    m = load("evidence-manifest.json")
    names = [x["path"] for x in m["files"]]
    assert len(names) == len(set(names))
    expected = {p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*") if p.is_file() and p.name != "evidence-manifest.json"}
    assert set(names) == expected, "manifest file set differs"
    for item in m["files"]:
        payload(ROOT / safe(item["path"]), item)
    assert sum(x["bytes"] for x in m["files"]) == m["payloadBytes"]
    assert len(names) == m["payloadFiles"]

def export(name, destination, complete, count, stable=False):
    data = load(name)
    assert data.get("sid", data.get("originalWitness", {}).get("sid")) == "S-1-5-21-587408064-856152790-2583328096-1003"
    if "complete" in data:
        assert data["complete"] is complete
    if complete:
        assert not data.get("fileErrors", [])
    files = data["files"]
    assert len(files) == count
    names = [x["relative"] for x in files]
    assert len(set(names)) == count
    for item in files:
        original = base64.b64decode(item["base64"], validate=True)
        assert len(original) == item["bytes"] and digest(original) == item["sha256"]
        assert original == payload(ROOT / destination / safe(item["relative"]), item)
        if stable:
            assert item["stableLength"] is True
    return data

manifest()

old = export("public-readable-raw/original-empty-host-recovery.json", "old-recovery-originals", True, 10)
new = export("public-readable-raw/final-same-sid-export.json", "repaired-originals", True, 25, True)
assert old["managerRawExit"] == 0 and old["originalHostExitCode"] == 0
assert old["nativeWaitConfirmed"] is True and old["forced"] is False
assert old["originalACLChanged"] is False and old["originalWitness"]["nativeOwnerAdopted"] is False
assert old["lookupSnapshotOnly"] is True and old["originalFailureSceneRetained"] is True
assert old["originalWitness"]["pid"] == 748
assert old["originalWitness"]["filetime"] == "134391474259863429"
assert old["originalWitness"]["noJob"] is True
launch = load("public-readable-raw/launch-result.json")
assert launch["rawTestExit"] == 0 and launch["hComponentPassed"] is True
assert launch["elevated"] is False and launch["session"] == 7 and launch["accepted"] is False
bootstrap = load("repaired-originals/public/bootstrap-admission.json")
assert bootstrap["noJob"] is True and bootstrap["identity"]["elevated"] is False
suspended = load("repaired-originals/public/suspended-original.json")
assert suspended["beforeResumeNoJob"] is True and suspended["bootstrapStillNoJob"] is True
assert suspended["originalCreationHandleRetained"] is True
identity = suspended["identity"]
assert identity["pid"] == 22216 and identity["creationFiletime"] == "134391493883001324"
assert identity["sid"] == launch["sid"] and identity["session"] == 7
assert identity["fileId"] == "844424931380260"
assert identity["elevated"] is False and identity["elevationType"] == 1
for name, generation, rawexit, forced in [("normal", 1, 0, False), ("hold", 2, 1, True), ("after-failure", 3, 0, False)]:
    after = load("repaired-originals/public/after-" + name + ".json")
    assert after["broker"] == identity
    assert after["brokerStillAlive"] is True and after["noJob"] is True
    assert after["originalCreationHandleRetained"] is True
    assert int(after["managerGeneration"]) == generation and after["managerExit"] == rawexit
    assert after["forcedManagerOnly"] is forced
    assert after["actualGUI"] is False and after["fixtureManager"] is True
    assert after["officialOwnerCreated"] is False and after["allAccepted"] is False
for name, generation in [("normal", 1), ("hold", 2), ("after-failure", 3), ("finish", 4)]:
    manager = load("repaired-originals/D/manager-" + name + ".json")
    assert int(manager["managerGeneration"]) == generation
    assert manager["oldConnectionReplayRefused"] is True
    assert manager["emptyHostExitRequested"] is (generation == 4)
    assert manager["allAccepted"] is False and manager["lifecycleAccepted"] is False
    assert manager["officialOwnerCreated"] is False
summary = load("repaired-originals/public/result.json")
assert summary["brokerExitedExplicitly"] is True
assert summary["sameOriginalBrokerAcrossFixtureManagers"] is True
assert summary["actualGUI"] is False and summary["officialOwnerCreated"] is False
assert summary["allAccepted"] is False and summary["OReplayed"] is False
events = [json.loads(line) for line in (ROOT / "repaired-originals/D/host-events.jsonl").read_text().splitlines()]
assert events[0]["event"] == "host-created" and events[0]["facts"]["identity"] == identity
assert [int(e["facts"]["managerGeneration"]) for e in events if e["event"] == "authenticated-empty-action"] == [1,2,3,4]
assert [int(e["facts"]["managerGeneration"]) for e in events if e["event"] == "same-connection-replay-refused"] == [1,2,3,4]
assert events[-1]["event"] == "empty-host-explicit-exit" and events[-1]["facts"]["forced"] is False
plan = load("cleanup-plan.json")
for item in plan["receipts"]:
    assert digest((ROOT / "public-readable-raw" / safe(item["name"])).read_bytes()) == item["sha256"]
cleanup = load("public-readable-raw/completed-task-cleanup.json")
assert cleanup["complete"] is True and len(cleanup["targets"]) == 2
assert cleanup["originalFailedBaseRetained"] is True and cleanup["accountOrProfileDeleted"] is False
assert cleanup["forcedProcessCleanup"] is False and cleanup["oldRejectedPathsRetried"] is False
for expected, target in zip(plan["targets"], cleanup["targets"]):
    assert target["path"].replace("\\", "/") == "C:/Users/marenben/" + expected["leaf"]
    assert target["removed"] is True and target["archiveVerified"] is True
    assert {x["relative"]:(x["bytes"], x["sha256"]) for x in target["files"]} == {x["relative"]:(x["bytes"], x["sha256"]) for x in expected["files"]}
    assert target["fileCount"] == len(expected["files"])
    assert target["bytes"] == sum(x["bytes"] for x in expected["files"])
assert sum(x["fileCount"] for x in cleanup["targets"]) == 22
assert sum(x["bytes"] for x in cleanup["targets"]) == 98829764
r = load("result.json")
assert r["allAccepted"] is False and r["productionExecuted"] is False
assert r["actualGUI"] is False and r["officialOwnerCreated"] is False
assert r["newEmptyHost"]["rawHostExitCode"] is None
assert r["overallCleanupComplete"] is False
assert len(load("status-matrix.json")["items"]) == 26
print(json.dumps({"verified": True, "scope": "empty-host-H-and-exact-cleanup", "allAccepted": False}))

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
    assert data["sid"] == "S-1-5-21-587408064-856152790-2583328096-1003"
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

export("same-sid-export01.json", "exported-originals", False, 3)
export("same-sid-export02.json", "export02-originals", True, 10, True)
r = load("result.json")
assert r["allAccepted"] is False and r["productionExecuted"] is False
assert r["originalDesktopRawExit"] == 101 and r["firstManagerRawError"] == 32
assert r["originalHandoffPassed"] is False
assert r["sameSIDExport02Complete"] is True
original = load("export02-originals/public/suspended-original.json")
assert original["beforeResumeNoJob"] is True
assert original["identity"]["pid"] == 748
assert original["identity"]["creationFiletime"] == "134391474259863429"
assert original["identity"]["elevated"] is False
assert original["identity"]["session"] == 7
stderr = (ROOT / "export02-originals/D/manager-normal.stderr.log").read_text(encoding="utf-8-sig")
assert "32" in stderr and "unwrap" in stderr
manifests = {}
for group, count in [("native02", 3), ("record01", 1), ("phase01", 2), ("library02", 1), ("product01", None)]:
    run = load(group + "/result.json")
    assert run["exitCode"] == 0 and run["profile"] == "debug"
    if count is not None:
        log = (ROOT / group / "stdout.log").read_text(encoding="utf-8-sig")
        assert "test result: ok. %s passed; 0 failed" % count in log
    else:
        assert run["elapsedMs"] == 143952
    sources = load(group + "/source-manifest.json")
    rust = {}
    for item in sources:
        data = payload(ROOT / group / "executed-source" / safe(item["path"]), item)
        if item["path"].endswith(".rs"):
            rust[item["path"]] = data
    assert len(rust) == 36
    manifests[group] = rust
assert all(x == manifests["product01"] for x in manifests.values())
assert load("final-source-manifest.json")["source"] == "ab87ad1261f927bf77a1c757342989105a40ad0a"
assert len(load("status-matrix.json")["items"]) == 18
print(json.dumps({"verified": True, "scope": "diagnosis-and-preparation-checkpoint", "allAccepted": False}))

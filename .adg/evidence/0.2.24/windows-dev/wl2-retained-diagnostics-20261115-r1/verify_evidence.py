"""Offline archive checks only; no Windows runtime or cleanup operations."""
from pathlib import Path, PurePosixPath
import hashlib, json
ROOT = Path(__file__).resolve().parent

def load(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))

def sha(data):
    return hashlib.sha256(data).hexdigest()

def safe(name):
    p = PurePosixPath(name)
    assert name and not p.is_absolute() and "\\" not in name and ":" not in name
    assert p.as_posix() == name and all(x not in ("", ".", "..") for x in p.parts)
    return p

def payload(file, item):
    data = file.read_bytes()
    assert len(data) == item["bytes"] and sha(data) == item["sha256"], str(file)
    return data

m = load("evidence-manifest.json")
names = [x["path"] for x in m["files"]]
assert len(names) == len(set(names))
assert set(names) == {f.relative_to(ROOT).as_posix() for f in ROOT.rglob("*") if f.is_file() and f.name != "evidence-manifest.json"}
for item in m["files"]:
    payload(ROOT / safe(item["path"]), item)
assert len(names) == m["payloadFiles"] and sum(x["bytes"] for x in m["files"]) == m["payloadBytes"]

sources = {}
for group in ["native01", "native02", "library01", "library02", "library03", "library04", "query01", "supervisor01", "product01"]:
    source = load(group + "/source-manifest.json")
    seen = set()
    for item in source:
        assert item["path"] not in seen
        seen.add(item["path"])
        payload(ROOT / group / "executed-source" / safe(item["path"]), item)
    sources[group] = {x["path"]: x["sha256"] for x in source if x["path"].endswith(".rs")}
    assert len(sources[group]) == 36
for group in ["query01", "library04", "supervisor01"]:
    assert sources[group] == sources["product01"]
changed = {k for k, v in sources["native02"].items() if sources["product01"].get(k) != v}
assert changed == {"apps/desktop/tauri/src/infrastructure/windows_supervisor024.rs", "apps/desktop/tauri/src/infrastructure/windows_supervisor024_tests.rs"}
assert all(v["standaloneNativeCompileScopeByteIdentical"] for v in load("execution-source-comparisons.json")["runs"])
for group, count in [("native02", 2), ("library04", 5), ("query01", 1), ("supervisor01", 7)]:
    assert load(group + "/result.json")["exitCode"] == 0
    log = (ROOT / group / "stdout.log").read_text(encoding="utf-8-sig")
    assert f"test result: ok. {count} passed; 0 failed" in log
for group in ["native01", "library01", "library02", "library03"]:
    assert load(group + "/result.json")["exitCode"] == 101
assert "code: 31" in (ROOT / "native01/stderr.log").read_text(encoding="utf-8-sig")
assert "failed fixture retained" in (ROOT / "native01/stderr.log").read_text(encoding="utf-8-sig")
assert "E0599" in (ROOT / "library02/stderr.log").read_text(encoding="utf-8-sig")
assert "rawExit=Some(1)" in (ROOT / "library03/stdout.log").read_text(encoding="utf-8-sig")
assert "rawExit=Some(0)" in (ROOT / "library04/stdout.log").read_text(encoding="utf-8-sig")
query = (ROOT / "query01/stdout.log").read_text(encoding="utf-8-sig")
assert "sc.exe 1060" in query and "schtasks.exe 1" in query
assert "role=Unknown(no official source)" in query
f = load("native01/failed-fixture-manifest.json")
assert f["count"] == 7 and f["bytes"] == 670767 and f["archiveVerified"] is True
assert f["deleted"] is False and f["nativeNormalExitAccepted"] is False
for item in f["files"]:
    payload(ROOT / "native01/failed-fixture-originals" / safe(item["path"]), item)
c = load("cleanup-native01-delete-rejected.json")
assert c["automaticApprovalReview"] == "rejected" and c["commandExecuted"] is False
assert c["retried"] is False and c["alternateDeletionUsed"] is False and c["rootRetained"] is True
assert load("product01/result.json")["exitCode"] == 0
assert load("product01/result.json")["profile"] == "debug"
r = load("result.json")
assert r["allAccepted"] is False and r["productionExecuted"] is False
assert r["actualGUI"] is False and r["overallCleanupComplete"] is False
assert r["uniquePassedTests"] == 12 and r["totalSuccessfulTestInvocations"] == 15
assert r["W2"] is False and r["dualSID"] is False and r["published"] is False
assert len(load("status-matrix.json")["items"]) == 23
print(json.dumps({"verified": True, "scope": "retained-native-and-S-D-directed-evidence", "allAccepted": False}))

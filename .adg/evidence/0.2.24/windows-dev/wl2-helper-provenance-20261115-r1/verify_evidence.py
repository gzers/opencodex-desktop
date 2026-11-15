"""Offline only: verify immutable bytes and scope; no runtime or cleanup actions."""
from pathlib import Path, PurePosixPath
import hashlib,json
ROOT=Path(__file__).resolve().parent

def load(n):return json.loads((ROOT/n).read_text(encoding="utf-8-sig"))
def sha(b):return hashlib.sha256(b).hexdigest()
def safe(n):
    p=PurePosixPath(n)
    assert n and not p.is_absolute() and "\\" not in n and ":" not in n
    assert p.as_posix()==n and all(v not in ("", ".", "..") for v in p.parts)
    return p

def payload(p,r):
    b=p.read_bytes();assert len(b)==r["bytes"] and sha(b)==r["sha256"],str(p)
    return b

m=load("evidence-manifest.json");names=[i["path"] for i in m["files"]]
assert len(names)==len(set(names)) and len(names)==m["payloadFiles"]
assert set(names)=={f.relative_to(ROOT).as_posix() for f in ROOT.rglob("*") if f.is_file() and f.name!="evidence-manifest.json"}
for i in m["files"]:payload(ROOT/safe(i["path"]),i)
assert sum(i["bytes"] for i in m["files"])==m["payloadBytes"]
source={}
for group,count in [("library01",36),("originals01",37),("originals02",37),("product01",37)]:
    rows=load(group+"/source-manifest.json");seen=set();rust={}
    for item in rows:
        assert item["path"] not in seen;seen.add(item["path"])
        b=payload(ROOT/group/"executed-source"/safe(item["path"]),item)
        if item["path"].endswith(".rs"):rust[item["path"]]=b
    assert len(rust)==count;source[group]=rust
    assert load(group+"/result.json")["exitCode"]==0
    assert load(group+"/result.json")["profile"]=="debug"
for group in ["library01","originals01","originals02"]:
    assert "test result: ok. 4 passed; 0 failed" in (ROOT/group/"stdout.log").read_text(encoding="utf-8-sig")
assert source["originals02"]==source["product01"]
helper="apps/desktop/tauri/src/infrastructure/windows_diagnostic_roles024.rs"
assert source["library01"][helper]==source["product01"][helper]
for item in load("final-source-manifest.json")["files"]:
    data=payload(ROOT/"final-source"/safe(item["path"]),item)
    assert data==source["product01"][item["path"]]
phase=source["product01"]["apps/desktop/tauri/src/infrastructure/windows_client_originals024/phase.rs"].decode("utf-8-sig")
assert "binding: Binding" in phase and "check_prefix_for_sid(&records, Some(&sid))" in phase
assert "session_dir.file_name()" in phase
assert "C:/task//H" in phase and "C:/task/H/" in phase
assert "terminal_record_observed: closed" in phase
binding=source["product01"]["apps/desktop/tauri/src/infrastructure/windows_client_originals024/binding.rs"].decode("utf-8-sig")
assert "deny_unknown_fields" in binding and "current_digest: String" in binding
assert ".components()" in binding and ".collect()" in binding
assert load("official-source-manifest.json")["innerSha256"]=="b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c"
for item in load("official-source-manifest.json")["files"]:payload(ROOT/"official-source"/safe(item["path"]),item)
launch=(ROOT/"official-source/src/service/health-sentinel-launch.ts").read_text(encoding="utf-8-sig")
assert 'argv: [input.execPath, input.entryPath]' in launch and 'if (!input.managedService)' in launch
priority=(ROOT/"official-source/src/service/windows-process-priority.ts").read_text(encoding="utf-8-sig")
assert 'deps.setPriority(0,' in priority
r=load("result.json")
assert r["allAccepted"] is False and r["productionExecuted"] is False and r["actualGUI"] is False
assert r["uniquePassedTests"]==8 and r["successfulTestInvocations"]==12
assert r["fullSDRetests"]==0 and r["officialStarts"]==0
assert r["overallCleanupComplete"] is False and r["rejectedCleanupRetried"] is False
assert r["W2"] is False and r["dualSID"] is False and r["published"] is False
assert len(load("status-matrix.json")["items"])==15
print(json.dumps({"verified":True,"scope":"helper-provenance-and-strict-pending-binding","allAccepted":False}))

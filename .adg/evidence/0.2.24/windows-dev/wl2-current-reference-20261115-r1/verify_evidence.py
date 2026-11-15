"""Offline evidence consistency only. Does not perform any native CAS/action."""
from pathlib import Path,PurePosixPath
import hashlib,json
ROOT=Path(__file__).resolve().parent

def load(n):return json.loads((ROOT/n).read_text(encoding="utf-8-sig"))
def safe(n):
    p=PurePosixPath(n)
    assert n and not p.is_absolute() and "\\" not in n and ":" not in n
    assert p.as_posix()==n and all(x not in ("", ".", "..") for x in p.parts)
    return p

def payload(p,i):
    b=p.read_bytes();assert len(b)==i["bytes"] and hashlib.sha256(b).hexdigest()==i["sha256"],str(p)
    return b

m=load("evidence-manifest.json");names=[i["path"] for i in m["files"]]
assert len(names)==len(set(names)) and len(names)==m["payloadFiles"]
assert set(names)=={f.relative_to(ROOT).as_posix() for f in ROOT.rglob("*") if f.is_file() and f.name!="evidence-manifest.json"}
for i in m["files"]:payload(ROOT/safe(i["path"]),i)
assert sum(i["bytes"] for i in m["files"])==m["payloadBytes"]
sources={}
for group,count in [("library01",7),("library02",8),("product01",None)]:
    r=load(group+"/result.json");assert r["exitCode"]==0 and r["profile"]=="debug"
    if count is not None:assert f"test result: ok. {count} passed; 0 failed" in (ROOT/group/"stdout.log").read_text(encoding="utf-8-sig")
    rows=load(group+"/source-manifest.json");rust={};seen=set()
    for i in rows:
        assert i["path"] not in seen;seen.add(i["path"])
        b=payload(ROOT/group/"executed-source"/safe(i["path"]),i)
        if i["path"].endswith(".rs"):rust[i["path"]]=b
    assert len(rust)==39;sources[group]=rust
assert sources["library02"]==sources["product01"]
for i in load("final-source-manifest.json")["files"]:
    assert payload(ROOT/"final-source"/safe(i["path"]),i)==(ROOT/"product01/executed-source"/safe(i["path"])).read_bytes()
src=load("product01/source.json")
assert src["head"]==load("result.json")["source"] and not src["diff"]
log=(ROOT/"library02/stdout.log").read_text(encoding="utf-8-sig")
for fragment in ["same current-reference fileID retained", "stale pointer/exclusive writer refused", "phase preserved", "no reconstructed permission", "foreign-directory create leaves no pointer", "duplicate and unknown current fields refused"]:
    assert fragment in log
ref=sources["product01"]["apps/desktop/tauri/src/infrastructure/windows_client_originals024/reference.rs"].decode("utf-8-sig")
assert "struct CurrentReference(Reference<Full>)" in ref
assert "FULL_DESCRIPTOR_QUERY_ACCESS" in ref and "descriptor(file)" in ref
assert ref.index("#[cfg(test)]")<ref.index("struct PartialDiagnostic")
assert ".share_mode(0)" in ref and "self.poisoned = true" in ref
phase=sources["product01"]["apps/desktop/tauri/src/infrastructure/windows_client_originals024/phase.rs"].decode("utf-8-sig")
assert "fn append_pending" in phase and "terminal_record_observed: closed" in phase
r=load("result.json")
assert r["allAccepted"] is False and r["productionExecuted"] is False and r["actualGUI"] is False
assert r["uniquePassedTests"]==8 and r["successfulTestInvocations"]==15
assert r["fullSecurityProductionPositive"] is False and r["partialDiagnosticCASMechanismOnly"] is True
assert r["fullSDRetests"]==0 and r["officialStarts"]==0
assert r["overallCleanupComplete"] is False and r["rejectedCleanupRetried"] is False
assert r["W2"] is False and r["dualSID"] is False and r["published"] is False
assert len(load("status-matrix.json")["items"])==13
print(json.dumps({"verified":True,"scope":"same-object-current-reference-and-pending-phase-mechanism","allAccepted":False}))

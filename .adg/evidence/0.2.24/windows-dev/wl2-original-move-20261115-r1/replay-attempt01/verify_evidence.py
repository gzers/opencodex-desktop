"""Offline archive verifier; never opens native move handles or runs cleanup."""
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
assert set(names)=={p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*") if p.is_file() and p.name!="evidence-manifest.json"}
for item in m["files"]:payload(ROOT/safe(item["path"]),item)
assert sum(x["bytes"] for x in m["files"])==m["payloadBytes"]
sources={}
for group,count in [("native01",2),("native02",1),("native03",4),("product01",None)]:
    run=load(group+"/result.json");assert run["exitCode"]==0 and run["profile"]=="debug"
    if count is not None:
        assert f"test result: ok. {count} passed; 0 failed" in (ROOT/group/"stdout.log").read_text(encoding="utf-8-sig")
    rows=load(group+"/source-manifest.json");rust={};seen=set()
    for item in rows:
        assert item["path"] not in seen;seen.add(item["path"])
        b=payload(ROOT/group/"executed-source"/safe(item["path"]),item)
        if item["path"].endswith(".rs"):rust[item["path"]]=b
    assert len(rust)==38;sources[group]=rust
assert sources["native03"]==sources["product01"]
for item in load("final-source-manifest.json")["files"]:
    b=payload(ROOT/"final-source"/safe(item["path"]),item)
    assert b==sources["product01"][item["path"]]
source=load("product01/source.json")
assert source["head"]==load("result.json")["source"] and not source["diff"]
log=(ROOT/"native03/stdout.log").read_text(encoding="utf-8-sig")
assert "native no-overwrite error raw=Some(183)" in log
assert "native restore occupied error raw=Some(183)" in log
assert "same native file object restored" in log
assert "sourceVolume=247287385 targetVolume=3089487301" in log
assert "copyFallback=false" in log
assert "same original native parent cannot mark a custody transition" in log
assert "ADS/hardlink/invalid target/expired-window refused" in log
r=load("result.json")
assert r["allAccepted"] is False and r["productionExecuted"] is False and r["actualGUI"] is False
assert r["OriginalSessionIssued"] is False and r["ClosedEligibilityIssued"] is False
assert r["uniquePassedTests"]==4 and r["successfulTestInvocations"]==7
assert r["firstError"] is None and r["fullSDRetests"]==0 and r["officialStarts"]==0
assert r["overallCleanupComplete"] is False and r["rejectedCleanupRetried"] is False
assert r["W2"] is False and r["dualSID"] is False and r["published"] is False
assert len(load("status-matrix.json")["items"])==14
assert load("review-and-provenance-limits.json")["allAccepted"] is False
print(json.dumps({"verified":True,"scope":"native-single-file-move-only","allAccepted":False}))

"""Offline evidence only; no Windows runtime, token, policy or cleanup operations."""
from pathlib import Path, PurePosixPath
import hashlib, json
ROOT=Path(__file__).resolve().parent

def load(name): return json.loads((ROOT/name).read_text(encoding="utf-8-sig"))
def sha(data): return hashlib.sha256(data).hexdigest()
def safe(name):
    p=PurePosixPath(name)
    assert name and not p.is_absolute() and "\\" not in name and ":" not in name
    assert p.as_posix()==name and all(x not in ("", ".", "..") for x in p.parts)
    return p

def payload(path,item):
    b=path.read_bytes();assert len(b)==item["bytes"] and sha(b)==item["sha256"],str(path)
    return b

m=load("evidence-manifest.json");names=[x["path"] for x in m["files"]]
assert len(names)==len(set(names))
assert set(names)=={f.relative_to(ROOT).as_posix() for f in ROOT.rglob("*") if f.is_file() and f.name!="evidence-manifest.json"}
for item in m["files"]:payload(ROOT/safe(item["path"]),item)
assert len(names)==m["payloadFiles"] and sum(x["bytes"] for x in m["files"])==m["payloadBytes"]
for group,count in [("native01",1),("query01",1),("library01",1)]:
    assert load(group+"/result.json")["exitCode"]==0
    assert f"test result: ok. {count} passed; 0 failed" in (ROOT/group/"stdout.log").read_text(encoding="utf-8-sig")
assert load("product01/result.json")["exitCode"]==0 and load("product01/result.json")["profile"]=="debug"
query=(ROOT/"query01/stdout.log").read_text(encoding="utf-8-sig")
assert "sc.exe 1060" in query and "schtasks.exe 1" in query and "role=Unknown(no official source)" in query
for group in ["native01","query01","library01","product01"]:
    source=load(group+"/source-manifest.json");seen=set()
    for item in source:
        assert item["path"] not in seen;seen.add(item["path"])
        payload(ROOT/group/"executed-source"/safe(item["path"]),item)
    assert sum(x["path"].endswith(".rs") for x in source)==36
final=load("final-source-manifest.json")
for item in final["files"]:payload(ROOT/"final-source"/safe(item["path"]),item)
for row in load("execution-source-comparisons.json")["runs"]:
    assert row["nativeCompileScopeByteIdentical"] is True
    if row["run"]=="product01":assert row["all36RustFinalByteIdentical"] is True
relative="apps/desktop/tauri/src/infrastructure/windows_diagnostic_roles024.rs"
def scope(path):
    s=path.read_text(encoding="utf-8-sig").replace("\r\n","\n")
    a=s.index("fn exact_root(");b=s.index("fn root_source(",a);part=s[a:b]
    a=s.index("    fn wl2_family_original_node_bun_grammar");b=s.index("    #[test]",a)
    return (part+s[a:b]).encode("utf-8")
old=scope(ROOT/"library01/executed-source"/relative);new=scope(ROOT/"final-source"/relative)
assert old==new and sha(old)==load("reused-pure-scope.json")["scopeSha256"]
assert load("official-source-manifest.json")["innerSha256"]=="b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c"
for item in load("official-source-manifest.json")["files"]:payload(ROOT/"official-source"/safe(item["path"]),item)
bin=(ROOT/"official-source/bin/ocx.mjs").read_text(encoding="utf-8-sig")
assert 'const NODE_LAUNCH_PROOF_PREFIX = "--ocx-internal-launch-proof="' in bin
assert 'randomBytes(32).toString("base64url")' in bin
assert 'probeSpawn(bun, [cliPath,' in bin
r=load("result.json")
assert r["allAccepted"] is False and r["productionExecuted"] is False and r["actualGUI"] is False
assert r["directedTestsPassed"]==3 and r["productionRoleRuntimePositive"] is False
assert r["ordinarySDRetests"]==0 and r["officialStarts"]==0
assert r["overallCleanupComplete"] is False and r["rejectedCleanupRetried"] is False
assert r["W2"] is False and r["dualSID"] is False and r["published"] is False
assert len(load("status-matrix.json")["items"])==16
print(json.dumps({"verified":True,"scope":"native-root-argv-and-source-bound-roles-increment","allAccepted":False}))

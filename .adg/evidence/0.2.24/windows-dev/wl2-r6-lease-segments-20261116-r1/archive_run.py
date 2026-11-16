from pathlib import Path
import hashlib,json,re,sys,zipfile
ROOT=Path(__file__).resolve().parent
CODE=Path(r"C:/Users/15119/.codex/worktrees/024-windows-native/code")
name=sys.argv[1]
assert re.fullmatch(r"library\d\d",name)
load=lambda p:json.loads(p.read_text(encoding="utf-8-sig"))
sha=lambda b:hashlib.sha256(b).hexdigest()
result=load(ROOT/name/"result.json")
text="".join((ROOT/name/f"{s}.log").read_text(encoding="utf-8-sig") for s in ["stdout","stderr"])
exe=CODE/"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/build/opencodex-desktop/b424977fd6e5186d/out/opencodex_desktop_lib-b424977fd6e5186d.exe"
executed="test result:" in text
artifact=dict(executed=executed,rawExit=result["exitCode"],accepted=False)
if executed:
    raw=exe.read_bytes()
    if exe.stat().st_mtime < (ROOT/name/"source.json").stat().st_mtime:
        prior=load(ROOT/"library02-artifact.json")
        assert sha(raw)==prior["sha256"]
        assert load(ROOT/name/"source-manifest.json")==load(ROOT/"library02/source-manifest.json")
        assert load(ROOT/name/"source.json")["arguments"]==load(ROOT/"library02/source.json")["arguments"]
        artifact["reusedExactArtifactFrom"]="library02"
    artifact.update(path=str(exe),bytes=len(raw),sha256=sha(raw))
else:
    artifact["reason"]="compile failed; no new executable attributed to this run"
with (ROOT/f"{name}-artifact.json").open("x",encoding="utf-8") as out:json.dump(artifact,out,indent=2)
scenes=[]
for i,path in enumerate(dict.fromkeys(re.findall(r"r4-retained-synthetic-fixture=([^\r\n]+)",text))):
    source=Path(path);target=CODE/f"test/windows024-native/target/r4-private-scenes/r6-segments-{name}-{i}.zip"
    files=[]
    with zipfile.ZipFile(target,"x",compression=zipfile.ZIP_DEFLATED) as archive:
        for p in sorted(source.rglob("*")):
            if p.is_file():
                raw=p.read_bytes();rel=p.relative_to(source).as_posix();archive.writestr(rel,raw)
                files.append(dict(path=rel,bytes=len(raw),sha256=sha(raw)))
    with zipfile.ZipFile(target) as archive:assert all(sha(archive.read(f["path"]))==f["sha256"] for f in files)
    scenes.append(dict(source=path,archive=str(target),archiveSha256=sha(target.read_bytes()),files=files,removed=False))
with (ROOT/f"{name}-private-scenes.json").open("x",encoding="utf-8") as out:
    json.dump(dict(scenes=scenes,deletionAttempted=False,nativeMetadataCloned=False),out,indent=2)
print(json.dumps(dict(run=name,executed=executed,scenes=len(scenes),files=sum(len(s["files"]) for s in scenes))))




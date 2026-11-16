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
executed="test result:" in text
artifact=dict(executed=executed,rawExit=result["exitCode"],accepted=False,profile=result["profile"])
if executed:
    candidates=re.findall(r"Running unittests[^\r\n]*\(([^\r\n]+\.exe)\)",text)
    if not candidates and (ROOT/name/'run-command.json').exists():candidates=[load(ROOT/name/'run-command.json')['program']]
    assert len(candidates)==1,candidates
    exe=Path(candidates[0].replace("\\","/"));exe=exe if exe.is_absolute() else CODE/exe
    exe=exe.resolve();assert exe.is_relative_to(CODE/"test/windows024-native/target/w1c-product")
    assert f"/{result['profile']}/" in exe.as_posix()
    raw=exe.read_bytes()
    if exe.stat().st_mtime < (ROOT/name/"source.json").stat().st_mtime:
        base="library01"
        prior=load(ROOT/f"{base}-artifact.json")
        assert sha(raw)==prior["sha256"]
        actual_inputs=load(ROOT/name/"source-manifest.json");prior_inputs=load(ROOT/base/"source-manifest.json")
        assert [r for r in actual_inputs if r["path"].endswith(".rs")]==[r for r in prior_inputs if r["path"].endswith(".rs")]
        assert result["profile"]==load(ROOT/base/"result.json")["profile"]
        actual_args=load(ROOT/name/"source.json")["arguments"];prior_args=load(ROOT/base/"source.json")["arguments"]
        assert actual_args[:actual_args.index('-j')]==prior_args[:prior_args.index('-j')]
        artifact["sameRustInputsAndCompilationFlags"]=True
        artifact["reusedExactArtifactFrom"]=base
        artifact["arguments"]=load(ROOT/name/"source.json")["arguments"]
    artifact.update(path=str(exe),bytes=len(raw),sha256=sha(raw))
else:
    artifact["reason"]="compile failed; no new executable attributed to this run"
with (ROOT/f"{name}-artifact.json").open("x",encoding="utf-8") as out:json.dump(artifact,out,indent=2)
scenes=[]
for i,path in enumerate(dict.fromkeys(re.findall(r"r4-retained-synthetic-fixture=([^\r\n]+)",text))):
    source=Path(path);target=CODE/f"test/windows024-native/target/r4-private-scenes/r6-capture-ready-{name}-{i}.zip"
    files=[]
    with zipfile.ZipFile(target,"x",compression=zipfile.ZIP_DEFLATED) as archive:
        for p in sorted(source.rglob("*")):
            if p.is_file():
                raw=p.read_bytes();rel=p.relative_to(source).as_posix();archive.writestr(rel,raw)
                files.append(dict(path=rel,bytes=len(raw),sha256=sha(raw)))
        # Only this declared task-created ADS is copied as observation data.
        # ZIP does not recreate NTFS security/file IDs/stream/link semantics.
        ads=source/"Profile/.codex/opencodex-catalog.json:synthetic-extra"
        try: raw=ads.read_bytes()
        except FileNotFoundError:pass
        else:
            rel="_observations/known-work-catalog-synthetic-extra-ADS.bin";archive.writestr(rel,raw)
            files.append(dict(path=rel,source=str(ads),bytes=len(raw),sha256=sha(raw),alternateStreamObservationOnly=True))
    with zipfile.ZipFile(target) as archive:assert all(sha(archive.read(f["path"]))==f["sha256"] for f in files)
    scenes.append(dict(source=path,archive=str(target),archiveSha256=sha(target.read_bytes()),files=files,removed=False))
with (ROOT/f"{name}-private-scenes.json").open("x",encoding="utf-8") as out:
    json.dump(dict(scenes=scenes,deletionAttempted=False,nativeMetadataCloned=False),out,indent=2)
print(json.dumps(dict(run=name,executed=executed,scenes=len(scenes),files=sum(len(s["files"]) for s in scenes))))











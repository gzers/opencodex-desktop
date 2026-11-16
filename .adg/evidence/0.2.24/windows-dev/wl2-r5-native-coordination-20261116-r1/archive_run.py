from pathlib import Path
import hashlib,json,re,sys,zipfile
ROOT=Path(__file__).resolve().parent
CODE=Path(r"C:/Users/15119/.codex/worktrees/024-windows-native/code")
name=sys.argv[1];assert name in ["library01","library02","library03","library04"]
load=lambda p:json.loads(p.read_text(encoding="utf-8-sig"))
sha=lambda b:hashlib.sha256(b).hexdigest()
exe=CODE/"test/windows024-native/target/w1c-product/x86_64-pc-windows-msvc/debug/build/opencodex-desktop/b424977fd6e5186d/out/opencodex_desktop_lib-b424977fd6e5186d.exe"
raw=exe.read_bytes()
with (ROOT/f"{name}-artifact.json").open("x",encoding="utf-8") as out:
    json.dump(dict(path=str(exe),bytes=len(raw),sha256=sha(raw),rawExit=load(ROOT/name/"result.json")["exitCode"],accepted=False),out,indent=2)
text="".join((ROOT/name/f"{s}.log").read_text(encoding="utf-8-sig") for s in ["stdout","stderr"])
scenes=[]
for i,path in enumerate(dict.fromkeys(re.findall(r"r4-retained-synthetic-fixture=([^\r\n]+)",text))):
    source=Path(path);target=CODE/f"test/windows024-native/target/r4-private-scenes/r5-coordination-{name}-{i}.zip"
    files=[]
    with zipfile.ZipFile(target,"x",compression=zipfile.ZIP_DEFLATED) as archive:
        for p in sorted(source.rglob("*")):
            if p.is_file():
                raw=p.read_bytes();rel=p.relative_to(source).as_posix();archive.writestr(rel,raw)
                files.append(dict(path=rel,bytes=len(raw),sha256=sha(raw)))
        # rglob does not enumerate NTFS streams. This exact synthetic stream
        # is a declared fixture, copied separately as observation data only.
        ads=source/"private-locks/synthetic-2.sqlite:foreign"
        try: raw=ads.read_bytes()
        except FileNotFoundError: pass
        else:
            rel="_observations/known-synthetic-2-foreign-ADS.bin";archive.writestr(rel,raw)
            files.append(dict(path=rel,source=str(ads),bytes=len(raw),sha256=sha(raw),alternateStreamObservationOnly=True))
    with zipfile.ZipFile(target) as archive:assert all(sha(archive.read(f["path"]))==f["sha256"] for f in files)
    scenes.append(dict(source=path,archive=str(target),archiveSha256=sha(target.read_bytes()),files=files,removed=False))
with (ROOT/f"{name}-private-scenes.json").open("x",encoding="utf-8") as out:
    json.dump(dict(scenes=scenes,deletionAttempted=False,nativeMetadataCloned=False),out,indent=2)
print(json.dumps(dict(run=name,scenes=len(scenes),files=sum(len(s["files"]) for s in scenes),bytes=sum(f["bytes"] for s in scenes for f in s["files"]))))

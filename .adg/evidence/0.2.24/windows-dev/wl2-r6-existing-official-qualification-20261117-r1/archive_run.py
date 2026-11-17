from pathlib import Path
import hashlib,json,re,sys,zipfile
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
name=sys.argv[1]
assert re.fullmatch(r'library\d\d',name)
result=load(ROOT/name/'result.json')
logs=''.join((ROOT/name/f'{s}.log').read_text(encoding='utf-8-sig') for s in ['stdout','stderr'])
executed='test result:' in logs
artifact=dict(executed=executed,rawExit=result['exitCode'],profile=result['profile'],accepted=False)
if executed:
    candidates=re.findall(r'Running unittests[^\r\n]*\(([^\r\n]+\.exe)\)',logs)
    if not candidates and (ROOT/name/'run-command.json').exists():candidates=[load(ROOT/name/'run-command.json')['program']]
    assert len(candidates)==1,candidates
    exe=Path(candidates[0].replace('\\','/'));exe=exe if exe.is_absolute() else CODE/exe
    exe=exe.resolve();assert exe.is_relative_to(CODE/'test/windows024-native/target/w1c-product')
    assert f"/{result['profile']}/" in exe.as_posix()
    raw=exe.read_bytes()
    artifact.update(path=str(exe),bytes=len(raw),sha256=sha(raw))
    artifact['buildInputTimeBeforeExe']=exe.stat().st_mtime >= (ROOT/name/'source.json').stat().st_mtime
    if not artifact['buildInputTimeBeforeExe']:
        reuse=load(ROOT/name/'run-command.json');prior=load(ROOT/f"{reuse['producer']}-artifact.json")
        assert prior['sha256']==sha(raw)
        artifact['reusedExactArtifactFrom']=reuse['producer']
else:artifact['reason']='no new executable attributed; see original raw compile result'
artifact_file=ROOT/f'{name}-artifact.json'
if artifact_file.exists():assert load(artifact_file)==artifact
else:
    with artifact_file.open('x',encoding='utf-8') as out:json.dump(artifact,out,indent=2)
scenes=[]
for i,path in enumerate(dict.fromkeys(re.findall(r'r4-retained-synthetic-fixture=([^\r\n]+)',logs))):
    source=Path(path).resolve()
    assert any(source.is_relative_to(p) for p in [Path('C:/Users/15119/AppData/Local/Temp'),Path('C:/Users/15119/.codex/worktrees/024-windows-native/code/test/windows024-native/target'),Path('D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/test/windows024-native/target')]),source
    target=CODE/f'test/windows024-native/target/r4-private-scenes/r6-existing-official-qualification-{name}-{i}.zip'
    files=[];directories=[]
    with zipfile.ZipFile(target,'x',compression=zipfile.ZIP_DEFLATED) as archive:
        for p in sorted(source.rglob('*')):
            if p.is_dir():directories.append(p.relative_to(source).as_posix())
            elif p.is_file():
                raw=p.read_bytes();rel=p.relative_to(source).as_posix();archive.writestr(rel,raw)
                files.append(dict(path=rel,bytes=len(raw),sha256=sha(raw)))
    with zipfile.ZipFile(target) as archive:
        assert sorted(archive.namelist())==sorted(f['path'] for f in files)
        assert all(sha(archive.read(f['path']))==f['sha256'] for f in files)
    scenes.append(dict(source=str(source),archive=str(target),archiveSha256=sha(target.read_bytes()),files=files,directories=directories,removed=False))
with (ROOT/f'{name}-private-scenes.json').open('x',encoding='utf-8') as out:
    json.dump(dict(scenes=scenes,deletionAttempted=False,nativeMetadataCloned=False),out,indent=2)
print(json.dumps(dict(run=name,executed=executed,rawExit=result['exitCode'],scenes=len(scenes),files=sum(len(s['files']) for s in scenes))))

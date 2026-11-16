from pathlib import Path
import hashlib,json,re,sys,zipfile
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
sha=lambda b:hashlib.sha256(b).hexdigest()
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
name=sys.argv[1];assert name in ['profile01','profile02']
result=load(ROOT/name/'result.json');producer=result['producer'];manifest=load(ROOT/producer/'source-manifest.json')
for row in manifest:
    raw=(ROOT/producer/'executed-source'/row['path']).read_bytes();assert sha(raw)==row['sha256'] and len(raw)==row['bytes'];assert raw==(CODE/row['path']).read_bytes()
program=Path(result['program']);assert sha(program.read_bytes())==result['programSha256']
with (ROOT/f'{name}-source-link.json').open('x',encoding='utf-8') as out:json.dump(dict(producer=producer,producerSourceManifest=f'{producer}/source-manifest.json',
    producerSourceManifestSha256=sha((ROOT/producer/'source-manifest.json').read_bytes()),inputs=manifest,sourceInputsByteIdentical=True,program=str(program),
    programSha256=result['programSha256'],actualArguments=result['arguments'],rawExit=result['rawExit'],newCompilation=False,diagnosticOnly=True),out,indent=2)
logs=''.join((ROOT/name/f'{s}.log').read_text(encoding='utf-8-sig') for s in ['stdout','stderr']);scenes=[]
for i,path in enumerate(dict.fromkeys(re.findall(r'r4-retained-synthetic-fixture=([^\r\n]+)',logs))):
    source=Path(path).resolve();assert any(source.is_relative_to(p) for p in [Path('C:/Users/15119/AppData/Local/Temp'),Path('D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/test/windows024-native/target')])
    target=CODE/f'test/windows024-native/target/r4-private-scenes/r6-completed-history-{name}-{i}.zip';files=[];directories=[]
    with zipfile.ZipFile(target,'x',compression=zipfile.ZIP_DEFLATED) as archive:
        for p in sorted(source.rglob('*')):
            rel=p.relative_to(source).as_posix()
            if p.is_dir():directories.append(rel)
            elif p.is_file():
                raw=p.read_bytes();archive.writestr(rel,raw);files.append(dict(path=rel,bytes=len(raw),sha256=sha(raw)))
    with zipfile.ZipFile(target) as archive:
        assert sorted(archive.namelist())==sorted(f['path'] for f in files)
        assert all(sha(archive.read(f['path']))==f['sha256'] for f in files)
    scenes.append(dict(source=str(source),archive=str(target),archiveSha256=sha(target.read_bytes()),files=files,directories=directories,removed=False))
with (ROOT/f'{name}-private-scenes.json').open('x',encoding='utf-8') as out:json.dump(dict(scenes=scenes,deletionAttempted=False,nativeMetadataCloned=False),out,indent=2)
print(json.dumps(dict(run=name,scenes=len(scenes),rawExit=result['rawExit'],sourceVerified=True,exeVerified=True,newBuild=False)))

from pathlib import Path
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parent
rows=[]
for index,name in enumerate(['.tmpErP935','.tmpvhhXRI']):
    scene=Path('C:/Users/15119/AppData/Local/Temp')/name
    def snapshot():
        files={p.relative_to(scene).as_posix():dict(bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in scene.rglob('*') if p.is_file()}
        dirs=sorted(p.relative_to(scene).as_posix() for p in scene.rglob('*') if p.is_dir())
        return dict(files=files,directories=dirs)
    before=snapshot();assert not (scene/'H').exists()
    assert before['files']['never-resumed-fixture.exe']['sha256']=='4699667b79d3aebd6e1e90c8b063f80a95b3f32c5c4a69ec49bd4c70217e4a38'
    target=ROOT/f'product03/failure-readonly-{index}.zip'
    with zipfile.ZipFile(target,'x',compression=zipfile.ZIP_DEFLATED) as archive:
        for name in sorted(before['files']):archive.write(scene/name,name)
    with zipfile.ZipFile(target) as archive:
        assert set(archive.namelist())==set(before['files'])
        for path,row in before['files'].items():
            raw=archive.read(path);assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
    assert snapshot()==before
    rows.append(dict(root=str(scene),**before,archive=target.name,archiveSha256=hashlib.sha256(target.read_bytes()).hexdigest(),retained=True))
(ROOT/'product03/failure-receive.json').write_text(json.dumps(dict(scenes=rows,error='raw2 missing privateH before OwnedRuntime CreateJob/CreateProcess; standalone history test did not call original pair helper which used to createH',nativeSourceOrder='process.rs create_suspended opens privateH at line432 before CreateJob442/CreateProcess494; original logs/source retained',cleanup='readonlyZIP/member/hash/dirs beforeafter only; two failedroots retained',currentR2Touched=False),indent=2),encoding='utf-8')
print(json.dumps(dict(archives=len(rows),files=sum(len(r['files']) for r in rows),retained=True)))

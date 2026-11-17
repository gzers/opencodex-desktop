from pathlib import Path
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parent
scene=Path('C:/Users/15119/AppData/Local/Temp/.tmpQ94pON')
def snapshot():
    return dict(files={p.relative_to(scene).as_posix():dict(bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in scene.rglob('*') if p.is_file()},directories=sorted(p.relative_to(scene).as_posix() for p in scene.rglob('*') if p.is_dir()))
before=snapshot();assert len(before['files'])==5 and before['files']['source/original.bin']['bytes']==29
archive=ROOT/'product01/failure-readonly.zip'
with zipfile.ZipFile(archive,'x',compression=zipfile.ZIP_DEFLATED) as out:
    for p in sorted(before['files']):out.write(scene/p,p)
with zipfile.ZipFile(archive) as out:
    assert set(out.namelist())==set(before['files'])
    for p,row in before['files'].items():
        raw=out.read(p);assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
assert snapshot()==before
(ROOT/'product01/failure-receive.json').write_text(json.dumps(dict(root=str(scene),**before,archive='failure-readonly.zip',archiveSha256=hashlib.sha256(archive.read_bytes()).hexdigest(),error='fixture TempDir close raw32 while unvalidated returned original remains in receiver; native query unvalidated refusal already passed',meaning='archive is exact5remaining files AFTER attempted fixture close, not claim of full untouched original scene; no actual childprocess in this source/worker-only fixture',cleanup='readonly archive only, failed root retained/no retry deletion',currentR2Touched=False),indent=2),encoding='utf-8')
print(json.dumps(dict(files=len(before['files']),archived=True,retained=True)))

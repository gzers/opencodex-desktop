from pathlib import Path
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parent
scene=Path('C:/Users/15119/AppData/Local/Temp/.tmpe2DUqC')
def snapshot():return dict(files={p.relative_to(scene).as_posix():dict(bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in scene.rglob('*') if p.is_file()},directories=sorted(p.relative_to(scene).as_posix() for p in scene.rglob('*') if p.is_dir()))
before=snapshot();assert len(before['files'])==4 and all(r['bytes']==0 for r in before['files'].values())
archive=ROOT/'product01/failure-readonly.zip'
with zipfile.ZipFile(archive,'x',compression=zipfile.ZIP_DEFLATED) as out:
    for p in sorted(before['files']):out.write(scene/p,p)
with zipfile.ZipFile(archive) as out:
    assert set(out.namelist())==set(before['files'])
    for p,row in before['files'].items():assert hashlib.sha256(out.read(p)).hexdigest()==row['sha256']
assert snapshot()==before
(ROOT/'product01/failure-receive.json').write_text(json.dumps(dict(root=str(scene),**before,archive='failure-readonly.zip',archiveSha256=hashlib.sha256(archive.read_bytes()).hexdigest(),error='new fixture incorrectly asserted original lease Option None after explicitrelease, existing implementation retains inactive drained shell',alreadyPassed='sameclaim epoch/key and all4actualranges busy between3stage borrows; explicit release/readback returned success',fix='test actual4ranges becomeacquirable+explicitunlock and originalinactivelease claimrefuses; no productionprimitive or budget changed',cleanup='readonly archive/member/hash/dirs beforeafter only, failedroot retained no delete/retry',currentR2Touched=False),indent=2),encoding='utf-8')
print(json.dumps(dict(files=4,archived=True,retained=True)))

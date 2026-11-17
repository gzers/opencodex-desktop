from pathlib import Path
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parent
scene=Path('C:/Users/15119/AppData/Local/Temp/.tmpJEMT1O')
def members():
    return {p.relative_to(scene).as_posix():dict(bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in scene.rglob('*') if p.is_file()}
before=members();assert len(before)==4
target=ROOT/'product02/failure-readonly.zip'
with zipfile.ZipFile(target,'x',compression=zipfile.ZIP_DEFLATED) as out:
    for p in sorted(before):out.write(scene/p,p)
with zipfile.ZipFile(target) as archive:
    assert set(archive.namelist())==set(before)
    for path,row in before.items():
        raw=archive.read(path);assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
assert members()==before
(ROOT/'product02/failure-receive.json').write_text(json.dumps(dict(root=str(scene),files=before,archive='failure-readonly.zip',archiveSha256=hashlib.sha256(target.read_bytes()).hexdigest(),error='new fixture attempted private source file in unprotected temp root; PermissionDenied before any source/worker/process created',cleanup='read-onlyZIP only; root retained, no retry delete',currentR2Touched=False),indent=2),encoding='utf-8')
print(json.dumps(dict(files=len(before),retained=True,archiveVerified=True)))

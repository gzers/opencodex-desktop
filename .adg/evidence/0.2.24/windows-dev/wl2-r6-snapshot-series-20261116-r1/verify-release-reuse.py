from pathlib import Path
import json,hashlib
ROOT=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
prior=load(ROOT/'library04/source-manifest.json');current=load(ROOT/'library05/source-manifest.json')
assert [r for r in prior if r['path'].endswith('.rs')]==[r for r in current if r['path'].endswith('.rs')]
old=load(ROOT/'library04/source.json');new=load(ROOT/'library05/source.json')
assert old['head']==new['head'] and old['rustc']==new['rustc']
a=old['arguments'];b=new['arguments'];assert a[:a.index('-j')]==b[:b.index('-j')]
changes=[]
for r in current:
    before=next(p for p in prior if p['path']==r['path'])
    if r!=before:changes.append(r['path'])
assert changes==['test/windows024-native/build-wl2.ps1'],changes
artifact=load(ROOT/'library04-artifact.json');path=Path(artifact['path']);raw=path.read_bytes()
assert len(raw)==artifact['bytes'] and hashlib.sha256(raw).hexdigest()==artifact['sha256']
assert artifact['profile']=='release'
with (ROOT/'release-executable-reuse-proof.json').open('x',encoding='utf-8') as out:
    json.dump(dict(actualExecutable=str(path),bytes=len(raw),sha256=artifact['sha256'],profile='release',rustInputsEqual=True,compilationFlagsEqual=True,rustcEqual=True,producerSourceHead=old['head'],onlyDifferentInput=changes,originalPassedRun='library04',next='cancel only current redundant compiler after parent PID verification; invoke this exact hash with original regression filters and capture raw exits'),out,indent=2)
print(json.dumps(dict(executable=str(path),sha256=artifact['sha256'],rustInputsEqual=True,onlyDifferentInput=changes)))

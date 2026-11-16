from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
rows=[];scenes=[]
for run in ['library01','library02']:
    current=load(ROOT/f'{run}-private-scenes.json')['scenes'];scenes.extend(current)
    text=(ROOT/run/'stdout.log').read_text(encoding='utf-8-sig')
    for name,body in re.findall(r'test ([^\n]+?) \.\.\. (.*?)(?=\ntest |\nfailures:|\ntest result:|\Z)',text,re.S):
        leaf=name.rsplit('::',1)[-1]
        if not leaf.startswith('wl2_r6_inventory_native_'):continue
        assert body.rstrip().endswith('ok')
        paths=re.findall(r'r4-retained-synthetic-fixture=([^\r\n]+)',body)
        assert len(paths)==(5 if 'exact_before_transfer' in leaf else 1)
        for path in paths:
            row=next(s for s in current if Path(s['source'])==Path(path))
            assert hashlib.sha256(Path(row['archive']).read_bytes()).hexdigest()==row['archiveSha256']
            assert set(row['directories'])==({'inventory'} if 'exact_before_transfer' in leaf else {'inventory-ancestors'})
            assert {f['path'] for f in row['files']}==({'inventory/original.bin','inventory/other.bin'} if 'exact_before_transfer' in leaf else set())
            rows.append(dict(run=run,test=leaf,**row))
assert len(rows)==12
with (ROOT/'success-cleanup-plan.json').open('x',encoding='utf-8') as f:json.dump(rows,f,indent=2)
with (ROOT/'cleanup-counts.json').open('x',encoding='utf-8') as f:
    json.dump(dict(scenes=len(scenes),ownCompletedScenes=len(rows),failedAndPendingRetained=len(scenes)-len(rows),crossVolumePendingRootsRetained=sum(s['source'].startswith('D:') for s in scenes)),f,indent=2)
previous=ROOT.parent/'wl2-r6-recovery-chain-20261116-r1'/'cleanup-completed.ps1'
with (ROOT/'cleanup-completed.ps1').open('xb') as f:f.write(previous.read_bytes())
print(json.dumps(dict(ownCompletedScenes=len(rows),failedPendingRetained=len(scenes)-len(rows))))

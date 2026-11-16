from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
rows=[];scenes=[]
for run in ['library01','library02']:
    current=load(ROOT/f'{run}-private-scenes.json')['scenes'];scenes.extend(current)
    text=(ROOT/run/'stdout.log').read_text(encoding='utf-8-sig')
    for name,body in re.findall(r'test ([^\n]+?) \.\.\. (.*?)(?=\ntest |\nfailures:|\ntest result:|\Z)',text,re.S):
        leaf=name.rsplit('::',1)[-1]
        if not leaf.startswith('wl2_r6_recovery_data_'):continue
        assert body.rstrip().endswith('ok')
        paths=re.findall(r'r4-retained-synthetic-fixture=([^\r\n]+)',body);assert len(paths)==1
        row=next(s for s in current if Path(s['source'])==Path(paths[0]));assert sha(Path(row['archive']).read_bytes())==row['archiveSha256']
        expected={'Profile','Profile/.codex','Profile/.codex/empty','Profile/.codex/nested','Profile/custody','coordination','Profile/.grok','Profile/grok-custody'}
        assert set(row['directories'])==expected,(leaf,row['directories'])
        rows.append(dict(run=run,test=leaf,**row))
assert len(rows)==7
with (ROOT/'success-cleanup-plan.json').open('x',encoding='utf-8') as out:json.dump(rows,out,indent=2)
with (ROOT/'cleanup-counts.json').open('x',encoding='utf-8') as out:json.dump(dict(ownCompletedScenes=len(rows),failedAndPendingRetained=len(scenes)-len(rows),crossVolumePendingRootsRetained=sum(s['source'].startswith('D:') for s in scenes)),out,indent=2)
print(json.dumps(dict(completedPureData=len(rows),pendingRetained=len(scenes)-len(rows))))

from pathlib import Path
import hashlib,json,re,sys
ROOT=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest()
rows=[];scenes=[]
for run in sys.argv[1:]:
    current=load(ROOT/f'{run}-private-scenes.json')['scenes'];scenes.extend(current)
    text=(ROOT/run/'stdout.log').read_text(encoding='utf-8-sig')
    for name,body in re.findall(r'test ([^\n]+?) \.\.\. (.*?)(?=\ntest |\nfailures:|\ntest result:|\Z)',text,re.S):
        leaf=name.rsplit('::',1)[-1]
        if leaf not in ['wl2_r6_production_content_fixed_bun_original_audit_source_projection_and_pin_rechecks','wl2_r6_production_content_exact_source_pin_denies_write_replace_and_wrong_hash']:continue
        if not body.rstrip().endswith('ok'):continue
        paths=re.findall(r'r4-retained-synthetic-fixture=([^\r\n]+)',body);assert len(paths)==1
        row=next(s for s in current if Path(s['source'])==Path(paths[0]));assert sha(Path(row['archive']).read_bytes())==row['archiveSha256']
        allowed={'fixed-audit-projection'} if 'fixed_bun' in leaf else {'synthetic-source-pin'}
        assert set(row['directories'])==allowed,(leaf,row['directories'])
        rows.append(dict(run=run,test=leaf,**row))
assert len(rows)==3
with (ROOT/'success-cleanup-plan.json').open('x',encoding='utf-8') as out:json.dump(rows,out,indent=2)
with (ROOT/'cleanup-counts.json').open('x',encoding='utf-8') as out:json.dump(dict(ownCompletedScenes=len(rows),failedAndPendingRetained=len(scenes)-len(rows),crossVolumePendingRootsRetained=sum(s['source'].startswith('D:') for s in scenes)),out,indent=2)
previous=ROOT.parent/'wl2-r6-recovery-chain-20261116-r1'/'cleanup-completed.ps1'
with (ROOT/'cleanup-completed.ps1').open('xb') as out:out.write(previous.read_bytes())
print(json.dumps(dict(completedOwnScenes=len(rows),failedPendingRetained=len(scenes)-len(rows))))

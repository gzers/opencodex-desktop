from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
rows=[];scenes=[]
for run in ['library01','library02','library03']:
    current=load(ROOT/f'{run}-private-scenes.json')['scenes'];scenes.extend(current)
    log=(ROOT/run/'stdout.log').read_text(encoding='utf-8-sig')
    matches=[body for name,body in re.findall(r'test ([^\n]+?) \.\.\. (.*?)(?=\ntest |\nfailures:|\ntest result:|\Z)',log,re.S) if name.endswith('wl2_r6_route_close_native_all_original_main_security_and_two_ancestor_sets_close_before_independent_config_write')]
    assert len(matches)==1 and matches[0].rstrip().endswith('ok')
    paths=re.findall(r'r4-retained-synthetic-fixture=([^\r\n]+)',matches[0]);assert len(paths)==1
    row=next(s for s in current if Path(s['source'])==Path(paths[0]));assert hashlib.sha256(Path(row['archive']).read_bytes()).hexdigest()==row['archiveSha256']
    assert set(row['directories'])=={'route-home'} and {r['path'] for r in row['files']}=={'route-home/config.json'}
    rows.append(dict(run=run,test='complete-own-route-native-candidate-with-independent-config-write',**row))
assert len(rows)==3
with (ROOT/'success-cleanup-plan.json').open('x',encoding='utf-8') as f:json.dump(rows,f,indent=2)
with (ROOT/'cleanup-counts.json').open('x',encoding='utf-8') as f:json.dump(dict(scenes=len(scenes),ownCompletedScenes=3,failedAndPendingRetained=len(scenes)-3,crossVolumePendingRootsRetained=sum(s['source'].startswith('D:') for s in scenes)),f,indent=2)
with (ROOT/'cleanup-completed.ps1').open('xb') as f:f.write((ROOT.parent/'wl2-r6-recovery-chain-20261116-r1'/'cleanup-completed.ps1').read_bytes())
print(json.dumps(dict(ownCompletedScenes=3,retainedNativeOrFailed=len(scenes)-3)))

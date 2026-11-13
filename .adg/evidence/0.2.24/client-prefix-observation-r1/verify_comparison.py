from pathlib import Path
import json,statistics
root=Path(__file__).resolve().parent
log=(root/'final-history.log').read_text();marker='CLOSED_PREFIX_COMPARISON '
row=next(json.loads(line.split(marker,1)[1]) for line in log.splitlines() if marker in line)
assert row['accepted'] is False
samples=row['samples'];assert [r['mode'] for r in samples]==['reference','scoped-union','scoped-union','reference']
for s in samples:
    metrics=s['metrics'];assert metrics['frame']['calls']==8 and metrics['frames_prefix']['calls']==2
    expected={'reference':{'root_provenance':10,'capture_file':64,'pinned_open':56,'protection':6683},'scoped-union':{'root_provenance':6,'capture_file':48,'pinned_open':42,'protection':5595}}[s['mode']]
    for key,value in expected.items():assert metrics[key]['calls']==value,(key,metrics[key],value)
    assert s['elapsedNanos']>=metrics['closed_history']['inclusive_nanos']>0
reference=[s['elapsedNanos']/1e6 for s in samples if s['mode']=='reference'];scoped=[s['elapsedNanos']/1e6 for s in samples if s['mode']=='scoped-union']
initial=next(json.loads(line.split('CLOSED_PREFIX_METRICS ',1)[1]) for line in (root/'baseline-prefix.log').read_text().splitlines() if 'CLOSED_PREFIX_METRICS ' in line)
assert initial['metrics']['root_provenance']['calls']==10 and initial['metrics']['protection']['calls']==6683
print(json.dumps({'accepted':False,'sameBinaryReverseOrder':True,'referenceMs':reference,'scopedMs':scoped,'meanChangePercent':round((1-statistics.mean(scoped)/statistics.mean(reference))*100,2),
 'counts':{'fullRootProvenance':[10,6],'captureFile':[64,48],'pinnedDirectoryOpen':[56,42],'protection':[6683,5595],'frames':[8,8]},
 'attributeCountReductionPercent':round((1-5595/6683)*100,2),
 'limits':'Two samples per mode, synthetic first-generation four-frame history, debug library host, not a general speedup or second normal Stop pass. Inclusive nested times are not summed. Earlier nonwinning implementations/logs remain retained.'},indent=2))

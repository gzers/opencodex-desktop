from pathlib import Path
import json,statistics
base=Path(__file__).resolve().parent;log=(base/'history-comparison.log').read_text();marker='HISTORICAL_PLAN_COMPARISON '
value=next(json.loads(l.split(marker,1)[1]) for l in log.splitlines() if marker in l)
samples=value['samples'];assert [s['reference'] for s in samples]==[True,False,False,True]
for sample in samples:
 m=sample['metrics'];assert m['retirement_plan_load']['calls']==(6 if sample['reference'] else 2)
 assert m['frame']['calls']==8 and m['frames_prefix']['calls']==2 and m['root_provenance']['calls']==10
 assert m['protection']['calls']==(6299 if sample['reference'] else 5927)
 assert m['capture_file']['calls']==(64 if sample['reference'] else 60)
reference=[s['elapsedNanos']/1e6 for s in samples if s['reference']];current=[s['elapsedNanos']/1e6 for s in samples if not s['reference']]
print(json.dumps({'accepted':False,'sameBinaryReversedOrder':True,'referenceMs':reference,'currentMs':current,'meanChangePercent':round((1-statistics.mean(current)/statistics.mean(reference))*100,2),'counts':{'planLoads':[6,2],'frames':[8,8],'fullRootProvenance':[10,10],'captureFile':[64,60],'protection':[6299,5927]},'unchangedBoundaries':'Initial full authenticated Plan with exact independent source; checkpoint after first full archive/tree/Grok/intent/readback; independent final plan/archive/tree/Grok/intent/readback; no cross-observation cache or current-movement shortcut.','limits':'Two samples per mode, synthetic debug library observation. No native service round or normalStop/candidate acceptance.'},indent=2))

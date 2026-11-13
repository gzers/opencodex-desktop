from pathlib import Path
import json,statistics
base=Path(__file__).resolve().parent;log=(base/'history-comparison.log').read_text();marker='CHILD_PIN_COMPARISON '
row=next(json.loads(l.split(marker,1)[1]) for l in log.splitlines() if marker in l)
assert [s['reference'] for s in row['samples']]==[True,False,False,True]
for sample in row['samples']:
 m=sample['metrics'];assert m['frame']['calls']==8 and m['root_provenance']['calls']==10 and m['capture_file']['calls']==60 and m['retirement_plan_load']['calls']==2
 assert m['pinned_open']['calls']==(68 if sample['reference'] else 30)
 assert m['protection']['calls']==(6430 if sample['reference'] else 6109)
ref=[s['elapsedNanos']/1e6 for s in row['samples'] if s['reference']];cur=[s['elapsedNanos']/1e6 for s in row['samples'] if not s['reference']]
unchanged=json.loads((base/'unchanged-authentication.json').read_text());assert unchanged['captureFileBodyByteIdentical'] and unchanged['rootCreationFileByteIdentical']
print(json.dumps({'accepted':False,'sameBinaryReversedOrder':True,'referenceMs':ref,'currentMs':cur,'meanChangePercent':round((1-statistics.mean(cur)/statistics.mean(ref))*100,2),'counts':{'frame':[8,8],'fullProvenance':[10,10],'captureFile':[60,60],'planLoad':[2,2],'pinnedOpen':[68,30],'protection':[6430,6109]},'sourceAuthenticationUnchanged':True,'rejectedGroupedFileCaptureNotShipped':True,'limits':'Four synthetic debug observations with metric/JSON/assertion overhead; only complete child-pin acquisition changed. No original Stop budget, fixed candidate or general acceleration claim.'},indent=2))

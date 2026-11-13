from pathlib import Path
import json,statistics
base=Path(__file__).resolve().parent;log=(base/'projection-comparison.log').read_text();marker='PROJECTED_HISTORY_COMPARISON '
v=next(json.loads(l.split(marker,1)[1]) for l in log.splitlines() if marker in l)
assert [s['reference'] for s in v['samples']]==[True,False,False,True]
for s in v['samples']:
 m=s['metrics'];assert m['frame']['calls']==(16 if s['reference'] else 8)
 assert m['root_provenance']['calls']==(20 if s['reference'] else 10)
 assert m['retirement_plan_load']['calls']==(4 if s['reference'] else 2)
 assert m['historical_full_pass']['calls']==(4 if s['reference'] else 2)
ref=[s['elapsedNanos']/1e6 for s in v['samples'] if s['reference']];current=[s['elapsedNanos']/1e6 for s in v['samples'] if not s['reference']]
fdlog=(base/'projection-controls-final.log').read_text();marker='HISTORY_FD_BOUNDARY '
fd=next(json.loads(l.split(marker,1)[1]) for l in fdlog.splitlines() if marker in l)
assert fd['isolatedProcess'] and fd['incompleteObservations']==32 and fd['finalHandles']<=fd['initialHandles']+1 and fd['noOriginalWrites']
print(json.dumps({'accepted':False,'sameBinaryReversedOrder':True,'referenceMs':ref,'currentMs':current,'meanChangePercent':round((1-statistics.mean(current)/statistics.mean(ref))*100,2),'counts':{'wholeHistoricalPasses':[4,2],'frames':[16,8],'fullRootProvenance':[20,10],'planLoads':[4,2]},'incompleteObservationResourceTest':fd,'integrityBoundary':'One entire authenticated pass before directory/scope projection, one fresh entire pass after; independently authenticated immutable equality digests, no retained fd/Plan/tree accumulation and no admitted Original before consuming finish.','limits':'Same synthetic debug-library held-index fixture; previous original secondStop failure remains until a separate new native round.'},indent=2))

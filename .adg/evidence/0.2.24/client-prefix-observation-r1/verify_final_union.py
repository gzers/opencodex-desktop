from pathlib import Path
import json,statistics
root=Path(__file__).resolve().parent
marker='CLOSED_PREFIX_COMPARISON '
v=next(json.loads(l.split(marker,1)[1]) for l in (root/'union-only-history.log').read_text().splitlines() if marker in l)
assert [s['mode'] for s in v['samples']]==['reference','scoped-union','scoped-union','reference']
for s in v['samples']:
 m=s['metrics'];assert m['frame']['calls']==8 and m['root_provenance']['calls']==10 and m['capture_file']['calls']==64 and m['pinned_open']['calls']==56
 assert m['protection']['calls']==(6683 if s['mode']=='reference' else 6299)
ref=[s['elapsedNanos']/1e6 for s in v['samples'] if s['mode']=='reference'];union=[s['elapsedNanos']/1e6 for s in v['samples'] if s['mode']!='reference']
rows=[]
for name in ['capture-reference','capture-optimized-before-single','capture-scoped-final','union-only-capture','union-only-capture-reference']:
 log=(root/(name+'.log')).read_text()
 report=next(json.loads(l.split('CAPTURE_DEADLINE_COMPARISON ',1)[1]) for l in log.splitlines() if 'CAPTURE_DEADLINE_COMPARISON ' in l)
 stages=[json.loads(l.split('SUCCESSOR_CAPTURE_COST ',1)[1]) for l in log.splitlines() if 'SUCCESSOR_CAPTURE_COST ' in l]
 assert len(stages)==3 and all(s['recorded'] for s in stages) and report['passed']
 rows.append({'label':name,'reference':report['reference'],'threeCaptureMs':[s['totalMs'] for s in stages],'protectionCalls':report['metrics']['protection']['calls'],'fullProvenanceCalls':report['metrics']['root_provenance']['calls']})
assert rows[0]['fullProvenanceCalls']==rows[3]['fullProvenanceCalls']==rows[4]['fullProvenanceCalls']==1089
assert rows[4]['reference'] and rows[3]['reference'] is False
assert rows[4]['protectionCalls']>rows[3]['protectionCalls']
assert rows[3]['protectionCalls']<rows[0]['protectionCalls']
print(json.dumps({'accepted':False,'finalImplementation':'ancestor-chain union only; per-frame complete provenance unchanged','sameBinaryReverseOrder':True,'referenceMs':ref,'ancestorUnionMs':union,'meanChangePercent':round((1-statistics.mean(union)/statistics.mean(ref))*100,2),'counts':{'fullProvenance':[10,10],'frames':[8,8],'captureFile':[64,64],'pinnedOpen':[56,56],'protection':[6683,6299]},'attributeCountReductionPercent':round((1-6299/6683)*100,2),'captureComparison':rows,'scopedProvenanceReuseRejected':True,'limits':'Small synthetic debug observation, two measurements per order. No original normalStop or release speed claim; previous secondStop failure remains.'},indent=2))

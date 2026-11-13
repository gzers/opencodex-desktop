from pathlib import Path
import json,statistics
base=Path(__file__).resolve().parent;log=(base/'prefix-controls.log').read_text();marker='INDEX_SCOPE_PREFIX_COMPARISON '
rows=[json.loads(l.split(marker,1)[1]) for l in log.splitlines() if marker in l]
assert len(rows)==2 and {r['secondGeneration'] for r in rows}=={False,True};verified=[]
for r in rows:
 assert r['accepted'] is False and r['head']['state']=='ready';s=r['samples'];assert [v['reference'] for v in s]==[True,False,False,True]
 ref=[v for v in s if v['reference']];cur=[v for v in s if not v['reference']]
 for before,after in zip(ref,cur):
  assert before['metrics']['frame']['calls']-after['metrics']['frame']['calls']==4
  assert before['metrics']['root_provenance']['calls']-after['metrics']['root_provenance']['calls']==4
  for k in ['historical_begin','historical_finish']:
   assert before['metrics'].get(k)==after['metrics'].get(k) or before['metrics'].get(k,{}).get('calls')==after['metrics'].get(k,{}).get('calls')
 verified.append({'secondGeneration':r['secondGeneration'],'referenceMs':[v['elapsedNanos']/1e6 for v in ref],'currentMs':[v['elapsedNanos']/1e6 for v in cur],'frameReads':[ref[0]['metrics']['frame']['calls'],cur[0]['metrics']['frame']['calls']],'historyObservationsUnchanged':True})
print(json.dumps({'accepted':False,'verified':verified,'scope':'Samebinary reversed synthetic acquisitions, not runtime Ready or original Stop success. Semantic controls separately strengthened and run on final test-only source delta.'},indent=2))

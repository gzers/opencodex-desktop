from pathlib import Path
import json,re
base=Path(__file__).resolve().parent;log=(base/'ready-delivery.log').read_text();marker='READY_ACQUISITION_COMPARISON '
rows=[json.loads(l.split(marker,1)[1]) for l in log.splitlines() if marker in l]
assert len(rows)==2 and {r['secondGeneration'] for r in rows}=={False,True}
verified=[]
for row in rows:
 assert row['accepted'] is False and row['head']['state']=='ready'
 before=row['reference'];after=row['optimized']
 assert before['protection']['calls']>after['protection']['calls'] and before['frame']['calls']>after['frame']['calls']
 if row['secondGeneration']:
  assert before['historical_begin']['calls']==before['historical_finish']['calls']==4
  assert after['historical_begin']['calls']==after['historical_finish']['calls']==2
  assert [before['frame']['calls'],after['frame']['calls']]==[50,26]
  assert [before['protection']['calls'],after['protection']['calls']]==[77774,40442]
 else:
  assert [before['frame']['calls'],after['frame']['calls']]==[18,10]
  assert [before['protection']['calls'],after['protection']['calls']]==[14046,7750]
 verified.append({'secondGeneration':row['secondGeneration'],'frameReads':[before['frame']['calls'],after['frame']['calls']],'protectionReads':[before['protection']['calls'],after['protection']['calls']],'sameReadyHead':True})
log=(base/'trace-result-controls.log').read_text()
assert re.search(r'stage=test.expired .*expired_after=true code=timeout',log)
assert re.search(r'stage=test.binding .*expired_after=false code=client_session_unconfirmed',log)
assert '1 passed; 0 failed' in log
mapped=(base/'final-mapping-build.log').read_text();assert 'code=timeout mapped_to=permission-denied' in mapped and 'code=client_native_coordination_expired mapped_to=permission-denied' in mapped
assert 'never print this detail' not in mapped
print(json.dumps({'accepted':False,'verifiedReadyAcquisitions':verified,'expiredDeadlineAndBudgetRemainingBindingRefusalRemainDistinct':True,'oldIoKindAndMessagesPreserved':True,'sensitiveErrorDetailNotLogged':True,'limits':'Synthetic persisted scope/live bindings, not actual attestation/normalStop. Counts include complete before/after acquisition and remove only two immediate extra full observations.'},indent=2))

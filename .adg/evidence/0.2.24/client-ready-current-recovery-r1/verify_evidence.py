from pathlib import Path
import json,hashlib,re,base64
base=Path('/private/tmp/ocxd024-ready-recovery-kybrs_7j');sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
before=json.loads((base/'before-recovery/index.json').read_text());after=json.loads((base/'current-recovery/index-final.json').read_text());a=before['head'];b=after['head'];originals=base/'current-recovery/registered-originals'
assert a['state']=='ready' and a['revision']==4 and b['state']=='restored' and b['revision']==6
assert a['references']==b['references'] and len(b['references']['phases'])==1
assert after['events'][:len(before['events'])]==before['events']
assert after['events'][-2]['state']=='recovery_prepared'
assert all(sha(p)==sha(originals/p.name) for p in (base/'before-recovery/registered-originals').iterdir())
assert not (originals/'pre-stop.json').exists() and not (originals/'stopped.json').exists()
plan=originals/('restore-'+b['restorePlan']['id']+'.json');assert sha(plan)==b['restorePlan']['sha256'];j=json.loads(plan.read_text())
assert j['schemaVersion']==4 and j['recoveryObservation']['sourceHead']==a
assert len(j['phases'])==3
assert j['phases'][0]==a['references']['originalSha256'] and j['phases'][1]==a['references']['phases'][0][1]
for name,field in [('original.json','originalSha256'),('startup-scope.json','scopeSha256'),('startup-live.json','liveSha256')]:assert sha(originals/name)==b['references'][field]
blob_checks=0
for p in originals.glob('tree-content-*.blob'):
 assert sha(p)==p.name[len('tree-content-'):-len('.blob')];blob_checks+=1
out=base/'current-recovery';prepared=json.loads((out/'missing-plan-prepare.json').read_text());completed=json.loads((out/'missing-plan-continue.json').read_text())
assert prepared['currentTargetFilesUnchanged'] and prepared['outcome']['revision']==5 and prepared['outcome']['status']=='recovery_prepared' and prepared['elapsedMs']<10000
assert completed['outcome']['revision']==6 and completed['outcome']['status']=='restored' and completed['elapsedMs']<10000
for label in ['ready-pending-status','prepare-continue','full-readback','final-status']:
 report=json.loads((out/(label+'.json')).read_text());assert report['exitCode']==0
 assert (out/(label+'.stdout.log')).read_text().count('test result: ok. 1 passed')==1
raw=(out/'full-readback.stdout.log').read_text();projection=json.loads(next(line[line.index('{'):] for line in raw.splitlines() if '{' in line))
assert projection['independentCurrentHashesMatchReceipt']
for field in ['codexTreeBase64','grokTreeBase64']:
 data=base64.b64decode(projection[field]);(out/(field[:-6]+'.json')).write_bytes(data)
required=json.loads((out/'ready-pending-live-status.json').read_text());final=json.loads((out/'final-live-status.json').read_text())
assert required['facts']['client_recovery']=='required' and final['facts']['client_recovery']=='not_required'
assert required['matrix']['runtime']==final['matrix']['runtime']=='stopped' and not required['can_start'] and final['can_start']
assert all(v['pid'] is None and v['port'] is None and not v['can_stop'] and not v['can_restart'] for v in [required,final])
summary={'accepted':False,'result':'passed-directed-ready-prefix-recovery','originalFailedStopRetained':True,'initialState':'ready','initialRevision':4,'preparedRevision':5,'restoredRevision':6,'originalReferenceCount':1,'planHashCount':3,'originalFramesUnchanged':True,'noHistoricalPreStopOrStopped':True,'originalEventsUnchanged':True,'currentPlanSourceMatchesOriginalHead':True,'prepareMs':prepared['elapsedMs'],'continueMs':completed['elapsedMs'],'currentTargetsUnchangedByPrepare':True,'fullNativeTreeReadbackMatches':True,'verifiedContentBlobs':blob_checks,'statusBefore':'required','statusAfter':'not_required','noServiceRestart':True,'candidateAcceptance':False}
(base/'verified-summary.json').write_text(json.dumps(summary,indent=2));print(json.dumps(summary,indent=2))

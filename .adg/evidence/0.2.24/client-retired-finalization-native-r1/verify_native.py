from pathlib import Path
import json,hashlib,base64,gzip,sqlite3,tempfile
base=Path(__file__).resolve().parent;out=base/'native-retired-r1';sha=lambda b:hashlib.sha256(b).hexdigest()
runner=json.loads((out/'runner-summary.json').read_text());summary=json.loads((out/'native/summary.json').read_text());source=json.loads((base/'compiled-source.json').read_text())
assert runner['accepted'] is False and runner['exitCode']==0 and runner['diagnosticPassed'] and runner['exactOneTestExecuted'] and runner['retiredCompletionMarker']
assert runner['binarySha256']==source['binarySha256'] and runner['globalLUnchanged'] and runner['globalLExclusiveReleased'] and runner['globalLSidecarsAbsent'] and runner['trackedGenerationsAbsent'] and not runner['ownedExecutablesRemaining'] and not runner.get('coordinatorIdentityChanges') and not runner.get('observationErrors')
assert summary['result']=='passed' and summary['accepted'] is False and summary['businessRequests'] is False
assert summary['start']['result']=='started' and summary['startElapsedMs']<30000 and summary['statusElapsedMs']<5000 and summary['status']['ready'] and summary['status']['statusObservation']=='attested-same-connection'
stop=summary['stop'];assert stop['result'] is None and stop['lifecycleState']=='stopped' and stop['clientRecoveryRequired'] and not stop['canStart'] and stop['failure']['code']=='client_original_restore_pending'
assert summary['stopElapsedMs']<10000
pending=summary['retiredPendingBeforeFinalize'];closed=summary['retiredClosedAfterFinalize'];assert pending['state']=='root_retirement_prepared' and pending['revision']==8 and pending['completionSha256'] is None and closed['state']=='root_absent' and closed['revision']==9 and closed['completionSha256']
expected=dict(pending);expected.update(state='root_absent',revision=9,completionSha256=closed['completionSha256']);assert closed==expected
preview=summary['retiredFinalizePreview'];assert preview['phase']=='root_retirement_prepared' and preview['revision']==8 and preview['attemptId']==pending['ticket'] and preview['phaseCount']==3 and preview['completionEvidence']=='unregistered' and preview['canFinalize'] and not preview['canContinue'] and not preview['canRollback'] and len(preview['confirmationDigest'])==64
first=summary['retiredFinalizeOutcome'];duplicate=summary['retiredDuplicateOutcome'];assert first==duplicate=={'attemptId':pending['ticket'],'revision':9,'status':'restored'}
assert summary['retiredFinalizeElapsedMs']<10000 and summary['retiredDuplicateElapsedMs']<10000
assert all(summary['retiredFinalizeOriginalArtifactsRestored'].values()) and len(summary['retiredFinalizeOriginalArtifactsRestored'])==7
assert all(summary['retiredDuplicateOriginalArtifactsRestored'].values()) and len(summary['retiredDuplicateOriginalArtifactsRestored'])==7
assert summary['retiredPostImageDriftRefused'] is True
snapshots=[]
for name in ['before','after-retired-finalize','after-retired-duplicate']:
 rows=json.loads((out/('native/'+name+'.json')).read_text());seen=set()
 for row in rows:
  assert row['path'] not in seen;seen.add(row['path'])
  if row['present']:
   raw=base64.b64decode(row['bytesBase64'],validate=True);assert sha(raw)==row['sha256']
  else:assert row['sha256'] is None and row['bytesBase64'] is None
 assert len(rows)==7;snapshots.append(rows)
assert snapshots[0]==snapshots[1]==snapshots[2]
compressed=(out/'owned-original-bytes.json.gz').read_bytes();meta=json.loads((out/'owned-byte-manifest.json').read_text());raw=gzip.decompress(compressed);assert sha(compressed)==meta['gzipSha256'] and sha(raw)==meta['rawSha256'] and len(raw)==meta['rawBytes'];payload=json.loads(raw);assert len(payload['files'])==51 and payload['rootExists'] is False
with tempfile.TemporaryDirectory(prefix='ocxd024-retired-finalize-offline-',dir='/private/tmp') as root:
 root=Path(root);seen=set()
 for item in payload['files']:
  path=Path(item['path']);assert not path.is_absolute() and '..' not in path.parts and str(path) not in seen;seen.add(str(path));b=base64.b64decode(item['rawBase64'],validate=True);assert len(b)==item['bytes'] and sha(b)==item['sha256'];p=root/path;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(b)
 manager=root/'.codex-desktop/desktop/manager-state';db=sqlite3.connect((manager/'client-session-index.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True);events=[json.loads(v[0]) for v in db.execute('select document from events order by revision')];binding=json.loads(db.execute('select document from binding').fetchone()[0]);head=json.loads(db.execute('select document from current_session').fetchone()[0]);db.close()
 assert events==summary['retiredFinalizeEvents'] and len(events)==9 and head==events[-1]==closed and events[-2]==pending
 assert [v['state'] for v in events]==['preparing','capture_reserved','captured','ready','stop_prepared','stopped','restore_prepared','root_retirement_prepared','root_absent']
 assert all(v['ticket']==pending['ticket'] for v in events)
 origin=binding['originalRoot'];assert origin['id']==pending['ticket'];refs=pending['references'];vault=manager/'client-originals'/pending['ticket']
 for name,key in [('intent.json','intentSha256'),('created.json','createdSha256'),('published.json','publishedSha256')]:assert sha((manager/'client-root-reservation'/name).read_bytes())==origin[key]
 original=(vault/'original.json').read_bytes();assert sha(original)==refs['originalSha256'];frame=json.loads(original);assert frame['originalRootAbsence']==origin and frame['schemaVersion']==5 and frame['previousSha256'] is None
 assert [phase for phase,digest in refs['phases']]==['ready','pre_stop','stopped'];prior=refs['originalSha256']
 for phase,expected in refs['phases']:
  b=(vault/{'ready':'ready.json','pre_stop':'pre-stop.json','stopped':'stopped.json'}[phase]).read_bytes();assert sha(b)==expected;f=json.loads(b);assert f['originalRootAbsence']==origin and f['attempt']==pending['ticket'] and f['previousSha256']==prior;prior=expected
 assert sha((vault/'startup-scope.json').read_bytes())==refs['scopeSha256'] and sha((vault/'startup-live.json').read_bytes())==refs['liveSha256']
 artifact=pending['restorePlan'];assert sha((vault/('restore-'+artifact['id']+'.json')).read_bytes())==artifact['sha256']
 for filename in ['original.json','ready.json','pre-stop.json','stopped.json']:
  phase=json.loads((vault/filename).read_bytes());trees=[phase['tree']]
  if phase['grok']['state']=='directory':trees.append(phase['grok']['tree'])
  for tree in trees:
   for entry in tree['entries']:
    if entry['kind']=='file':
     blob=(vault/('tree-content-'+entry['sha256']+'.blob')).read_bytes();assert len(blob)==entry['bytes'] and sha(blob)==entry['sha256']
 movement=pending['rootRetirement'];assert movement['sourceRevision']==7;raw=(vault/('root-retirement-'+movement['plan']['id']+'.json')).read_bytes();assert sha(raw)==movement['plan']['sha256'];plan=json.loads(raw);assert plan['source']==events[6] and plan['origin']==origin
 raw=(vault/('root-retirement-'+movement['plan']['id']+'-Retire-readback.json')).read_bytes();assert sha(raw)==closed['completionSha256'];receipt=json.loads(raw);assert receipt['sourceHead']==events[6] and receipt['planSha256']==movement['plan']['sha256'] and receipt['currentRootAbsent'] and receipt['direction']=='retire'
 intent=json.loads((vault/('root-retirement-'+movement['plan']['id']+'-Retire-intent.json')).read_bytes());assert intent=={'schemaVersion':1,'planSha256':movement['plan']['sha256'],'direction':'retire'}
 archive=vault/('root-retired-'+movement['plan']['id']);assert archive.is_dir() and not (archive/'config.toml.ocx-write.lock').exists() and not (root/'.codex').exists()
 assert (root/'.grok/config.toml').read_bytes()==b'owned-test-drift-after-image=1\n'
cleanup=json.loads((out/'cleanup-native.json').read_text());assert cleanup['rawOwnedBytesCapturedBeforeCleanup'] and cleanup['noOwnedProcessesByExecutableInventory'] and cleanup['globalLExclusiveReleased'] and cleanup['preexistingGlobalLNeverDeletedRebuiltOrRestored'] and len(cleanup['exactOwnedCoordinatorsExclusivelyReacquiredAndRemoved'])==3
log=gzip.decompress((out/'manager.txt.gz').read_bytes()).decode();assert 'original-native-launcher-and-service confirmed code=0' in log and 'failed phase=root-close code=owned_retired_close_interrupted' in log
assert 'captured-tcp-listeners absent count=4' in log
print(json.dumps({'accepted':False,'sourceCommit':source['baseCommit'],'directedRetiredFinalizePassed':True,'originalStopRemainsFailed':True,'startStatusStopMs':[summary['startElapsedMs'],summary['statusElapsedMs'],summary['stopElapsedMs']],'explicitFinalizeMs':summary['retiredFinalizeElapsedMs'],'exactDuplicateMs':summary['retiredDuplicateElapsedMs'],'pendingRevision':8,'closedRevision':9,'rawEvents':9,'duplicateDidNotAppendRevision':True,'trackedSourceByteSnapshotsEqual':7,'postImageCurrentGrokDriftRefusedAndPreserved':True,'originalPhasePlanReadbackChainVerified':True,'rawProfileFilesVerified':51,'ownedCoordinatorsRemoved':3,'globalLUnchangedAndReleased':True,'offlineReplayRemoved':True,'scope':'One explicit retired metadata recovery/duplicate/drift-refusal test, not original normalStop pass or general interrupted movement/fixedapp/wholeversion acceptance'},indent=2))

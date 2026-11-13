from pathlib import Path
import json,gzip,base64,hashlib,tempfile,sqlite3
base=Path(__file__).resolve().parent;out=base/'native-tail-r1';sha=lambda b:hashlib.sha256(b).hexdigest()
runner=json.loads((out/'runner-summary.json').read_text());s=json.loads((out/'native/summary.json').read_text());assert runner['exitCode']==101 and runner['accepted'] is False and not runner['diagnosticPassed']
assert runner['globalLUnchanged'] and runner['globalLExclusiveReleased'] and runner['globalLSidecarsAbsent'] and runner['trackedGenerationsAbsent'] and not runner['ownedExecutablesRemaining'] and not runner.get('observationErrors') and not runner.get('coordinatorIdentityChanges')
assert s['start']['result']=='started' and s['startElapsedMs']==7436 and s['statusElapsedMs']==229 and s['status']['ready'] and s['status']['statusObservation']=='attested-same-connection'
assert 'stop' not in s and 'stopElapsedMs' not in s and 'rootCycles' not in s
log=gzip.decompress((out/'manager.txt.gz').read_bytes()).decode();assert 'failed phase=wait-os-absence' in log and 'Invalid argument (os error 22)' in log and 'read recovery process arguments' in log and 'duration_ms=1863' in log
assert 'original-native-exit confirmed code=0' not in log and 'root-absence-complete' not in log
stderr=gzip.decompress((out/'native.stderr.txt.gz').read_bytes()).decode();assert 'verified Stop with incomplete recovery must not remain running or claim success' in stderr
raw=gzip.decompress((out/'owned-original-bytes.json.gz').read_bytes());meta=json.loads((out/'owned-byte-manifest.json').read_text());assert sha(raw)==meta['rawSha256'] and len(raw)==meta['rawBytes'];payload=json.loads(raw);assert len(payload['files'])==40
with tempfile.TemporaryDirectory(prefix='ocxd024-procargs-failure-replay-') as root:
 root=Path(root);seen=set()
 for item in payload['files']:
  p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and str(p) not in seen;seen.add(str(p));b=base64.b64decode(item['rawBase64'],validate=True);assert len(b)==item['bytes'] and sha(b)==item['sha256'];path=root/p;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(b)
 manager=root/'.codex-desktop/desktop/manager-state';db=sqlite3.connect((manager/'client-session-index.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True);events=[json.loads(row[0]) for row in db.execute('select document from events order by revision')];head=json.loads(db.execute('select document from current_session').fetchone()[0]);binding=json.loads(db.execute('select document from binding').fetchone()[0]);db.close()
 assert [v['state'] for v in events]==['preparing','capture_reserved','captured','ready','stop_prepared'] and head==events[-1] and head['revision']==5
 refs=head['references'];origin=binding['originalRoot'];vault=manager/'client-originals'/head['ticket'];assert origin['id']==head['ticket']
 for name,key in [('intent.json','intentSha256'),('created.json','createdSha256'),('published.json','publishedSha256')]:assert sha((manager/'client-root-reservation'/name).read_bytes())==origin[key]
 b=(vault/'original.json').read_bytes();assert sha(b)==refs['originalSha256'];original=json.loads(b);assert original['originalRootAbsence']==origin
 assert [phase for phase,digest in refs['phases']]==['ready','pre_stop'];previous=refs['originalSha256']
 for phase,digest in refs['phases']:
  b=(vault/('ready.json' if phase=='ready' else 'pre-stop.json')).read_bytes();assert sha(b)==digest;frame=json.loads(b);assert frame['previousSha256']==previous and frame['attempt']==head['ticket'];previous=digest
 assert not (vault/'stopped.json').exists() and head.get('restorePlan') is None and head.get('rootRetirement') is None
cleanup=json.loads((out/'cleanup-native.json').read_text());assert cleanup['rawOwnedBytesCapturedBeforeCleanup'] and cleanup['noOwnedProcessesByExecutableInventory'] and cleanup['globalLExclusiveReleased'] and cleanup['preexistingGlobalLNeverDeletedRebuiltOrRestored'] and len(cleanup['exactOwnedCoordinatorsExclusivelyReacquiredAndRemoved'])==3
print(json.dumps({'accepted':False,'actualNativeRounds':1,'firstStartMs':7436,'firstSignedStatusMs':229,'firstStop':'failed-before-normal-exit-proof','originalError':'read recovery process arguments / EINVAL','headState':'StopPrepared5','sourceAndReadyPreStopChainsVerified':True,'noNormalExitOrRestorationSuccessInferred':True,'stopDTOUnavailableDueAssertionOrder':True,'secondCycleNotExecuted':True,'rawProfileFiles':40,'globalLUnchangedAndReleased':True,'ownedCoordinatorsRemoved':3},indent=2))

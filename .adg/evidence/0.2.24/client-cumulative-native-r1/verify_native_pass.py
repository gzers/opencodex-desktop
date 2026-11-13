from pathlib import Path
import json,hashlib,base64,gzip,sqlite3,tempfile,uuid
base=Path(__file__).resolve().parent;out=base/'native-cumulative-r1';sha=lambda b:hashlib.sha256(b).hexdigest()
runner=json.loads((out/'runner-summary.json').read_text());summary=json.loads((out/'native/summary.json').read_text());cycles=json.loads((out/'native/root-cycles.json').read_text())
assert runner['accepted'] is False and summary['accepted'] is False and cycles['accepted'] is False
assert runner['diagnosticPassed'] and runner['exitCode']==0 and runner['exactOneTestExecuted'] and runner['twoCycleCompletionMarker']
assert runner['globalLUnchanged'] and runner['globalLExclusiveReleased'] and runner['globalLSidecarsAbsent']
assert runner['trackedGenerationsAbsent'] and not runner['ownedExecutablesRemaining'] and not runner.get('observationErrors') and not runner.get('coordinatorIdentityChanges')
assert summary['result']=='passed' and summary['rootCycles']==cycles and cycles['passed']
for flag in ['firstStoppedGate','firstReadonlyStateUnchanged','firstPreviewGate','secondLiveGate','secondStoppedGate','secondReadonlyStateUnchanged','secondPreviewGate','oldEvidenceUnchangedDuringSecondStart','oldBindingEventsAndVaultUnchanged','distinctRootAttemptPlanAndPid','secondReadyGate','secondStopGate','recordsAbsent']:
    assert cycles[flag] is True,flag
for field,budget in [('startElapsedMs',30000),('statusElapsedMs',5000),('stopElapsedMs',10000)]:assert summary[field]<budget
for field,budget in [('secondStartElapsedMs',30000),('secondLiveStatusElapsedMs',5000),('secondStopElapsedMs',10000),('firstStoppedStatusElapsedMs',5000),('secondStoppedStatusElapsedMs',5000)]:assert cycles[field]<budget
for value in [summary['stop'],cycles['secondStop']]:
    assert value['result']=='stopped' and value['lifecycleState']=='stopped' and value['canStart'] and not value['canStop'] and not value['canRestart'] and not value['clientRecoveryRequired'] and value.get('failure') is None
for prefix in ['first','second']:
    observed=cycles[prefix+'StoppedStatus']['snapshot']
    assert observed['matrix']['runtime']=='stopped' and observed['pid'] is None and observed['port'] is None and observed['can_start'] and not observed['can_stop'] and not observed['can_restart'] and observed['facts']['client_recovery']=='not_required'
for raw,record in [(summary['status'],summary['record']),(cycles['secondLiveStatus']['official'],cycles['secondRecord'])]:
    assert raw['status']=='running' and raw['ready'] and raw['officialReadiness']=='ready' and raw['statusObservation']=='attested-same-connection'
    assert int(raw['pid'])==record['pid'] and raw['port']==record['port'] and record['pid']>0 and record['port']>0
payload=gzip.decompress((out/'owned-original-bytes.json.gz').read_bytes());meta=json.loads((out/'owned-byte-manifest.json').read_text())
assert sha(payload)==meta['rawSha256'] and len(payload)==meta['rawBytes'] and sha((out/'owned-original-bytes.json.gz').read_bytes())==meta['gzipSha256']
payload=json.loads(payload);assert not payload['rootExists'] and len(payload['files'])==meta['fileCount']
with tempfile.TemporaryDirectory(prefix='ocxd024-two-root-offline-') as replay:
    replay=Path(replay);seen=set()
    for item in payload['files']:
        p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and str(p) not in seen;seen.add(str(p))
        raw=base64.b64decode(item['rawBase64'],validate=True);assert len(raw)==item['bytes'] and sha(raw)==item['sha256']
        target=replay/p;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw)
    manager=replay/'.codex-desktop/desktop/manager-state'
    c=sqlite3.connect((manager/'client-session-index.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True)
    raw_events=[r[0] for r in c.execute('SELECT document FROM events ORDER BY revision')];events=[json.loads(r) for r in raw_events]
    binding_raw=c.execute('SELECT document FROM binding').fetchone()[0];binding=json.loads(binding_raw)
    head=json.loads(c.execute('SELECT document FROM current_session').fetchone()[0]);c.close()
    old=json.loads((out/'native/first-closed-index-documents.json').read_text())
    assert binding_raw==base64.b64decode(old['bindingBase64']) and raw_events[:9]==[base64.b64decode(b) for b in old['eventsBase64']]
    assert events==json.loads((out/'native/two-cycle-index-events.json').read_text()) and head==events[-1]==cycles['secondClosed']
    assert len(events)==18 and cycles['firstClosed']==events[8] and cycles['secondReadyIndex']==events[12]
    expected=['preparing','capture_reserved','captured','ready','stop_prepared','stopped','restore_prepared','root_retirement_prepared','root_absent']
    assert [h['state'] for h in events]==expected*2 and [h['revision'] for h in events]==list(range(1,19))
    origins=[];identities=[];plan_ids=[]
    for start in [0,9]:
        segment=events[start:start+9];ticket=segment[0]['ticket'];assert str(uuid.UUID(ticket))==ticket
        assert all(h['ticket']==ticket for h in segment)
        if start==0:active=binding;assert 'rootGeneration' not in segment[0]
        else:
            generation=segment[0]['rootGeneration'];assert generation['preparingRevision']==10 and generation['predecessorRevision']==9 and generation['predecessorSha256']==sha(raw_events[8])
            active=json.loads(bytes(generation['binding']))
        scope=json.loads(bytes(active['scope']));origin=active['originalRoot'];assert origin['id']==ticket
        assert all('rootGeneration' not in h for h in segment[1:])
        origin_slot=manager/('client-root-reservation' if start==0 else 'client-root-generations/'+ticket)
        for filename,key in [('intent.json','intentSha256'),('created.json','createdSha256'),('published.json','publishedSha256')]:assert sha((origin_slot/filename).read_bytes())==origin[key]
        assert json.loads((origin_slot/'created.json').read_bytes())['rootIdentity']==scope[8]
        refs=segment[-1]['references'];vault=manager/'client-originals'/ticket
        raw=(vault/'original.json').read_bytes();assert sha(raw)==refs['originalSha256'];frame=json.loads(raw)
        assert frame['schemaVersion']==5 and frame['originalRootAbsence']==origin and frame['clientRoot']==scope[8]
        assert frame['tree']['totalBytes']==0 and frame['previousSha256'] is None
        assert frame['tree']['coordination'] and frame['tree']['totalBytes']==0
        assert all(row['kind']=='coordination' and row['path'] in {'.opencodex-native-main.claim.sqlite'+suffix for suffix in ['', '-journal', '-wal', '-shm']} for row in frame['tree']['entries'])
        assert sha((vault/'startup-scope.json').read_bytes())==refs['scopeSha256']
        assert [p for p,h in refs['phases']]==['ready','pre_stop','stopped'] and refs['liveSha256'] is not None
        previous=refs['originalSha256']
        for phase,expected_sha in refs['phases']:
            filename={'ready':'ready.json','pre_stop':'pre-stop.json','stopped':'stopped.json'}[phase]
            phase_raw=(vault/filename).read_bytes();assert sha(phase_raw)==expected_sha
            phase_frame=json.loads(phase_raw);assert phase_frame['attempt']==ticket and phase_frame['previousSha256']==previous and phase_frame['originalRootAbsence']==origin
            previous=expected_sha
        movement=segment[-1]['rootRetirement'];assert movement==segment[-2]['rootRetirement'] and movement['sourceRevision']==start+7
        plan_raw=(vault/('root-retirement-'+movement['plan']['id']+'.json')).read_bytes();assert sha(plan_raw)==movement['plan']['sha256']
        plan=json.loads(plan_raw);assert plan['source']==segment[6] and plan['origin']==origin
        readback=vault/('root-retirement-'+movement['plan']['id']+'-Retire-readback.json');assert sha(readback.read_bytes())==segment[-1]['completionSha256']
        assert (vault/('root-retired-'+movement['plan']['id'])).is_dir()
        origins.append(origin);identities.append(scope[8]);plan_ids.append(movement['plan']['id'])
    assert origins[0]['id']!=origins[1]['id'] and identities[0]!=identities[1] and plan_ids[0]!=plan_ids[1]
cleanup=json.loads((out/'cleanup-native.json').read_text());assert cleanup['rawOwnedBytesCapturedBeforeCleanup'] and cleanup['noOwnedProcessesByExecutableInventory'] and cleanup['globalLExclusiveReleased'] and cleanup['preexistingGlobalLNeverDeletedRebuiltOrRestored']
assert len(cleanup['exactOwnedCoordinatorsExclusivelyReacquiredAndRemoved'])==3
print(json.dumps({'accepted':False,'directedTwoOfficialCyclesPassed':True,'rawEvents':18,'independentOriginalPhasePlanReadbackChains':2,'originalBindingAndFirstNineDocumentsUnchanged':True,'distinctAttemptsRootIdentitiesAndPlans':True,'globalLUnchangedAndReleased':True,'ownedCoordinatorsRemoved':3,'offlineReplayRemoved':True,'scope':'release-library two-cycle diagnostic, not fixed application/installer or whole version acceptance'},indent=2))

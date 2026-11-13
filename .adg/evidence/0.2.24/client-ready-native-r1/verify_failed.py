from pathlib import Path
import json,hashlib,base64,gzip,sqlite3,tempfile
base=Path(__file__).resolve().parent;out=base/'native-ready-r1';sha=lambda b:hashlib.sha256(b).hexdigest()
runner=json.loads((out/'runner-summary.json').read_text());summary=json.loads((out/'native/summary.json').read_text());cycles=json.loads((out/'native/root-cycles.json').read_text())
assert runner['exitCode']==101 and runner['diagnosticPassed'] is False and runner['accepted'] is False
assert summary['result']=='failed' and summary['rootCycles']==cycles and cycles['passed'] is False
assert runner['globalLUnchanged'] and runner['globalLExclusiveReleased'] and runner['globalLSidecarsAbsent'] and runner['trackedGenerationsAbsent'] and not runner['ownedExecutablesRemaining']
assert not runner.get('observationErrors') and not runner.get('coordinatorIdentityChanges')
assert summary['start']['result']=='started' and summary['stop']['result']=='stopped'
for k,budget in [('startElapsedMs',30000),('statusElapsedMs',5000),('stopElapsedMs',10000)]:assert summary[k]<budget
assert summary['firstAttestedStatusGate'] and cycles['firstStoppedGate'] and cycles['firstReadonlyStateUnchanged'] and cycles['firstPreviewGate']
assert cycles['secondStart']['result']=='started' and cycles['secondStartElapsedMs']<30000
assert cycles['secondLiveGate'] and cycles['secondLiveStatusElapsedMs']<5000
failure=cycles['secondStop'];assert failure['result'] is None and failure['lifecycleState']=='stopped' and failure['clientRecoveryRequired'] and not failure['canStart'] and failure['failure']['code']=='client_original_restore_pending'
assert cycles['secondStopElapsedMs']>=10000 and cycles['secondStoppedGate'] is False
assert cycles['oldEvidenceUnchangedDuringSecondStart'] and cycles['oldBindingEventsAndVaultUnchanged']
assert cycles['secondClosed']['state']=='restore_prepared' and cycles['secondClosed']['revision']==16 and cycles['closedGenerationCount'] is None
for raw,record in [(summary['status'],summary['record']),(cycles['secondLiveStatus']['official'],cycles['secondRecord'])]:
    assert raw['status']=='running' and raw['ready'] and raw['officialReadiness']=='ready' and raw['statusObservation']=='attested-same-connection'
    assert int(raw['pid'])==record['pid'] and raw['port']==record['port'] and record['pid']>0 and record['port']>0
compressed=(out/'owned-original-bytes.json.gz').read_bytes();meta=json.loads((out/'owned-byte-manifest.json').read_text());payload=gzip.decompress(compressed)
assert len(payload)==meta['rawBytes'] and sha(payload)==meta['rawSha256'] and sha(compressed)==meta['gzipSha256']
payload=json.loads(payload);assert payload['rootExists'] and len(payload['files'])==meta['fileCount']
assert not any(Path(f['path']).name in ['admin-api-token','runtime-port.json'] for f in payload['files'])
with tempfile.TemporaryDirectory(prefix='ocxd024-two-root-failure-offline-') as d:
    d=Path(d);seen=set()
    for f in payload['files']:
        p=Path(f['path']);assert not p.is_absolute() and '..' not in p.parts and str(p) not in seen;seen.add(str(p))
        raw=base64.b64decode(f['rawBase64'],validate=True);assert len(raw)==f['bytes'] and sha(raw)==f['sha256']
        target=d/p;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw)
    manager=d/'.codex-desktop/desktop/manager-state'
    db=sqlite3.connect((manager/'client-session-index.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True)
    raw_events=[r[0] for r in db.execute('SELECT document FROM events ORDER BY revision')];events=[json.loads(b) for b in raw_events]
    head=json.loads(db.execute('SELECT document FROM current_session').fetchone()[0]);binding_raw=db.execute('SELECT document FROM binding').fetchone()[0];binding=json.loads(binding_raw);db.close()
    old=json.loads((out/'native/first-closed-index-documents.json').read_text())
    assert binding_raw==base64.b64decode(old['bindingBase64']) and raw_events[:9]==[base64.b64decode(b) for b in old['eventsBase64']]
    assert len(events)==16 and head==events[-1]==cycles['secondClosed'] and cycles['firstClosed']==events[8] and cycles['secondReadyIndex']==events[12]
    expected=['preparing','capture_reserved','captured','ready','stop_prepared','stopped','restore_prepared','root_retirement_prepared','root_absent']
    assert [h['state'] for h in events]==expected+expected[:7] and [h['revision'] for h in events]==list(range(1,17))
    assert events==json.loads((out/'native/two-cycle-index-events.json').read_text())
    identities=[];attempts=[]
    for start in [0,9]:
        segment=events[start:];ticket=segment[0]['ticket'];last=segment[8] if start==0 else segment[-1]
        if start==0:active=binding;slot=manager/'client-root-reservation'
        else:
            generation=segment[0]['rootGeneration'];assert generation['preparingRevision']==10 and generation['predecessorRevision']==9 and generation['predecessorSha256']==sha(raw_events[8])
            active=json.loads(bytes(generation['binding']));slot=manager/'client-root-generations'/ticket
        origin=active['originalRoot'];scope=json.loads(bytes(active['scope']));assert origin['id']==ticket
        for filename,key in [('intent.json','intentSha256'),('created.json','createdSha256'),('published.json','publishedSha256')]:assert sha((slot/filename).read_bytes())==origin[key]
        assert json.loads((slot/'created.json').read_bytes())['rootIdentity']==scope[8]
        vault=manager/'client-originals'/ticket;refs=last['references'];raw=(vault/'original.json').read_bytes();assert sha(raw)==refs['originalSha256']
        frame=json.loads(raw);assert frame['schemaVersion']==5 and frame['originalRootAbsence']==origin and frame['clientRoot']==scope[8]
        assert frame['tree']['coordination'] and frame['tree']['totalBytes']==0
        assert all(row['kind']=='coordination' and row['path'] in {'.opencodex-native-main.claim.sqlite'+suffix for suffix in ['', '-journal', '-wal', '-shm']} for row in frame['tree']['entries'])
        assert sha((vault/'startup-scope.json').read_bytes())==refs['scopeSha256'] and [p for p,h in refs['phases']]==['ready','pre_stop','stopped'] and refs['liveSha256'] is not None
        prior=refs['originalSha256']
        for phase,digest in refs['phases']:
            raw=(vault/{'ready':'ready.json','pre_stop':'pre-stop.json','stopped':'stopped.json'}[phase]).read_bytes();assert sha(raw)==digest
            f=json.loads(raw);assert f['attempt']==ticket and f['previousSha256']==prior and f['originalRootAbsence']==origin;prior=digest
        if start==0:
            movement=last['rootRetirement'];assert movement['sourceRevision']==7
            raw=(vault/('root-retirement-'+movement['plan']['id']+'.json')).read_bytes();assert sha(raw)==movement['plan']['sha256'] and json.loads(raw)['source']==events[6]
            raw=(vault/('root-retirement-'+movement['plan']['id']+'-Retire-readback.json')).read_bytes();assert sha(raw)==last['completionSha256']
        else:assert last['completionSha256'] is None and 'rootRetirement' not in last and (d/'.codex').is_dir()
        identities.append(scope[8]);attempts.append(ticket)
    assert identities[0]!=identities[1] and attempts[0]!=attempts[1]
cleanup=json.loads((out/'cleanup-native.json').read_text());assert cleanup['diagnosticPassed'] is False and cleanup['rawOwnedBytesCapturedBeforeCleanup'] and cleanup['noOwnedProcessesByExecutableInventory']
assert cleanup['globalLExclusiveReleased'] and cleanup['preexistingGlobalLNeverDeletedRebuiltOrRestored'] and not cleanup['exactDigestAdoptTemporaryResiduals']
assert len(cleanup['exactOwnedCoordinatorsExclusivelyReacquiredAndRemoved'])==3
log=(out/'manager.log').read_text();assert log.count('stop-proof: original-native-launcher-and-service confirmed code=0')==2
assert 'failed phase=root-artifacts-held code=client_session_unconfirmed' in log
print(json.dumps({'accepted':False,'directedTwoCyclesPassed':False,'firstCycle':{'startMs':summary['startElapsedMs'],'statusMs':summary['statusElapsedMs'],'stopMs':summary['stopElapsedMs'],'stoppedStatusMs':cycles['firstStoppedStatusElapsedMs']},'secondCycle':{'startMs':cycles['secondStartElapsedMs'],'statusMs':cycles['secondLiveStatusElapsedMs'],'stopMs':cycles['secondStopElapsedMs'],'stopResult':'failed-client-restore-pending','retainedState':'restore_prepared','revision':16},'firstBindingAndNineEventsUnchanged':True,'twoIndependentOriginalAndThreePhaseChainsVerified':True,'secondRootNotRetiredAndNoCompletionInvented':True,'bothActualLauncherAndServiceNormalExitObserved':True,'globalLUnchangedAndReleased':True,'ownedCoordinatorsRemoved':3,'offlineReplayRemoved':True,'scope':'new frozen4163514fe trace hypothesis round retained as failed; prior10736ms and12878ms rounds unchanged, no budget relaxation or whole-version acceptance'},indent=2))

from pathlib import Path
import json,gzip,base64,hashlib,sqlite3,tempfile
b=Path(__file__).resolve().parent;sha=lambda data:hashlib.sha256(data).hexdigest();verified=[]
for label in ['cost-diagnostic','prefix-optimized','retained-writer','facts-coalesced']:
 runner=json.loads((b/label/'runner-summary.json').read_text());assert runner['exitCode']==101 and not runner['ownedProcessesRemaining'] and runner['accepted'] is False
 compressed=(b/label/'failed-original-bytes.json.gz').read_bytes();raw=gzip.decompress(compressed);manifest=json.loads((b/label/'failed-byte-manifest.json').read_text());assert sha(raw)==manifest['rawSha256'] and len(raw)==manifest['rawBytes'] and sha(compressed)==manifest['gzipSha256']
 payload=json.loads(raw);assert payload['originalRootStillExists']==(label in ['cost-diagnostic','prefix-optimized'])
 with tempfile.TemporaryDirectory(prefix='ocxd024-first-root-offline-') as d:
  scene=Path(d)
  for e in payload['files']:
   p=Path(e['path']);assert not p.is_absolute() and '..' not in p.parts;data=base64.b64decode(e['rawBase64']);assert len(data)==e['bytes'] and sha(data)==e['sha256'];dest=scene/p;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(data)
  manager=scene/'.codex-desktop/desktop/manager-state';c=sqlite3.connect((manager/'client-session-index.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True);head=json.loads(c.execute('select document from current_session').fetchone()[0]);events=[json.loads(r[0]) for r in c.execute('select document from events order by revision')];binding=json.loads(c.execute('select document from binding').fetchone()[0]);c.close()
  assert [h['state'] for h in events[:7]]==['preparing','capture_reserved','captured','ready','stop_prepared','stopped','restore_prepared']
  assert head==events[-1] and len(events)==head['revision'] and head['completionSha256'] is None
  refs=head['references'];vault=manager/'client-originals'/refs['attempt'];originalraw=(vault/'original.json').read_bytes();assert sha(originalraw)==refs['originalSha256'];original=json.loads(originalraw);assert original['schemaVersion']==5 and original['originalRootAbsence']==binding['originalRoot']
  assert sha((vault/'startup-scope.json').read_bytes())==refs['scopeSha256'] and sha((vault/'startup-live.json').read_bytes())==refs['liveSha256'];live=json.loads((vault/'startup-live.json').read_text())
  previous=refs['originalSha256']
  for phase,filename,digest in [(name,{'ready':'ready.json','pre_stop':'pre-stop.json','stopped':'stopped.json'}[name],digest) for name,digest in refs['phases']]:
   rawframe=(vault/filename).read_bytes();assert sha(rawframe)==digest;frame=json.loads(rawframe);assert frame['previousSha256']==previous and frame['originalRootAbsence']==binding['originalRoot'];previous=digest
   if phase=='ready':assert live['readySha256']==digest
  if True:
   assert head['state']=='root_retirement_prepared' and head['revision']==8
   movement=head['rootRetirement'];assert movement['sourceRevision']==7
   rawplan=(vault/('root-retirement-'+movement['plan']['id']+'.json')).read_bytes();assert sha(rawplan)==movement['plan']['sha256'];plan=json.loads(rawplan);assert plan['source']==events[6] and plan['origin']==binding['originalRoot'] and plan['schemaVersion']==2
   receipt=vault/('root-retirement-'+movement['plan']['id']+'-Retire-readback.json')
   if label=='facts-coalesced':
    assert receipt.exists();readback=json.loads(receipt.read_text());assert readback['sourceHead']==events[6] and readback['planSha256']==movement['plan']['sha256'] and readback['currentRootAbsent']
    tree=json.loads(json.dumps(plan['executionTree']));tree['entries']=[e for e in tree['entries'] if e['path']!='config.toml.ocx-write.lock'];tree['totalBytes']=sum(e.get('bytes',0) for e in tree['entries'] if e['kind']=='file');assert readback['executionTreeSha256']==sha(json.dumps(tree,separators=(',',':'),ensure_ascii=False).encode())
   else:assert not receipt.exists()
  else:assert head['state']=='restore_prepared' and head['revision']==7
 report=json.loads((b/label/'native/summary.json').read_text());assert report['start']['result']=='started' and report['startElapsedMs']<30000 and report['stop']['result'] is None and report['stop']['clientRecoveryRequired']
 log=(b/label/'manager.log').read_text();assert 'original-native-exit confirmed code=0' in log and 'original-native-launcher-and-service confirmed code=0' in log and 'captured-tcp-listeners absent count=4' in log
 verified.append({'round':label,'startPassedWithin30s':True,'normalServiceLauncherExitAndFourListenersAbsent':True,'independentState':head['state'],'indexRevision':head['revision'],'allPhaseAndSourceReferencesExact':True,'StopAccepted':False,'stopElapsedMs':report['stopElapsedMs']})
print(json.dumps({'accepted':False,'verifiedFailedRounds':verified,'failureRemains':'Stop10 still failed; final round disk readback does not independently close RootRetirementPrepared; next Start unaccepted'},indent=2))

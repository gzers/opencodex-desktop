from pathlib import Path
import json,hashlib,gzip,base64,sqlite3,tempfile
b=Path(__file__).resolve().parent;sha=lambda data:hashlib.sha256(data).hexdigest();verified=[]
for status in json.loads((b/'round-status.json').read_text())['rounds']:
 label=status['label'];manifest=json.loads((b/label/'failed-byte-manifest.json').read_text());compressed=(b/label/'failed-original-bytes.json.gz').read_bytes();raw=gzip.decompress(compressed);assert sha(compressed)==manifest['gzipSha256'] and len(raw)==manifest['rawBytes'] and sha(raw)==manifest['rawSha256'];payload=json.loads(raw)
 with tempfile.TemporaryDirectory(prefix='ocxd024-tail-offline-') as d:
  root=Path(d)
  for entry in payload['files']:
   p=Path(entry['path']);assert not p.is_absolute() and '..' not in p.parts;raw=base64.b64decode(entry['rawBase64']);assert len(raw)==entry['bytes'] and sha(raw)==entry['sha256'];dest=root/p;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(raw)
  manager=root/'.codex-desktop/desktop/manager-state';c=sqlite3.connect((manager/'client-session-index.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True);head=json.loads(c.execute('select document from current_session').fetchone()[0]);events=[json.loads(r[0]) for r in c.execute('select document from events order by revision')];binding=json.loads(c.execute('select document from binding').fetchone()[0]);c.close()
  assert head==json.loads((b/label/'failed-index-projection.json').read_text())['head']==events[-1] and len(events)==head['revision']
  assert [e['state'] for e in events[:7]]==['preparing','capture_reserved','captured','ready','stop_prepared','stopped','restore_prepared']
  assert head['completionSha256'] is None and head['state']==status['headState'] and head['revision']==status['headRevision']
  refs=head['references'];vault=manager/'client-originals'/refs['attempt'];raw=(vault/'original.json').read_bytes();assert sha(raw)==refs['originalSha256'];frame=json.loads(raw);assert frame['schemaVersion']==5 and frame['originalRootAbsence']==binding['originalRoot']
  assert sha((vault/'startup-scope.json').read_bytes())==refs['scopeSha256'] and sha((vault/'startup-live.json').read_bytes())==refs['liveSha256']
  previous=refs['originalSha256']
  for phase,digest in refs['phases']:
   filename={'ready':'ready.json','pre_stop':'pre-stop.json','stopped':'stopped.json'}[phase];raw=(vault/filename).read_bytes();assert sha(raw)==digest;v=json.loads(raw);assert v['previousSha256']==previous and v['originalRootAbsence']==binding['originalRoot'];previous=digest
  if head['state']=='root_retirement_prepared':
   movement=head['rootRetirement'];assert movement['sourceRevision']==7
   p=vault/('root-retirement-'+movement['plan']['id']+'.json');raw=p.read_bytes();assert sha(raw)==movement['plan']['sha256'];plan=json.loads(raw);assert plan['source']==events[6] and plan['origin']==binding['originalRoot']
   receipt=vault/('root-retirement-'+movement['plan']['id']+'-Retire-readback.json');assert receipt.exists()==status['terminalReceiptOnDisk']
   if receipt.exists():
    r=json.loads(receipt.read_text());assert r['planSha256']==movement['plan']['sha256'] and r['sourceHead']==events[6] and r['currentRootAbsent']
    assert events[8]['state']=='root_absent' and events[8]['completionSha256']==sha(receipt.read_bytes()) and events[9]['state']=='root_retirement_prepared'
  assert payload['originalRootStillExists']==status['rootExists']
 report=json.loads((b/label/'native/summary.json').read_text());assert report['start']['result']=='started' and report['startElapsedMs']<30000 and report['stop']['result'] is None and report['stop']['clientRecoveryRequired']
 log=(b/label/'manager.log').read_text();assert 'original-native-launcher-and-service confirmed code=0' in log and 'captured-tcp-listeners absent count=4' in log
 verified.append({'round':label,'exactRawBytesAndIndependentReferencesVerified':True,'indexState':head['state'],'revision':head['revision'],'sourceEligibility':status['eligibleSource'],'Stop10Accepted':False})
print(json.dumps({'accepted':False,'verified':verified,'rejectedSharedPrefixNotAcceptanceEvidence':True,'nextStartNotAccepted':True},indent=2))

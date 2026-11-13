from pathlib import Path
import json,base64,hashlib,sqlite3,tempfile
b=Path(__file__).resolve().parent;sha=lambda raw:hashlib.sha256(raw).hexdigest()
log=(b/'history-delivery-tests.log').read_text();rows=[json.loads(line.split('CLOSED_ROOT_HISTORY_EVIDENCE ',1)[1]) for line in log.splitlines() if 'CLOSED_ROOT_HISTORY_EVIDENCE ' in line];assert len(rows)==1
row=rows[0];assert row['accepted'] is False and row['storageOnlySyntheticClose'] and row['readonlyHistoryPassesWithNewCurrentRoots'] and row['currentAbsenceAndStartStillRefuse']
with tempfile.TemporaryDirectory(prefix='ocxd024-closed-history-verify-') as d:
 root=Path(d)
 for entry in row['files']:
  p=Path(entry['path']);assert not p.is_absolute() and '..' not in p.parts;raw=base64.b64decode(entry['rawBase64']);assert len(raw)==entry['bytes'] and sha(raw)==entry['sha256'];target=root/p;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw)
 manager=root/'desktop/manager-state';c=sqlite3.connect((manager/'client-session-index.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True)
 head=json.loads(c.execute('select document from current_session').fetchone()[0]);events=[json.loads(r[0]) for r in c.execute('select document from events order by revision')];binding=json.loads(c.execute('select document from binding').fetchone()[0]);c.close()
 assert head==row['closed'] and head['revision']==5 and head['state']=='root_absent' and events[-1]==head and len(events)==5
 assert [h['state'] for h in events]==['preparing','capture_reserved','captured','root_retirement_prepared','root_absent']
 source=row['source'];assert source==events[2];movement=head['rootRetirement'];assert movement['sourceRevision']==source['revision']==3 and events[3]['rootRetirement']==movement and events[3]['references']==head['references']
 refs=head['references'];vault=manager/'client-originals'/refs['attempt'];raw=(vault/'original.json').read_bytes();assert sha(raw)==refs['originalSha256'];frame=json.loads(raw);assert frame['schemaVersion']==5 and frame['originalRootAbsence']==row['origin']==binding['originalRoot'];assert sha((vault/'startup-scope.json').read_bytes())==refs['scopeSha256']
 raw=(vault/('root-retirement-'+movement['plan']['id']+'.json')).read_bytes();assert sha(raw)==movement['plan']['sha256'];plan=json.loads(raw);assert plan['source']==source and plan['origin']==row['origin']
 raw=(vault/('root-retirement-'+movement['plan']['id']+'-Retire-readback.json')).read_bytes();assert sha(raw)==head['completionSha256'];receipt=json.loads(raw);assert receipt['sourceHead']==source and receipt['planSha256']==movement['plan']['sha256'] and receipt['currentRootAbsent'] and receipt['rootIdentity']==plan['executionTree']['root']==row['sameArchiveIdentity']
 assert receipt['executionTreeSha256']==sha(json.dumps(plan['executionTree'],separators=(',',':'),ensure_ascii=False).encode())
 assert (root/'profile/.codex/auth.json').read_bytes()==b'new synthetic user root' and (root/'profile/.grok/config.toml').read_bytes()==b'new current Grok setting'
 assert plan['grok']['state']=='absent' and frame['grok']['state']=='absent'
 print(json.dumps({'accepted':False,'verified':True,'independentSourcePlanClosureAndByteReferencesExact':True,'closedArchiveIdentityBound':True,'newLogicalRootAndGrokAreSeparateFromHistory':True,'onlyStorageSyntheticCloseNoStartPermission':True},indent=2))

from pathlib import Path
import json,base64,hashlib,sqlite3,gzip
b=Path(__file__).resolve().parent;sha=lambda v:hashlib.sha256(v).hexdigest();out=b/'offline-native-delivery-projections';out.mkdir(mode=0o700);rows=[]
for label in ['official-delivery','official-delivery-late']:
 data=json.loads(gzip.decompress((b/label/'root-transient-original-bytes.json.gz').read_bytes()));known=json.loads((b/label/'root-transient-references.json').read_text());scene=out/label;scene.mkdir(mode=0o700)
 for e in data['files']:
  p=Path(e['path']);assert not p.is_absolute() and '..' not in p.parts
  raw=base64.b64decode(e['rawBase64']);assert sha(raw)==e['sha256'] and len(raw)==e['bytes'];dest=scene/p;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(raw)
 conn=sqlite3.connect((scene/'desktop/manager-state/client-session-index.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True);head=json.loads(conn.execute('select document from current_session').fetchone()[0]);events=[json.loads(r[0]) for r in conn.execute('select document from events order by revision')];binding=json.loads(conn.execute('select document from binding').fetchone()[0]);conn.close()
 source=known['source'];assert source==events[2] and events[3]==known['prepared'];refs=source['references'];vault=scene/'desktop/manager-state/client-originals'/refs['attempt'];planraw=(vault/('root-retirement-'+known['planId']+'.json')).read_bytes();assert sha(planraw)==known['planSha256'];plan=json.loads(planraw)
 assert plan['schemaVersion']==2 and plan['source']==source and plan['origin']==binding['originalRoot']==known['originalRoot']
 assert sha((vault/'original.json').read_bytes())==refs['originalSha256'] and sha((vault/'startup-scope.json').read_bytes())==refs['scopeSha256']
 original=json.loads((vault/'original.json').read_text());assert original['originalRootAbsence']==plan['origin'];assert not any(e['path']=='config.toml.ocx-write.lock' for e in original['tree']['entries'])
 marker=next(e for e in plan['executionTree']['entries'] if e['path']=='config.toml.ocx-write.lock');image=plan['writeGuard'];assert marker['identity']==image['identity'] and marker['protection']==image['protection'] and marker['sha256']==sha(bytes(image['bytes']))
 assert all(not e['path'].endswith(('-journal','-wal','-shm')) for e in plan['executionTree']['entries'])
 tree=json.loads(json.dumps(plan['executionTree']));tree['entries']=[e for e in tree['entries'] if e['path']!='config.toml.ocx-write.lock'];tree['totalBytes']=sum(e.get('bytes',0) for e in tree['entries'] if e['kind']=='file')
 receipt=vault/('root-retirement-'+known['planId']+'-Retire-readback.json');slot=vault/('root-retired-'+known['planId'])
 if label=='official-delivery':
  assert head==known['closed'] and head['state']=='root_absent' and head['revision']==5
  raw=receipt.read_bytes();assert sha(raw)==head['completionSha256'];r=json.loads(raw);assert r['sourceHead']==source and r['planSha256']==known['planSha256'] and r['currentRootAbsent'];assert r['executionTreeSha256']==sha(json.dumps(tree,separators=(',',':'),ensure_ascii=False).encode())
  assert not (slot/'config.toml.ocx-write.lock').exists()
 else:
  assert head==known['prepared'] and head['state']=='root_retirement_prepared' and head['completionSha256'] is None and known['closed'] is None and known['expectedRefusal'] and not receipt.exists()
  conflicts=list(vault.glob('*-conflict-*.json'));assert len(conflicts)==1;conflict=json.loads(conflicts[0].read_text());assert conflict['expected']==plan['executionTree']['root'] and conflict['observed']!=conflict['expected'];assert (slot/'external').read_bytes()==b'late external root'
  retained=scene/'profile/retained-original-root/config.toml.ocx-write.lock';assert retained.read_bytes()==bytes(image['bytes'])
 assert len(events)==head['revision'] and events[-1]==head
 summary=json.loads((b/label/'native-summary.json').read_text());assert summary['result']=='passed' and summary['elapsedMs']<summary['budgetMs']==10000
 rows.append({'scene':label,'rawBytesAndReferenceHashesVerified':True,'exactGuardBoundAfterClaimJournalRelease':True,'state':head['state'],'terminalReceiptVerified':receipt.exists(),'elapsedMs':summary['elapsedMs'],'scope':'actual official Guard/native locks; explicit synthetic lifecycle permit'})
(b/'verified-native-delivery.json').write_text(json.dumps({'accepted':False,'verified':rows,'noProductionStartStopAcceptance':True},indent=2));print('verified',len(rows),'native composition scenarios')

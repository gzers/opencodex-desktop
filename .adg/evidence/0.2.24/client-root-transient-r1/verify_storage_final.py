from pathlib import Path
import json,hashlib,base64,sqlite3,tempfile
b=Path(__file__).resolve().parent;sha=lambda data:hashlib.sha256(data).hexdigest()
rows=[]
for line in (b/'root-transient-final-tests.log').read_text().splitlines():
 for tag in ['ROOT_TRANSIENT_EVIDENCE ','ROOT_TRANSIENT_EXIT_EVIDENCE ']:
  if tag in line:rows.append(json.loads(line.split(tag,1)[1]))
assert len(rows)==5
out=b/'offline-storage-final-projections';out.mkdir(mode=0o700);verified=[]
for row in rows:
 label=row.get('point','positive');scene=out/label;scene.mkdir(mode=0o700)
 for entry in row['files']:
  p=Path(entry['path']);assert not p.is_absolute() and '..' not in p.parts
  raw=base64.b64decode(entry['rawBase64']);assert len(raw)==entry['bytes'] and sha(raw)==entry['sha256'];dest=scene/p;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(raw)
 conn=sqlite3.connect((scene/'desktop/manager-state/client-session-index.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True)
 head=json.loads(conn.execute('select document from current_session').fetchone()[0]);events=[json.loads(r[0]) for r in conn.execute('select document from events order by revision')];binding=json.loads(conn.execute('select document from binding').fetchone()[0]);conn.close()
 source=row.get('source',row.get('known',{}).get('source'));assert events[2]==source and source['revision']==3
 movement=head['rootRetirement'];assert movement['sourceRevision']==3
 refs=source['references'];vault=scene/'desktop/manager-state/client-originals'/refs['attempt'];planraw=(vault/('root-retirement-'+movement['plan']['id']+'.json')).read_bytes();assert sha(planraw)==movement['plan']['sha256'];plan=json.loads(planraw)
 assert plan['schemaVersion']==2 and plan['source']==source and plan['origin']==binding['originalRoot']
 frame=json.loads((vault/'original.json').read_text());assert sha((vault/'original.json').read_bytes())==refs['originalSha256'] and sha((vault/'startup-scope.json').read_bytes())==refs['scopeSha256']
 assert not any(e['path']=='config.toml.ocx-write.lock' for e in frame['tree']['entries'])
 image=plan['writeGuard'];assert image['state']=='file' and image['protection']['mode']==0o600
 marker=next(e for e in plan['executionTree']['entries'] if e['path']=='config.toml.ocx-write.lock');assert marker['identity']==image['identity'] and marker['protection']==image['protection'] and marker['bytes']==len(image['bytes']) and marker['sha256']==sha(bytes(image['bytes']))
 tree=json.loads(json.dumps(plan['executionTree']));tree['entries']=[e for e in tree['entries'] if e['path']!='config.toml.ocx-write.lock'];tree['totalBytes']=sum(e.get('bytes',0) for e in tree['entries'] if e['kind']=='file')
 receipt=vault/('root-retirement-'+movement['plan']['id']+'-Retire-readback.json')
 if receipt.exists():
  value=json.loads(receipt.read_text());assert value['executionTreeSha256']==sha(json.dumps(tree,separators=(',',':'),ensure_ascii=False).encode());assert value['sourceHead']==source and value['currentRootAbsent'] and value['planSha256']==movement['plan']['sha256']
 if label=='positive':assert head==row['closed'] and head['state']=='root_absent' and sha(receipt.read_bytes())==head['completionSha256']
 else:assert row['actualWaitExitCode']==91 and head==row['known']['head'] and head['state']=='root_retirement_prepared' and head['completionSha256'] is None
 assert len(events)==head['revision'] and events[-1]==head
 verified.append({'scene':label,'allBytesAndReferenceHashesVerified':True,'markerExactIdentityAndBytes':True,'originalTreeDoesNotContainMarker':True,'releasedTreeReceiptChecked':receipt.exists(),'independentState':head['state'],'diskReceiptDoesNotClosePending':label!='positive'})
(b/'verified-storage-final.json').write_text(json.dumps({'accepted':False,'scope':'offline original-byte supplement only; synthetic marker permissions are not native authority','verified':verified},indent=2));print('verified',len(verified),'storage scenarios')

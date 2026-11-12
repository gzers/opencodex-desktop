from pathlib import Path
import json,base64,hashlib,sqlite3
base=Path('/private/tmp/ocxd024-root-index-e8pu52gt');log=(base/'root-index-logical-scope-tests.log').read_text();sha=lambda b:hashlib.sha256(b).hexdigest();rows=[]
for line in log.splitlines():
 for marker,positive in [('INDEXED_ROOT_EXIT_EVIDENCE ',False),('INDEXED_ROOT_ABSENT_EVIDENCE ',True)]:
  if marker in line:rows.append((positive,json.loads(line.split(marker,1)[1])))
assert len(rows)==4 and sum(pos for pos,row in rows)==1
out=base/'offline-byte-projections';out.mkdir(mode=0o700);verified=[]
for positive,row in rows:
 label='positive' if positive else row['point'];assert row['accepted'] is False
 if not positive:assert row['childExitCode']==91 and row['storageOnlyNoProductionNativePermission']
 scene=out/label;scene.mkdir(mode=0o700);files={}
 for item in row['files']:
  p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts;b=base64.b64decode(item['rawBase64']);assert len(b)==item['bytes'] and sha(b)==item['sha256'];target=scene/p;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(b);files[str(p)]=item
 manager=scene/'manager-state';conn=sqlite3.connect((manager/'client-session-index.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True);head=json.loads(conn.execute('SELECT document FROM current_session').fetchone()[0]);events=[json.loads(r[0]) for r in conn.execute('SELECT document FROM events ORDER BY revision')];binding=json.loads(conn.execute('SELECT document FROM binding').fetchone()[0]);conn.close()
 expected=row['closed'] if positive else row['head'];assert head==expected and len(events)==head['revision'] and events[-1]==head
 movement=head['rootRetirement'];source=next(e for e in events if e['revision']==movement['sourceRevision']);assert source['revision']==3 and source['state']=='captured' and source['references']==head['references'] and source.get('rootRetirement') is None
 refs=source['references'];vault=manager/'client-originals'/refs['attempt'];planfile=vault/('root-retirement-'+movement['plan']['id']+'.json');assert sha(planfile.read_bytes())==movement['plan']['sha256'];plan=json.loads(planfile.read_text());assert plan['source']==source and plan['origin']==binding['originalRoot'] and plan['origin']['id']==source['ticket']
 assert sha((vault/'original.json').read_bytes())==refs['originalSha256'] and sha((vault/'startup-scope.json').read_bytes())==refs['scopeSha256']
 frame=json.loads((vault/'original.json').read_text());assert frame['originalRootAbsence']==plan['origin'] and frame['clientRoot']==plan['executionTree']['root']
 assert events[3]['state']=='root_retirement_prepared' and events[3]['rootRetirement']==movement
 if head['state']=='root_absent':
  assert head['revision']==5 and row['currentRootAbsent'] and head['completionSha256']
  raw=(vault/('root-retirement-'+movement['plan']['id']+'-Retire-readback.json')).read_bytes();assert sha(raw)==head['completionSha256'];receipt=json.loads(raw);assert receipt['planSha256']==movement['plan']['sha256'] and receipt['sourceHead']==source and receipt['currentRootAbsent']
  assert receipt['rootIdentity']==plan['executionTree']['root'] and receipt['logicalClient']==json.loads((vault/'startup-scope.json').read_text())['client']
  assert receipt['executionTreeSha256']==sha(json.dumps(plan['executionTree'],separators=(',',':'),ensure_ascii=False).encode())
 else:
  assert head['state']=='root_retirement_prepared' and head['revision']==4 and head['completionSha256'] is None
 if positive:assert row['prepared']==events[3] and row['source']==source and row['storageOnlyNoProductionAuthority']
 verified.append({'scene':label,'allRawByteHashesVerified':True,'sourceEventAndImmutablePlanExactMatch':True,'originFrameAndStartupScopeReferencesMatch':True,'indexPhase':head['state'],'indexRevision':head['revision'],'terminalReceiptVerifiedOnlyWhenIndependentlyClosed':head['state']=='root_absent','noNativeServiceAuthorizationClaim':True})
(base/'verified-evidence.json').write_text(json.dumps({'accepted':False,'scope':'Offline original-byte/SQLite supplement to real storage/index tests; synthetic move factory is not production service permission','verified':verified},indent=2));print('verified',len(verified),'root-index scenarios')

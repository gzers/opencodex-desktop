from pathlib import Path
import json,base64,hashlib,sqlite3
base=Path('/private/tmp/ocxd024-root-retirement-c7h34dv0');sha=lambda b:hashlib.sha256(b).hexdigest()
rows=[]
for line in (base/'root-retirement-delivery-tests.log').read_text().splitlines():
 marker='ROOT_RETIRE_CRASH_EVIDENCE '
 if marker in line:rows.append(json.loads(line.split(marker,1)[1]))
assert len(rows)==4 and {r['point'] for r in rows}=={'intent','after_native','before_readback','readback'}
marker='ROOT_RETIRE_POSITIVE_EVIDENCE ';positive=[json.loads(line.split(marker,1)[1]) for line in (base/'archive-bootstrap-tests.log').read_text().splitlines() if marker in line];assert len(positive)==1
out=base/'offline-byte-projections';out.mkdir(mode=0o700);verified=[]
for label,row in [('positive',positive[0])]+[(r['point'],r) for r in rows]:
 assert row['accepted'] is False
 if label!='positive':assert row['actualChildExitCode']==91 and row['sameRootPreserved'] and row['rootAbsentAfterResume']
 scene=out/label;scene.mkdir(mode=0o700);files={}
 for item in row['files']:
  p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts;raw=base64.b64decode(item['rawBase64']);assert len(raw)==item['bytes'] and sha(raw)==item['sha256'];target=scene/p;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(raw);files[str(p)]=item
 source=row['sourceHead'];refs=source['references'];vault=scene/'desktop/manager-state/client-originals'/refs['attempt']
 candidates=[p for p in vault.glob('root-retirement-*.json') if sha(p.read_bytes())==row['planSha256']];assert len(candidates)==1
 planfile=candidates[0];plan=json.loads(planfile.read_text());assert plan['source']==source and plan['origin']['id']==refs['attempt']==source['ticket'];assert plan['schemaVersion']==1
 original=vault/'original.json';scope=vault/'startup-scope.json';assert sha(original.read_bytes())==refs['originalSha256'] and sha(scope.read_bytes())==refs['scopeSha256']
 frame=json.loads(original.read_text());logical=json.loads(scope.read_text())['client'];assert frame['schemaVersion']==5 and frame['originalRootAbsence']==plan['origin']
 assert plan['executionTree']['root']==frame['clientRoot'] and plan['executionTree']['rootProtection']==frame['tree']['rootProtection'] and frame['tree']['entries']==[]
 for p,k in [('intent.json','intentSha256'),('created.json','createdSha256'),('published.json','publishedSha256')]:assert sha((scene/'desktop/manager-state/client-root-reservation'/p).read_bytes())==plan['origin'][k]
 conn=sqlite3.connect((scene/'desktop/manager-state/client-session-index.sqlite').as_uri()+'?mode=ro&immutable=1',uri=True);head=json.loads(conn.execute('SELECT document FROM current_session').fetchone()[0]);binding=json.loads(conn.execute('SELECT document FROM binding').fetchone()[0]);events=[json.loads(v[0]) for v in conn.execute('SELECT document FROM events ORDER BY revision')];conn.close()
 assert head==source and head['state']=='captured' and head['revision']==3 and len(events)==3 and binding['originalRoot']==plan['origin']
 receipt=vault/('root-retirement-'+plan['id']+'-Retire-readback.json');value=json.loads(receipt.read_text());assert value['planSha256']==row['planSha256'] and value['sourceHead']==source and value['originalRootAbsent'] and value['currentRootAbsent'] and value['logicalClient']==logical and value['archivedSlot']=='root-retired-'+plan['id'] and value['rootIdentity']==frame['clientRoot']
 assert value['executionTreeSha256']==sha(json.dumps(plan['executionTree'],separators=(',',':'),ensure_ascii=False).encode()) and value['grokSha256']==sha(json.dumps(plan['grok'],separators=(',',':'),ensure_ascii=False).encode())
 if label=='positive':assert row['rootAbsent'] and row['archivedRootIdentity']==frame['clientRoot']
 verified.append({'scene':label,'allOriginalByteHashesMatch':True,'immutablePlanSourceMatchesIndependentCapturedHead':True,'sourceHeadRemainsCapturedNotCompleted':True,'rootOriginAndFrameScopeHashesMatch':True,'readbackMatchesPlanTreeAndLogicalClient':True,'wholeDirectoryMovedWithoutNewOriginal':True})
(base/'verified-evidence.json').write_text(json.dumps({'accepted':False,'scope':'Offline byte/SQLite supplement to filesystem tests; not a live process or production Stop certificate','verifiedScenes':verified,'sourceIndexStillCapturedInAllScenes':True,'noIndependentRootRestoredTerminalClaim':True},indent=2));print('verified',len(verified),'whole-root storage scenarios')

from pathlib import Path
import json,base64,hashlib,sqlite3
base=Path('/private/tmp/ocxd024-root-originals-9v8it4h8');log=(base/'root-final-capture-tests.log').read_text();marker='ROOT_ORIGINAL_CRASH_EVIDENCE '
rows=[json.loads(line.split(marker,1)[1]) for line in log.splitlines() if marker in line];assert len(rows)==4
assert {r['point'] for r in rows}=={'directory_created','codex_archived','grok_archived','original_published'}
out=base/'offline-byte-projections';out.mkdir(mode=0o700);verified=[]
for row in rows:
 assert row['childExitCode']==91 and row['indexState']=='capture_reserved' and row['indexRevision']==2 and not row['originalIndependentlyRegistered']
 scene=out/row['point'];scene.mkdir(mode=0o700);files={}
 for item in row['files']:
  path=Path(item['path']);assert not path.is_absolute() and '..' not in path.parts;raw=base64.b64decode(item['rawBase64']);assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256'];p=scene/path;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(raw);files[item['path']]=item
 manager=scene/'desktop/manager-state';db=manager/'client-session-index.sqlite';c=sqlite3.connect(db.as_uri()+'?mode=ro&immutable=1',uri=True);head=json.loads(c.execute('SELECT document FROM current_session').fetchone()[0]);binding=json.loads(c.execute('SELECT document FROM binding').fetchone()[0]);events=[json.loads(r[0]) for r in c.execute('SELECT document FROM events ORDER BY revision')];c.close()
 assert head['revision']==2 and head['state']=='capture_reserved' and head['references'] is None and len(events)==2 and [v['state'] for v in events]==['preparing','capture_reserved'] and events[-1]==head
 intent=json.loads((manager/'client-root-reservation/intent.json').read_text());assert intent['originalRootAbsent'] and intent['id']==head['ticket']==binding['originalRoot']['id'];assert binding['originalRoot']['intentSha256']==hashlib.sha256((manager/'client-root-reservation/intent.json').read_bytes()).hexdigest()
 original=manager/'client-originals'/head['ticket']/'original.json';assert original.exists()==row['originalFilePresent']==(row['point']=='original_published')
 if original.exists():
  frame=json.loads(original.read_text());assert frame['schemaVersion']==5 and frame['originalRootAbsence']==binding['originalRoot'] and frame['phase']=='original' and frame['attempt']==head['ticket'] and frame['tree']['entries']==[]
 verified.append({'point':row['point'],'actualChildExitCode':91,'independentIndexRevision':2,'indexReferencesAbsent':True,'rootAbsenceSourceBoundToIndex':True,'originalFilePresentButNotAdopted':row['originalFilePresent'],'allRawByteHashesMatch':True})
marker='ROOT_CAPTURE_REGISTRATION_EVIDENCE ';positive=[json.loads(line.split(marker,1)[1]) for line in log.splitlines() if marker in line];assert len(positive)==1;p=positive[0];head=p['head'];frame_raw=base64.b64decode(p['originalFrameRawBase64']);scope_raw=base64.b64decode(p['scopeRawBase64']);frame=json.loads(frame_raw);scope=json.loads(scope_raw)
assert head['state']=='captured' and head['revision']==3 and head['references']['phases']==[] and head['references']['liveSha256'] is None
assert hashlib.sha256(frame_raw).hexdigest()==head['references']['originalSha256']==scope['originalSha256']
assert hashlib.sha256(scope_raw).hexdigest()==head['references']['scopeSha256']
assert frame['schemaVersion']==5 and frame['originalRootAbsence']==p['rootOrigin'] and frame['tree']['entries']==[] and frame['attempt']==head['ticket']==p['rootOrigin']['id'] and scope['attempt']==head['ticket']
(base/'verified-evidence.json').write_text(json.dumps({'accepted':False,'scope':'Offline byte/SQLite supplement to real child tests; no service authority','crashScenes':verified,'positiveRegisteredCapture':{'state':'captured','revision':3,'originIndependentReferencePreserved':True,'frameSchema':5,'createdExecutionDirectoryEmpty':True,'originalUserRootAbsentExplicitlyPreserved':True,'originalAndScopeDigestsMatchIndependentHead':True,'noReadyOrCompletionClaim':True}},indent=2))
(base/'registered-positive-frame.json').write_bytes(frame_raw);(base/'registered-positive-scope.json').write_bytes(scope_raw);print('Verified4 actual crash scenes and1 independently registered original/scope')

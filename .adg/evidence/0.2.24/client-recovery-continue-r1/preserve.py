from pathlib import Path
import json,sqlite3,hashlib,shutil,stat,sys
base=Path('/private/tmp/ocxd024-recovery-continue-kxrb590b')
for label in sys.argv[1:]:
 out=base/label;r=json.loads((out/'runner-summary.json').read_text());root=Path(r['temporaryRoot'])/'fixtures';data=root/'profile/.codex-desktop/desktop';idx=data/'manager-state/client-session-index.sqlite';c=sqlite3.connect(idx.as_uri()+'?mode=ro&immutable=1',uri=True);head=json.loads(c.execute('SELECT document FROM current_session').fetchone()[0]);events=[json.loads(row[0]) for row in c.execute('SELECT document FROM events ORDER BY revision')];assert c.execute('PRAGMA user_version').fetchone()[0]==3;c.close();refs=head['references'];vault=data/'manager-state/client-originals'/refs['attempt'];saved=out/'preserved-session';saved.mkdir();shutil.copy2(idx,saved/idx.name);(saved/'index-readback.json').write_text(json.dumps({'head':head,'events':events},indent=2));expected={'original.json':refs['originalSha256'],'startup-scope.json':refs['scopeSha256'],'startup-live.json':refs['liveSha256']}
 for phase,sha in refs['phases']:expected[{'ready':'ready.json','pre_stop':'pre-stop.json','stopped':'stopped.json'}[phase]]=sha
 if head.get('restorePlan'):expected['restore-'+head['restorePlan']['id']+'.json']=head['restorePlan']['sha256']
 if head.get('completionSha256'):expected['completion-readback.json']=head['completionSha256']
 for name,sha in expected.items():
  raw=(vault/name).read_bytes();assert hashlib.sha256(raw).hexdigest()==sha;shutil.copy2(vault/name,saved/name)
 for p in vault.glob('restore-*-*.json'):
  if (saved/p.name).exists():assert (saved/p.name).read_bytes()==p.read_bytes()
  else:shutil.copy2(p,saved/p.name)
 contents=[]
 def visit(v,frame,scope):
  if isinstance(v,list):
   for x in v:visit(x,frame,scope)
  elif isinstance(v,dict):
   if 'sha256' in v and 'bytes' in v and len(str(v['sha256']))==64:
    p=vault/('tree-content-'+v['sha256']+'.blob')
    if p.exists():
     raw=p.read_bytes();assert hashlib.sha256(raw).hexdigest()==v['sha256'] and len(raw)==v['bytes'] and stat.S_IMODE(p.stat().st_mode)==0o400;contents.append({'frame':frame,'scope':scope,'sha256':v['sha256'],'bytes':len(raw)})
   for x in v.values():visit(x,frame,scope)
 for name in ['original.json','ready.json','pre-stop.json','stopped.json']:
  f=json.loads((vault/name).read_text());assert f['schemaVersion']==4
  for scope in ['tree','grok']:visit(f[scope],name,scope)
 (saved/'content-readback.json').write_text(json.dumps(contents,indent=2))
 scope=json.loads((vault/'startup-scope.json').read_text());f=json.loads((vault/'ready.json').read_text());catalog=json.loads(bytes(f['capture']['images'][0]['bytes']));rows=[{'slug':v['slug'],'displayName':v.get('display_name')} for v in catalog['models']]
 if label=='native':
  expectedroutes=['native-fixture/claude-opus-5-5','native-fixture/gpt-6-sol','native-fixture/vendor-gemini-3.5-pro'];assert scope['declaredRoutes']==expectedroutes and sorted(v['slug'] for v in rows)==expectedroutes
  assert not any('unselected-live-fixture' in v['slug'] or 'stale-known-model' in v['slug'] for v in rows)
 (out/'route-selection-proof.json').write_text(json.dumps({'indexState':head['state'],'independentBeforeDiscoveryRoutes':scope['declaredRoutes'],'actualReadyRows':rows,'localDiscovery':json.loads((out/'native/summary.json').read_text()).get('localModelDiscovery'),'disabledRoutedModelExcluded':label=='native','refsAndContentChecks':len(contents)},indent=2));print(label,head['revision'],len(contents),'content refs;',len(rows),'actual rows')

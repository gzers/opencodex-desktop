from pathlib import Path
import json,sqlite3,hashlib,shutil,stat,base64,subprocess,os
base=Path('/private/tmp/ocxd024-session-close-v9lwsp8w/evidence');out=base/'native';r=json.loads((out/'runner-summary.json').read_text());root=Path(r['temporaryRoot'])/'fixtures';data=root/'profile/.codex-desktop/desktop';idx=data/'manager-state/client-session-index.sqlite';c=sqlite3.connect(idx.as_uri()+'?mode=ro&immutable=1',uri=True);version=c.execute('PRAGMA user_version').fetchone()[0];head=json.loads(c.execute('SELECT document FROM current_session').fetchone()[0]);events=[json.loads(row[0]) for row in c.execute('SELECT document FROM events ORDER BY revision')];c.close();assert version==3 and head['revision']==14 and head['state']=='restored' and [e['revision'] for e in events]==list(range(1,15))
dest=out/'preserved-sessions';dest.mkdir();shutil.copy2(idx,dest/idx.name);(dest/'index-readback.json').write_text(json.dumps({'schema':version,'head':head,'events':events},indent=2))
completed=[e for e in events if e['state']=='restored'];assert len(completed)==2 and completed[0]['revision']==7 and completed[0]['ticket']!=completed[1]['ticket'];rows=[]
for e in completed:
 refs=e['references'];vault=data/'manager-state/client-originals'/refs['attempt'];saved=dest/refs['attempt'];saved.mkdir();expected={'original.json':refs['originalSha256'],'startup-scope.json':refs['scopeSha256'],'startup-live.json':refs['liveSha256'],'completion-readback.json':e['completionSha256'],'restore-'+e['restorePlan']['id']+'.json':e['restorePlan']['sha256']}
 for phase,sha in refs['phases']:expected[{'ready':'ready.json','pre_stop':'pre-stop.json','stopped':'stopped.json'}[phase]]=sha
 for name,sha in expected.items():
  raw=(vault/name).read_bytes();assert hashlib.sha256(raw).hexdigest()==sha;shutil.copy2(vault/name,saved/name)
 for p in vault.glob('restore-*-*.json'):
  target=saved/p.name
  if target.exists():assert target.read_bytes()==p.read_bytes()
  else:shutil.copy2(p,target)
 rec=json.loads((vault/'completion-readback.json').read_text());assert rec['references']==refs and rec['plan']==e['restorePlan'];outcome=vault/('restore-'+e['restorePlan']['id']+'-outcome.json');assert hashlib.sha256(outcome.read_bytes()).hexdigest()==rec['outcomeSha256'] and json.loads(outcome.read_text())=={'schemaVersion':1,'planSha256':e['restorePlan']['sha256'],'decision':'complete'}
 entries=[]
 def visit(v,frame,scope):
  if isinstance(v,list):
   for x in v:visit(x,frame,scope)
  elif isinstance(v,dict):
   if 'sha256' in v and 'bytes' in v and len(str(v['sha256']))==64:
    p=vault/('tree-content-'+v['sha256']+'.blob')
    if p.exists():
     raw=p.read_bytes();assert hashlib.sha256(raw).hexdigest()==v['sha256'] and len(raw)==v['bytes'] and stat.S_IMODE(p.stat().st_mode)==0o400;entries.append({'frame':frame,'scope':scope,'sha256':v['sha256'],'bytes':len(raw)})
   for x in v.values():visit(x,frame,scope)
 for name in ['original.json','ready.json','pre-stop.json','stopped.json']:
  f=json.loads((vault/name).read_text());assert f['schemaVersion']==4
  for scope in ['tree','grok']:visit(f[scope],name,scope)
 (saved/'content-readback.json').write_text(json.dumps(entries,indent=2));rows.append({'ticket':e['ticket'],'revision':e['revision'],'attempt':refs['attempt'],'immutableBoundFiles':len(expected),'contentReferencesVerified':len(entries),'completionSha256':e['completionSha256'],'archivedPostimages':[p.name for p in saved.glob('restore-*-old-*')]})
(dest/'two-session-readback.json').write_text(json.dumps({'twoIndependentOriginalsRetained':True,'all14EventsRetained':True,'bothCompletedReceiptsIndependentlyBound':True,'sessions':rows},indent=2))
# Read-only exact native final snapshots against second receipt.
rs=[json.loads(l) for l in (base/'read-only-build.jsonl').read_text().splitlines() if l.startswith('{')];exe=Path(next(x['executable'] for x in rs if x.get('reason')=='compiler-artifact' and x.get('target',{}).get('name')=='opencodex_desktop_lib' and x.get('executable')))
p=subprocess.run([str(exe),'--exact','infrastructure::macos_client_originals::completion::tests::inspect_owned_completion_receipt_without_rewriting_it','--ignored','--nocapture'],env=dict(os.environ,OCXD024_COMPLETION_READBACK_ROOT=str(root),TMPDIR='/private/tmp/'),capture_output=True);(out/'final-tree-readback.stdout.log').write_bytes(p.stdout);(out/'final-tree-readback.stderr.log').write_bytes(p.stderr);assert p.returncode==0
v=json.loads(next(l for l in p.stdout.decode().splitlines() if l.startswith('{')));assert v['receiptSha256']==head['completionSha256'] and v['independentCurrentHashesMatchReceipt']
for name in ['codex','grok']:(out/(name+'-final-tree.json')).write_bytes(base64.b64decode(v[name+'TreeBase64']))
print('retained2successful sessions,14events,current native tree matches final receipt')

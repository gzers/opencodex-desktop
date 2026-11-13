from pathlib import Path
import json,sqlite3,hashlib,shutil,stat,sys
base=Path('/private/tmp/ocxd024-tree-close-5qznbx9d/evidence')
def copy_once(src,dst):
 if dst.exists():assert src.read_bytes()==dst.read_bytes()
 else:shutil.copy2(src,dst)
for label in sys.argv[1:]:
 out=base/label;r=json.loads((out/'runner-summary.json').read_text());root=Path(r['temporaryRoot'])/'fixtures';data=root/'profile/.codex-desktop/desktop';idx=data/'manager-state/client-session-index.sqlite';c=sqlite3.connect(idx.as_uri()+'?mode=ro&immutable=1',uri=True);head=json.loads(c.execute('SELECT document FROM current_session').fetchone()[0]);events=[json.loads(row[0]) for row in c.execute('SELECT document FROM events ORDER BY revision')];c.close();dest=out/'preserved-session';dest.mkdir(exist_ok=True);shutil.copy2(idx,dest/idx.name);(dest/'index-readback.json').write_text(json.dumps({'head':head,'events':events},indent=2));refs=head['references'];vault=data/'manager-state/client-originals'/refs['attempt'];expected={'original.json':refs['originalSha256'],'startup-scope.json':refs['scopeSha256'],'startup-live.json':refs['liveSha256']}
 for phase,sha in refs['phases']:expected[{'ready':'ready.json','pre_stop':'pre-stop.json','stopped':'stopped.json'}[phase]]=sha
 if head.get('restorePlan'):expected['restore-'+head['restorePlan']['id']+'.json']=head['restorePlan']['sha256']
 for name,sha in expected.items():
  raw=(vault/name).read_bytes();assert hashlib.sha256(raw).hexdigest()==sha;copy_once(vault/name,dest/name)
 for p in vault.glob('restore-*-*.json'):copy_once(p,dest/p.name)
 if (vault/'completion-readback.json').exists():
  receipt=json.loads((vault/'completion-readback.json').read_text());assert receipt['references']==refs and receipt['plan']==head['restorePlan'];copy_once(vault/'completion-readback.json',dest/'completion-readback.json');assert stat.S_IMODE((vault/'completion-readback.json').stat().st_mode)==0o400
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
  if (vault/name).exists():
   f=json.loads((vault/name).read_text());assert f['schemaVersion']==4
   for scope in ['tree','grok']:visit(f[scope],name,scope)
 (dest/'content-readback.json').write_text(json.dumps(contents,indent=2))
 print(label,'revision',head['revision'],'refs',len(expected),'content checks',len(contents),'completion',(dest/'completion-readback.json').exists())

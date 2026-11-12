from pathlib import Path
import json,sqlite3,hashlib,shutil,stat,sys
base=Path('/private/tmp/ocxd024-stop-profile-31v_ju2g/evidence')
def copy_once(src,dst):
 if dst.exists():assert src.read_bytes()==dst.read_bytes()
 else:shutil.copy2(src,dst)
for label in sys.argv[1:]:
 out=base/label;work=Path(json.loads((out/'runner-summary.json').read_text())['temporaryRoot'])
 data=work/'fixtures/profile/.codex-desktop/desktop';idx=data/'manager-state/client-session-index.sqlite'
 conn=sqlite3.connect(idx.as_uri()+'?mode=ro&immutable=1',uri=True)
 head=json.loads(conn.execute('SELECT document FROM current_session').fetchone()[0]);events=[json.loads(r[0]) for r in conn.execute('SELECT document FROM events ORDER BY revision')];conn.close()
 preserved=out/'preserved-session';preserved.mkdir(exist_ok=True);shutil.copy2(idx,preserved/idx.name)
 (preserved/'index-readback.json').write_text(json.dumps({'head':head,'events':events},indent=2))
 refs=head['references'];attempt=data/'manager-state/client-originals'/refs['attempt']
 expected={'original.json':refs['originalSha256'],'startup-scope.json':refs['scopeSha256'],'startup-live.json':refs['liveSha256']}
 for phase,sha in refs['phases']:expected[{'ready':'ready.json','pre_stop':'pre-stop.json','stopped':'stopped.json'}[phase]]=sha
 if head.get('restorePlan'):
  plan=head['restorePlan'];expected['restore-'+plan['id']+'.json']=plan['sha256']
 for name,sha in expected.items():
  raw=(attempt/name).read_bytes();assert hashlib.sha256(raw).hexdigest()==sha;copy_once(attempt/name,preserved/name)
 for p in attempt.glob('restore-*-preparing.json'):copy_once(p,preserved/p.name)
 rows=[]
 def visit(value,frame,scope):
  if isinstance(value,list):
   for v in value:visit(v,frame,scope)
  elif isinstance(value,dict):
   if 'sha256' in value and 'bytes' in value and len(str(value['sha256']))==64:
    sha=value['sha256'];p=attempt/('tree-content-'+sha+'.blob')
    if p.exists():
     raw=p.read_bytes();assert hashlib.sha256(raw).hexdigest()==sha;assert len(raw)==value['bytes'];assert stat.S_IMODE(p.stat().st_mode)==0o400
     rows.append({'frame':frame,'scope':scope,'sha256':sha,'bytes':len(raw),'mode':'0400'})
   for v in value.values():visit(v,frame,scope)
 for name in ['original.json','ready.json','pre-stop.json','stopped.json']:
  f=json.loads((attempt/name).read_text());assert f['schemaVersion']==4
  for scope in ['tree','grok']:visit(f[scope],name,scope)
 (preserved/'content-readback.json').write_text(json.dumps(rows,indent=2))
 stages=[]
 if head.get('restorePlan'):
  plan=json.loads((attempt/('restore-'+head['restorePlan']['id']+'.json')).read_text());stages=[]
  for i,s in enumerate(plan['staged']):
   if s is None:continue
   p=work/'fixtures/profile/.codex'/('.ocxd-restore-'+plan['id']+'-'+str(i));raw=p.read_bytes();stages.append({'slot':i,'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw),'stat':{'device':p.stat().st_dev,'inode':p.stat().st_ino},'planIdentity':s})
   if i==0:copy_once(p,preserved/'prepared-original-0')
 (preserved/'stage-readback.json').write_text(json.dumps(stages,indent=2))
 print(label,'revision',head['revision'],'bound',len(expected),'content checks',len(rows),'stages',len(stages))

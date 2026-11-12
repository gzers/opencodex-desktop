from pathlib import Path
import json,sqlite3,hashlib,shutil,stat
base=Path('/private/tmp/ocxd024-live-restore-3fgay2dl/evidence/native');r=json.loads((base/'runner-summary.json').read_text());work=Path(r['temporaryRoot']);data=work/'fixtures/profile/.codex-desktop/desktop';idx=data/'manager-state/client-session-index.sqlite';conn=sqlite3.connect(idx.as_uri()+'?mode=ro&immutable=1',uri=True);head=json.loads(conn.execute('SELECT document FROM current_session').fetchone()[0]);events=[json.loads(row[0]) for row in conn.execute('SELECT document FROM events ORDER BY revision')];conn.close();assert head['revision']==6
out=base/'preserved-session';out.mkdir();shutil.copy2(idx,out/idx.name);(out/'index-readback.json').write_text(json.dumps({'head':head,'events':events},indent=2));refs=head['references'];vault=data/'manager-state/client-originals'/refs['attempt'];expected={'original.json':refs['originalSha256'],'startup-scope.json':refs['scopeSha256'],'startup-live.json':refs['liveSha256']}
for phase,sha in refs['phases']:expected[{'ready':'ready.json','pre_stop':'pre-stop.json','stopped':'stopped.json'}[phase]]=sha
planref=head['restorePlan'];expected['restore-'+planref['id']+'.json']=planref['sha256']
for name,sha in expected.items():
 raw=(vault/name).read_bytes();assert hashlib.sha256(raw).hexdigest()==sha;shutil.copy2(vault/name,out/name)
for p in vault.glob('restore-'+planref['id']+'-*.json'):shutil.copy2(p,out/p.name)
plan=json.loads((vault/('restore-'+planref['id']+'.json')).read_text());terminal=vault/('restore-'+planref['id']+'-outcome.json');value=json.loads(terminal.read_text());assert value=={'schemaVersion':1,'planSha256':planref['sha256'],'decision':'complete'} and stat.S_IMODE(terminal.stat().st_mode)==0o400
names=['opencodex-catalog.json','models_cache.json','opencodex-catalog-audit.jsonl'];original=json.loads((vault/'original.json').read_text())['capture']['images'];stopped=json.loads((vault/'stopped.json').read_text())['capture']['images'];rows=[]
for i,name in enumerate(names):
 target=work/'fixtures/profile/.codex'/name;stage=target.parent/('.ocxd-restore-'+planref['id']+'-'+str(i));assert not stage.exists();saved=plan['staged'][i];arch=vault/('restore-'+planref['id']+'-old-'+str(i))
 raw=target.read_bytes() if target.exists() else None
 expectedbytes=bytes(original[i]['bytes']) if original[i]['state']=='file' else None
 assert raw==expectedbytes
 if saved:
  st=target.stat();assert (st.st_dev,st.st_ino)==(saved['device'],saved['inode'])
 archivebytes=arch.read_bytes() if arch.exists() else None;stoppedbytes=bytes(stopped[i]['bytes']) if stopped[i]['state']=='file' else None
 assert archivebytes==stoppedbytes
 if arch.exists():shutil.copy2(arch,out/arch.name)
 rows.append({'artifact':name,'originalBytesRestored':True,'stagesAbsent':True,'archivedStoppedPostimageMatches':True,'targetSha256':hashlib.sha256(raw).hexdigest() if raw is not None else None,'archiveSha256':hashlib.sha256(archivebytes).hexdigest() if archivebytes is not None else None})
(out/'publication-readback.json').write_text(json.dumps({'terminalPlanAndDecisionMatch':True,'artifacts':rows},indent=2))
contents=[]
def visit(v,frame,scope):
 if isinstance(v,list):
  for x in v:visit(x,frame,scope)
 elif isinstance(v,dict):
  if 'sha256' in v and 'bytes' in v and len(str(v['sha256']))==64:
   p=vault/('tree-content-'+v['sha256']+'.blob')
   if p.exists():
    raw=p.read_bytes();assert hashlib.sha256(raw).hexdigest()==v['sha256'] and len(raw)==v['bytes'] and stat.S_IMODE(p.stat().st_mode)==0o400
    contents.append({'frame':frame,'scope':scope,'sha256':v['sha256'],'bytes':len(raw)})
  for x in v.values():visit(x,frame,scope)
for name in ['original.json','ready.json','pre-stop.json','stopped.json']:
 f=json.loads((vault/name).read_text());assert f['schemaVersion']==4
 for scope in ['tree','grok']:visit(f[scope],name,scope)
(out/'content-readback.json').write_text(json.dumps(contents,indent=2));print('verified actual3publication/outcome/archive; content refs',len(contents))

from pathlib import Path
import os,json,hashlib,base64,stat,subprocess,sys,tempfile
HERE=Path(__file__).parent.resolve()
ALLOWED={'codex/config.toml','codex/opencodex-catalog.json','codex/models_cache.json','codex/opencodex-catalog-audit.jsonl','grok/config.toml.bak-opencodex'}
def digest(b):return hashlib.sha256(b).hexdigest()
def same(a,b):return a['state']==b['state'] and (a['state']=='absent' or (a['sha256']==b['sha256'] and a['mode']==b['mode']))
def read(root,rel):
 if rel not in ALLOWED:raise ValueError('path-outside-fixed-write-set')
 path=root/rel
 for parent in [path.parent]:
  while parent!=root:
   s=parent.lstat()
   if not stat.S_ISDIR(s.st_mode) or stat.S_ISLNK(s.st_mode):raise ValueError('unsafe-parent')
   parent=parent.parent
 try:before=path.lstat()
 except FileNotFoundError:return {'state':'absent'}
 if not stat.S_ISREG(before.st_mode) or stat.S_ISLNK(before.st_mode) or before.st_nlink!=1:raise ValueError('unsafe-target')
 fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW)
 try:
  opened=os.fstat(fd);data=b''
  while True:
   chunk=os.read(fd,65536)
   if not chunk:break
   data+=chunk
  after=os.fstat(fd);last=path.lstat()
  key=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns,s.st_nlink)
  if not (key(before)==key(opened)==key(after)==key(last)):raise ValueError('unstable-read')
 finally:os.close(fd)
 return {'state':'file','sha256':digest(data),'bytesBase64':base64.b64encode(data).decode(),'mode':stat.S_IMODE(before.st_mode)}
def canonical(plan):return json.dumps(plan,sort_keys=True,separators=(',',':')).encode()
def put(root,rel,value):
 path=root/rel
 if value['state']=='absent':path.unlink();return
 data=base64.b64decode(value['bytesBase64']);assert digest(data)==value['sha256']
 tmp=path.parent/(path.name+'.fixture-txn-tmp');fd=os.open(tmp,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
 try:
  os.write(fd,data);os.fchmod(fd,value['mode']);os.fsync(fd)
 finally:os.close(fd)
 os.replace(tmp,path)
 fd=os.open(path.parent,os.O_RDONLY)
 try:os.fsync(fd)
 finally:os.close(fd)
def restore(root,plan,expected,fail_at=None,crash_after=None):
 if digest(canonical(plan))!=expected:raise ValueError('plan-integrity-changed')
 for x in plan:
  if not same(read(root,x['path']),x['after']):raise ValueError('postimage-changed')
 # Immutable synthetic postimage evidence remains outside restored client tree.
 backup=root/'transaction-postimages.json';backup.write_bytes(canonical(plan));os.chmod(backup,0o600)
 journal=root/'transaction-state.json';journal.write_text(json.dumps({'state':'applying','planSha256':expected}));fd=os.open(journal,os.O_RDONLY);os.fsync(fd);os.close(fd)
 try:
  for i,x in enumerate(plan):
   if fail_at==i:raise RuntimeError('injected-sync-write-failure')
   if not same(read(root,x['path']),x['after']):raise ValueError('late-conflict')
   put(root,x['path'],x['before'])
   if crash_after==i:os._exit(91)
  for x in plan:assert same(read(root,x['path']),x['before'])
  journal.write_text(json.dumps({'state':'restored','planSha256':expected}))
 except Exception:
  recover(root,plan,expected);raise
 return len(plan)
def recover(root,plan,expected):
 if digest(canonical(plan))!=expected:raise ValueError('plan-integrity-changed')
 for x in plan:
  now=read(root,x['path'])
  if not same(now,x['before']) and not same(now,x['after']):raise ValueError('recovery-external-change')
 for x in reversed(plan):
  if not same(read(root,x['path']),x['after']):put(root,x['path'],x['after'])
 (root/'transaction-state.json').write_text(json.dumps({'state':'rolled-back-to-sealed-postimage','planSha256':expected}))
def setup(label,case='pipeline-attempt4'):
 root=Path(json.loads((HERE/'fixture.json').read_text())['root'])/'probe-cases-r2'/label;root.mkdir(parents=True)
 before=json.loads((HERE/case/'before.json').read_text())['files'];after=json.loads((HERE/case/'after.json').read_text())['files'];b={x['path']:x for x in before};plan=[]
 for a in after:
  if a['path'] not in ALLOWED:continue
  path=root/a['path'];path.parent.mkdir(exist_ok=True)
  if a['state']=='file':path.write_bytes(base64.b64decode(a['bytesBase64']));path.chmod(a['mode'])
  if not same(b[a['path']],a):plan.append({'path':a['path'],'before':b[a['path']],'after':a})
 return root,plan,digest(canonical(plan)),before
checks=[]
if len(sys.argv)>1 and sys.argv[1]=='crash-child':
 root=Path(sys.argv[2]);plan=json.loads((root/'plan.json').read_text());restore(root,plan,sys.argv[3],crash_after=0);raise AssertionError('did not crash')
for case in ['pipeline-attempt4','pipeline-preexisting']:
 root,plan,h,before=setup(case,case);assert restore(root,plan,h)>0
 for x in before:
  if x['path'] in ALLOWED:assert same(read(root,x['path']),x)
 assert (root/'transaction-postimages.json').exists();checks.append(case+': actual official changed artifacts restore exact originals/modes; synthetic audit retained in transaction evidence')
for kind in ['external-change','symlink','hardlink','missing','unreadable','tampered-plan','parent-link']:
 root,plan,h,_=setup(kind);target=root/plan[-1]['path'];first=read(root,plan[0]['path'])
 if kind=='external-change':target.write_text('synthetic external changed after seal')
 if kind=='symlink':target.unlink();target.symlink_to(root/plan[0]['path'])
 if kind=='hardlink':target.unlink();os.link(root/plan[0]['path'],target)
 if kind=='missing':target.unlink()
 if kind=='unreadable':target.chmod(0)
 if kind=='tampered-plan':plan[-1]['path']='codex/../../outside'
 if kind=='parent-link':parent=target.parent;other=root/'moved';parent.rename(other);parent.symlink_to(other,target_is_directory=True)
 try:restore(root,plan,h)
 except (ValueError,OSError):pass
 else:raise AssertionError(kind+' not refused')
 assert digest((root/plan[0]['path']).read_bytes())==first['sha256'];checks.append(kind+': whole preflight refused without earlier file write')
 if kind=='unreadable':target.chmod(0o600)
root,plan,h,_=setup('sync-failure')
try:restore(root,plan,h,fail_at=1)
except RuntimeError:pass
else:raise AssertionError('no failure')
assert all(same(read(root,x['path']),x['after']) for x in plan);checks.append('mid-batch injected failure rolls back to sealed official postimage')
root,plan,h,_=setup('process-crash');(root/'plan.json').write_bytes(canonical(plan));p=subprocess.run([sys.executable,str(Path(__file__)), 'crash-child',str(root),h],capture_output=True,text=True,timeout=10);assert p.returncode==91,p.stderr;recover(root,plan,h);assert all(same(read(root,x['path']),x['after']) for x in plan);restore(root,plan,h);checks.append('actual child abrupt exit after one write: fresh parent recovers postimage then exact restore succeeds')
root,plan,h,_=setup('unknown-extra');extra=root/'codex/user-added.txt';extra.write_text('user-content');restore(root,plan,h);assert extra.read_text()=='user-content';checks.append('unknown extra user file preserved and full-tree mismatch remains unaccepted')
root,plan,h,_=setup('unknown-recovery');put(root,plan[0]['path'],plan[0]['before']);target=root/plan[-1]['path'];target.write_text('new user value before restart');before=read(root,plan[0]['path'])
try:recover(root,plan,h)
except ValueError:pass
else:raise AssertionError('unrecognized state overwritten')
assert same(read(root,plan[0]['path']),before);checks.append('recovery refuses external contents before touching other files')
result={'result':'pass','checks':checks,'checkCount':len(checks),'scope':'POSIX isolated candidate compensation primitives against actual official pipeline captures; not product/native Windows acceptance','limits':['Only fixed synthetic transaction paths; observed afterimage is not authority by itself for daily user data','No POSIX advisory lock asserted to exclude arbitrary writers; Windows exclusive-handle/official-coordinator integration still required','Actual crash is bounded after-one-write child death; not exhaustive crash or production recovery proof','No unknown artifact cleanup and no extra lock exception']};(HERE/'compensation-result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2));print(json.dumps({'result':'pass','checks':len(checks)}))

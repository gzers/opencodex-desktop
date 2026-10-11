import subprocess,json,hashlib,fcntl,os,re,sys
from pathlib import Path
from datetime import datetime,timezone
root=Path('/private/tmp/ocx-0110-03e0-candidate-3wpphyvk/test-data')
exe=Path('/private/tmp/ocx-0110-03e0-candidate-3wpphyvk/test-install/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop')
daily='/Applications/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop'
out=Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/.adg/evidence/OCX-0110-20261011-03E0-MACOS-SAME-ROOT-LIFECYCLE')
rows=subprocess.run(['ps','-axo','pid=,lstart=,command='],capture_output=True,text=True,check=True).stdout.splitlines()
processes=[]
for row in rows:
 parts=row.strip().split(None,6)
 if len(parts)!=7: continue
 pid,started,cmd=int(parts[0]),' '.join(parts[1:6]),parts[6]
 if cmd not in (str(exe),daily): continue
 env={}
 if cmd==str(exe):
  raw=subprocess.run(['ps','eww','-p',str(pid),'-o','command='],capture_output=True,text=True,check=True).stdout
  for key in ['OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY']:
   m=re.search(r'(?:^| )'+key+r'=([^ ]+)',raw)
   env[key]=m.group(1).strip() if m else None
 processes.append({'pid':pid,'started':started,'executable':cmd,'sandbox_env':env})
lock=root/'manager-state/app.lock'
lockinfo={'path':str(lock),'exists':lock.exists()}
if lock.exists():
 lockinfo['metadata_before_probe']=json.loads(lock.read_text())
 with lock.open('rb') as handle:
  try:
   fcntl.flock(handle.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)
  except BlockingIOError: lockinfo['nonblocking_exclusive_acquired']=False
  else:
   lockinfo['nonblocking_exclusive_acquired']=True
   fcntl.flock(handle.fileno(),fcntl.LOCK_UN)
 lockinfo['metadata_after_probe']=json.loads(lock.read_text())
backups=[]
for item in sorted((root/'backups').rglob('*')):
 if item.is_file():
  data=item.read_bytes()
  backups.append({'relative_path':str(item.relative_to(root)),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
source='03e0ad942c16fe2ef75473eb6693012b3ae684ac'
value={'captured_at':datetime.now(timezone.utc).isoformat(),'source_sha':source,'test_executable':str(exe),'test_exe_sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'source_sha_embedded':source.encode() in exe.read_bytes(),'data_root':str(root),'processes':processes,'lock':lockinfo,'backups':backups,'snapshot_method':'ps filtered to two exact executable paths; environmental fields allowlisted; flock read-only no lock deletion/truncation; backup hashes only, no preference payload exported'}
target=out/(sys.argv[1]+'.json')
with target.open('x') as handle: json.dump(value,handle,ensure_ascii=False,indent=2);handle.write(chr(10))
print(json.dumps(value,ensure_ascii=False))

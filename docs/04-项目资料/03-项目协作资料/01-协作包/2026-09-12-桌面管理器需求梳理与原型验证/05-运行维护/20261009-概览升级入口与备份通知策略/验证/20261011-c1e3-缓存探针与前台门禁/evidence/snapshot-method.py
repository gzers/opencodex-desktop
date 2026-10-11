import subprocess, json, hashlib, fcntl, re, sys
from pathlib import Path
from datetime import datetime, timezone
base=Path('/private/tmp/ocx-0110-c1e3-candidate-hqjf10k6')
old=Path('/private/tmp/ocx-0110-369a-candidate-tgpmm_zq')
locations={'old':old,'new':base}
executables={name:str(path/'test-install/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop') for name,path in locations.items()}
executables['daily']='/Applications/OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop'
rows=subprocess.run(['ps','-axo','pid=,lstart=,command='],capture_output=True,text=True,check=True).stdout.splitlines()
processes=[]
for row in rows:
    parts=row.strip().split(None,6)
    if len(parts)!=7: continue
    pid,started,cmd=int(parts[0]),' '.join(parts[1:6]),parts[6]
    if cmd not in executables.values(): continue
    role=next(key for key,value in executables.items() if value==cmd)
    env={}
    if role!='daily':
        raw=subprocess.run(['ps','eww','-p',str(pid),'-o','command='],capture_output=True,text=True,check=True).stdout
        for key in ['OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY']:
            match=re.search(r'(?:^| )'+key+r'=([^ ]+)',raw)
            env[key]=match.group(1).strip() if match else None
    processes.append({'pid':pid,'started':started,'executable':cmd,'role':role,'sandbox_env':env})
roots={}
for name,path in locations.items():
    root=path/'test-data'
    lock=root/'manager-state/app.lock'
    info={'path':str(lock),'exists':lock.exists()}
    if lock.exists():
        data=lock.read_bytes()
        info['sha256_before_probe']=hashlib.sha256(data).hexdigest()
        info['metadata_before_probe']=json.loads(data)
        with lock.open('rb') as handle:
            try: fcntl.flock(handle.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)
            except BlockingIOError: info['nonblocking_exclusive_acquired']=False
            else:
                info['nonblocking_exclusive_acquired']=True
                fcntl.flock(handle.fileno(),fcntl.LOCK_UN)
        info['sha256_after_probe']=hashlib.sha256(lock.read_bytes()).hexdigest()
    backups=[]
    for item in sorted((root/'backups').rglob('*')):
        if item.is_file():
            data=item.read_bytes()
            backups.append({'relative_path':str(item.relative_to(root)),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
    exe=Path(executables[name])
    roots[name]={'data_root':str(root),'lock':info,'backups':backups,'exe_sha256':hashlib.sha256(exe.read_bytes()).hexdigest() if exe.exists() else None,'source_embedded':b'c1e3c09d8ed33238543ce5943e439991b8099088' in exe.read_bytes() if exe.exists() and name=='new' else None}
value={'captured_at':datetime.now(timezone.utc).isoformat(),'source_sha':'c1e3c09d8ed33238543ce5943e439991b8099088','processes':processes,'roots':roots,'method':'ps filtered to exact old/new/daily executables; only allowlisted sandbox env exported; read-only nonblocking flock; backup digest only'}
target=base/(sys.argv[1]+'.json')
with target.open('x') as handle: json.dump(value,handle,ensure_ascii=False,indent=2);handle.write(chr(10))
print(json.dumps(value,ensure_ascii=False))

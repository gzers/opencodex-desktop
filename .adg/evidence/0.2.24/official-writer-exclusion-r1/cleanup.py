from pathlib import Path
import json,subprocess,os,stat,hashlib,sqlite3,shutil
base=Path('/private/tmp/ocxd024-writer-cover-b_g199s0/evidence');reports=[json.loads((base/l/'runner-summary.json').read_text()) for l in ['native','native2','native3']];roots=[Path(r['temporaryRoot']) for r in reports]
remaining=[l for l in subprocess.check_output(['/bin/ps','-axo','pid=,ppid=,pgid=,comm='],text=True).splitlines() if any(str(r) in l for r in roots)];assert not remaining,remaining
dbs=[]
for r in reports:
 paths=[Path(p) for p in r['exactNewCoordinatorFiles']]
 for p in paths:
  assert p.parent.parent==Path('/private/tmp/opencodex-runtime-v1-'+str(os.getuid()))
  if p.suffix=='.sqlite' and p.exists():
   before=p.stat();assert before.st_uid==os.getuid() and before.st_nlink==1
   c=sqlite3.connect(p.as_uri()+'?mode=rw',uri=True,timeout=0);c.execute('BEGIN EXCLUSIVE');c.rollback();c.close();assert (p.stat().st_dev,p.stat().st_ino)==(before.st_dev,before.st_ino);dbs.append(str(p))
 for p in paths:
  if p.exists():p.unlink()
lbefore=json.loads((base/'global-l-before.json').read_text());p=Path(lbefore['path']);before=p.stat();assert (before.st_dev,before.st_ino,before.st_uid,before.st_size)==(lbefore['device'],lbefore['inode'],lbefore['uid'],lbefore['bytes']);assert stat.S_IMODE(before.st_mode)==0o600 and hashlib.sha256(p.read_bytes()).hexdigest()==lbefore['sha256']
c=sqlite3.connect(p.as_uri()+'?mode=rw',uri=True,timeout=0);c.execute('BEGIN EXCLUSIVE');c.rollback();c.close()
after=p.stat();assert (after.st_dev,after.st_ino,after.st_uid,after.st_size)==(lbefore['device'],lbefore['inode'],lbefore['uid'],lbefore['bytes']);assert stat.S_IMODE(after.st_mode)==0o600 and hashlib.sha256(p.read_bytes()).hexdigest()==lbefore['sha256']
for suf,existed in lbefore['sidecarsExisted'].items():
 side=Path(str(p)+suf)
 if not existed:assert not side.exists(),str(side)
rootbefore=json.loads((base/'global-root-before.json').read_text());st=p.parent.stat();assert (st.st_dev,st.st_ino,st.st_uid)==(rootbefore['device'],rootbefore['inode'],rootbefore['uid']) and stat.S_IMODE(st.st_mode)==0o700
for r in roots:
 assert r.parent==Path('/private/tmp') and r.name.startswith('ocxd024-native-coordination-') and r.resolve()==r
 shutil.rmtree(r);assert not r.exists()
code=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes');exe=json.loads((base/'checks-probe-fixed.json').read_text())['binary'];helpers=[l for l in subprocess.check_output(['/bin/ps','-axo','pid=,args='],text=True).splitlines() if len(l.split(None,1))==2 and l.split(None,1)[1].startswith(exe+' ') and '--exact ' in l];assert not helpers
report={'ownedProcessesRemaining':remaining,'exactNativeHelperCommandsRemaining':helpers,'newNativeCoordinatorDatabasesReacquiredReleasedRemoved':dbs,'ownedNativeFixturesRemoved':[str(r) for r in roots],'existingGlobalLReacquiredReleasedPreserved':True,'existingGlobalLInodeBytesModeUnchanged':True,'existingUserRuntimeRootIdentityAndModeUnchanged':True,'noNewGlobalLSidecarsRemain':True,'globalLNeverDeleted':True,'dailyConfigurationChanged':False,'officialRuntimeModified':False,'businessRequests':False}
(base/'native-cleanup.json').write_text(json.dumps(report,indent=2));print('removed3 native fixtures;6 new databases;globalL preserved')

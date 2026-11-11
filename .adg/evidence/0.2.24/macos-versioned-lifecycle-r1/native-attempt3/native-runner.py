from pathlib import Path
import hashlib,json,os,subprocess,shutil,signal,time
base=Path('/private/tmp/ocxd024-versioned-lifecycle-ihwdax9o')
code=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes')
root=base/'fixtures-r3';root.mkdir(exist_ok=True);(root/'tmp').mkdir(exist_ok=True)
profile=root/'profile';profile.mkdir(exist_ok=True)
out=base/'evidence/native-attempt3';out.mkdir(exist_ok=True)
resources=root/'resources';resources.mkdir(exist_ok=True)
source=code/'apps/desktop/tauri/target/release/bundle/macos/OpenCodeX Desktop.app/Contents/Resources/runtime-bundle'
for name in ['runtime-bundle.manifest.json','runtime-bundle-0.2.24-darwin-arm64.tar.gz']:

 if not (resources/name).exists():shutil.copy2(source/name,resources/name)
 else:assert (source/name).stat().st_size==(resources/name).stat().st_size
assert hashlib.sha256((resources/'runtime-bundle.manifest.json').read_bytes()).hexdigest()=='e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3'
exe=root/'lib-test';shutil.copy2(code/'apps/desktop/tauri/target/debug/deps/opencodex_desktop_lib-625891ad1214e6fb',exe)
client=profile/'.codex';client.mkdir(mode=0o700,exist_ok=True)
coord=Path('/private/tmp')/('opencodex-runtime-v1-'+str(os.getuid()))/'catalog-write-locks'/hashlib.sha256(str(client.resolve()).encode()).hexdigest()
locks=[Path(str(coord)+s) for s in ['.sqlite','.sqlite-journal','.sqlite-wal','.sqlite-shm']]
assert coord.parent.is_dir(), 'current official per-user coordinator namespace must already exist; do not initialize somebody else namespace'
assert not any(p.exists() for p in locks), 'fresh exact private CODEX_HOME hash required'
policy='(version 1)\n(allow default)\n(deny network*)\n(allow network-inbound (local ip "localhost:*"))\n(allow network-outbound (remote ip "localhost:*"))\n(deny file-write*)\n(allow file-write* (subpath '+json.dumps(str(root))+'))\n(allow file-write* (literal "/dev/null"))\n(deny file-read* (subpath "/Users/ezio"))\n(deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))\n'
native_coord=coord.parent.parent/'native-write-locks'/coord.name
locks += [Path(str(native_coord)+s) for s in ['.sqlite','.sqlite-journal','.sqlite-wal','.sqlite-shm']]
canonical_home=str(client.resolve());state=str(client.resolve()/'state_5.sqlite')
history_digest=hashlib.sha256((str(len(canonical_home))+':'+canonical_home+str(len(state))+':'+state).encode()).hexdigest()
history_coord=coord.parent.parent/'history-write-locks'/history_digest
assert history_coord.parent.is_dir(),'official namespace must exist; do not open whole namespace'
locks += [Path(str(history_coord)+s) for s in ['.sqlite','.sqlite-journal','.sqlite-wal','.sqlite-shm']]

assert not any(p.exists() for p in locks),'fresh owned coordinator inputs'
policy+=''.join('(allow file-write* (literal '+json.dumps(str(p))+'))\n' for p in locks)
import re
pattern='^'+re.escape(str(native_coord.parent/'x')).removesuffix('x')+'\\.'+native_coord.name+'\\.sqlite\\.adopt-[0-9]+-[0-9a-f-]+\\.tmp(-journal|-wal|-shm)?$'
# coord already contains home hash, without .sqlite; exact new HOME scope.
policy+='(allow file-write* (regex '+json.dumps(pattern)+'))\n'

sb=base/'native.sb';sb.write_text(policy);shutil.copy2(sb,out/'native.sb')
env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(profile),'CODEX_HOME':str(client),'TMPDIR':str(root/'tmp')+'/','LANG':'en_US.UTF-8','OCXD024_NATIVE_ROOT':str(root)}
report={'accepted':False,'hypothesis':'Native2 Start succeeded within30s. Its Stop was refused because official history restoration could not open the unlisted exact private history coordination DB; official restore diagnostic confirmed this. Allow only canonical private HOME/state_5 length-prefixed hash and sidecars, enter actual runtime worker for status, use actual Grok backup path. Product byte restoration and30/5/10s assertions unchanged.','locks':[str(p) for p in locks],'kernelBoundary':'all daily HOME reads and external network/write denied; only own profile, synthetic loopback endpoint and exact catalog coordinator database allowed','cleanup':{}}
(out/'native-hypothesis.json').write_text(json.dumps(report,indent=2)+'\n')
for label,argv in [('external-network',['/usr/bin/nc','-v','-G','1','-z','192.0.2.1','443']),('daily-home',['/bin/cat','/Users/ezio/.zshrc'])]:
 p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(sb),*argv],capture_output=True,env=env,cwd=root)
 (out/(label+'.stdout.log')).write_bytes(p.stdout);(out/(label+'.stderr.log')).write_bytes(p.stderr)
 assert p.returncode and b'Operation not permitted' in p.stderr,label
report['unitPassedEvidence']='native-attempt2/after-fix.stdout.log; unchanged product source and admission tests'
start=time.monotonic()
with (out/'native.stdout.log').open('wb') as stdout,(out/'native.stderr.log').open('wb') as stderr:
 p=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(sb),str(exe),'--exact','commands::lifecycle_versioned_tests::official299_versioned_start_status_stop_preserves_native_client','--ignored','--nocapture','--test-threads=1'],stdout=stdout,stderr=stderr,env=env,cwd=root,start_new_session=True)
 report['testPid']=p.pid
 try:report['nativeExitCode']=p.wait(timeout=280)
 except subprocess.TimeoutExpired:
  report['nativeTimedOut']=True
  os.killpg(p.pid,signal.SIGTERM)
  try:report['nativeExitCode']=p.wait(timeout=5)
  except subprocess.TimeoutExpired:report['nativeExitCode']=None
report['elapsedSeconds']=round(time.monotonic()-start,3)
if (root/'evidence/native').exists():shutil.copytree(root/'evidence/native',out/'native')
# Only selected synthetic manager logs; never archive runtime record/key/token.
log=profile/'.codex-desktop/desktop/logs/app.log'
if log.exists():shutil.copy2(log,out/'native-manager.log')
rows=subprocess.check_output(['/bin/ps','-axo','pid=,ppid=,pgid=,comm='],text=True)
owned=[]
for row in rows.splitlines():
 parts=row.split(None,3)
 if len(parts)!=4:continue
 pid,ppid,pgid,image=parts
 if image.startswith(str(root/'program ')) or (int(pgid)==p.pid):
  owned.append({'pid':int(pid),'ppid':int(ppid),'pgid':int(pgid),'image':image})
report['cleanup']['ownedProcessesAfterNative']=owned
report['cleanup']['forcedCleanupUsed']=False
report['cleanup']['fixtureRetained']=True
# Stop failures are preserved. Do not signal detached services or delete their files.
# Only a fully empty original process set permits exact fixture deletion here.
if not owned and p.poll() is not None:
 for path in locks:
  if path.exists():
   st=path.lstat();assert st.st_uid==os.getuid() and st.st_nlink==1 and path.is_file() and not path.is_symlink()
   path.unlink()
 shutil.rmtree(root)
 report['cleanup']['fixtureRetained']=False
report['cleanup']['fixtureAbsent']=not root.exists()
report['cleanup']['exactNewCoordinatorFilesAbsent']=all(not p.exists() for p in locks)
(out/'native-runner-summary.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
shutil.copy2(Path(__file__),out/'native-runner.py')
print(json.dumps(report,ensure_ascii=False,indent=2),flush=True)

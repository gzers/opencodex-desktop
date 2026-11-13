from pathlib import Path
import subprocess,json,shutil,time,hashlib,os
b=Path('/private/tmp/ocxd024-coordination-7ni3ejam');root=b/'fixtures';out=b/'evidence'
code=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes');exe=root/'library-tests'
shutil.copy2(code/'apps/desktop/tauri/target/debug/deps/opencodex_desktop_lib-625891ad1214e6fb',exe)
env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(root/'profile'),'CODEX_HOME':str(root/'profile/.codex'),'TMPDIR':str(root/'tmp')+'/','LANG':'en_US.UTF-8','OCXD_COORDINATION_ROOT':str(root)}
for label,args in [('external-network',['/usr/bin/nc','-v','-G','1','-z','192.0.2.1','443']),('daily-home',['/bin/cat','/Users/ezio/.zshrc'])]:
 p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(b/'fixture.sb'),*args],env=env,cwd=root,capture_output=True)
 (out/(label+'.stdout.log')).write_bytes(p.stdout);(out/(label+'.stderr.log')).write_bytes(p.stderr)
 assert p.returncode and b'Operation not permitted' in p.stderr,label
p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(b/'fixture.sb'),str(exe),'official_client_coordination','--nocapture','--test-threads=1'],env=env,cwd=root,capture_output=True)
(out/'protocol.stdout.log').write_bytes(p.stdout);(out/'protocol.stderr.log').write_bytes(p.stderr);(out/'protocol.json').write_text(json.dumps({'exitCode':p.returncode})+'\n')
assert p.returncode==0,'protocol or source refusal tests failed'
source_files=['apps/desktop/tauri/src/infrastructure/client_coordination299.mjs','apps/desktop/tauri/src/infrastructure/official_client_coordination.rs','apps/desktop/tauri/src/infrastructure/official_client_coordination_tests.rs','apps/desktop/tauri/src/infrastructure/macos_client_originals.rs']
(out/'source.json').write_text(json.dumps({'baseCommit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=code,text=True).strip(),'files':[{'path':n,'sha256':hashlib.sha256((code/n).read_bytes()).hexdigest()} for n in source_files],'libraryTestSha256':hashlib.sha256(exe.read_bytes()).hexdigest()},indent=2)+'\n')
start=time.monotonic()
with (out/'native.stdout.log').open('wb') as stdout,(out/'native.stderr.log').open('wb') as stderr:
 p=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(b/'fixture.sb'),str(exe),'--exact','infrastructure::official_client_coordination::tests::official299_real_coordinator_excludes_other_connections_and_releases_after_failure','--ignored','--nocapture','--test-threads=1'],cwd=root,env=env,stdout=stdout,stderr=stderr,start_new_session=True)
 report={'accepted':False,'nativeTestPid':p.pid,'noServiceModelOrSync':True,'startedMonotonic':start}
 (out/'running.json').write_text(json.dumps(report,indent=2)+'\n')
 try:report['exitCode']=p.wait(timeout=240)
 except subprocess.TimeoutExpired:report['observationTimeout']=True
report['elapsedSeconds']=round(time.monotonic()-start,3)
if (root/'native-summary.json').exists():shutil.copy2(root/'native-summary.json',out/'native-summary.json')
report['ownedProcessesRemaining']=[line for line in subprocess.check_output(['/bin/ps','-axo','pid=,ppid=,pgid=,comm='],text=True).splitlines() if str(root) in line]
report['fixtureRetainedForIndependentCleanup']=True
(out/'runner-summary.json').write_text(json.dumps(report,indent=2)+'\n');shutil.copy2(Path(__file__),out/'run.py')
print(json.dumps(report,indent=2),flush=True)

from pathlib import Path
import subprocess,json,shutil,hashlib,time
b=Path('/private/tmp/ocxd024-native-coordination-45apu6s5');root=b/'fixtures-r2';out=b/'evidence/official-attempt2';out.mkdir(exist_ok=True);code=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes');exe=root/'library-tests';shutil.copy2(code/'apps/desktop/tauri/target/debug/deps/opencodex_desktop_lib-625891ad1214e6fb',exe)
env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(root/'profile'),'CODEX_HOME':str(root/'profile/codex'),'TMPDIR':str(root/'tmp')+'/','LANG':'en_US.UTF-8','OCXD_NATIVE_COORDINATION_ROOT':str(root),'OCXD_NATIVE_COORDINATION_REUSE':str(b/'fixtures/program')}
for label,args in [('external-network',['/usr/bin/nc','-v','-G','1','-z','192.0.2.1','443']),('daily-home',['/bin/cat','/Users/ezio/.zshrc'])]:
 p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(b/'fixture-r2.sb'),*args],env=env,cwd=root,capture_output=True)
 (out/(label+'.stdout.log')).write_bytes(p.stdout);(out/(label+'.stderr.log')).write_bytes(p.stderr);assert p.returncode and b'Operation not permitted' in p.stderr,label
paths=['apps/desktop/tauri/src/infrastructure/client_coordination299.mjs','apps/desktop/tauri/src/infrastructure/official_client_coordination.rs','apps/desktop/tauri/src/infrastructure/native_client_coordination.rs','apps/desktop/tauri/src/infrastructure/native_client_coordination_tests.rs','apps/desktop/tauri/src/infrastructure/macos_client_originals.rs','apps/desktop/tauri/src/infrastructure/macos_client_restore.rs']
(out/'source.json').write_text(json.dumps({'baseCommit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=code,text=True).strip(),'files':[{'path':p,'sha256':hashlib.sha256((code/p).read_bytes()).hexdigest()} for p in paths],'librarySha256':hashlib.sha256(exe.read_bytes()).hexdigest()},indent=2)+'\n')
started=time.monotonic()
with (out/'official-native.stdout.log').open('wb') as stdout,(out/'official-native.stderr.log').open('wb') as stderr:
 p=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(b/'fixture-r2.sb'),str(exe),'--exact','infrastructure::official_client_coordination::native::tests::official299_native_handover_holds_through_work_and_preserves_original_ownership','--ignored','--nocapture','--test-threads=1'],env=env,cwd=root,stdout=stdout,stderr=stderr,start_new_session=True)
 (out/'running.json').write_text(json.dumps({'pid':p.pid,'accepted':False,'nativeServiceStarted':False})+'\n')
 try:exit_code=p.wait(timeout=240)
 except subprocess.TimeoutExpired:exit_code=None
r={'accepted':False,'exitCode':exit_code,'elapsedSeconds':round(time.monotonic()-started,3),'pid':p.pid,'fixturesRetained':True,'ownedProcessesRemaining':[x for x in subprocess.check_output(['/bin/ps','-axo','pid=,ppid=,pgid=,comm='],text=True).splitlines() if str(root) in x]}
if (root/'native-summary.json').exists():shutil.copy2(root/'native-summary.json',out/'native-summary.json')
for name in ['official-initialize.stdout.log','official-initialize.stderr.log']:
 if (root/name).exists():shutil.copy2(root/name,out/name)
(out/'runner-summary.json').write_text(json.dumps(r,indent=2)+'\n');shutil.copy2(Path(__file__),out/'run.py');print(json.dumps(r,indent=2),flush=True)

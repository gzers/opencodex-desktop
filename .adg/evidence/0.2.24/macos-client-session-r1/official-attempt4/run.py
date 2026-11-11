from pathlib import Path
import subprocess,json,shutil,hashlib,time
b=Path(__file__).parent;r=b/'fixtures-r4';o=b/'evidence/official-attempt4';o.mkdir(exist_ok=True);code=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes');exe=r/'tests';shutil.copy2(code/'apps/desktop/tauri/target/debug/deps/opencodex_desktop_lib-625891ad1214e6fb',exe)
env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(r/'profile'),'CODEX_HOME':str(r/'profile/.codex'),'TMPDIR':str(r/'tmp')+'/','LANG':'en_US.UTF-8','OCXD_CLIENT_SESSION_ROOT':str(r),'OCXD_CLIENT_SESSION_REUSE':str(b/'fixtures/program')}
for label,args in [('unit-fixed',['macos_client_originals::session','--nocapture','--test-threads=1'])]:
 p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(b/'native-r4.sb'),str(exe),*args],env=env,cwd=r,capture_output=True)
 (o/(label+'.stdout.log')).write_bytes(p.stdout);(o/(label+'.stderr.log')).write_bytes(p.stderr);(o/(label+'.json')).write_text(json.dumps({'exitCode':p.returncode})+'\n');print(label,p.returncode,p.stdout.decode()[-500:],p.stderr.decode()[-1500:],flush=True)
 assert p.returncode==0,'unit failure: do not start native official service'
for label,args in [('external-network',['/usr/bin/nc','-v','-G','1','-z','192.0.2.1','443']),('daily-home',['/bin/cat','/Users/ezio/.zshrc'])]:
 p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(b/'native-r4.sb'),*args],env=env,cwd=r,capture_output=True)
 (o/(label+'.stdout.log')).write_bytes(p.stdout);(o/(label+'.stderr.log')).write_bytes(p.stderr);assert p.returncode and b'Operation not permitted' in p.stderr
paths=['apps/desktop/tauri/src/infrastructure/macos_client_session.rs','apps/desktop/tauri/src/infrastructure/macos_client_session_tests.rs','apps/desktop/tauri/src/infrastructure/macos_client_originals.rs','apps/desktop/tauri/src/infrastructure/native_client_coordination.rs','apps/desktop/tauri/src/infrastructure/native_client_coordination_tests.rs','apps/desktop/tauri/src/commands/runtime_recovery_v2_owner.rs']
(o/'source.json').write_text(json.dumps({'baseCommit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=code,text=True).strip(),'files':[{'path':p,'sha256':hashlib.sha256((code/p).read_bytes()).hexdigest()} for p in paths],'librarySha256':hashlib.sha256(exe.read_bytes()).hexdigest()},indent=2)+'\n')
started=time.monotonic()
with (o/'official-native.stdout.log').open('wb') as stdout,(o/'official-native.stderr.log').open('wb') as stderr:
 p=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(b/'native-r4.sb'),str(exe),'--exact','infrastructure::macos_client_originals::session::tests::official299_real_startup_originals_ready_binding_and_completed_stop_roundtrip','--ignored','--nocapture','--test-threads=1'],env=env,cwd=r,stdout=stdout,stderr=stderr,start_new_session=True)
 (o/'running.json').write_text(json.dumps({'pid':p.pid,'accepted':False,'scope':'official299 new synthetic private profile only'})+'\n')
 try:exitcode=p.wait(timeout=240)
 except subprocess.TimeoutExpired:exitcode=None
report={'accepted':False,'exitCode':exitcode,'pid':p.pid,'elapsedSeconds':round(time.monotonic()-started,3),'fixtureRetained':True,'ownedProcessesRemaining':[line for line in subprocess.check_output(['/bin/ps','-axo','pid=,ppid=,pgid=,comm='],text=True).splitlines() if str(r) in line]}
for name in ['native-summary.json','independent-session-references.json','completed-session-references.json']:
 if (r/name).exists():shutil.copy2(r/name,o/name)
log=r/'profile/desktop/logs/app.log'
if log.exists():shutil.copy2(log,o/'manager.log')
(o/'runner-summary.json').write_text(json.dumps(report,indent=2)+'\n');shutil.copy2(Path(__file__),o/'run.py');print(json.dumps(report,indent=2),flush=True)

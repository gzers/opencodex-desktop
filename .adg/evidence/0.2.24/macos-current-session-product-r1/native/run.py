from pathlib import Path
import subprocess,json,shutil,hashlib,time
b=Path(__file__).parent;r=b/'fixtures';o=Path('/private/tmp/ocxd024-current-session-jhbqv7af/evidence/native');code=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes');exe=r/'tests';shutil.copy2(code/'apps/desktop/tauri/target/debug/deps/opencodex_desktop_lib-625891ad1214e6fb',exe)
env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(r/'profile'),'CODEX_HOME':str(r/'profile/.codex'),'TMPDIR':str(r/'tmp')+'/','LANG':'en_US.UTF-8','OCXD024_NATIVE_ROOT':str(r)}
for label,args in [('external-network',['/usr/bin/nc','-v','-G','1','-z','192.0.2.1','443']),('daily-home',['/bin/cat','/Users/ezio/.zshrc'])]:
 p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(b/'fixture.sb'),*args],cwd=r,env=env,capture_output=True)
 (o/(label+'.stdout.log')).write_bytes(p.stdout);(o/(label+'.stderr.log')).write_bytes(p.stderr);assert p.returncode and b'Operation not permitted' in p.stderr
paths=['apps/desktop/tauri/src/commands/mod.rs','apps/desktop/tauri/src/commands/client_session_lifecycle.rs','apps/desktop/tauri/src/commands/lifecycle_versioned_tests.rs','apps/desktop/tauri/src/infrastructure/macos_client_session_index.rs','apps/desktop/tauri/src/infrastructure/macos_client_session_index_tests.rs','apps/desktop/tauri/src/infrastructure/macos_client_session.rs','apps/desktop/tauri/src/infrastructure/process_runner.rs','apps/desktop/tauri/src/modules/process/mod.rs','apps/desktop/tauri/src/types/process_action.rs']
(o/'source.json').write_text(json.dumps({'baseCommit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=code,text=True).strip(),'files':[{'path':p,'sha256':hashlib.sha256((code/p).read_bytes()).hexdigest()} for p in paths],'librarySha256':hashlib.sha256(exe.read_bytes()).hexdigest()},indent=2)+'\n')
started=time.monotonic()
with (o/'native.stdout.log').open('wb') as stdout,(o/'native.stderr.log').open('wb') as stderr:
 p=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(b/'fixture.sb'),str(exe),'--exact','commands::lifecycle_versioned_tests::official299_versioned_start_status_stop_preserves_native_client','--ignored','--nocapture','--test-threads=1'],cwd=r,env=env,stdout=stdout,stderr=stderr,start_new_session=True)
 (o/'running.json').write_text(json.dumps({'pid':p.pid,'accepted':False,'scope':'actual product action against only new syntheticprofile'})+'\n')
 try:exit_code=p.wait(timeout=240)
 except subprocess.TimeoutExpired:exit_code=None
if (r/'evidence/native').exists():shutil.copytree(r/'evidence/native',o/'native')
log=r/'profile/.codex-desktop/desktop/logs/app.log'
if log.exists():shutil.copy2(log,o/'manager.log')
report={'accepted':False,'exitCode':exit_code,'elapsedSeconds':round(time.monotonic()-started,3),'pid':p.pid,'temporaryRoot':str(b),'fixturesRetained':True,'ownedProcessesRemaining':[x for x in subprocess.check_output(['/bin/ps','-axo','pid=,ppid=,pgid=,comm='],text=True).splitlines() if str(r) in x]}
(o/'runner-summary.json').write_text(json.dumps(report,indent=2)+'\n');shutil.copy2(Path(__file__),o/'run.py');print(json.dumps(report,indent=2),flush=True)

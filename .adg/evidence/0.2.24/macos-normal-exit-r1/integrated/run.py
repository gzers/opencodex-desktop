from pathlib import Path
import subprocess,json,shutil,hashlib,os,re,time
base=Path(__file__).parent;root=base/'fixtures';profile=root/'profile';out=Path('/private/tmp/ocxd024-exit-witness-5s9m14me/evidence/integrated')
code=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes');resources=root/'resources';resources.mkdir()
source=code/'apps/desktop/tauri/target/release/bundle/macos/OpenCodeX Desktop.app/Contents/Resources/runtime-bundle'
for name in ['runtime-bundle.manifest.json','runtime-bundle-0.2.24-darwin-arm64.tar.gz']:shutil.copy2(source/name,resources/name)
assert hashlib.sha256((resources/'runtime-bundle.manifest.json').read_bytes()).hexdigest()=='e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3'
exe=root/'tests';shutil.copy2(code/'apps/desktop/tauri/target/debug/deps/opencodex_desktop_lib-625891ad1214e6fb',exe)
client=profile/'.codex';client.mkdir(mode=0o700)
namespace=Path('/private/tmp')/('opencodex-runtime-v1-'+str(os.getuid()));digest=hashlib.sha256(str(client).encode()).hexdigest();locks=[]
for kind in ['native-write-locks','catalog-write-locks']:
 assert (namespace/kind).is_dir()
 for suffix in ['.sqlite','.sqlite-journal','.sqlite-wal','.sqlite-shm']:locks.append(Path(str(namespace/kind/digest)+suffix))
home=str(client);state=str(client/'state_5.sqlite');history=hashlib.sha256((str(len(home))+':'+home+str(len(state))+':'+state).encode()).hexdigest()
assert (namespace/'history-write-locks').is_dir()
for suffix in ['.sqlite','.sqlite-journal','.sqlite-wal','.sqlite-shm']:locks.append(Path(str(namespace/'history-write-locks'/history)+suffix))
assert not any(p.exists() for p in locks)
policy='(version 1)\n(allow default)\n(deny network*)\n(allow network-inbound (local ip "localhost:*"))\n(allow network-outbound (remote ip "localhost:*"))\n(deny file-write*)\n(allow file-write* (subpath '+json.dumps(str(root))+'))\n(allow file-write* (literal "/dev/null"))\n(deny file-read* (subpath "/Users/ezio"))\n(deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))\n'+''.join('(allow file-write* (literal '+json.dumps(str(p))+'))\n' for p in locks)
pattern='^'+re.escape(str(namespace/'native-write-locks'/('.'+digest+'.sqlite.adopt-')))+'[0-9]+-[0-9a-f-]+'+re.escape('.tmp')+'(-journal|-wal|-shm)?$';policy+='(allow file-write* (regex '+json.dumps(pattern)+'))\n'
sb=base/'fixture.sb';sb.write_text(policy);shutil.copy2(sb,out/'fixture.sb')
env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(profile),'CODEX_HOME':str(client),'TMPDIR':str(root/'tmp')+'/','LANG':'en_US.UTF-8','OCXD024_NATIVE_ROOT':str(root)}
for label,argv in [('external-network',['/usr/bin/nc','-v','-G','1','-z','192.0.2.1','443']),('daily-home',['/bin/cat','/Users/ezio/.zshrc'])]:
 p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(sb),*argv],capture_output=True,cwd=root,env=env)
 (out/(label+'.stdout.log')).write_bytes(p.stdout);(out/(label+'.stderr.log')).write_bytes(p.stderr);assert p.returncode and b'Operation not permitted' in p.stderr
paths=['apps/desktop/tauri/src/infrastructure/macos_process_exit.rs','apps/desktop/tauri/src/infrastructure/macos_process_exit_tests.rs','apps/desktop/tauri/src/commands/runtime_recovery_v2_owner.rs','apps/desktop/tauri/src/commands/lifecycle_versioned_tests.rs']
(out/'source.json').write_text(json.dumps({'baseCommit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=code,text=True).strip(),'files':[{'path':p,'sha256':hashlib.sha256((code/p).read_bytes()).hexdigest()} for p in paths],'librarySha256':hashlib.sha256(exe.read_bytes()).hexdigest()},indent=2)+'\n')
started=time.monotonic()
with (out/'native.stdout.log').open('wb') as stdout,(out/'native.stderr.log').open('wb') as stderr:
 p=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(sb),str(exe),'--exact','commands::lifecycle_versioned_tests::official299_versioned_start_status_stop_preserves_native_client','--ignored','--nocapture','--test-threads=1'],cwd=root,env=env,stdout=stdout,stderr=stderr,start_new_session=True)
 (out/'running.json').write_text(json.dumps({'pid':p.pid,'accepted':False,'ownedSyntheticRuntimeOnly':True})+'\n')
 try:exit_code=p.wait(timeout=240)
 except subprocess.TimeoutExpired:exit_code=None
if (root/'evidence/native').exists():shutil.copytree(root/'evidence/native',out/'native')
log=profile/'.codex-desktop/desktop/logs/app.log'
if log.exists():shutil.copy2(log,out/'manager.log')
report={'accepted':False,'exitCode':exit_code,'pid':p.pid,'elapsedSeconds':round(time.monotonic()-started,3),'temporaryRoot':str(base),'exactNewCoordinatorFiles':[str(p) for p in locks],'ownedProcessesRemaining':[line for line in subprocess.check_output(['/bin/ps','-axo','pid=,ppid=,pgid=,comm='],text=True).splitlines() if str(root) in line],'fixtureRetained':True}
(out/'runner-summary.json').write_text(json.dumps(report,indent=2)+'\n');shutil.copy2(Path(__file__),out/'run.py');print(json.dumps(report,indent=2),flush=True)

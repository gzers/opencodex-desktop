from pathlib import Path
import subprocess,json,shutil,hashlib,tempfile,os,re,time
base=Path(__file__).resolve().parent;out=base/'descriptor';out.mkdir()
work=Path(tempfile.mkdtemp(prefix='ocxd024-versioned-lifecycle-',dir='/private/tmp'));root=work/'fixtures';root.mkdir(mode=0o700);profile=root/'profile';profile.mkdir(mode=0o700);(root/'tmp').mkdir(mode=0o700);res=root/'resources';res.mkdir()
code=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes');source=code/'apps/desktop/tauri/target/release/bundle/macos/OpenCodeX Desktop.app/Contents/Resources/runtime-bundle'
for name in ['runtime-bundle.manifest.json']:shutil.copy2(source/name,res/name)
assert hashlib.sha256((res/'runtime-bundle.manifest.json').read_bytes()).hexdigest()=='e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3'
compiled=code/'apps/desktop/tauri/target/debug/deps/opencodex_desktop_lib-114f658f3f2e2b97';expected_binary=hashlib.sha256(compiled.read_bytes()).hexdigest()
exe=root/'tests';shutil.copy2(compiled,exe);assert hashlib.sha256(exe.read_bytes()).hexdigest()==expected_binary
client=profile/'.codex';digest=hashlib.sha256(str(client).encode()).hexdigest();namespace=Path('/private/tmp')/('opencodex-runtime-v1-'+str(os.getuid()));locks=[]
for kind in ['native-write-locks','catalog-write-locks']:
 for suffix in ['.sqlite','.sqlite-journal','.sqlite-wal','.sqlite-shm']:locks.append(Path(str(namespace/kind/digest)+suffix))
h=str(client);state=str(client/'state_5.sqlite');hd=hashlib.sha256((str(len(h))+':'+h+str(len(state))+':'+state).encode()).hexdigest()
for suffix in ['.sqlite','.sqlite-journal','.sqlite-wal','.sqlite-shm']:locks.append(Path(str(namespace/'history-write-locks'/hd)+suffix))
assert not any(p.exists() for p in locks)
policy='(version 1)\n(allow default)\n(deny network*)\n(allow network-inbound (local ip "localhost:*"))\n(allow network-outbound (remote ip "localhost:*"))\n(deny file-write*)\n(allow file-write* (subpath '+json.dumps(str(root))+'))\n(allow file-write* (literal "/dev/null"))\n(deny file-read* (subpath "/Users/ezio"))\n(deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))\n'+''.join('(allow file-write* (literal '+json.dumps(str(p))+'))\n' for p in locks)
lpath=namespace/'client-desktop-lifecycle.sqlite'
import stat,sqlite3
ls=lpath.lstat();assert stat.S_ISREG(ls.st_mode) and ls.st_nlink==1 and ls.st_uid==os.getuid() and stat.S_IMODE(ls.st_mode)==0o600
assert not any(Path(str(lpath)+suffix).exists() for suffix in ['-journal','-wal','-shm'])
before={'device':ls.st_dev,'inode':ls.st_ino,'mode':stat.S_IMODE(ls.st_mode),'uid':ls.st_uid,'gid':ls.st_gid,'bytes':ls.st_size,'sha256':hashlib.sha256(lpath.read_bytes()).hexdigest()}
c=sqlite3.connect(lpath.as_uri()+'?mode=rw',uri=True,timeout=0);c.execute('begin exclusive');c.execute('rollback');c.close()
(out/'global-l-before.json').write_text(json.dumps(before,indent=2))
for suffix in ['','-journal','-wal','-shm']:policy+='(allow file-write* (literal '+json.dumps(str(lpath)+suffix)+'))'+chr(10)
policy+='(allow file-write-mode (literal '+json.dumps(str(namespace))+'))'+chr(10)
pattern='^'+re.escape(str(namespace/'native-write-locks'/('.'+digest+'.sqlite.adopt-')))+'[0-9]+-[0-9a-f-]+'+re.escape('.tmp')+'(-journal|-wal|-shm)?$';policy+='(allow file-write* (regex '+json.dumps(pattern)+'))\n';sb=work/'fixture.sb';sb.write_text(policy);(out/'fixture.sb').write_text(policy)
env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(profile),'CODEX_HOME':str(client),'TMPDIR':str(root/'tmp')+'/','LANG':'en_US.UTF-8','OCXD024_NATIVE_ROOT':str(root)}
env['OCXD024_ALIAS_GRAPH']='1'
env['OCXD024_RUNTIME_REUSE']='/private/tmp/ocxd024-versioned-lifecycle-ofvb723j/fixtures/program 中文 space'
env['OCXD024_FRESH_CLIENTS']='codex-absent'
report={'accepted':False,'hypothesis':'Diagnostic confirmed all artifact restoration and official release succeeded; post-apply-stopped returned timeout. Same-call Index duplication was removed and reached RootRetirementPrepared but Stop remained timeout. New hypothesis: replace reopening all pinned directories with exact retained-fd plus per-component NOFOLLOW checks, two complete ACL/xattrs passes including in-scan xattr negative control; unchanged assertions/budgets. Measure final Stop and reject if over10s. Reuse verified immutable runtime input, new Profile with no .codex. Production Start creates root via retained custody and independent capture; typed real Ready/PreStop/Stopped transitions authorize normal-Stop factory only after independent plan registration. Same exact root must retire with official writer released, complete readback and RootAbsent within original30/5/10 budgets. This is diagnostic first Start/Stop; next Start remains unaccepted.','temporaryRoot':str(work),'exactNewCoordinatorFiles':[str(p) for p in locks],'noBusinessRequests':True}
(out/'hypothesis.json').write_text(json.dumps(report,indent=2)+'\n')
for label,argv in [('external-network',['/usr/bin/nc','-v','-G','1','-z','192.0.2.1','443']),('daily-home',['/bin/cat','/Users/ezio/.zshrc'])]:
 p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(sb),*argv],env=env,cwd=root,capture_output=True);(out/(label+'.stdout.log')).write_bytes(p.stdout);(out/(label+'.stderr.log')).write_bytes(p.stderr);assert p.returncode and b'Operation not permitted' in p.stderr
paths=sorted(set(subprocess.check_output(['git','diff','--name-only'],cwd=code,text=True).splitlines()+subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=code,text=True).splitlines()))
(out/'source.json').write_text(json.dumps({'baseCommit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=code,text=True).strip(),'files':[{'path':n,'sha256':hashlib.sha256((code/n).read_bytes()).hexdigest()} for n in paths],'binarySha256':hashlib.sha256(exe.read_bytes()).hexdigest()},indent=2)+'\n')
start=time.monotonic()
with (out/'native.stdout.log').open('wb') as stdout,(out/'native.stderr.log').open('wb') as stderr:
 p=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(sb),str(exe),'--exact','commands::lifecycle_versioned_tests::official299_versioned_start_status_stop_preserves_native_client','--ignored','--nocapture','--test-threads=1'],env=env,cwd=root,stdout=stdout,stderr=stderr,start_new_session=True)
 report['pid']=p.pid;(out/'running.json').write_text(json.dumps({'pid':p.pid,'temporaryRoot':str(work),'accepted':False})+'\n')
 try:report['exitCode']=p.wait(timeout=240)
 except subprocess.TimeoutExpired:report['exitCode']=None
report['elapsedSeconds']=round(time.monotonic()-start,3)
if (root/'evidence/native').exists():shutil.copytree(root/'evidence/native',out/'native')
log=profile/'.codex-desktop/desktop/logs/app.log'
if log.exists():shutil.copy2(log,out/'manager.log')
report['ownedProcessesRemaining']=[line for line in subprocess.check_output(['/bin/ps','-axo','pid=,ppid=,pgid=,comm='],text=True).splitlines() if str(root) in line]
(out/'runner-summary.json').write_text(json.dumps(report,indent=2)+'\n');shutil.copy2(Path(__file__),out/'native_runner.py');print(json.dumps(report,indent=2),flush=True)

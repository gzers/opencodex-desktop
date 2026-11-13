from pathlib import Path
import subprocess,json,hashlib,os,re,time,shutil,tempfile
base=Path('/private/tmp/ocxd024-writer-cover-b_g199s0');out=base/'evidence/native';out.mkdir()
work=Path(tempfile.mkdtemp(prefix='ocxd024-native-coordination-',dir='/private/tmp'));root=work/'fixtures';root.mkdir(mode=0o700);profile=root/'profile';profile.mkdir(mode=0o700);(root/'tmp').mkdir();res=root/'resources';res.mkdir()
code=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes');src=code/'apps/desktop/tauri/target/release/bundle/macos/OpenCodeX Desktop.app/Contents/Resources/runtime-bundle'
for name in ['runtime-bundle.manifest.json','runtime-bundle-0.2.24-darwin-arm64.tar.gz']:shutil.copy2(src/name,res/name)
assert hashlib.sha256((res/'runtime-bundle.manifest.json').read_bytes()).hexdigest()=='e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3'
checked=json.loads((base/'evidence/checks.json').read_text());exe=root/'library-tests';shutil.copy2(checked['binary'],exe);assert hashlib.sha256(exe.read_bytes()).hexdigest()==checked['sha256']
client=profile/'codex';digest=hashlib.sha256(str(client).encode()).hexdigest();namespace=Path('/private/tmp')/('opencodex-runtime-v1-'+str(os.getuid()));locks=[]
for kind in ['native-write-locks','catalog-write-locks']:
 for suffix in ['.sqlite','.sqlite-journal','.sqlite-wal','.sqlite-shm']:locks.append(Path(str(namespace/kind/digest)+suffix))
assert not any(p.exists() for p in locks)
lpath=namespace/'client-desktop-lifecycle.sqlite';assert json.loads((base/'evidence/global-l-before.json').read_text())['existedBefore']
policy='(version 1)'+chr(10)+'(allow default)'+chr(10)+'(deny network*)'+chr(10)+'(deny file-write*)'+chr(10)+'(allow file-write* (subpath '+json.dumps(str(root))+'))'+chr(10)+'(allow file-write* (literal "/dev/null"))'+chr(10)+'(deny file-read* (subpath "/Users/ezio"))'+chr(10)+'(deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))'+chr(10)
for path in locks+[Path(str(lpath)+s) for s in ['','-journal','-wal','-shm']]:policy+='(allow file-write* (literal '+json.dumps(str(path))+'))'+chr(10)
pattern='^'+re.escape(str(namespace/'native-write-locks'/('.'+digest+'.sqlite.adopt-')))+'[0-9]+-[0-9a-f-]+'+re.escape('.tmp')+'(-journal|-wal|-shm)?$';policy+='(allow file-write* (regex '+json.dumps(pattern)+'))'+chr(10);sb=work/'fixture.sb';sb.write_text(policy);(out/'fixture.sb').write_text(policy)
env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(profile),'CODEX_HOME':str(client),'TMPDIR':str(root/'tmp')+'/','LANG':'en_US.UTF-8','OCXD_NATIVE_COORDINATION_ROOT':str(root)}
for label,argv in [('network',['/usr/bin/nc','-v','-G','1','-z','192.0.2.1','443']),('daily-home',['/bin/cat','/Users/ezio/.zshrc'])]:
 p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(sb),*argv],env=env,cwd=root,capture_output=True);(out/(label+'.stdout.log')).write_bytes(p.stdout);(out/(label+'.stderr.log')).write_bytes(p.stderr);assert p.returncode and b'Operation not permitted' in p.stderr
report={'accepted':False,'temporaryRoot':str(work),'exactNewCoordinatorFiles':[str(p) for p in locks],'preexistingGlobalL':str(lpath),'noServiceOrBusinessRequests':True,'binarySha256':checked['sha256'],'hypothesis':'Actual official K mutators and L-protected direct catalog callback must refuse while native owns all4 locks, then succeed after release. Preserve existing user-global L inode/content/mode, never delete it.'}
(out/'hypothesis.json').write_text(json.dumps(report,indent=2));start=time.monotonic()
with (out/'native.stdout.log').open('wb') as stdout,(out/'native.stderr.log').open('wb') as stderr:
 p=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(sb),str(exe),'--exact','infrastructure::official_client_coordination::native::tests::official299_native_handover_holds_through_work_and_preserves_original_ownership','--ignored','--nocapture','--test-threads=1'],env=env,cwd=root,stdout=stdout,stderr=stderr,start_new_session=True)
 report['pid']=p.pid;(out/'running.json').write_text(json.dumps(report));
 try:report['exitCode']=p.wait(timeout=240)
 except subprocess.TimeoutExpired:report['exitCode']=None
report['elapsedSeconds']=round(time.monotonic()-start,3)
for name in ['native-summary.json','official-initialize.stdout.log','official-initialize.stderr.log','official-writers-blocked.stdout.log','official-writers-blocked.stderr.log','official-writers-released.stdout.log','official-writers-released.stderr.log']:
 if (root/name).exists():shutil.copy2(root/name,out/name)
report['ownedProcessesRemaining']=[line for line in subprocess.check_output(['/bin/ps','-axo','pid=,ppid=,pgid=,comm='],text=True).splitlines() if str(root) in line]
(out/'runner-summary.json').write_text(json.dumps(report,indent=2));shutil.copy2(__file__,out/'native_runner.py');print(json.dumps(report,indent=2))

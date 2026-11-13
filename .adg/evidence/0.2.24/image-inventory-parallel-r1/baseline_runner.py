from pathlib import Path
import subprocess,json,hashlib,shutil,tempfile,os,re,time
b=Path(__file__).resolve().parent;out=b/'baseline';out.mkdir();work=Path(tempfile.mkdtemp(prefix='ocxd024-versioned-lifecycle-',dir='/private/tmp'));root=work/'fixtures';root.mkdir(mode=0o700);profile=root/'profile';profile.mkdir(mode=0o700);(profile/'desktop').mkdir(mode=0o700);(root/'tmp').mkdir(mode=0o700);res=root/'resources';res.mkdir(mode=0o700)
repo=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes');source=repo/'apps/desktop/tauri/target/release/bundle/macos/OpenCodeX Desktop.app/Contents/Resources/runtime-bundle';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
for name in ['runtime-bundle.manifest.json','runtime-bundle-0.2.24-darwin-arm64.tar.gz']:shutil.copy2(source/name,res/name)
assert sha(res/'runtime-bundle.manifest.json')=='e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3'
log=(b/'baseline-compile-tests.log').read_text();match=re.search(r'Running unittests src/lib.rs \(([^)]+)\)',log);assert match;binary=repo/'apps/desktop/tauri'/match.group(1);exe=root/'library-tests';shutil.copy2(binary,exe);assert sha(exe)==sha(binary)
policy='(version 1)\n(allow default)\n(deny network*)\n(deny file-write*)\n(allow file-write* (subpath '+json.dumps(str(root))+'))\n(allow file-write* (literal "/dev/null"))\n(deny file-read* (subpath "/Users/ezio"))\n(deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))\n';sb=work/'fixture.sb';sb.write_text(policy);(out/'fixture.sb').write_text(policy)
env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(profile),'TMPDIR':str(root/'tmp')+'/','LANG':'en_US.UTF-8','OCXD024_IMAGE_SCAN_FIXTURE_ROOT':str(root)}
for label,argv in [('external-network',['/usr/bin/nc','-v','-G','1','-z','192.0.2.1','443']),('daily-home',['/bin/cat','/Users/ezio/.zshrc'])]:
 p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(sb),*argv],env=env,cwd=root,capture_output=True);(out/(label+'.stderr.log')).write_bytes(p.stderr);assert p.returncode and b'Operation not permitted' in p.stderr
report={'accepted':False,'temporaryRoot':str(work),'fixture':str(root),'binarySha256':sha(exe),'sourceBase':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),'sourceFileSha256':sha(repo/'apps/desktop/tauri/src/modules/runtime/program_image.rs'),'noServiceOrGlobalCoordinatorOrBusinessCalls':True}
(out/'hypothesis.json').write_text(json.dumps(report,indent=2));t=time.monotonic()
with (out/'native.stdout.log').open('wb') as stdout,(out/'native.stderr.log').open('wb') as stderr:
 p=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(sb),str(exe),'--exact','modules::runtime::program_image::tests::official299_complete_image_inventory_stage_cost','--ignored','--nocapture','--test-threads=1'],cwd=root,env=env,stdout=stdout,stderr=stderr,start_new_session=True);report['pid']=p.pid;(out/'running.json').write_text(json.dumps(report))
 try:report['exitCode']=p.wait(timeout=240)
 except subprocess.TimeoutExpired:report['exitCode']=None
report['elapsedSeconds']=round(time.monotonic()-t,3);report['ownedProcessesRemaining']=[line for line in subprocess.check_output(['ps','-axo','pid=,ppid=,comm='],text=True).splitlines() if str(root) in line]
if (root/'image-scan-baseline.json').exists():shutil.copy2(root/'image-scan-baseline.json',out/'image-scan-baseline.json')
(out/'runner-summary.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2),flush=True)

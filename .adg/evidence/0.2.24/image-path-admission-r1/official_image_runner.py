from pathlib import Path
import subprocess,json,hashlib,tempfile,shutil,os,re,time,stat,sqlite3,ctypes,sys,base64
base=Path(__file__).resolve().parent
code=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes')
out=base/'official-image-path';out.mkdir()
work=Path(tempfile.mkdtemp(prefix='ocxd024-versioned-lifecycle-',dir='/private/tmp'))
(out/'setup.json').write_text(json.dumps({'temporaryRoot':str(work),'accepted':False}));root=work/'fixtures';root.mkdir(mode=0o700);profile=root/'profile';profile.mkdir(mode=0o700);(root/'tmp').mkdir(mode=0o700)
resources=root/'resources';resources.mkdir(mode=0o700)
source=code/'apps/desktop/tauri/target/release/bundle/macos/OpenCodeX Desktop.app/Contents/Resources/runtime-bundle'
sha=lambda data:hashlib.sha256(data).hexdigest()
resource_rows=[]
for name,expected in [('runtime-bundle.manifest.json','e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3'),('runtime-bundle-0.2.24-darwin-arm64.tar.gz','5b10c1a888000d54d14783622d9d8330c33f4e1b14a5f163d007337d0424ec7f')]:
    assert sha((source/name).read_bytes())==expected
    shutil.copyfile(source/name,resources/name);assert sha((resources/name).read_bytes())==expected
    resource_rows.append({'source':str(source/name),'copiedTo':str(resources/name),'bytes':(resources/name).stat().st_size,'sha256':expected})
manifest=json.loads((resources/'runtime-bundle.manifest.json').read_bytes())
assert manifest['runtimeVersion']=='2.99.0' and manifest['runtimeArchiveSha256']=='b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c'
compiled_receipt=json.loads((base/'tested-source.json').read_text());compiled=Path(compiled_receipt['testBinaryPath']);assert sha(compiled.read_bytes())==compiled_receipt['testBinarySha256']
for item in compiled_receipt['files']:assert sha((code/item['path']).read_bytes())==item['sha256']
exe=root/'tests';shutil.copy2(compiled,exe)
assert sha(exe.read_bytes())==sha(compiled.read_bytes())
policy='(version 1)\n(allow default)\n(deny network*)\n(deny file-write*)\n(allow file-write* (subpath '+json.dumps(str(root))+'))\n(allow file-write* (literal "/dev/null"))\n(deny file-read* (subpath "/Users/ezio"))\n(deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))\n'
sandbox=work/'fixture.sb';sandbox.write_text(policy);(out/'fixture.sb').write_text(policy)
env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(profile),'TMPDIR':str(root/'tmp')+'/','LANG':'en_US.UTF-8','OCXD024_IMAGE_SCAN_FIXTURE_ROOT':str(root),'OCXD024_IMAGE_PATH_COMPARE':'1'}
for label,args in [('daily-home',['/usr/bin/head','-c','1','/Users/ezio/.zshrc']),('external-network',['/usr/bin/nc','-v','-G','1','-z','192.0.2.1','443'])]:
 p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(sandbox),*args],env=env,cwd=root,capture_output=True)
 (out/(label+'.stderr.log')).write_bytes(p.stderr);assert p.returncode and b'Operation not permitted' in p.stderr and not p.stdout
start=time.monotonic()
with (out/'stdout.log').open('wb') as stdout,(out/'stderr.log').open('wb') as stderr:
 p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(sandbox),str(exe),'--exact','modules::runtime::program_image::tests::official299_complete_image_inventory_stage_cost','--ignored','--nocapture','--test-threads=1'],env=env,cwd=root,stdout=stdout,stderr=stderr)
report={'accepted':False,'exitCode':p.returncode,'elapsedSeconds':round(time.monotonic()-start,3),'temporaryRoot':str(work),'binarySha256':sha(exe.read_bytes()),'resources':resource_rows,'serviceActions':0,'allNetworkDenied':True,'dailyHomeDenied':True}
comparison=root/'image-path-comparison.json'
if comparison.exists():shutil.copy2(comparison,out/comparison.name)
text=(out/'stdout.log').read_text();report['exactOneTestPassed']=p.returncode==0 and 'running 1 test' in text and '1 passed; 0 failed' in text and 'OFFICIAL_IMAGE_PATH_COMPARISON ' in text
(out/'runner-summary.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2),flush=True)
sys.exit(0 if report['exactOneTestPassed'] else 1)

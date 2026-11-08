import json,os,subprocess,tempfile,time,plistlib,hashlib,pwd
from pathlib import Path
repo=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes');out=Path('/private/tmp/ocxd023-1f1-mac-port-evidence-attempt2')
assert not (out/'launch.json').exists(),'Fresh native run required'
install=Path(tempfile.mkdtemp(prefix='ocxd023-1f1-port-I-',dir='/private/tmp')).resolve();root=Path(tempfile.mkdtemp(prefix='ocxd023-1f1-port-U-',dir='/private/tmp')).resolve();app=install/'OpenCodeX Desktop.app'
subprocess.run(['/usr/bin/ditto',str(repo/'apps/desktop/tauri/target/release/bundle/macos/OpenCodeX Desktop.app'),str(app)],check=True)
info=plistlib.loads((app/'Contents/Info.plist').read_bytes());assert info['CFBundleShortVersionString']=='0.2.23';exe=app/'Contents/MacOS'/info['CFBundleExecutable']
for path in [root/'home',root/'tmp'] :path.mkdir()
daily=pwd.getpwuid(os.getuid()).pw_dir
profile=root/'guard.sb';profile.write_text('(version 1)\n(allow default)\n(deny network-outbound)\n(deny file-read* (subpath '+json.dumps(daily)+'))\n(deny file-write* (subpath '+json.dumps(daily)+'))\n(deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))\n')
env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','LANG':'en_US.UTF-8','USER':pwd.getpwuid(os.getuid()).pw_name,'HOME':str(root/'home'),'CFFIXED_USER_HOME':str(root/'home'),'TMPDIR':str(root/'tmp')+'/','TMP':str(root/'tmp'),'TEMP':str(root/'tmp'),'CODEX_HOME':str(root/'home/.codex')}
# Positive bind proof is essential: outbound denial must not fake a failed port check.
bind=subprocess.run(['/usr/bin/sandbox-exec','-f',str(profile),'/usr/bin/python3','-c','import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1]); s.close()'],env=env,cwd=root,capture_output=True,text=True,timeout=15)
(out/'bind-guard.log').write_text(bind.stdout+bind.stderr);assert bind.returncode==0,bind.stderr
block=subprocess.run(['/usr/bin/sandbox-exec','-f',str(profile),'/usr/bin/nc','-v','-G','1','-z','192.0.2.1','443'],env=env,cwd=root,capture_output=True,text=True,timeout=10)
(out/'outbound-guard.log').write_text(block.stdout+block.stderr);assert block.returncode!=0 and 'Operation not permitted' in block.stderr
with (out/'stdout.log').open('wb') as log:proc=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(profile),str(exe)],cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
report={'pid':proc.pid,'executable':str(exe),'app':str(app),'installation':str(install),'root':str(root),'home':str(root/'home'),'dataRoot':str(root/'home/.codex-desktop/desktop'),'opencodexHome':str(root/'home/.codex-desktop/opencodex/home'),'binarySha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'source':'1f1dcb9d08319ab900a836d7c9537826931992e7','outboundBlocked':True,'localBindAllowed':True,'realServiceStarted':False,'accepted':False}
(out/'launch.json').write_text(json.dumps(report,indent=2))
print(json.dumps(report),flush=True)
deadline=time.monotonic()+240
while time.monotonic()<deadline:
 assert proc.poll() is None,'Owned native app exited'
 logPath=Path(report['dataRoot'])/'logs/app.log'
 if logPath.exists():
  text=logPath.read_text(errors='replace')
  if 'integrated runtime provisioned and verified' in text:
   assert 'commit='+report['source'] in text,'Native source differs'
   (out/'startup-app.log').write_text(text)
   print('native task app provisioned; ready for actual UI checks',flush=True);break
  if 'integrated runtime provisioning failed' in text:raise RuntimeError('Actual offline supply failed; retain owned root and logs')
 time.sleep(.5)
else:raise RuntimeError('Actual offline supply not proved inside original 240-second observation window')

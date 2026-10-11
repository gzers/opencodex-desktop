import hashlib,json,plistlib,shutil,subprocess,time
from pathlib import Path
base=Path('/private/tmp/ocx-0110-deb740-foreground')
base.mkdir(exist_ok=False)
old=Path('/private/tmp/ocx-0110-c1e3-candidate-hqjf10k6')
original=Path('/private/tmp/ocx-0110-clock-audit/candidate/original/OpenCodeX Desktop.app')
app=base/'test-install/OpenCodeX Desktop.app'
app.parent.mkdir()
shutil.copytree(original,app,symlinks=True)
info=app/'Contents/Info.plist'
value=plistlib.loads(info.read_bytes())
value['CFBundleIdentifier']='com.gzers.opencodex.desktop.sandbox0110deb740'
info.write_bytes(plistlib.dumps(value))
commands=[]
for argv in [['codesign','--force','--deep','--sign','-',str(app)],['codesign','--verify','--deep','--strict','--verbose=2',str(app)],['clang','-dynamiclib','-Wall','-Wextra','-Werror','/Users/ezio/.codex/worktrees/ocx-0110-main/test/smoke/macos-update-probe.c','-o',str(base/'update-probe.dylib')]]:
 r=subprocess.run(argv,capture_output=True,text=True)
 commands.append(dict(argv=argv,exit_code=r.returncode,stdout=r.stdout,stderr=r.stderr))
 assert r.returncode==0,r.stderr
for name in ['foreground-monitor-ready','foreground-monitor-ready.swift']:
 shutil.copy2(old/name,base/name)
root=base/'test-data'
for name in ['manager-state/data-root.json','manager-state/preferences.json','manager-state/runtime.json','manager-state/update-schedule.json','cache/updates/manager-stable.json','cache/updates/panel-latest.json']:
 target=root/name
 target.parent.mkdir(parents=True,exist_ok=True)
 shutil.copy2(old/'test-data'/name,target)
files={}
for name in ['manager-state/update-schedule.json','cache/updates/manager-stable.json','cache/updates/panel-latest.json']:
 data=(root/name).read_bytes()
 files[name]={'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data)}
(base/'cache-before.json').write_text(json.dumps({'files':files,'schedule':json.loads((root/'manager-state/update-schedule.json').read_text()),'fixture':'Copied prior isolated test cache and future deadlines; no user data; cache-hit scenario only'},indent=2))
exe=app/'Contents/MacOS/opencodex-desktop'
identity={'source_sha':'deb740c09857b5f0bef7c9b4090fe5e8587bb4b3','prepared_at':time.time(),'app':str(app),'bundle_id':value['CFBundleIdentifier'],'commands':commands,'test_exe_sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'original_exe_sha256':hashlib.sha256((original/'Contents/MacOS/opencodex-desktop').read_bytes()).hexdigest(),'source_embedded':b'deb740c09857b5f0bef7c9b4090fe5e8587bb4b3' in exe.read_bytes(),'test_copy_adhoc_only':True,'original_preserved':True}
assert identity['source_embedded']
(base/'test-identity.json').write_text(json.dumps(identity,indent=2))
launch=(old/'probe-launch-bundle.py').read_text().replace(str(old),str(base)).replace('c1e3c09d8ed33238543ce5943e439991b8099088',identity['source_sha'])
(base/'launch.py').write_text(launch)
verify=(old/'probe-verify-phase.py').read_text().replace(str(old),str(base)).replace('probe-cache-before.json','cache-before.json')
(base/'verify.py').write_text(verify)
print(json.dumps(identity))

import os,sys,json,subprocess,shutil,hashlib
from pathlib import Path
base=Path(__file__).parent
label=sys.argv[1];port=int(sys.argv[2]);root=base/label
assert label in ['baseline','candidate']
for name in ['sandbox/opencodex-home','sandbox/home/AppData/Roaming','sandbox/home/AppData/Local','sandbox/home/.codex','home/AppData/Roaming','home/AppData/Local','home/.codex','temp']:(root/name).mkdir(parents=True,exist_ok=True)
source=base.parent/'post021-materials/candidate/opencodex-desktop.exe' if label=='baseline' else base.parent.parent/'feature-0.2.19-fixes/apps/desktop/tauri/target/debug/opencodex-desktop.exe'
exe=root/'opencodex-desktop.exe';shutil.copy2(source,exe)
shutil.copy2(base.parent/'post021-materials/candidate/WebView2Loader.dll',root/'WebView2Loader.dll')
if label=='candidate':subprocess.run([sys.executable,str(base.parent/'post021-materials/manifest-candidate.py'),str(exe)],check=True)
env={key:value for key,value in os.environ.items() if key.upper() in ['SYSTEMROOT','WINDIR','COMSPEC','PATHEXT','PROCESSOR_ARCHITECTURE','NUMBER_OF_PROCESSORS','SYSTEMDRIVE']}
env.update({'OPENCODEX_SANDBOX':'1','OPENCODEX_SANDBOX_ROOT':str(root/'sandbox'),'OPENCODEX_SANDBOX_BOUNDARY':str(root),'OPENCODEX_WINDOWS_SMOKE_CDP_PORT':str(port),'HOME':str(root/'home'),'USERPROFILE':str(root/'home'),'APPDATA':str(root/'home/AppData/Roaming'),'LOCALAPPDATA':str(root/'home/AppData/Local'),'CODEX_HOME':str(root/'home/.codex'),'OPENCODEX_HOME':str(root/'sandbox/opencodex-home'),'TEMP':str(root/'temp'),'TMP':str(root/'temp'),'PATH':str(base/'tools/node_modules/.bin')+';C:/Windows/System32;C:/Windows'})
with (root/'stdout.log').open('w',encoding='utf-8') as out,(root/'stderr.log').open('w',encoding='utf-8') as err:
 child=subprocess.Popen([str(exe)],env=env,cwd=root,stdout=out,stderr=err)
 record={'pid':child.pid,'exe':str(exe),'sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'source':str(source),'port':port,'sandbox':str(root/'sandbox'),'environment':env}
 (root/'launch.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
 print(json.dumps({key:record[key] for key in ['pid','port','sandbox','sha256']}),flush=True)
 print('exit='+str(child.wait()),flush=True)

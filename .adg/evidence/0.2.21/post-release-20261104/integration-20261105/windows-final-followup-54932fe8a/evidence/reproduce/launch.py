import os,sys,json,subprocess,shutil,hashlib
from pathlib import Path
base=Path(__file__).resolve().parent;workspace=base.parent.parent
label=sys.argv[1];port=int(sys.argv[2]);assert label in ['model','model-fresh','model-final','model-complete','model-regression','integrated']
root=base/label;root.mkdir(exist_ok=True)
for name in ['sandbox/opencodex-home','sandbox/home','home/AppData/Roaming','home/AppData/Local','home/.codex','temp']:(root/name).mkdir(parents=True,exist_ok=True)
source=workspace/'feature-0.2.19-fixes/apps/desktop/tauri/target/debug/opencodex-desktop.exe';exe=root/'opencodex-desktop.exe';shutil.copy2(source,exe)
shutil.copy2(source.parent/'WebView2Loader.dll',root/'WebView2Loader.dll')
subprocess.run([sys.executable,str(base.parent/'post021-materials/manifest-candidate.py'),str(exe)],check=True,stdout=(root/'manifest.log').open('w'))
env={k:v for k,v in os.environ.items() if k.upper() in ['SYSTEMROOT','WINDIR','COMSPEC','PATHEXT','PROCESSOR_ARCHITECTURE','NUMBER_OF_PROCESSORS','SYSTEMDRIVE']}
env.update({'OPENCODEX_SANDBOX':'1','OPENCODEX_SANDBOX_ROOT':str(root/'sandbox'),'OPENCODEX_SANDBOX_BOUNDARY':str(root),'OPENCODEX_WINDOWS_SMOKE_CDP_PORT':str(port),'HOME':str(root/'home'),'USERPROFILE':str(root/'home'),'APPDATA':str(root/'home/AppData/Roaming'),'LOCALAPPDATA':str(root/'home/AppData/Local'),'CODEX_HOME':str(root/'home/.codex'),'OPENCODEX_HOME':str(root/'sandbox/opencodex-home'),'TEMP':str(root/'temp'),'TMP':str(root/'temp'),'PATH':str(base.parent/'post021-restart-panel/tools/node_modules/.bin')+';C:/Users/15119/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin;C:/Windows/System32;C:/Windows'})
ca=base/'tls/ca.pem'
if ca.exists():env['SSL_CERT_FILE']=str(ca);env['SSL_CERT_DIR']=str(base/'tls/empty')
with(root/'stdout.log').open('w',encoding='utf-8')as out,(root/'stderr.log').open('w',encoding='utf-8')as err:
 p=subprocess.Popen([str(exe)],cwd=root,env=env,stdout=out,stderr=err)
 record={'pid':p.pid,'exe':str(exe),'sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'sourceCommit':subprocess.check_output(['git','-C',str(workspace/'post021-windows-materials'),'rev-parse','HEAD'],text=True).strip(),'sandbox':str(root/'sandbox'),'port':port,'environment':env}
 (root/'launch.json').write_text(json.dumps(record,indent=2)+chr(10),encoding='utf-8');print(json.dumps({k:v for k,v in record.items() if k!='environment'}),flush=True);print('exit='+str(p.wait()),flush=True)

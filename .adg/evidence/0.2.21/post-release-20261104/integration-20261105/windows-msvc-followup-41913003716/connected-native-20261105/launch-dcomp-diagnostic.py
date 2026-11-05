import os,sys,json,hashlib,subprocess,shutil
from pathlib import Path
b=Path(__file__).resolve().parent;w=b.parent.parent
role=sys.argv[1];assert role in ['panel','cloud','target','perf','baseline']
port=int(sys.argv[2]);root=w/'.toolchain/post021-windows-final-msvc-41913003716'/role
for n in ['sandbox/opencodex-home','sandbox/home/.local/bin','sandbox/manager-state','home/.codex','home/AppData/Local','home/AppData/Roaming','temp']:(root/n).mkdir(parents=True,exist_ok=True)
src=b/'candidate/opencodex-desktop.exe';assert hashlib.sha256(src.read_bytes()).hexdigest()=='dd6f37cf2f6094cf9254ead08217cf8456e190cb213d3198c1b528172f1a0184'
exe=root/'opencodex-desktop.exe';shutil.copy2(src,exe)
node=Path('C:/Users/15119/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node.exe');tools=root/'sandbox/home/.local/bin';shutil.copy2(node,tools/'node.exe')
for name,script in [('npm',node.parent/'node_modules/npm/bin/npm-cli.js')]:
 (tools/(name+'.cmd')).write_text('@echo off'+chr(13)+chr(10)+chr(34)+'%~dp0node.exe'+chr(34)+' '+chr(34)+str(script)+chr(34)+' %*'+chr(13)+chr(10),encoding='utf-8',newline='')
probe=w/'.toolchain/post021-windows-final/status-reviewed.exe';shutil.copy2(probe,root/'status-only.exe')
shutil.copy2(probe,tools/'ocx.exe')
runtime=root/'sandbox/manager-state/runtime.json'
if not runtime.exists():runtime.write_text(json.dumps({'schema_version':1,'source':'explicit','path':str(root/'status-only.exe'),'resolved_version':None,'resolved_at':None,'history':[]}),encoding='utf-8')
config=root/'sandbox/opencodex-home/config.json'
if not config.exists():config.write_text(json.dumps({'defaultProvider':'local-fixture','providers':{'local-fixture':{'name':'Local scope fixture','adapter':'openai-chat','authMode':'key','baseUrl':'http://127.0.0.1:60443/v1','models':['local/raw-model'],'selectedModels':['local/raw-model'],'disabled':True}},'codexAutoStart':False,'oauthOpenBrowser':False,'syncOnStart':False,'autoUpdate':False,'claudeCodeEnabled':False,'clientIntegrations':{'codex':False,'grok':False,'claude':False}}),encoding='utf-8')
env={k:v for k,v in os.environ.items()if k.upper()in ['SYSTEMROOT','WINDIR','COMSPEC','PATHEXT','SYSTEMDRIVE','PROCESSOR_ARCHITECTURE','NUMBER_OF_PROCESSORS']}
env.update({'OPENCODEX_SANDBOX':'1','OPENCODEX_SANDBOX_ROOT':str(root/'sandbox'),'OPENCODEX_SANDBOX_BOUNDARY':str(root),'OPENCODEX_WINDOWS_SMOKE_CDP_PORT':str(port),'HOME':str(root/'home'),'USERPROFILE':str(root/'home'),'APPDATA':str(root/'home/AppData/Roaming'),'LOCALAPPDATA':str(root/'home/AppData/Local'),'CODEX_HOME':str(root/'home/.codex'),'OPENCODEX_HOME':str(root/'sandbox/opencodex-home'),'TEMP':str(root/'temp'),'TMP':str(root/'temp'),'PATH':str(tools)+';C:/Windows/System32;C:/Windows','DO_NOT_TRACK':'1'})
if role=='baseline':env['WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS']='--disable-direct-composition'
if (b/'tls/ca.pem').exists():env.update(SSL_CERT_FILE=str(b/'tls/ca.pem'),SSL_CERT_DIR=str(b/'tls/empty'))
with(root/'stdout.log').open('w',encoding='utf-8')as out,(root/'stderr.log').open('w',encoding='utf-8')as err:
 p=subprocess.Popen([str(exe)],cwd=root,env=env,stdout=out,stderr=err)
 identity={'pid':p.pid,'exe':str(exe),'sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'buildCommit':'127543d2d0d9c2725f1a357710cd43bc69e3c44d','port':port,'role':role,'sandbox':str(root/'sandbox'),'environment':env}
 (root/'launch.json').write_text(json.dumps(identity,indent=2),encoding='utf-8');print(json.dumps({k:v for k,v in identity.items()if k!='environment'}),flush=True);print('exit='+str(p.wait()),flush=True)

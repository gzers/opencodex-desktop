import json,subprocess,hashlib,shutil
from pathlib import Path
base=Path(__file__).resolve().parent;previous=base/'integrated';root=base/'first-scope-reviewed'
assert not root.exists(),'fresh root required'
for name in ['sandbox/opencodex-home','sandbox/home/.local/bin','home/AppData/Roaming','home/AppData/Local','home/.codex','temp']:(root/name).mkdir(parents=True,exist_ok=True)
for p in (previous/'sandbox/home/.local/bin').iterdir():shutil.copy2(p,root/'sandbox/home/.local/bin'/p.name)
config={'defaultProvider':'scope-fixture','providers':{'scope-fixture':{'name':'范围合成渠道','adapter':'openai-chat','authMode':'key','baseUrl':'http://127.0.0.1:60443/v1','models':['vendor/raw-model'],'selectedModels':['vendor/raw-model'],'disabled':True}}}
(root/'sandbox/opencodex-home/config.json').write_text(json.dumps(config,ensure_ascii=False),encoding='utf-8')
old=json.loads((previous/'launch.json').read_text(encoding='utf-8'));env={k:v.replace(str(previous),str(root)) for k,v in old['environment'].items()};env['OPENCODEX_WINDOWS_SMOKE_CDP_PORT']='9446'
exe=previous/'opencodex-desktop.exe';assert hashlib.sha256(exe.read_bytes()).hexdigest()=='76dc1049f4b6c3995993b2c1457ae06e1d3627669e20bd8c62ffdd98054b778a'
with(root/'stdout.log').open('w')as out,(root/'stderr.log').open('w')as err:
 p=subprocess.Popen([str(exe)],env=env,cwd=root,stdout=out,stderr=err);(root/'identity.json').write_text(json.dumps({'pid':p.pid,'exe':str(exe),'dataRoot':str(root/'sandbox'),'port':9446,'environment':env},indent=2),encoding='utf-8');print('fresh scope pid='+str(p.pid),flush=True);print('exit='+str(p.wait()),flush=True)

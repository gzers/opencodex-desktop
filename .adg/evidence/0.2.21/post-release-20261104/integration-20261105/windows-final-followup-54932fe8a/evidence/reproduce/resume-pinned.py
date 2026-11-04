import json,subprocess,hashlib
from pathlib import Path
base=Path(__file__).resolve().parent
root=base/'integrated'
launch=json.loads((root/'launch.json').read_text(encoding='utf-8'))
exe=Path(launch['exe'])
assert exe.resolve().parent==root.resolve()
assert hashlib.sha256(exe.read_bytes()).hexdigest()=='76dc1049f4b6c3995993b2c1457ae06e1d3627669e20bd8c62ffdd98054b778a'
env=launch['environment']
assert env['OPENCODEX_SANDBOX']=='1' and Path(env['OPENCODEX_SANDBOX_ROOT']).resolve()==(root/'sandbox').resolve()
with(root/'resume.stdout').open('w',encoding='utf-8')as out,(root/'resume.stderr').open('w',encoding='utf-8')as err:
 p=subprocess.Popen([str(exe)],env=env,cwd=root,stdout=out,stderr=err)
 (base/'resume-identity.json').write_text(json.dumps({'pid':p.pid,'exe':str(exe),'sha256':launch['sha256'],'frozenBuild':'bd7c0fe8ccecfa39e0706c885dc233f3854dab74','cdpPort':9444},indent=2),encoding='utf-8')
 print('pinned native pid='+str(p.pid),flush=True)
 print('exit='+str(p.wait()),flush=True)

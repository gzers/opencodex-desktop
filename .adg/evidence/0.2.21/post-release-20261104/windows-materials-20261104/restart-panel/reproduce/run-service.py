import sys,subprocess,os,json
from pathlib import Path
base=Path(__file__).parent;label=sys.argv[1];port=sys.argv[2];root=base/label
assert label in ['baseline','candidate']
env=json.loads((root/'launch.json').read_text())['environment']
env.update({'PATH':str(base/'tools/node_modules/.bin')+';C:/Users/15119/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin;C:/Windows/System32;C:/Windows','DO_NOT_TRACK':'1'})
bun=base/'tools/node_modules/@oven/bun-windows-x64/bin/bun.exe'
with (root/'server.stdout').open('w',encoding='utf-8') as out,(root/'server.stderr').open('w',encoding='utf-8') as err:
 p=subprocess.Popen([str(bun),str(base/'official-server.ts'),label,port],env=env,cwd=root,stdout=out,stderr=err)
 print('official fixture pid='+str(p.pid),flush=True);print('exit='+str(p.wait()),flush=True)

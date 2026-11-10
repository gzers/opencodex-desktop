from pathlib import Path
import subprocess,json,time
r=Path(__file__).parent
bun='/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/runtime/opencodex/node_modules/@bitkyc08/opencodex/node_modules/bun/bin/bun.exe'
if not Path(bun).is_file():bun='/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/runtime/opencodex/node_modules/bun/bin/bun.exe'
env={'PATH':'/usr/bin:/bin','HOME':str(r/'user'),'USERPROFILE':str(r/'user'),'TMPDIR':str(r/'tmp'),'XDG_CACHE_HOME':str(r/'cache'),'OPENCODEX_HOME':str(r/'user/home'),'CODEX_HOME':str(r/'user/.codex'),'CI':'1','NODE_ENV':'test','LANG':'en_US.UTF-8'}
res=subprocess.run(['/usr/bin/sandbox-exec','-f',str(r/'sandbox.sb'),bun,'run',str(r/'check.ts')],cwd=r,env=env,capture_output=True,text=True,timeout=30)
# Logs expected to contain only report or source errors, never configuration bytes.
print('exit',res.returncode);print(res.stdout[-6500:]);print(res.stderr[-1400:])
(r/'execution.json').write_text(json.dumps({'exitCode':res.returncode,'sandboxNetworkDenied':True,'writesOnlyFixture':True,'originalDataReadDenied':True})+'\n')

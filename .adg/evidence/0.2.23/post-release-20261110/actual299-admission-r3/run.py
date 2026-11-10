from pathlib import Path
import subprocess,os,json,time
r=Path(__file__).parent
bun='/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/runtime/opencodex/node_modules/bun/bin/bun.exe'
env={'PATH':'/usr/bin:/bin','HOME':str(r/'fixture-user'),'USERPROFILE':str(r/'fixture-user'),'TMPDIR':str(r/'tmp'),'XDG_CACHE_HOME':str(r/'cache'),'OPENCODEX_HOME':str(r/'fixture-user/.codex-desktop/opencodex/home'),'CODEX_HOME':str(r/'fixture-user/.codex'),'CI':'1','NODE_ENV':'test','LANG':'en_US.UTF-8'}
start=time.monotonic()
try:
 result=subprocess.run(['/usr/bin/sandbox-exec','-f',str(r/'sandbox.sb'),bun,'run',str(r/'probe.ts')],cwd=r,env=env,capture_output=True,text=True,timeout=35)
 (r/'execution.stdout.log').write_text(result.stdout);(r/'execution.stderr.log').write_text(result.stderr)
 print('exit',result.returncode,'seconds',round(time.monotonic()-start,2));print(result.stdout[-2200:]);print(result.stderr[-1800:])
except subprocess.TimeoutExpired as e:
 (r/'execution.timeout.json').write_text(json.dumps({'timeout':35,'stdout':str(e.stdout),'stderr':str(e.stderr)}));print('bounded timeout')

from pathlib import Path
import subprocess,json,time
r=Path('/var/folders/fq/l0j4lwrs5ld6gpfx2pppv2640000gn/T/ocxd-official299-gates-1g7e493p').resolve();e=Path(__file__).parent.resolve()
bun='/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/runtime/opencodex/node_modules/bun/bin/bun.exe'
env={'PATH':'/usr/bin:/bin','HOME':str(r/'user'),'USERPROFILE':str(r/'user'),'TMPDIR':str(r/'tmp'),'XDG_CACHE_HOME':str(r/'user/cache'),'OPENCODEX_HOME':str(r/'user/opencodex'),'CODEX_HOME':str(r/'user/codex'),'CI':'1','NODE_ENV':'test','LANG':'en_US.UTF-8','PROBE_ROOT':str(r)}
(r/'probe.ts').write_bytes((e/'display-probe.ts').read_bytes())
start=time.monotonic();p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(r/'probe.sb'),bun,'run',str(r/'probe.ts')],cwd=r,env=env,capture_output=True,text=True,timeout=40)
a=e/'display-attempt1';a.mkdir(exist_ok=False);(a/'stdout.log').write_text(p.stdout);(a/'stderr.log').write_text(p.stderr);(a/'execution.json').write_text(json.dumps({'exit':p.returncode,'elapsedSeconds':time.monotonic()-start,'sandboxNetworkDenied':True,'writesOnlyFixture':True},indent=2))
if (r/'out/display-result.json').exists():(a/'result.json').write_bytes((r/'out/display-result.json').read_bytes())
print('exit',p.returncode);print(p.stdout[-1500:]);print(p.stderr[-2500:])

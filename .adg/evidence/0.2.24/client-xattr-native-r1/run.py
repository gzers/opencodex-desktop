from pathlib import Path
import os,sys,time,subprocess,json
r=Path(__file__).parent;name=sys.argv[1];cmd=sys.argv[2:];env=os.environ.copy();env['TMPDIR']=str(r/'unit-tmp');t=time.monotonic()
with (r/(name+'.log')).open('wb') as out:p=subprocess.run(cmd,stdout=out,stderr=subprocess.STDOUT,env=env)
(r/(name+'.json')).write_text(json.dumps({'command':cmd,'exitCode':p.returncode,'elapsedSeconds':round(time.monotonic()-t,3)}))
print(json.dumps({'name':name,'exitCode':p.returncode}),flush=True)
print('\n'.join(v for v in (r/(name+'.log')).read_text(errors='replace').splitlines() if len(v)<1400)[-6000:],flush=True)
sys.exit(p.returncode)

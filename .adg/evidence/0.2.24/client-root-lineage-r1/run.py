import json,os,subprocess,sys,time
from pathlib import Path
r=Path(__file__).parent
name=sys.argv[1]
cmd=sys.argv[2:]
env=os.environ.copy();env['TMPDIR']=str(r/'unit-tmp')
t=time.monotonic()
with (r/(name+'.log')).open('wb') as out:
 p=subprocess.run(cmd,stdout=out,stderr=subprocess.STDOUT,env=env)
(r/(name+'.json')).write_text(json.dumps({'command':cmd,'exitCode':p.returncode,'elapsedSeconds':round(time.monotonic()-t,3)}))
print(json.dumps({'name':name,'exitCode':p.returncode,'root':str(r)}),flush=True)
print('\n'.join(line for line in (r/(name+'.log')).read_text(errors='replace').splitlines() if len(line)<1200)[-5000:],flush=True)
sys.exit(p.returncode)

from pathlib import Path
import sys,subprocess,os,json,time
r=Path(__file__).resolve().parent;name,code,*cmd=sys.argv[1:];log=r/(name+'.log');report=r/(name+'.json');assert not log.exists() and not report.exists()
env=os.environ.copy();env['TMPDIR']=json.loads((r/'hypothesis.json').read_text())['tempRoot'];env['PYTHONDONTWRITEBYTECODE']='1';t=time.monotonic()
with log.open('xb') as f:p=subprocess.run(cmd,cwd=code,env=env,stdout=f,stderr=subprocess.STDOUT)
report.write_text(json.dumps({'command':cmd,'exitCode':p.returncode,'seconds':round(time.monotonic()-t,3),'windowsNativeExecutedHere':False},indent=2)+'\n');print(report.read_text());print(log.read_text(errors='replace')[-4500:]);sys.exit(p.returncode)

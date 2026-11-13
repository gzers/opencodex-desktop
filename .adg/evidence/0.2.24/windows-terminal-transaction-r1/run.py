"""Immutable isolated checks for the shared terminal transaction."""
from pathlib import Path
import json,os,subprocess,sys,time
here=Path(__file__).resolve().parent
name,code,*command=sys.argv[1:]
assert name.replace('-','').isalnum()
path=here/(name+'.log');record=here/(name+'.json')
assert not path.exists() and not record.exists()
env=os.environ.copy();env['TMPDIR']=json.loads((here/'hypothesis.json').read_text())['temporaryRoot'];env['PYTHONDONTWRITEBYTECODE']='1'
t=time.monotonic()
with path.open('xb') as output: result=subprocess.run(command,cwd=code,env=env,stdout=output,stderr=subprocess.STDOUT)
record.write_text(json.dumps({'command':command,'cwd':code,'exitCode':result.returncode,'elapsedSeconds':round(time.monotonic()-t,3),'windowsNativeExecuted':False},indent=2)+'\n')
print(record.read_text());print(path.read_text(errors='replace')[-4500:]);sys.exit(result.returncode)

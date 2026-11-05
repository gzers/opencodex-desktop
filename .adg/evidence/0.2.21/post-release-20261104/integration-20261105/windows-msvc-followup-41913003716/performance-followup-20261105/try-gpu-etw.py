import json,subprocess,time
from pathlib import Path
b=Path('.toolchain/post021-msvc-followup');out=b/'gpu-etw-admission';out.mkdir(exist_ok=True);exe='C:/Windows/System32/wpr.exe'
def run(args):
 r=subprocess.run([exe,*args],capture_output=True,text=True,timeout=15);return {'args':args,'exitCode':r.returncode,'stdout':r.stdout,'stderr':r.stderr}
state=run(['-status']);report={'preflight':state,'scope':'GPU/desktop composition scheduling diagnostics; no file/network/registry provider requested'}
assert state['exitCode']==0 and 'not recording'in(state['stdout']+state['stderr']).lower(),'existing recording or unknown status; no start'
started=run(['-start','GPU','-filemode']);report['start']=started
if started['exitCode']==0:
 try:time.sleep(10)
 finally:report['stop']=run(['-stop',str((out/'gpu-scheduling.etl').resolve())])
report['at']=time.time();(out/'result.json').write_text(json.dumps(report,indent=2)+chr(10),encoding='utf-8');print(json.dumps(report))

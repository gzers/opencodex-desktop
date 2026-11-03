import os,sys,json,subprocess,datetime
from pathlib import Path
root=Path('/tmp/ocx020-final-review-agent')
engine=sys.argv[1]
assert engine in ('chromium','webkit')
records=[]
for suite in ['model-dialogs','extensions','secondary','model-completion']:
 out=root/'reports'/(suite+'-'+engine+'-r1');out.mkdir()
 temp=root/'browser-temp'/(suite+'-'+engine);temp.mkdir()
 env=os.environ.copy();env.update(AUDIT_ENGINE=engine,AUDIT_OUT=str(out),AUDIT_SOURCE='bda7c92da4e1f2e02e6bc2c641e0747748b98dd5',TMPDIR=str(temp))
 started=datetime.datetime.now(datetime.timezone.utc).isoformat()
 with (out/'execution.log').open('w') as log:
  try:
   p=subprocess.run(['node',str(root/'harness'/(suite+'.mjs'))],cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=360)
   code=p.returncode
  except subprocess.TimeoutExpired:code=124
 rec={'suite':suite,'engine':engine,'startedAt':started,'finishedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exitCode':code,'output':str(out)}
 (out/'execution.json').write_text(json.dumps(rec,indent=2)+'\n');records.append(rec)
 print(json.dumps(rec),flush=True)
(root/'receipts'/('matrix-runs-'+engine+'.json')).write_text(json.dumps(records,indent=2)+'\n')

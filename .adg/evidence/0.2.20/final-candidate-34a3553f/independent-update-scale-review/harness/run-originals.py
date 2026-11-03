from pathlib import Path
import subprocess,os,json,sys,datetime
R=Path('/tmp/ocx020-final-review-agent/update-scale-fix')
engine=sys.argv[1];assert engine in ['chromium','webkit']
identity=json.loads((R/'receipts/source-identity-before.json').read_text())
records=[]
for suite in ['update-scale-contract','update-hosts']:
 out=R/'reports'/f'{suite}-{engine}-r1';out.mkdir()
 temp=R/'browser-temp'/f'{suite}-{engine}';temp.mkdir()
 env=os.environ.copy();overrides={'AUDIT_ENGINE':engine,'AUDIT_OUT':str(out),'AUDIT_SOURCE':'working-UpdateCenter-sha256:'+identity['UpdateCenterSha256'],'TMPDIR':str(temp)};env.update(overrides)
 argv=['node',str(R/'harness'/f'{suite}.mjs')];record={'argv':argv,'envOverrides':overrides,'cwd':str(R),'startedAt':datetime.datetime.now(datetime.timezone.utc).isoformat()}
 with (out/'execution.log').open('w') as f:
  try:p=subprocess.run(argv,cwd=R,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=180);record['exitCode']=p.returncode
  except subprocess.TimeoutExpired:record['exitCode']=124
 record['finishedAt']=datetime.datetime.now(datetime.timezone.utc).isoformat();(out/'execution.json').write_text(json.dumps(record,indent=2)+chr(10));records.append(record);print(json.dumps({'suite':suite,'engine':engine,'exitCode':record['exitCode'],'out':str(out)}),flush=True)
(R/'receipts'/f'original-runs-{engine}.json').write_text(json.dumps(records,indent=2)+chr(10))

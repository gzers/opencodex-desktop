from pathlib import Path
import json,subprocess,time
c=json.loads(Path('/private/tmp/ocxd024consumers-active.json').read_text());r=Path(c['root']);e=Path(c['evidence']);runs=[]
checks=[('handle-final','modules::runtime::tests::'),('source-dto-final','commands::runtime::tests::source_dto_'),('resource-source-final','fixed_resource_resolver_authenticates_descriptor_and_both_bundled_and_staged_sources'),('attested-final','infrastructure::attested_status::tests::')]
for label,filter in checks:
 start=time.monotonic();policy='loopback.sb' if label=='attested-final' else 'sandbox.sb';p=subprocess.run(['/usr/bin/sandbox-exec','-f',str(r/policy),str(r/'test'),filter,'--nocapture','--test-threads=1'],cwd=r,env=c['env'],capture_output=True,text=True,timeout=85);(e/(label+'.stdout.log')).write_text(p.stdout);(e/(label+'.stderr.log')).write_text(p.stderr);item={'label':label,'filter':filter,'exit':p.returncode,'elapsedSeconds':time.monotonic()-start,'summary':[s for s in p.stdout.splitlines() if 'test result:' in s]};runs.append(item);(e/'runs-final.json').write_text(json.dumps(runs,indent=2));print(json.dumps(item),flush=True)
(e/'finished-final.json').write_text(json.dumps({'allPassed':all(x['exit']==0 for x in runs)},indent=2))

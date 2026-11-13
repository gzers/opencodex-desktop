from pathlib import Path
import json,subprocess,hashlib
base=Path('/private/tmp/ocxd024-exit-pair-kqi11hjd/evidence');records=[json.loads(l) for l in (base/'build-final.jsonl').read_text().splitlines() if l.startswith('{')]
exe=Path(next(r['executable'] for r in records if r.get('reason')=='compiler-artifact' and r.get('target',{}).get('name')=='opencodex_desktop_lib' and r.get('executable')));result={'binary':str(exe),'sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'checks':[]}
for label,pattern in [('pair','infrastructure::macos_process_pair::tests::'),('exit','infrastructure::macos_process_exit::tests::'),('session','infrastructure::macos_client_originals::session::tests::'),('binding','commands::runtime_recovery_v2_owner::tests::')]:
 with (base/(label+'-final.stdout.log')).open('wb') as stdout,(base/(label+'-final.stderr.log')).open('wb') as stderr:p=subprocess.run([str(exe),pattern,'--nocapture','--test-threads=2'],stdout=stdout,stderr=stderr)
 output=(base/(label+'-final.stdout.log')).read_text();result['checks'].append({'name':label,'exit':p.returncode,'summary':next((l for l in output.splitlines() if l.startswith('test result:')),None)});(base/'checks-final.json').write_text(json.dumps(result,indent=2));print(label,p.returncode,flush=True)
 assert p.returncode==0 and 'running 0 tests' not in output,(label,p.returncode)

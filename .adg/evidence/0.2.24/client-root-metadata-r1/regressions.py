from pathlib import Path
import subprocess,sys,json
r=Path(__file__).parent
filters=['infrastructure::macos_client_originals::root_creation::','infrastructure::macos_client_originals::root_retirement::history_tests::','infrastructure::macos_client_originals::session::root_stop_binding_tests::','infrastructure::macos_client_originals::root_originals_tests::','root_restore_registration_','commands::lifecycle_versioned_tests::']
results=[]
for i,f in enumerate(filters):
 p=subprocess.run([sys.executable,str(r/'run.py'),f'regression-{i+1}','cargo','test','--manifest-path','apps/desktop/tauri/Cargo.toml','--lib',f,'--','--nocapture','--test-threads=1']);results.append({'name':f'regression-{i+1}','filter':f,'exitCode':p.returncode})
(r/'regression-summary.json').write_text(json.dumps(results,indent=2))
sys.exit(int(any(v['exitCode'] for v in results)))

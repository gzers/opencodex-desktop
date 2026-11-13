from pathlib import Path
import subprocess,sys,json
r=Path(__file__).parent
filters=['root_retirement::index_tests::','root_stop_binding_tests::','successor_capture_publication_rechecks_custody_and_deadline_after_sql_and_commit','commands::lifecycle_versioned_tests::']
results=[]
for i,f in enumerate(filters):
 p=subprocess.run([sys.executable,str(r/'run.py'),f'regression-{i+1}','cargo','test','--manifest-path','apps/desktop/tauri/Cargo.toml','--lib',f,'--','--nocapture','--test-threads=1'])
 results.append({'filter':f,'exitCode':p.returncode})
(r/'regression-summary.json').write_text(json.dumps(results,indent=2))
sys.exit(int(any(v['exitCode'] for v in results)))

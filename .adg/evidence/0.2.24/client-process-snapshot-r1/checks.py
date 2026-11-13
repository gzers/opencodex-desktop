from pathlib import Path
import sys,subprocess,json
r=Path(__file__).parent
filters=['commands::runtime_recovery_v2_owner::','commands::lifecycle_versioned_tests::','infrastructure::macos_process_exit::','infrastructure::macos_process_pair::']
results=[]
for i,f in enumerate(filters):
 p=subprocess.run([sys.executable,str(r/'run.py'),f'regression-{i+1}','cargo','test','--manifest-path','apps/desktop/tauri/Cargo.toml','--lib',f,'--','--nocapture','--test-threads=1']);results.append({'name':f'regression-{i+1}','filter':f,'exitCode':p.returncode})
for name,cmd in [('clippy',['cargo','clippy','--manifest-path','apps/desktop/tauri/Cargo.toml','--all-targets','--','-D','warnings']),('format',['cargo','fmt','--manifest-path','apps/desktop/tauri/Cargo.toml','--','--check']),('included-test-format',['rustfmt','--edition','2021','--check','apps/desktop/tauri/src/commands/runtime_recovery_macos_snapshot_tests.rs']),('diff-check',['git','diff','--check'])]:
 p=subprocess.run([sys.executable,str(r/'run.py'),name,*cmd]);results.append({'name':name,'exitCode':p.returncode})
(r/'checks-summary.json').write_text(json.dumps(results,indent=2))
sys.exit(int(any(v['exitCode'] for v in results)))

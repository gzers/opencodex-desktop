from pathlib import Path
import subprocess,sys,json
r=Path(__file__).parent
commands=[('command-regression',['cargo','test','--manifest-path','apps/desktop/tauri/Cargo.toml','--lib','commands::lifecycle_versioned_tests::','--','--nocapture','--test-threads=1']),('clippy',['cargo','clippy','--manifest-path','apps/desktop/tauri/Cargo.toml','--all-targets','--','-D','warnings']),('format',['cargo','fmt','--manifest-path','apps/desktop/tauri/Cargo.toml','--','--check']),('diff-check',['git','diff','--check'])]
results=[]
for name,cmd in commands:
 p=subprocess.run([sys.executable,str(r/'run.py'),name,*cmd]);results.append({'name':name,'exitCode':p.returncode})
(r/'finish-summary.json').write_text(json.dumps(results,indent=2))
sys.exit(int(any(v['exitCode'] for v in results)))

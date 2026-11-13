from pathlib import Path
import subprocess,sys,json
here=Path(__file__).resolve().parent;code=sys.argv[1]
checks=[('backend-terminal',['cargo','test','--manifest-path','apps/desktop/tauri/Cargo.toml','--lib','modules::runtime::terminal_cli::','--offline','--','--test-threads=1']),('backend-uninstall',['cargo','test','--manifest-path','apps/desktop/tauri/Cargo.toml','--lib','modules::data_root::uninstall::','--offline','--','--test-threads=1']),('console-client',['cargo','test','--manifest-path','apps/desktop/tauri/Cargo.toml','--bin','ocxd','--offline','--','--test-threads=1']),('clippy',['cargo','clippy','--manifest-path','apps/desktop/tauri/Cargo.toml','--all-targets','--offline','--','-D','warnings']),('format',['cargo','fmt','--manifest-path','apps/desktop/tauri/Cargo.toml','--','--check']),('diff-check',['git','diff','--check'])]
results=[]
for name,cmd in checks:
 p=subprocess.run([sys.executable,str(here/'run.py'),name,code,*cmd]);results.append({'name':name,'exitCode':p.returncode})
(here/'backend-summary.json').write_text(json.dumps(results,indent=2)+'\n')
sys.exit(int(any(v['exitCode'] for v in results)))

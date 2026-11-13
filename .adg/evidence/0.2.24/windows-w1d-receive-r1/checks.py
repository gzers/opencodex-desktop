from pathlib import Path
import subprocess,sys,json
r=Path(__file__).resolve().parent;code=sys.argv[1]
checks=[('terminal',['cargo','test','--manifest-path','apps/desktop/tauri/Cargo.toml','--lib','modules::runtime::terminal_cli::','--offline','--','--test-threads=1']),('layout',['cargo','test','--manifest-path','apps/desktop/tauri/Cargo.toml','--lib','modules::runtime::program_layout::','--offline','--','--test-threads=1']),('clippy',['cargo','clippy','--manifest-path','apps/desktop/tauri/Cargo.toml','--all-targets','--offline','--','-D','warnings']),('format',['cargo','fmt','--manifest-path','apps/desktop/tauri/Cargo.toml','--','--check']),('diff-check',['git','diff','--check'])]
rows=[]
for name,cmd in checks:
 p=subprocess.run([sys.executable,str(r/'run.py'),name,code,*cmd]);rows.append({'name':name,'exitCode':p.returncode})
(r/'host-checks.json').write_text(json.dumps(rows,indent=2)+'\n');sys.exit(int(any(v['exitCode'] for v in rows)))

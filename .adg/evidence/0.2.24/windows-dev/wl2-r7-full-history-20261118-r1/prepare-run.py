from pathlib import Path
import json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
paths=subprocess.check_output(['git','diff','--name-only','--','apps/desktop/tauri/src/infrastructure'],cwd=CODE).decode().splitlines()
paths+=subprocess.check_output(['git','ls-files','--others','--exclude-standard','apps/desktop/tauri/src/infrastructure'],cwd=CODE).decode().splitlines()
assert len(paths)==19 and len(set(paths))==19
assert all(p.startswith(('apps/desktop/tauri/src/infrastructure/windows_client_originals024/','apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024/')) or p=='apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024.rs' for p in paths)
rustfmt=Path('D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustfmt.exe')
subprocess.run([str(rustfmt),'--edition','2021','--config','skip_children=true',*paths],cwd=CODE,check=True)
(ROOT/'sources.json').write_text(json.dumps(paths,indent=2),encoding='utf-8')
text=(ROOT.parent/'wl2-r7-history-primitives-20261118-r1/run-product.ps1').read_text(encoding='utf-8').replace('wl2_r7_history_primitives_native_','wl2_r7_full_history_native_')
(ROOT/'run-product.ps1').write_text(text,encoding='utf-8',newline='\n')
(ROOT/'inputs.json').write_text(json.dumps(dict(codeBefore=subprocess.check_output(['git','rev-parse','HEAD'],cwd=CODE).decode().strip(),mainInput=subprocess.check_output(['git','rev-parse','origin/main'],cwd=CODE).decode().strip(),docsInput=subprocess.check_output(['git','rev-parse','origin/docs/governance-main'],cwd=CODE).decode().strip(),authority='TEAM29/IMP164/r7',expectedChangedSources=19),indent=2),encoding='utf-8')
print('formatted 19 actual Windows exclusive source inputs and prepared exact snapshot build')

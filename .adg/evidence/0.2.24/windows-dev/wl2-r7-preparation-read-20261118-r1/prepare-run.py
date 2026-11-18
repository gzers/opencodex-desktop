from pathlib import Path
import json,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
base='apps/desktop/tauri/src/infrastructure/windows_client_originals024/'
files=[base+p for p in ['v3/journal/preparation.rs','v3/journal/preparation/read.rs','v3/journal/preparation_tests.rs','v3/journal/preparation/read_tests.rs','v3/capture/preparation/v2.rs','v3/capture/preparation.rs']]
rustfmt='D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustfmt.exe'
subprocess.run([rustfmt,'--edition','2021','--config','skip_children=true',*files],cwd=CODE,check=True)
(ROOT/'sources.json').write_text(json.dumps(files,indent=2),encoding='utf-8')
text=(ROOT.parent/'wl2-r7-private-native-20261118-r1/run-product.ps1').read_text(encoding='utf-8').replace('wl2_r7_private_','wl2_r7_preparation_read_native_')
(ROOT/'run-product.ps1').write_text(text,encoding='utf-8',newline=chr(10))
(ROOT/'inputs.json').write_text(json.dumps(dict(codeBefore=subprocess.check_output(['git','rev-parse','HEAD'],cwd=CODE).decode().strip(),mainInput=subprocess.check_output(['git','rev-parse','origin/main'],cwd=CODE).decode().strip(),docsInput=subprocess.check_output(['git','rev-parse','origin/docs/governance-main'],cwd=CODE).decode().strip(),authority='TEAM33/IMP171/r7 unchanged'),indent=2),encoding='utf-8')
print('6 original Windows journal preparation read/mapping/test files prepared')

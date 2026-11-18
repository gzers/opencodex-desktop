from pathlib import Path
import json,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
base='apps/desktop/tauri/src/infrastructure/windows_client_originals024/'
files=[base+p for p in ['tree/anchored.rs','tree/anchored/before_resume.rs','tree/anchored/before_resume/membership.rs','tree/anchored/before_resume/membership/tests.rs','tree/anchored/capture/abort_return/released.rs','v3/capture/preparation/v2.rs','v3/capture/preparation/v2/tests.rs']]
rustfmt='D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustfmt.exe'
subprocess.run([rustfmt,'--edition','2021','--config','skip_children=true',*files],cwd=CODE,check=True)
(ROOT/'sources.json').write_text(json.dumps(files,indent=2),encoding='utf-8')
text=(ROOT.parent/'wl2-r7-directory-read-20261118-r1/run-product.ps1').read_text(encoding='utf-8').replace('wl2_r7_directory_read_native_','wl2_r7_tree_membership_native_')
(ROOT/'run-product.ps1').write_text(text,encoding='utf-8',newline=chr(10))
(ROOT/'inputs.json').write_text(json.dumps(dict(codeBefore=subprocess.check_output(['git','rev-parse','HEAD'],cwd=CODE).decode().strip(),mainInput=subprocess.check_output(['git','rev-parse','origin/main'],cwd=CODE).decode().strip(),docsInput=subprocess.check_output(['git','rev-parse','origin/docs/governance-main'],cwd=CODE).decode().strip(),authority='TEAM33/IMP171/r7; latestfetchTLSfailed no unseenrevisionclaimed'),indent=2),encoding='utf-8')
print('7 Windows original tree/V2 membership/retainedcapture locator sources prepared')

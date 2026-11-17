from pathlib import Path
import json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
base='apps/desktop/tauri/src/infrastructure/'
paths=[base+'windows_client_originals024/recovery_worker.rs',base+'windows_client_originals024/recovery_worker/canonical_trait.rs',base+'windows_client_originals024/recovery_worker/canonical_trait/tests.rs',base+'windows_official_lifecycle024/actions/recovery.rs',base+'windows_official_lifecycle024/actions/recovery/driver.rs']
assert all((CODE/p).exists() for p in paths)
rustfmt='D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustfmt.exe'
subprocess.run([rustfmt,'--edition','2021','--config','skip_children=true',*paths],cwd=CODE,check=True)
(ROOT/'sources.json').write_text(json.dumps(paths,indent=2),encoding='utf-8')
text=(ROOT.parent/'wl2-r7-full-history-20261118-r1/run-product.ps1').read_text(encoding='utf-8').replace('wl2_r7_full_history_native_','wl2_r7_canonical_trait_')
(ROOT/'run-product.ps1').write_text(text,encoding='utf-8',newline='\n')
(ROOT/'inputs.json').write_text(json.dumps(dict(codeBefore=subprocess.check_output(['git','rev-parse','HEAD'],cwd=CODE).decode().strip(),mainInput='fb755b8ea63d8a3fc9146dc4db491931919905b0',docsInput=subprocess.check_output(['git','rev-parse','origin/docs/governance-main'],cwd=CODE).decode().strip(),authority='TEAM30/IMP166/r7'),indent=2),encoding='utf-8')
print('5 original Windows source paths formatted and test snapshot prepared')

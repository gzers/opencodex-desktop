from pathlib import Path
import json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
base='apps/desktop/tauri/src/infrastructure/windows_owner_host024/'
paths=[base+'mod.rs',base+'recovery_control.rs',base+'recovery_control/wire.rs',base+'recovery_control/wire_tests.rs']
rustfmt='D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustfmt.exe'
subprocess.run([rustfmt,'--edition','2021','--config','skip_children=true',*paths],cwd=CODE,check=True)
(ROOT/'sources.json').write_text(json.dumps(paths,indent=2),encoding='utf-8')
text=(ROOT.parent/'wl2-r7-canonical-trait-20261118-r1/run-product.ps1').read_text(encoding='utf-8').replace('wl2_r7_canonical_trait_','wl2_r7_private_wire_v2_')
(ROOT/'run-product.ps1').write_text(text,encoding='utf-8',newline='\n')
(ROOT/'inputs.json').write_text(json.dumps(dict(codeBefore=subprocess.check_output(['git','rev-parse','HEAD'],cwd=CODE).decode().strip(),mainInput='4ea82e8a61fe00c1f76334db5e39654f49f4342e',docsInput=subprocess.check_output(['git','rev-parse','origin/docs/governance-main'],cwd=CODE).decode().strip(),authority='TEAM31/IMP168/r7'),indent=2),encoding='utf-8')
print('4 Windows host strictV2 DATA source snapshots prepared; native server/client notyetimplemented')

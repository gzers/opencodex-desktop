from pathlib import Path
import json,re,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
base='apps/desktop/tauri/src/'
paths=[base+'bin/ocxd.rs']+[base+'infrastructure/windows_owner_host024/'+p for p in ['existing_qualification.rs','empty_host.rs','recovery_control.rs','recovery_control/wire.rs','recovery_control/wire_tests.rs','recovery_control/server.rs','recovery_control/client.rs']]
rustfmt='D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustfmt.exe'
subprocess.run([rustfmt,'--edition','2021','--config','skip_children=true',*paths],cwd=CODE,check=True)
(ROOT/'sources.json').write_text(json.dumps(paths,indent=2),encoding='utf-8')
text=(ROOT.parent/'wl2-r7-private-wire-20261118-r1/run-product.ps1').read_text(encoding='utf-8').replace('wl2_r7_private_wire_v2_','wl2_r7_private_')
(ROOT/'run-product.ps1').write_text(text,encoding='utf-8',newline='\n')
binary,n=re.subn(r'^\$arguments=@\(.*\)$',"$arguments=@('build','--locked','--offline','--config','profile.dev.package.opencodex-desktop.debug=0','--config','profile.dev.package.opencodex-desktop.incremental=false','--config','profile.dev.package.opencodex-desktop.codegen-units=16','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--bin','ocxd','-j','1')",text,flags=re.M);assert n==1
binary=binary.replace("[string]$Run='product01'","[string]$Run='binary02'")
(ROOT/'run-binary.ps1').write_text(binary,encoding='utf-8',newline='\n')
print('8 Windows internal host/client/server/dispatch sources, full MSVC library tests and actualocxd binary build prepared')

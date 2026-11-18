from pathlib import Path
import json,re,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
prefix='apps/desktop/tauri/src/infrastructure/'
files=[prefix+'windows_official_runtime024/paths/preparation.rs']+[prefix+'windows_client_originals024/'+p for p in ['v3/capture/preparation.rs','v3/capture/preparation/v2.rs','v3/capture/preparation/directory_read.rs','v3/capture/preparation/v2/tests.rs','v3/capture/preparation/v2/tests/directory_read_tests.rs']]
rustfmt='D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustfmt.exe'
subprocess.run([rustfmt,'--edition','2021','--config','skip_children=true',*files],cwd=CODE,check=True)
(ROOT/'sources.json').write_text(json.dumps(files,indent=2),encoding='utf-8')
text=(ROOT.parent/'wl2-r7-preparation-read-20261118-r1/run-product.ps1').read_text(encoding='utf-8').replace('wl2_r7_preparation_read_native_','wl2_r7_directory_read_native_')
(ROOT/'run-product.ps1').write_text(text,encoding='utf-8',newline=chr(10))
backend,count=re.subn(r'^\$arguments=@\(.*\)$',"$arguments=@('check','--locked','--offline','--manifest-path','test/windows024-native/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1')",text,flags=re.M);assert count==1
backend=backend.replace("[string]$Run='product01'","[string]$Run='backend02'").replace("'test/windows024-native/target/w1c-product'","'test/windows024-native/target/wl2-primitives'")
(ROOT/'run-backend.ps1').write_text(backend,encoding='utf-8',newline=chr(10))
(ROOT/'inputs.json').write_text(json.dumps(dict(codeBefore=subprocess.check_output(['git','rev-parse','HEAD'],cwd=CODE).decode().strip(),mainInput=subprocess.check_output(['git','rev-parse','origin/main'],cwd=CODE).decode().strip(),docsInput=subprocess.check_output(['git','rev-parse','origin/docs/governance-main'],cwd=CODE).decode().strip(),authority='TEAM33/IMP171/r7 unchanged; latestfetchTLSfailed no unseenrevisionconsumed'),indent=2),encoding='utf-8')
print('6 actual Windows source inputs formatted, native directory tests and separate backend compile prepared')

from pathlib import Path
import json,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
base='apps/desktop/tauri/src/infrastructure/windows_client_originals024/'
files=[base+p for p in ['before_resume_record_v2.rs','before_resume_record_v2/tests.rs','source.rs','adapter/before_resume.rs','v3/capture/preparation.rs','v3/capture/preparation/bound.rs','v3/capture/preparation/v2/tests.rs','v3/capture/preparation/v2/tests/binding_tests.rs']]
rustfmt='D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustfmt.exe'
subprocess.run([rustfmt,'--edition','2021','--config','skip_children=true',*files],cwd=CODE,check=True)
(ROOT/'sources.json').write_text(json.dumps(files,indent=2),encoding='utf-8')
text=(ROOT.parent/'wl2-r7-tree-membership-20261118-r1/run-product.ps1').read_text(encoding='utf-8').replace('wl2_r7_tree_membership_native_','wl2_r7_bound_preparation_')
(ROOT/'run-product.ps1').write_text(text,encoding='utf-8',newline=chr(10))
git=lambda *args:subprocess.check_output(['git',*args],cwd=CODE).decode().strip()
(ROOT/'inputs.json').write_text(json.dumps(dict(codeBefore=git('rev-parse','HEAD'),mainActuallyBuiltAndConsumed='88b8fc33d5ec21b54c494cdb529fd68026b3f8b9',docsActuallyConsumed='4a09aa011c18d2880ed63bfb211694d49ed84669',latestFetchedMainRef=git('rev-parse','origin/main'),latestFetchedDocsRef=git('rev-parse','origin/docs/governance-main'),latestContentsFullyConsumed=False,r7GitBlobUnchanged='f9d39b1600974a2e4353e347034f29ce775300c0',authority='TEAM33/IMP171/r7 last fullconsumption; latest main tree names noWindowsexclusiveoverlap, sharedchanges remainunread/unmerged'),indent=2),encoding='utf-8')
print('8 original Windows strictV2/Owner-bound preparation/source-context and test files prepared')

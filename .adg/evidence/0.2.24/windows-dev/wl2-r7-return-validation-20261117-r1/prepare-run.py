from pathlib import Path
import json,re
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
base='apps/desktop/tauri/src/infrastructure/'
files=[base+'windows_client_originals024/'+p for p in [
 'tree/anchored/capture/abort_return.rs','tree/anchored/capture/abort_return/released.rs','v3/capture.rs','adapter/before_resume.rs','recovery_worker.rs','recovery_worker/return_validation.rs','tree/anchored/before_resume/driver_worker_tests.rs','tree/anchored/before_resume/release_tests.rs','tree/anchored/before_resume/return_validation_tests.rs','recovery_worker/tests.rs','recovery_worker/tests/return_validation_tests.rs'
]]+[base+'windows_official_lifecycle024.rs',base+'windows_official_lifecycle024/actions/recovery.rs']
assert len(files)==13 and all((CODE/p).exists() for p in files)
(ROOT/'sources.json').write_text(json.dumps(files,indent=2),encoding='utf-8')
text=(ROOT.parent/'wl2-r7-v2-durability-20261117-r1/run-product.ps1').read_text(encoding='utf-8')
text=text.replace('wl2_r7_v2_storage_native_','wl2_r7_return_validation_native_')
text,n=re.subn(r'^\$relative=@\(.*\)$',"$relative=@(Get-Content -LiteralPath (Join-Path $EvidenceRoot 'sources.json') -Raw | ConvertFrom-Json)",text,flags=re.M);assert n==1
text=text.replace("[IO.File]::Create((Join-Path $runRoot 'stdout.log'))","[IO.File]::Open((Join-Path $runRoot 'stdout.log'),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)").replace("[IO.File]::Create((Join-Path $runRoot 'stderr.log'))","[IO.File]::Open((Join-Path $runRoot 'stderr.log'),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)")
(ROOT/'run-product.ps1').write_text(text,encoding='utf-8',newline='\n')
print('prepared exact 13 source snapshots and read-sharing raw log capture')

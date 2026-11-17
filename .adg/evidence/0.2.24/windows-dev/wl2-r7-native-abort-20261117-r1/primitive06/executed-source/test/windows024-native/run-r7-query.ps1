param([Parameter(Mandatory)][string]$Evidence,[ValidateSet('all','scan-tests','fixed-scan','collector-source','observer-build','abort')][string]$Mode='all',[string]$ScanRoot)
$ErrorActionPreference='Stop'
$evidencePath=[IO.Path]::GetFullPath($Evidence)
if(Test-Path -LiteralPath $evidencePath){throw 'fresh immutable evidence directory required'}
[void][IO.Directory]::CreateDirectory($evidencePath)
$repo=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
if((Get-Location).Path -cne $repo){throw 'run at exact code checkout root'}
$toolchain='D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain'
. "$PSScriptRoot/msvc-env.ps1" -Tools "$toolchain/msvc-w1c/msvc" -RustRoot $toolchain
$env:RUSTC="$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe"
$env:CARGO_TARGET_DIR="$PSScriptRoot/target/r7-query-proof"
$cargo="$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/cargo.exe"
$paths=@('test/windows024-native/run-r7-query.ps1',
 'apps/desktop/tauri/src/infrastructure/hash.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/bounded/query.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/before_resume_record.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/r7_readonly.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/r7_readonly/objects.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/tree/streams.rs',
 'test/windows024-native/r7-scene-locators.json',
 'apps/desktop/tauri/src/modules/runtime/windows_lifecycle024_contract.rs',
 'apps/desktop/tauri/src/modules/runtime/program_image.rs',
 'apps/desktop/tauri/src/modules/runtime/program_image_windows_tests.rs',
 'apps/desktop/tauri/src/modules/runtime/program_layout.rs',
 'apps/desktop/tauri/src/modules/backup/safety.rs')
foreach($directory in @('test/windows024-native/r7-query-harness',
 'apps/desktop/tauri/src/infrastructure/windows_official_runtime024',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/before_resume_record')){
 $paths+=@(Get-ChildItem -LiteralPath $directory -Recurse -File|ForEach-Object {$_.FullName.Substring($repo.Length+1).Replace([char]92,[char]47)})
}
$source=@()
foreach($relative in $paths|Sort-Object -Unique){
 $item=Get-Item -LiteralPath $relative -Force
 if($item.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'source reparse refused'}
 $dest=Join-Path (Join-Path $evidencePath 'executed-source') $relative
 [void][IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($dest))
 [IO.File]::Copy($item.FullName,$dest,$false)
 $source+=@{path=$relative;bytes=$item.Length;sha256=(Get-FileHash -LiteralPath $dest -Algorithm SHA256).Hash.ToLowerInvariant()}
}
$arguments=@('test','--locked','--offline','--manifest-path','test/windows024-native/r7-query-harness/Cargo.toml','--target','x86_64-pc-windows-msvc','-j','1','r7_','--','--test-threads=1','--nocapture')
if($Mode -eq 'scan-tests'){$arguments=@('test','--locked','--offline','--manifest-path','test/windows024-native/r7-query-harness/Cargo.toml','--target','x86_64-pc-windows-msvc','-j','1','r7_image_','--','--test-threads=1','--nocapture')}
if($Mode -in @('fixed-scan','collector-source')){
 if([string]::IsNullOrEmpty($ScanRoot)){throw 'explicit copied image required'}
 $env:OCXD024_R7_FIXED_SCAN_ROOT=[IO.Path]::GetFullPath($ScanRoot)
 $arguments=@('test','--locked','--offline','--manifest-path','test/windows024-native/r7-query-harness/Cargo.toml','--target','x86_64-pc-windows-msvc','-j','1','r7_image_fixed_copy_stage_metrics_worker1_and_default4','--','--ignored','--test-threads=1','--nocapture')
 if($Mode -eq 'collector-source'){$arguments=@('test','--locked','--offline','--manifest-path','test/windows024-native/r7-query-harness/Cargo.toml','--target','x86_64-pc-windows-msvc','-j','1','r7_collector_full_source_native_manifest_current_and_full_inventory_are_bound','--','--ignored','--test-threads=1','--nocapture')}
}
if($Mode -eq 'observer-build'){$arguments=@('build','--locked','--offline','--manifest-path','test/windows024-native/r7-query-harness/Cargo.toml','--target','x86_64-pc-windows-msvc','-j','1','--bin','r7-scene-observer')}
if($Mode -eq 'abort'){$arguments=@('test','--locked','--offline','--manifest-path','test/windows024-native/r7-query-harness/Cargo.toml','--target','x86_64-pc-windows-msvc','-j','1','--test','abort','r7_abort_','--','--test-threads=1','--nocapture')}
@{head=(git rev-parse HEAD);source=$source;arguments=$arguments;rustc=(& $env:RUSTC --version);scope='actual shared query/backend and strict attachment; not Tauri product or ordinary marenben lifecycle acceptance';rootProduct=$false}|ConvertTo-Json -Depth 7|Set-Content -LiteralPath (Join-Path $evidencePath 'source.json') -Encoding utf8
$watch=[Diagnostics.Stopwatch]::StartNew()
& $cargo @arguments 1> (Join-Path $evidencePath 'stdout.log') 2> (Join-Path $evidencePath 'stderr.log')
$rc=$LASTEXITCODE
if($Mode -in @('fixed-scan','collector-source')){[Environment]::SetEnvironmentVariable('OCXD024_R7_FIXED_SCAN_ROOT',$null,'Process')}
@{raw=$rc;elapsedMs=$watch.ElapsedMilliseconds;arguments=$arguments;rootProduct=$false;ordinaryMarenbenAcceptance=$false;originalR2Touched=$false;restored=$false;currentR2Forced=$false;syntheticForcedPathExercised=($Mode -eq 'abort');normalStop=$false;closed=$false;publicStartOpen=$false}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $evidencePath 'result.json') -Encoding utf8
exit $rc

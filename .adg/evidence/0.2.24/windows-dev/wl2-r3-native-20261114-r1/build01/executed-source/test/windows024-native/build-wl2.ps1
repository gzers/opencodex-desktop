param([Parameter(Mandatory)][string]$Evidence,[string]$Mode='build')
$ErrorActionPreference='Stop'
$toolchain='D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain'
. "$PSScriptRoot/msvc-env.ps1" -Tools "$toolchain/msvc-w1c/msvc" -RustRoot $toolchain
$env:RUSTC="$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe"
$env:RUSTDOC="$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustdoc.exe"
$env:CARGO_TARGET_DIR="$PSScriptRoot/target/w1c-product"
$cargo="$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/cargo.exe"
$arguments=@('build','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--bins','-j','1')
if($Mode -eq 'test'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','wl2_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'r3'){$arguments=@('test','--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--lib','-j','1','wl2_r3_','--','--test-threads=1','--nocapture')}
if($Mode -eq 'native'){
 $env:CARGO_TARGET_DIR="$PSScriptRoot/target/wl2-primitives"
 $arguments=@('test','--locked','--offline','--manifest-path','test/windows024-native/Cargo.toml','--target','x86_64-pc-windows-msvc','-j','1','wl2_','--','--test-threads=1','--nocapture')
}
if($Mode -eq 'native-r3'){
 $env:CARGO_TARGET_DIR="$PSScriptRoot/target/wl2-primitives"
 $arguments=@('test','--locked','--offline','--manifest-path','test/windows024-native/Cargo.toml','--target','x86_64-pc-windows-msvc','-j','1','wl2_r3_','--','--test-threads=1','--nocapture')
}
$head=git rev-parse HEAD
$diff=git diff --binary
@{head=$head;diff=$diff;arguments=$arguments;rustc=(& $env:RUSTC --version);time=[datetime]::Now.ToString('o')}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath "$Evidence/source.json" -Encoding utf8
$files=@(git ls-files --cached --others --exclude-standard -- 'apps/desktop/tauri/src/infrastructure/windows*' 'apps/desktop/tauri/src/infrastructure/mod.rs' 'test/windows024-native')
$manifest=@()
foreach($relative in $files){
 $item=Get-Item -LiteralPath $relative
 if($item.PSIsContainer){throw 'source manifest directory entry refused'}
 $target=Join-Path (Join-Path $Evidence 'executed-source') $relative
 [void][IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($target))
 [IO.File]::Copy($item.FullName,$target,$false)
 $manifest+=@{path=$relative;bytes=$item.Length;sha256=(Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant()}
}
ConvertTo-Json -InputObject @($manifest) -Depth 5|Set-Content -LiteralPath "$Evidence/source-manifest.json" -Encoding utf8
$watch=[Diagnostics.Stopwatch]::StartNew()
& $cargo @arguments 1> "$Evidence/stdout.log" 2> "$Evidence/stderr.log"
$rc=$LASTEXITCODE
@{exitCode=$rc;elapsedMs=$watch.ElapsedMilliseconds;arguments=$arguments;profile='debug';accepted=$false}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath "$Evidence/result.json" -Encoding utf8
exit $rc

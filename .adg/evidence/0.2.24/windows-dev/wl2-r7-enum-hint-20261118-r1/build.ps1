param([string]$Mode)
$ErrorActionPreference = 'Stop'
$toolchain = 'D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain'
. './test/windows024-native/msvc-env.ps1' -Tools "$toolchain/msvc-w1c/msvc" -RustRoot $toolchain
$rustbin = "$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin"
$env:RUSTC = "$rustbin/rustc.exe"
$env:RUSTDOC = "$rustbin/rustdoc.exe"
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'test/windows024-native/target/w1c-product'
python "$PSScriptRoot/build.py" "$rustbin/cargo.exe" $Mode 'test/windows024-native/target/r7-preparation-scan-trace-20261118-r1'
exit $LASTEXITCODE

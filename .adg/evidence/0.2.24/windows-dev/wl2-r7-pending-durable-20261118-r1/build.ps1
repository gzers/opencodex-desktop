param([Parameter(Mandatory)][string]$Run, [Parameter(Mandatory)][string]$Mode)
$ErrorActionPreference = 'Stop'
$toolchain = 'D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain'
. './test/windows024-native/msvc-env.ps1' -Tools "$toolchain/msvc-w1c/msvc" -RustRoot $toolchain
$rustbin = "$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin"
$env:RUSTC = "$rustbin/rustc.exe"
$env:RUSTDOC = "$rustbin/rustdoc.exe"
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'test/windows024-native/target/w1c-product'
python "$Run/build.py" $Mode "$rustbin/cargo.exe" $env:CARGO_TARGET_DIR
exit $LASTEXITCODE

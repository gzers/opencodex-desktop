param([Parameter(Mandatory)][string]$CodeRoot,[Parameter(Mandatory)][string]$EvidenceRoot,[string]$Run='product01',[string]$Filter='wl2_r7_private_wire_v2_')
$ErrorActionPreference='Stop'
$toolchain='D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain'
. (Join-Path $CodeRoot 'test/windows024-native/msvc-env.ps1') -Tools "$toolchain/msvc-w1c/msvc" -RustRoot $toolchain
$env:RUSTC="$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe"
$env:RUSTDOC="$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustdoc.exe"
$env:CARGO_TARGET_DIR=Join-Path $CodeRoot 'test/windows024-native/target/w1c-product'
$cargo="$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/cargo.exe"
$runRoot=Join-Path $EvidenceRoot $Run
if(Test-Path -LiteralPath $runRoot){throw 'refuse overwrite existing run'}
$null=New-Item -ItemType Directory -Path $runRoot
$utf8=[Text.UTF8Encoding]::new($false)
$arguments=@('test','--locked','--offline','--config','profile.test.package.opencodex-desktop.debug=0','--config','profile.test.package.opencodex-desktop.incremental=false','--config','profile.test.package.opencodex-desktop.codegen-units=16','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','1',$Filter,'--','--test-threads=1','--nocapture')
$relative=@(Get-Content -LiteralPath (Join-Path $EvidenceRoot 'sources.json') -Raw | ConvertFrom-Json)
$sources=@(foreach($file in $relative){
  $from=Join-Path $CodeRoot $file
  $to=Join-Path (Join-Path $runRoot 'executed-source') $file
  $null=New-Item -ItemType Directory -Path (Split-Path $to) -Force
  [IO.File]::Copy($from,$to,$false)
  @{path=$file;bytes=(Get-Item -LiteralPath $from).Length;sha256=(Get-FileHash -LiteralPath $from -Algorithm SHA256).Hash.ToLowerInvariant()}
})
[IO.File]::WriteAllText((Join-Path $runRoot 'source.json'),(@{arguments=$arguments;sources=$sources;currentR2Touched=$false;codeRoot=$CodeRoot}|ConvertTo-Json -Depth 10),$utf8)
$info=[Diagnostics.ProcessStartInfo]::new()
$info.FileName=$cargo
$info.WorkingDirectory=$CodeRoot
$info.UseShellExecute=$false
$info.CreateNoWindow=$true
$info.RedirectStandardOutput=$true
$info.RedirectStandardError=$true
foreach($argument in $arguments){$info.ArgumentList.Add($argument)}
$proc=[Diagnostics.Process]::new()
$proc.StartInfo=$info
$out=[IO.File]::Open((Join-Path $runRoot 'stdout.log'),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
$err=[IO.File]::Open((Join-Path $runRoot 'stderr.log'),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
$watch=[Diagnostics.Stopwatch]::StartNew()
try{
  if(!$proc.Start()){throw 'process start failed'}
  $outCopy=$proc.StandardOutput.BaseStream.CopyToAsync($out)
  $errCopy=$proc.StandardError.BaseStream.CopyToAsync($err)
  $proc.WaitForExit()
  $outCopy.GetAwaiter().GetResult()
  $errCopy.GetAwaiter().GetResult()
  $watch.Stop()
  $raw=$proc.ExitCode
  [IO.File]::WriteAllText((Join-Path $runRoot 'result.json'),(@{arguments=$arguments;raw=$raw;elapsedMs=$watch.ElapsedMilliseconds;currentR2Touched=$false;productionAccepted=$false}|ConvertTo-Json -Depth 10),$utf8)
}finally{$out.Dispose();$err.Dispose();$proc.Dispose()}
Write-Output "$Run raw=$raw elapsedMs=$($watch.ElapsedMilliseconds)"
exit $raw

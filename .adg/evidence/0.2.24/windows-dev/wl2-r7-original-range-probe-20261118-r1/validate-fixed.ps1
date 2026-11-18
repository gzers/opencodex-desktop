param([Parameter(Mandatory)][string]$CodeRoot,[Parameter(Mandatory)][string]$EvidenceRoot)
$ErrorActionPreference='Stop'
$toolchain='D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain'
. (Join-Path $CodeRoot 'test/windows024-native/msvc-env.ps1') -Tools "$toolchain/msvc-w1c/msvc" -RustRoot $toolchain
$env:RUSTC="$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe"
$env:RUSTDOC="$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustdoc.exe"
$env:CARGO_TARGET_DIR=Join-Path $CodeRoot 'test/windows024-native/target/w1c-product'
$cargo="$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/cargo.exe"
$runRoot=Join-Path $EvidenceRoot 'fixed'
if (!(Test-Path -LiteralPath $runRoot -PathType Container)) { throw "Compile output missing" }
$utf8=[Text.UTF8Encoding]::new($false)
function SaveJson([string]$Name,$Data) {
    $path=Join-Path $runRoot $Name
    if(Test-Path -LiteralPath $path){throw "Refuse overwrite $path"}
    [IO.File]::WriteAllText($path,($Data|ConvertTo-Json -Depth 12),$utf8)
}
function Run-Raw([string]$Name,[string]$Program,[string[]]$Arguments) {
    $info=[Diagnostics.ProcessStartInfo]::new()
    $info.FileName=$Program
    $info.WorkingDirectory=$CodeRoot
    $info.UseShellExecute=$false
    $info.CreateNoWindow=$true
    $info.RedirectStandardOutput=$true
    $info.RedirectStandardError=$true
    foreach($argument in $Arguments){$info.ArgumentList.Add($argument)}
    $proc=[Diagnostics.Process]::new()
    $proc.StartInfo=$info
    $out=[IO.File]::Open((Join-Path $runRoot "$Name.stdout.log"),'CreateNew','Write','Read')
    $err=[IO.File]::Open((Join-Path $runRoot "$Name.stderr.log"),'CreateNew','Write','Read')
    $watch=[Diagnostics.Stopwatch]::StartNew()
    try {
        if(!$proc.Start()){throw 'process start failed'}
        $outCopy=$proc.StandardOutput.BaseStream.CopyToAsync($out)
        $errCopy=$proc.StandardError.BaseStream.CopyToAsync($err)
        $proc.WaitForExit()
        $outCopy.GetAwaiter().GetResult()
        $errCopy.GetAwaiter().GetResult()
        $watch.Stop()
        $raw=$proc.ExitCode
        SaveJson "$Name.result.json" @{program=$Program;arguments=$Arguments;raw=$raw;elapsedMs=$watch.ElapsedMilliseconds}
    }finally{$out.Dispose();$err.Dispose();$proc.Dispose()}
    Write-Host "$Name raw=$raw elapsedMs=$($watch.ElapsedMilliseconds)"
    return $raw
}
$compile=Get-Content (Join-Path $runRoot 'compile.result.json') -Raw|ConvertFrom-Json
if($compile.raw -ne 0){throw "Compile raw $($compile.raw)"}
$artifacts=@(Get-Content (Join-Path $runRoot 'compile.stdout.log')|ForEach-Object {$_|ConvertFrom-Json}|Where-Object {$_.reason -ceq 'compiler-artifact' -and $_.target.name -ceq 'opencodex_desktop_lib' -and $_.profile.test -eq $true -and $_.executable})
if($artifacts.Count -ne 1){throw "Found $($artifacts.Count) test executables"}
$image=$artifacts[0].executable
SaveJson 'image.json' @{path=$image;bytes=(Get-Item -LiteralPath $image).Length;sha256=(Get-FileHash -LiteralPath $image).Hash.ToLowerInvariant()}
$filters=@('windows_official_lifecycle024::start_worker::tests','windows_owner_host024::recovery_control','wl2_r7_isolation_','--test-threads=1','--nocapture')
$tests=Run-Raw 'focused' $image $filters
$normal=Run-Raw 'normal-check' $cargo @('check','--locked','--offline','--config','profile.dev.package.opencodex-desktop.debug=0','--config','profile.dev.package.opencodex-desktop.incremental=false','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','4')
if($tests -ne 0){exit $tests}
exit $normal
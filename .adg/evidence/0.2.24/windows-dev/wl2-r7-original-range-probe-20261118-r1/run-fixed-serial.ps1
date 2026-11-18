param([Parameter(Mandatory)][string]$CodeRoot,[Parameter(Mandatory)][string]$EvidenceRoot)
$ErrorActionPreference='Stop'
$toolchain='D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain'
. (Join-Path $CodeRoot 'test/windows024-native/msvc-env.ps1') -Tools "$toolchain/msvc-w1c/msvc" -RustRoot $toolchain
$env:RUSTC="$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustc.exe"
$env:RUSTDOC="$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/rustdoc.exe"
$env:CARGO_TARGET_DIR=Join-Path $CodeRoot 'test/windows024-native/target/w1c-product'
$cargo="$toolchain/rustup/toolchains/stable-x86_64-pc-windows-msvc/bin/cargo.exe"
$runRoot=Join-Path $EvidenceRoot 'fixed-serial'
$null=New-Item -ItemType Directory -Path $runRoot
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
$paths=@(git -C $CodeRoot diff --name-only --no-renames 2>$null)
if ($paths.Count -ne 7) { throw "Unexpected source count $($paths.Count)" }
$sources=@(foreach ($file in $paths) {
    $from=Join-Path $CodeRoot $file
    $to=Join-Path (Join-Path $runRoot 'executed-source') $file
    $null=New-Item -ItemType Directory -Path (Split-Path $to) -Force
    [IO.File]::Copy($from,$to,$false)
    @{path=$file;bytes=(Get-Item -LiteralPath $from).Length;sha256=(Get-FileHash -LiteralPath $from).Hash.ToLowerInvariant()}
})
SaveJson 'sources.json' @{baseline=(git -C $CodeRoot rev-parse HEAD);sources=$sources}
$baseArgs=@('--locked','--offline','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','4')
$buildArgs=@('test','--no-run','--message-format=json','--config','profile.test.package.opencodex-desktop.debug=0','--config','profile.test.package.opencodex-desktop.incremental=false','--config','profile.test.package.opencodex-desktop.codegen-units=16')+$baseArgs
$compiled=Run-Raw 'compile' $cargo $buildArgs
if($compiled -ne 0){exit $compiled}
$artifacts=@(Get-Content (Join-Path $runRoot 'compile.stdout.log')|ForEach-Object {$_|ConvertFrom-Json}|Where-Object {$_.reason -ceq 'compiler-artifact' -and $_.target.name -ceq 'opencodex_desktop_lib' -and $_.profile.test -eq $true -and $_.executable})
if($artifacts.Count -ne 1){throw "Found $($artifacts.Count) test executables"}
$image=$artifacts[0].executable
SaveJson 'image.json' @{path=$image;bytes=(Get-Item -LiteralPath $image).Length;sha256=(Get-FileHash -LiteralPath $image).Hash.ToLowerInvariant()}
$filters=@('windows_official_lifecycle024::start_worker::tests','windows_owner_host024::recovery_control','wl2_r7_isolation_','--test-threads=1','--nocapture')
$tests=Run-Raw 'focused' $image $filters
$normal=Run-Raw 'normal-check' $cargo @('check','--locked','--offline','--config','profile.dev.package.opencodex-desktop.debug=0','--config','profile.dev.package.opencodex-desktop.incremental=false','--manifest-path','apps/desktop/tauri/Cargo.toml','--target','x86_64-pc-windows-msvc','--features','custom-protocol','--lib','-j','4')
if($tests -ne 0){exit $tests}
exit $normal
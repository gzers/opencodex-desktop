$ErrorActionPreference = 'Stop'
$codeRoot = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$evidence = 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-BASELINE-KNOWN-FOLDERS'
$runtime = Join-Path $codeRoot 'test/runtime/baseline-known-folder-readonly'
$probe = 'C:\Users\15119\.codex\tmp\ocx-0110-known-folder-probe.ps1'
$pwsh = 'C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\native\powershell\pwsh.exe'
if (Test-Path -LiteralPath $runtime) { throw 'Do not reuse a prior boundary experiment' }
if (![IO.Path]::GetFullPath($runtime).StartsWith((Join-Path $codeRoot 'test/runtime') + '\',
    [StringComparison]::OrdinalIgnoreCase)) { throw 'Experiment root escaped task scope' }
$queryHome = Join-Path $runtime 'home'
$temp = Join-Path $runtime 'temp'
New-Item -ItemType Directory -Path $evidence, $queryHome, $temp,
    (Join-Path $queryHome 'AppData/Roaming'), (Join-Path $queryHome 'AppData/Local') -Force | Out-Null
$records = @()
foreach ($mode in @('control', 'environment-override')) {
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $pwsh
    $info.WorkingDirectory = $runtime
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    foreach ($argument in @('-NoProfile','-NonInteractive','-File',$probe,
        '-FixtureRoot',$runtime,'-ProbeMode',$mode)) { [void]$info.ArgumentList.Add($argument) }
    $info.Environment.Clear()
    $info.Environment['SystemRoot'] = 'C:\Windows'
    $info.Environment['WINDIR'] = 'C:\Windows'
    $info.Environment['COMSPEC'] = 'C:\Windows\System32\cmd.exe'
    $info.Environment['PATH'] = 'C:\Windows\System32;C:\Windows'
    $info.Environment['TEMP'] = $temp
    $info.Environment['TMP'] = $temp
    $info.Environment['DOTNET_CLI_TELEMETRY_OPTOUT'] = '1'
    foreach ($key in @('HOME','USERPROFILE','APPDATA','LOCALAPPDATA')) {
        $value = [Environment]::GetEnvironmentVariable($key)
        if ($value) { $info.Environment[$key] = $value }
    }
    if ($mode -eq 'environment-override') {
        $info.Environment['HOME'] = $queryHome
        $info.Environment['USERPROFILE'] = $queryHome
        $info.Environment['APPDATA'] = Join-Path $queryHome 'AppData/Roaming'
        $info.Environment['LOCALAPPDATA'] = Join-Path $queryHome 'AppData/Local'
        $info.Environment['OPENCODEX_SANDBOX'] = '1'
        $info.Environment['OPENCODEX_SANDBOX_ROOT'] = $runtime
    }
    $process = $null
    try {
        $process = [Diagnostics.Process]::Start($info)
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        if (!$process.WaitForExit(30000)) { throw 'Read-only native probe exceeded bounded observation' }
        if (!$stdout.Wait(5000) -or !$stderr.Wait(5000)) { throw 'Probe output did not close' }
        $stderr.Result | Set-Content -LiteralPath (Join-Path $evidence "$mode-stderr.log") -Encoding utf8
        $stdout.Result | Set-Content -LiteralPath (Join-Path $evidence "$mode-observation.json") -Encoding utf8
        if ($process.ExitCode -ne 0) { throw 'Native Known Folder probe failed' }
        $records += ($stdout.Result | ConvertFrom-Json -DateKind String)
    } finally {
        if ($process) {
            if (!$process.HasExited) { $process.Kill($true); [void]$process.WaitForExit(5000) }
            $process.Dispose()
        }
    }
}
$comparison = @()
foreach ($name in @('profile','roaming','local')) {
    $before = $records[0].rows | Where-Object folder -eq $name
    $after = $records[1].rows | Where-Object folder -eq $name
    $comparison += [ordered]@{
        folder=$name
        sameNativePathBeforeAndAfterOverride=($before.nativePathSha256 -ceq $after.nativePathSha256)
        nativePathInsideFixture=$after.nativePathInsideFixture
        overriddenEnvironmentInsideFixture=$after.environmentPathInsideFixture
        overriddenEnvironmentMatchesNativePath=$after.nativePathEqualsEnvironment
    }
}
[ordered]@{
    recordedAtUtc=[DateTime]::UtcNow.ToString('o')
    fixtureRoot=$runtime
    actualApiExperiment=$true
    comparison=$comparison
    fullBaselineIsolationProved=$false
    baselineApplicationLaunched=$false
    settingsOrProfileChanged=$false
    rawRealUserPathsPublished=$false
    sourceScope='native API boundary; old official baseline source separately requests dirs6/home_dir'
} | ConvertTo-Json -Depth 7 |
    Set-Content -LiteralPath (Join-Path $evidence 'known-folder-comparison.json') -Encoding utf8
$comparison | ConvertTo-Json -Depth 5

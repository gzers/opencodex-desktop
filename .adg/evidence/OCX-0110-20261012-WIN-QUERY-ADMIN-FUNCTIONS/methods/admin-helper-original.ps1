$ErrorActionPreference = 'Stop'
$codeRoot = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$evidence = 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-QUERY-ADMIN-FUNCTIONS'
$testExe = Join-Path $codeRoot 'test/software/query-job-source/fix-final/windows-backend/opencodex_desktop_lib-713299640ffef8db.exe'
$testSha = '097421b7971853e5ef38667100de6eee0b96366adfee5221dfb86ca640c4e23c'
$node = 'C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe'
$runtime = Join-Path $codeRoot 'test/runtime/query-tree-admin-functions-485'
$queryHome = Join-Path $runtime 'home'
$temp = Join-Path $runtime 'temp'
. (Join-Path $codeRoot 'test/smoke/windows-process-token.ps1')
$readerToken = Get-WindowsHarnessToken
Assert-WindowsProcessMode -Observation $readerToken -Mode administrator
if ((Get-FileHash -LiteralPath $testExe -Algorithm SHA256).Hash.ToLowerInvariant() -cne $testSha) {
    throw 'Exact CI Rust test EXE hash differs'
}
if (Test-Path -LiteralPath $runtime) { throw 'Do not reuse a prior administrator fixture' }
if (![IO.Path]::GetFullPath($runtime).StartsWith((Join-Path $codeRoot 'test/runtime') + '\',
    [StringComparison]::OrdinalIgnoreCase)) { throw 'Fixture escaped task root' }
$desktopBefore = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'")
if ($desktopBefore.Count -ne 1 -or $desktopBefore[0].ProcessId -ne 38924) {
    throw 'Retained single application differs'
}
New-Item -ItemType Directory -Path $evidence, $queryHome, $temp -Force | Out-Null
$info = [Diagnostics.ProcessStartInfo]::new()
$info.FileName = $testExe
$info.WorkingDirectory = $runtime
$info.UseShellExecute = $false
$info.CreateNoWindow = $true
$info.RedirectStandardOutput = $true
$info.RedirectStandardError = $true
foreach ($argument in @('modules::about::remote::windows_tests::', '--test-threads=1', '--nocapture')) {
    [void]$info.ArgumentList.Add($argument)
}
$info.Environment.Clear()
$info.Environment['SystemRoot'] = 'C:\Windows'
$info.Environment['WINDIR'] = 'C:\Windows'
$info.Environment['COMSPEC'] = 'C:\Windows\System32\cmd.exe'
$info.Environment['PATH'] = (Split-Path -Parent $node) + ';C:\Windows\System32;C:\Windows'
$info.Environment['HOME'] = $queryHome
$info.Environment['USERPROFILE'] = $queryHome
$info.Environment['APPDATA'] = Join-Path $queryHome 'AppData/Roaming'
$info.Environment['LOCALAPPDATA'] = Join-Path $queryHome 'AppData/Local'
$info.Environment['TEMP'] = $temp
$info.Environment['TMP'] = $temp
$info.Environment['OPENCODEX_TEST_NODE'] = $node
$info.Environment['OPENCODEX_SANDBOX'] = '1'
$info.Environment['OPENCODEX_SANDBOX_ROOT'] = $runtime
$process = $null
$stdout = $null
$stderr = $null
$report = [ordered]@{
    startedAtUtc = [DateTime]::UtcNow.ToString('o')
    sourceCommit = '4852820b89f7f9df7d54bc623e94f3f55d884023'
    ciTestExecutableSha256 = $testSha
    testSourceBlob = '3dc48b4251cd8465f933bd59387e77dcfc8f339b'
    filter = 'modules::about::remote::windows_tests::'
    helperPid = $PID
    helperToken = $readerToken
    testProcessToken = $null
    ordinaryDesktopPid = 38924
    runtimeRoot = $runtime
    credentialsInherited = $false
    status = 'preparing'
    desktopApplicationLaunched = $false
    packageLifecycleScriptsExecuted = $false
    securitySettingsChanged = $false
    scope = 'actual administrator-token production Rust query process functions; not elevated desktop UI acceptance'
}
try {
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $process = [Diagnostics.Process]::Start($info)
    $stdout = $process.StandardOutput.ReadToEndAsync()
    $stderr = $process.StandardError.ReadToEndAsync()
    $report.testProcessToken = Get-WindowsProcessToken -Process $process -ExpectedExecutable $testExe
    Assert-WindowsProcessMode -Observation $report.testProcessToken -Mode administrator
    if (!$process.WaitForExit(45000)) { throw 'Fixed query regression exceeded bounded observation' }
    $report.elapsedMs = $clock.Elapsed.TotalMilliseconds
    $report.exitCode = $process.ExitCode
    $report.status = if ($process.ExitCode -eq 0) { 'recorded-admin-functions-passed' } else { 'recorded-admin-functions-failed' }
} catch {
    $report.status = 'admin-test-harness-failed'
    $report.error = $_.Exception.Message
    throw
} finally {
    if ($process) {
        if (!$process.HasExited) { $process.Kill($true); [void]$process.WaitForExit(5000) }
        if ($stdout -and $stdout.Wait(5000)) {
            $stdout.Result | Set-Content -LiteralPath (Join-Path $evidence 'admin-test-stdout.log') -Encoding utf8
        }
        if ($stderr -and $stderr.Wait(5000)) {
            $stderr.Result | Set-Content -LiteralPath (Join-Path $evidence 'admin-test-stderr.log') -Encoding utf8
        }
        $process.Dispose()
    }
    $desktopAfter = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'")
    $report.finishedAtUtc = [DateTime]::UtcNow.ToString('o')
    $report.desktopCountAfter = $desktopAfter.Count
    $report.sameDesktopLifecycleRetained = $desktopAfter.Count -eq 1 -and
        $desktopAfter[0].ProcessId -eq $desktopBefore[0].ProcessId -and
        $desktopAfter[0].CreationDate -eq $desktopBefore[0].CreationDate
    $report | ConvertTo-Json -Depth 9 |
        Set-Content -LiteralPath (Join-Path $evidence 'admin-function-test-result.json') -Encoding utf8
}
if ($report.exitCode -ne 0) { throw 'Recorded administrator query tests failed; preserve logs' }

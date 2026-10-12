$ErrorActionPreference = 'Stop'
$codeRoot = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$evidence = 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-QUERY-LIVE-NPM'
$fixture = Join-Path $codeRoot 'test/runtime/query-tree-real-npm-485'
$originalNode = 'C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe'
$npmCli = 'D:\Libraries\Nodejs\versions\22.12.0\node_modules\npm\bin\npm-cli.js'
$testExe = Join-Path $codeRoot 'test/software/query-job-source/fix-final/windows-backend/opencodex_desktop_lib-713299640ffef8db.exe'
$testSha = '097421b7971853e5ef38667100de6eee0b96366adfee5221dfb86ca640c4e23c'
$source = '4852820b89f7f9df7d54bc623e94f3f55d884023'
if ((Get-FileHash -LiteralPath $testExe -Algorithm SHA256).Hash.ToLowerInvariant() -cne $testSha) {
    throw 'Exact CI test executable identity differs'
}
if (Test-Path -LiteralPath $fixture) { throw 'Do not reuse a prior live query fixture' }
if (![IO.Path]::GetFullPath($fixture).StartsWith((Join-Path $codeRoot 'test/runtime') + '\',
    [StringComparison]::OrdinalIgnoreCase)) { throw 'Fixture escaped task runtime root' }
$npmPackage = Get-Content -LiteralPath 'D:\Libraries\Nodejs\versions\22.12.0\node_modules\npm\package.json' -Raw |
    ConvertFrom-Json
if ($npmPackage.name -ne 'npm' -or $npmPackage.version -ne '10.9.0') { throw 'Configured npm identity changed' }
foreach ($path in @($originalNode, $npmCli, $testExe)) {
    if (!(Test-Path -LiteralPath $path -PathType Leaf)) { throw 'Required pinned local tool missing' }
}
$queryHome = Join-Path $fixture 'home'
$bin = Join-Path $queryHome '.local/bin'
$temp = Join-Path $fixture 'temp'
New-Item -ItemType Directory -Path $evidence -Force | Out-Null
New-Item -ItemType Directory -Path $queryHome, $bin, $temp,
    (Join-Path $queryHome 'AppData/Roaming'), (Join-Path $queryHome 'AppData/Local') -Force | Out-Null
$node = Join-Path $bin 'node.exe'
Copy-Item -LiteralPath $originalNode -Destination $node
$wrapper = "@echo off`r`nsetlocal DisableDelayedExpansion`r`n""%~dp0node.exe"" ""$npmCli"" %*`r`nexit /b %errorlevel%`r`n"
[IO.File]::WriteAllText((Join-Path $bin 'npm.cmd'), $wrapper, [Text.UTF8Encoding]::new($false))
$beforeDesktop = Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'"
if (@($beforeDesktop).Count -ne 1 -or $beforeDesktop.ProcessId -ne 38924) {
    throw 'Original one-desktop invariant changed'
}
$info = [Diagnostics.ProcessStartInfo]::new()
$info.FileName = $testExe
$info.WorkingDirectory = $fixture
$info.UseShellExecute = $false
$info.CreateNoWindow = $true
$info.RedirectStandardOutput = $true
$info.RedirectStandardError = $true
foreach ($argument in @('--exact', 'modules::about::remote::tests::live_remote_query_returns_version_and_integrity',
    '--ignored', '--test-threads=1', '--nocapture')) { [void]$info.ArgumentList.Add($argument) }
$info.Environment.Clear()
$info.Environment['SystemRoot'] = 'C:\Windows'
$info.Environment['WINDIR'] = 'C:\Windows'
$info.Environment['COMSPEC'] = 'C:\Windows\System32\cmd.exe'
$info.Environment['PATH'] = $bin + ';C:\Windows\System32;C:\Windows'
$info.Environment['HOME'] = $queryHome
$info.Environment['USERPROFILE'] = $queryHome
$info.Environment['APPDATA'] = Join-Path $queryHome 'AppData/Roaming'
$info.Environment['LOCALAPPDATA'] = Join-Path $queryHome 'AppData/Local'
$info.Environment['TEMP'] = $temp
$info.Environment['TMP'] = $temp
$info.Environment['OPENCODEX_SANDBOX'] = '1'
$info.Environment['OPENCODEX_SANDBOX_ROOT'] = $fixture
$process = $null
$stdout = $null
$stderr = $null
$report = [ordered]@{
    startedAtUtc = [DateTime]::UtcNow.ToString('o')
    sourceCommit = $source
    ciTestExecutableSha256 = $testSha
    npmVersion = $npmPackage.version
    nodeSha256 = (Get-FileHash -LiteralPath $node -Algorithm SHA256).Hash.ToLowerInvariant()
    originalNodeSha256 = (Get-FileHash -LiteralPath $originalNode -Algorithm SHA256).Hash.ToLowerInvariant()
    npmCliSha256 = (Get-FileHash -LiteralPath $npmCli -Algorithm SHA256).Hash.ToLowerInvariant()
    fixtureRoot = $fixture
    home = $queryHome
    npmWrapper = $wrapper
    credentialEnvironmentInherited = $false
    test = 'modules::about::remote::tests::live_remote_query_returns_version_and_integrity'
    applicationExecutableLaunched = $false
    packageLifecycleScriptsExecuted = $false
    currentDesktopPidBefore = 38924
    status = 'preparing'
    scope = 'one explicit ignored Rust live npm metadata query under separate HOME/cache; not new desktop UI or installation acceptance'
}
try {
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $process = [Diagnostics.Process]::Start($info)
    $report.testProcessPid = $process.Id
    $report.testProcessCreationUtc = $process.StartTime.ToUniversalTime().ToString('o')
    $stdout = $process.StandardOutput.ReadToEndAsync()
    $stderr = $process.StandardError.ReadToEndAsync()
    if (!$process.WaitForExit(45000)) { throw 'Live query test exceeded bounded observation' }
    $report.elapsedMs = $clock.Elapsed.TotalMilliseconds
    $report.exitCode = $process.ExitCode
    $report.status = if ($process.ExitCode -eq 0) { 'recorded-live-test-passed' } else { 'recorded-live-test-failed' }
} catch {
    $report.status = 'live-test-harness-failed'
    $report.error = $_.Exception.Message
    throw
} finally {
    if ($process) {
        if (!$process.HasExited) { $process.Kill($true); [void]$process.WaitForExit(5000) }
        if ($stdout -and $stdout.Wait(5000)) {
            $stdout.Result | Set-Content -LiteralPath (Join-Path $evidence 'native-test-stdout.log') -Encoding utf8
        }
        if ($stderr -and $stderr.Wait(5000)) {
            $stderr.Result | Set-Content -LiteralPath (Join-Path $evidence 'native-test-stderr.log') -Encoding utf8
        }
        $process.Dispose()
    }
    $afterDesktop = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'")
    $report.finishedAtUtc = [DateTime]::UtcNow.ToString('o')
    $report.currentDesktopCountAfter = $afterDesktop.Count
    $report.currentDesktopPidAfter = if ($afterDesktop.Count -eq 1) { $afterDesktop[0].ProcessId } else { $null }
    $report.retainedOriginalDesktopLifecycle = $afterDesktop.Count -eq 1 -and $afterDesktop[0].ProcessId -eq 38924 -and
        $afterDesktop[0].CreationDate -eq $beforeDesktop.CreationDate
    $report | ConvertTo-Json -Depth 8 |
        Set-Content -LiteralPath (Join-Path $evidence 'real-npm-test-result.json') -Encoding utf8
}
$report | ConvertTo-Json -Depth 8
if ($report.exitCode -ne 0) { throw 'Recorded live production query failed; preserve the original logs' }

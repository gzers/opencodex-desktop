$ErrorActionPreference = 'Stop'
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskEvidence = $PSScriptRoot
$taskExe = Join-Path $taskCode 'test\software\downloads\11792440963\windows-backend\opencodex_desktop_lib-713299640ffef8db.exe'
$taskExpectedHash = 'bcb9d346a829da51b50b4b85ae32258488e66aa9c25d5868dace0d3a21276a76'
$taskNode = Join-Path $taskCode 'test\software\node-ci-24.21.0\node.exe'
$taskNodeHash = 'ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32'
$taskRuntime = Join-Path $taskCode 'test\runtime\update-focus-stage-probe-20261013'
$taskResultPath = Join-Path $taskEvidence 'local-stage-result.json'
if ((Test-Path -LiteralPath $taskRuntime) -or (Test-Path -LiteralPath $taskResultPath)) {
    throw 'Do not overwrite or restart the diagnostic stage sample'
}
foreach ($taskIdentity in @(@($taskExe, $taskExpectedHash), @($taskNode, $taskNodeHash))) {
    if ((Get-FileHash -LiteralPath $taskIdentity[0] -Algorithm SHA256).Hash.ToLowerInvariant() -cne $taskIdentity[1]) {
        throw 'Exact diagnostic test EXE or CI Node identity differs'
    }
}
. (Join-Path $taskCode 'test\smoke\windows-process-token.ps1')
$taskHelperToken = Get-WindowsHarnessToken
Assert-WindowsProcessMode -Observation $taskHelperToken -Mode ordinary
$taskBeforeApps = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'")
if ($taskBeforeApps.Count -ne 1 -or $taskBeforeApps[0].ProcessId -ne 37276) {
    throw 'One original desktop required'
}
$taskState = Join-Path $taskCode 'test\runtime\windows0110\data\manager-state'
$taskBeforeHashes = [ordered]@{}
foreach ($taskName in @('preferences.json', 'data-root.json', 'backup-cleanup-policy.json', 'update-schedule.json')) {
    $taskBeforeHashes[$taskName] = (Get-FileHash -LiteralPath (Join-Path $taskState $taskName) -Algorithm SHA256).Hash.ToLowerInvariant()
}
$taskHome = Join-Path $taskRuntime 'home'
$taskTemp = Join-Path $taskRuntime 'temp'
$taskRoot = Join-Path $taskRuntime 'fixture'
New-Item -ItemType Directory -Path $taskHome, $taskTemp, $taskRoot | Out-Null
$taskInfo = [Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName = $taskExe
$taskInfo.WorkingDirectory = $taskRoot
$taskInfo.UseShellExecute = $false
$taskInfo.CreateNoWindow = $true
$taskInfo.RedirectStandardOutput = $true
$taskInfo.RedirectStandardError = $true
foreach ($taskArg in @('windows_tests::', '--test-threads=7', '--nocapture')) {
    [void]$taskInfo.ArgumentList.Add($taskArg)
}
$taskInfo.Environment.Clear()
foreach ($taskPair in @{
    SystemRoot = 'C:\Windows'; WINDIR = 'C:\Windows'; COMSPEC = 'C:\Windows\System32\cmd.exe'
    PATH = (Split-Path -Parent $taskNode) + ';C:\Windows\System32;C:\Windows'
    HOME = $taskHome; USERPROFILE = $taskHome
    APPDATA = (Join-Path $taskHome 'AppData\Roaming'); LOCALAPPDATA = (Join-Path $taskHome 'AppData\Local')
    TEMP = $taskTemp; TMP = $taskTemp; OPENCODEX_TEST_NODE = $taskNode
    OPENCODEX_SANDBOX = '1'; OPENCODEX_SANDBOX_ROOT = $taskRoot
}.GetEnumerator()) { $taskInfo.Environment[$taskPair.Key] = $taskPair.Value }
$taskRecord = [ordered]@{
    sourceCommit = 'be7395acfce0ec1e36b83300b576db408f8c35bb'; ciRunId = 38352525092
    testExeSha256 = $taskExpectedHash; nodeExeSha256 = $taskNodeHash
    actualNodeVersion = (& $taskNode --version); requestedTestThreads = 7
    argv = @($taskInfo.ArgumentList); helperToken = $taskHelperToken; testToken = $null
    runtime = $taskRuntime; startedAtUtc = [datetime]::UtcNow.ToString('o')
    inheritedCredentials = $false; desktopApplicationStarted = $false; foregroundInputIssued = $false
    productionProbeSeconds = 2; fixtureSourceChanged = $false; testTimeoutBudgetAltered = $false
    originalDataHashes = $taskBeforeHashes; status = 'starting'
    originalCiCauseIsolated = $false; originalCiFailureCleared = $false; fullNativeAccepted = $false
}
$taskProcess = $null
$taskStdout = $null
$taskStderr = $null
try {
    $taskClock = [Diagnostics.Stopwatch]::StartNew()
    $taskProcess = [Diagnostics.Process]::Start($taskInfo)
    $taskStdout = $taskProcess.StandardOutput.ReadToEndAsync()
    $taskStderr = $taskProcess.StandardError.ReadToEndAsync()
    $taskRecord.testToken = Get-WindowsProcessToken -Process $taskProcess -ExpectedExecutable $taskExe
    Assert-WindowsProcessMode -Observation $taskRecord.testToken -Mode ordinary
    if (-not $taskProcess.WaitForExit(45000)) { throw 'Diagnostic stage observation exceeded45s' }
    $taskRecord.exitCode = $taskProcess.ExitCode
    $taskRecord.elapsedMs = $taskClock.Elapsed.TotalMilliseconds
    $taskRecord.status = if ($taskProcess.ExitCode -eq 0) { 'actual-local-stage-tests-passed' } else { 'actual-local-stage-tests-failed' }
} catch {
    $taskRecord.status = 'failed-or-interrupted'
    $taskRecord.error = $_.Exception.Message
    throw
} finally {
    if ($taskProcess) {
        if (-not $taskProcess.HasExited) {
            $taskRecord.outerCleanupUsed = $true
            $taskProcess.Kill($true)
            [void]$taskProcess.WaitForExit(5000)
        }
        if ($taskStdout -and $taskStdout.Wait(5000)) {
            [IO.File]::WriteAllText((Join-Path $taskEvidence 'local-stage-stdout.log'), $taskStdout.Result, [Text.UTF8Encoding]::new($false))
        }
        if ($taskStderr -and $taskStderr.Wait(5000)) {
            [IO.File]::WriteAllText((Join-Path $taskEvidence 'local-stage-stderr.log'), $taskStderr.Result, [Text.UTF8Encoding]::new($false))
        }
        $taskProcess.Dispose()
    }
    $taskAfterApps = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'")
    $taskRecord.sameOriginalDesktopLifecycle = $taskAfterApps.Count -eq 1 -and
        $taskAfterApps[0].ProcessId -eq 37276 -and $taskAfterApps[0].CreationDate -eq $taskBeforeApps[0].CreationDate
    $taskAfterHashes = [ordered]@{}
    $taskRecord.originalDataBytesPreserved = $true
    foreach ($taskName in $taskBeforeHashes.Keys) {
        $taskAfterHashes[$taskName] = (Get-FileHash -LiteralPath (Join-Path $taskState $taskName) -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($taskAfterHashes[$taskName] -cne $taskBeforeHashes[$taskName]) { $taskRecord.originalDataBytesPreserved = $false }
    }
    $taskRecord.afterDataHashes = $taskAfterHashes
    $taskRecord.finishedAtUtc = [datetime]::UtcNow.ToString('o')
    $taskRecord | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskResultPath -Encoding utf8
}
$taskRecord | ConvertTo-Json -Depth 8

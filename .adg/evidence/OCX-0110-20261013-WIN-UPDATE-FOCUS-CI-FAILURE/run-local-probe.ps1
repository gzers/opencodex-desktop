param([Parameter(Mandatory = $true)][ValidateSet('serial', 'concurrent', 'serial-ci-node', 'concurrent-ci-node')][string]$Mode)
$ErrorActionPreference = 'Stop'
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskEvidence = $PSScriptRoot
$taskSource = 'd975b2351920bc027c931c8135b5605fde7a46c2'
$taskExe = $taskCode + '\test\software\downloads\11790820992\windows-backend\opencodex_desktop_lib-713299640ffef8db.exe'
$taskExpectedHash = 'e5aaed4c66d4bc362c1a63f9485e3fdf806f620e7be6a39d503c5e7c0018a4fc'
$taskRuntime = $taskCode + '\test\runtime\update-focus-ci-probe-' + $Mode
if (Test-Path -LiteralPath $taskRuntime) { throw 'Do not overwrite or restart this sample' }
if ((Get-FileHash -LiteralPath $taskExe -Algorithm SHA256).Hash.ToLowerInvariant() -cne $taskExpectedHash) {
    throw 'Exact failed-run test EXE hash differs'
}
. ($taskCode + '\test\smoke\windows-process-token.ps1')
$taskHelperToken = Get-WindowsHarnessToken
Assert-WindowsProcessMode -Observation $taskHelperToken -Mode ordinary
$taskBeforeApps = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'")
if ($taskBeforeApps.Count -ne 1 -or $taskBeforeApps[0].ProcessId -ne 37276) { throw 'One original desktop required' }
$taskRoot = $taskRuntime + '\fixture'
$taskHome = $taskRuntime + '\home'
$taskTemp = $taskRuntime + '\temp'
New-Item -ItemType Directory -Path $taskRoot, $taskHome, $taskTemp | Out-Null
$taskNode = 'C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe'
if ($Mode.EndsWith('-ci-node')) {
    $taskNodeIdentity = Get-Content -LiteralPath (Join-Path $taskEvidence 'exact-ci-node-identity.json') -Raw | ConvertFrom-Json -DateKind String
    $taskNode = $taskCode + '\test\software\node-ci-24.21.0\node.exe'
    if ($taskNodeIdentity.nodeExecutable -cne $taskNode -or $taskNodeIdentity.version -cne 'v24.21.0' -or
        (Get-FileHash -LiteralPath $taskNode -Algorithm SHA256).Hash.ToLowerInvariant() -cne $taskNodeIdentity.nodeExeSha256) {
        throw 'Exact isolated CI Node identity differs'
    }
}
$taskThreads = if ($Mode.StartsWith('serial')) { 1 } else { 7 }
$taskInfo = [Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName = $taskExe
$taskInfo.WorkingDirectory = $taskRoot
$taskInfo.UseShellExecute = $false
$taskInfo.CreateNoWindow = $true
$taskInfo.RedirectStandardOutput = $true
$taskInfo.RedirectStandardError = $true
foreach ($taskArg in @('windows_tests::', ('--test-threads=' + $taskThreads), '--nocapture')) {
    [void]$taskInfo.ArgumentList.Add($taskArg)
}
$taskInfo.Environment.Clear()
foreach ($taskPair in @{
    SystemRoot = 'C:\Windows'; WINDIR = 'C:\Windows'; COMSPEC = 'C:\Windows\System32\cmd.exe'
    PATH = (Split-Path -Parent $taskNode) + ';C:\Windows\System32;C:\Windows'
    HOME = $taskHome; USERPROFILE = $taskHome
    APPDATA = $taskHome + '\AppData\Roaming'; LOCALAPPDATA = $taskHome + '\AppData\Local'
    TEMP = $taskTemp; TMP = $taskTemp; OPENCODEX_TEST_NODE = $taskNode
    OPENCODEX_SANDBOX = '1'; OPENCODEX_SANDBOX_ROOT = $taskRoot
}.GetEnumerator()) { $taskInfo.Environment[$taskPair.Key] = $taskPair.Value }
$taskProcess = $null
$taskStdout = $null
$taskStderr = $null
$taskRecord = [ordered]@{
    sourceCommit = $taskSource; ciRunId = 38349316806
    testExeSha256 = $taskExpectedHash; requestedTestThreads = $taskThreads
    nodeExecutable = $taskNode; actualNodeVersion = (& $taskNode --version)
    helperToken = $taskHelperToken; testToken = $null; mode = $Mode
    runtime = $taskRuntime; startedAtUtc = [datetime]::UtcNow.ToString('o')
    inheritedCredentials = $false; desktopApplicationStarted = $false
    originalProductionProbeSeconds = 2; testTimeoutBudgetAltered = $false
    fixtureSourceModified = $false; status = 'starting'
}
try {
    $taskClock = [Diagnostics.Stopwatch]::StartNew()
    $taskProcess = [Diagnostics.Process]::Start($taskInfo)
    $taskStdout = $taskProcess.StandardOutput.ReadToEndAsync()
    $taskStderr = $taskProcess.StandardError.ReadToEndAsync()
    $taskRecord.testToken = Get-WindowsProcessToken -Process $taskProcess -ExpectedExecutable $taskExe
    Assert-WindowsProcessMode -Observation $taskRecord.testToken -Mode ordinary
    if (-not $taskProcess.WaitForExit(45000)) { throw 'Actual matched-run probe observation exceeded45s' }
    $taskRecord.exitCode = $taskProcess.ExitCode
    $taskRecord.elapsedMs = $taskClock.Elapsed.TotalMilliseconds
    $taskRecord.status = if ($taskProcess.ExitCode -eq 0) { 'actual-local-tests-passed' } else { 'actual-local-tests-failed' }
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
            [IO.File]::WriteAllText((Join-Path $taskEvidence ($Mode + '-stdout.log')), $taskStdout.Result, [Text.UTF8Encoding]::new($false))
        }
        if ($taskStderr -and $taskStderr.Wait(5000)) {
            [IO.File]::WriteAllText((Join-Path $taskEvidence ($Mode + '-stderr.log')), $taskStderr.Result, [Text.UTF8Encoding]::new($false))
        }
        $taskProcess.Dispose()
    }
    $taskAfterApps = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'")
    $taskRecord.sameOriginalDesktopLifecycle = $taskAfterApps.Count -eq 1 -and
        $taskAfterApps[0].ProcessId -eq 37276 -and $taskAfterApps[0].CreationDate -eq $taskBeforeApps[0].CreationDate
    $taskRecord.finishedAtUtc = [datetime]::UtcNow.ToString('o')
    $taskRecord | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $taskEvidence ($Mode + '-result.json')) -Encoding utf8
}
$taskRecord | ConvertTo-Json -Depth 7

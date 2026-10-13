param([ValidateSet('default', 'four')][string]$Mode = 'default')
$ErrorActionPreference = 'Stop'
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskEvidence = $PSScriptRoot
$taskExe = Join-Path $taskCode 'test\software\downloads\11790820992\windows-backend\opencodex_desktop_lib-713299640ffef8db.exe'
$taskExeHash = 'e5aaed4c66d4bc362c1a63f9485e3fdf806f620e7be6a39d503c5e7c0018a4fc'
$taskNode = Join-Path $taskCode 'test\software\node-ci-24.21.0\node.exe'
$taskNodeHash = 'ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32'
$taskRuntime = Join-Path $taskCode ('test\runtime\update-focus-full-lib-' + $Mode)
$taskResultPath = Join-Path $taskEvidence ($Mode + '-result.json')
if ((Test-Path -LiteralPath $taskRuntime) -or (Test-Path -LiteralPath $taskResultPath)) {
    throw 'Do not overwrite or restart an existing diagnostic sample'
}
foreach ($taskIdentity in @(@($taskExe, $taskExeHash), @($taskNode, $taskNodeHash))) {
    if ((Get-FileHash -LiteralPath $taskIdentity[0] -Algorithm SHA256).Hash.ToLowerInvariant() -cne $taskIdentity[1]) {
        throw 'Exact failed-run test EXE or exact CI Node identity differs'
    }
}
. (Join-Path $taskCode 'test\smoke\windows-process-token.ps1')
$taskHelperToken = Get-WindowsHarnessToken
Assert-WindowsProcessMode -Observation $taskHelperToken -Mode ordinary
$taskBeforeApps = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'")
if ($taskBeforeApps.Count -ne 1 -or $taskBeforeApps[0].ProcessId -ne 37276) {
    throw 'The original single desktop lifecycle is required'
}
$taskStateRoot = Join-Path $taskCode 'test\runtime\windows0110\data\manager-state'
$taskDataNames = @('preferences.json', 'data-root.json', 'backup-cleanup-policy.json', 'update-schedule.json')
$taskBeforeHashes = [ordered]@{}
foreach ($taskName in $taskDataNames) {
    $taskBeforeHashes[$taskName] = (Get-FileHash -LiteralPath (Join-Path $taskStateRoot $taskName) -Algorithm SHA256).Hash.ToLowerInvariant()
}
$taskHome = Join-Path $taskRuntime 'home'
$taskRoot = Join-Path $taskRuntime 'fixture'
$taskTemp = Join-Path $taskRuntime 'temp'
New-Item -ItemType Directory -Path $taskHome, $taskRoot, $taskTemp | Out-Null
$taskInfo = [Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName = $taskExe
$taskInfo.WorkingDirectory = $taskRoot
$taskInfo.UseShellExecute = $false
$taskInfo.CreateNoWindow = $true
$taskInfo.RedirectStandardOutput = $true
$taskInfo.RedirectStandardError = $true
if ($Mode -eq 'four') { [void]$taskInfo.ArgumentList.Add('--test-threads=4') }
[void]$taskInfo.ArgumentList.Add('--nocapture')
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
    sourceCommit = 'd975b2351920bc027c931c8135b5605fde7a46c2'; ciRunId = 38349316806
    nativeSourceCommit = 'eae85322ebca3b755c034db97cbebf2ea5bf71fc'
    testExeSha256 = $taskExeHash; nodeExeSha256 = $taskNodeHash
    nodeExecutable = $taskNode; actualNodeVersion = (& $taskNode --version)
    mode = $Mode; argv = @($taskInfo.ArgumentList); hostLogicalProcessors = [Environment]::ProcessorCount
    runtime = $taskRuntime; workingDirectory = $taskRoot; helperToken = $taskHelperToken
    isolatedEnvironmentKeys = @($taskInfo.Environment.Keys | Sort-Object)
    inheritedCredentials = $false; desktopApplicationStarted = $false; foregroundInputIssued = $false
    testFilterApplied = $false; ignoredTestsEnabled = $false; productionProbeDeadlineSeconds = 2
    testAssertionsChanged = $false; fixtureSourceChanged = $false; outerObservationLimitSeconds = 600
    fullCiWorkspaceMatched = $false; localPassClearsCiFailure = $false; ciRerunRequested = $false
    originalDataHashes = $taskBeforeHashes; startedAtUtc = [datetime]::UtcNow.ToString('o'); status = 'starting'
}
function Write-TaskRecord {
    [IO.File]::WriteAllText($taskResultPath, ($taskRecord | ConvertTo-Json -Depth 8) + "`n", [Text.UTF8Encoding]::new($false))
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
    $taskRecord.status = 'running'
    Write-TaskRecord
    while (-not $taskProcess.WaitForExit(1000)) {
        if ($taskClock.Elapsed.TotalSeconds -gt 600) { throw 'Whole-library diagnostic observation limit exceeded' }
    }
    $taskRecord.exitCode = $taskProcess.ExitCode
    $taskRecord.elapsedMs = $taskClock.Elapsed.TotalMilliseconds
    $taskRecord.status = if ($taskProcess.ExitCode -eq 0) { 'actual-local-full-library-passed' } else { 'actual-local-full-library-failed' }
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
    $taskAfterHashes = [ordered]@{}
    $taskRecord.originalDataBytesPreserved = $true
    foreach ($taskName in $taskDataNames) {
        $taskAfterHashes[$taskName] = (Get-FileHash -LiteralPath (Join-Path $taskStateRoot $taskName) -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($taskAfterHashes[$taskName] -cne $taskBeforeHashes[$taskName]) { $taskRecord.originalDataBytesPreserved = $false }
    }
    $taskRecord.afterDataHashes = $taskAfterHashes
    $taskRecord.dailyExecutableSha256 = (Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe' -Algorithm SHA256).Hash.ToLowerInvariant()
    $taskRecord.finishedAtUtc = [datetime]::UtcNow.ToString('o')
    Write-TaskRecord
}
$taskRecord | ConvertTo-Json -Depth 8

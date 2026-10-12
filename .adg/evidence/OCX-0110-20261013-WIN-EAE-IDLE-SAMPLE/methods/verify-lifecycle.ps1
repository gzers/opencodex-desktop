$ErrorActionPreference = 'Stop'
$taskBase = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\windows0110'
$taskSession = Get-Content -LiteralPath ($taskBase + '\control\current-session.json') -Raw | ConvertFrom-Json
if ($taskSession.sourceCommit -cne 'eae85322ebca3b755c034db97cbebf2ea5bf71fc' -or
    $taskSession.status -cne 'running' -or $taskSession.applicationPid -ne 37276) {
    throw 'Retained final application identity differs'
}
$taskExpected = @(
    @{ Id = 37276; Started = '2026-10-12T14:53:12.0088010Z'; Path = $taskSession.executable },
    @{ Id = 24300; Started = '2026-10-12T14:53:10.4430287Z'; Path = 'C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\native\powershell\pwsh.exe' },
    @{ Id = 3920; Started = '2026-10-12T14:55:52.1278297Z'; Path = 'C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\native\powershell\pwsh.exe' }
)
$taskHandles = @()
try {
    $taskReadbacks = foreach ($taskIdentity in $taskExpected) {
        $taskProcess = Get-Process -Id $taskIdentity.Id -ErrorAction Stop
        $taskHandles += $taskProcess
        $taskHandle = $taskProcess.SafeHandle
        if ($taskHandle.IsInvalid -or $taskProcess.HasExited -or
            $taskProcess.StartTime.ToUniversalTime() -ne [datetime]::Parse($taskIdentity.Started).ToUniversalTime() -or
            $taskProcess.Path -cne $taskIdentity.Path) {
            throw 'A retained process handle or creation identity differs'
        }
        [ordered]@{ processId = $taskProcess.Id; creationTimeUtc = $taskProcess.StartTime.ToUniversalTime().ToString('o')
            executable = $taskProcess.Path; liveHandleVerified = $true; hasExited = $taskProcess.HasExited }
    }
    $taskAppCount = @(Get-CimInstance Win32_Process -Filter "Name = 'opencodex-desktop.exe'").Count
    if ($taskAppCount -ne 1) { throw 'One-isolated-desktop constraint violated' }
    $taskImageHash = (Get-FileHash -LiteralPath $taskSession.executable -Algorithm SHA256).Hash.ToLowerInvariant()
    $taskDailyHash = (Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe' -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($taskImageHash -cne 'f5e4a2eef0ffc364a2a57b47b1463015faf3433a15ff021d544d6e161004f00c' -or
        $taskDailyHash -cne '43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67') {
        throw 'Candidate or preserved daily image hash differs'
    }
    [ordered]@{
        collectedAtUtc = [datetime]::UtcNow.ToString('o')
        sourceCommit = $taskSession.sourceCommit
        applicationPid = 37276; supervisorPid = 24300; monitorPid = 3920
        applicationExecSession = 80657; monitorExecSession = 46484
        desktopApplicationCount = $taskAppCount
        processReadbacks = @($taskReadbacks)
        applicationToken = $taskSession.applicationToken
        executableSha256 = $taskImageHash; dailyExecutableSha256 = $taskDailyHash
        noUiInputsWhileRdpDisconnected = $true
        syntheticDatesOnlyForNewlyOwnedFixtures = $true
        clocksOrTimersChanged = $false
        manualUpdateCheckOrInstallInvoked = $false
        physicalDpiUiMatrixAccepted = $false
        fullRetentionSafetyMatrixAccepted = $false
        pairedPerformanceAccepted = $false
        realCyclesAccepted = $false
        fullNativeGateAccepted = $false
        releaseAccepted = $false
    } | ConvertTo-Json -Depth 7
} finally {
    foreach ($taskHandleOwner in $taskHandles) { $taskHandleOwner.Dispose() }
}

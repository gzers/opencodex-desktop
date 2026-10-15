param([ValidateSet('first-screen','maximized','restored','minimized','resumed','settings','before-exit')][string]$Phase='first-screen',
      [switch]$ObserveExit)
$ErrorActionPreference = 'Stop'
$taskIdentity = [Security.Principal.WindowsIdentity]::GetCurrent()
$taskPrincipal = [Security.Principal.WindowsPrincipal]::new($taskIdentity)
if (!$taskPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'The read-only administrator observer needs an administrator token.'
}
$taskInvocation = Join-Path $PSScriptRoot ("admin-observer-$Phase-invocation.json")
if (Test-Path -LiteralPath $taskInvocation) { throw 'This observation phase already exists.' }
[ordered]@{
    atUtc=[DateTime]::UtcNow.ToString('o'); phase=$Phase; observerPid=$PID
    administrator=$true; observeExit=[bool]$ObserveExit
    sendsInput=$false; changesSettings=$false; startsApplication=$false; terminatesApplication=$false
} | ConvertTo-Json | Set-Content -LiteralPath $taskInvocation -Encoding utf8
try {
    & (Join-Path $PSScriptRoot 'read-state.ps1') -Phase $Phase -Mode administrator `
        1> (Join-Path $PSScriptRoot "admin-observer-$Phase.stdout.original.txt") `
        2> (Join-Path $PSScriptRoot "admin-observer-$Phase.stderr.original.txt")
    if ($ObserveExit) {
        $taskLaunch = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-launch.json') -Raw | ConvertFrom-Json
        $taskProcess = Get-Process -Id $taskLaunch.pid
        if ($taskProcess.StartTime.ToUniversalTime().Ticks -ne ([datetimeoffset]$taskLaunch.creationTimeUtc).UtcDateTime.Ticks) {
            throw 'Administrator product PID identity changed.'
        }
        $taskHandle = $taskProcess.Handle
        [ordered]@{
            atUtc=[DateTime]::UtcNow.ToString('o'); pid=$taskProcess.Id
            creationTimeUtc=$taskLaunch.creationTimeUtc; handleHeld=$true
            observerOnly=$true; timeoutSeconds=600
        } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-exit-observer-ready.json') -Encoding utf8
        $taskExited = $taskProcess.WaitForExit(600000)
        [ordered]@{
            atUtc=[DateTime]::UtcNow.ToString('o'); pid=$taskLaunch.pid
            creationTimeUtc=$taskLaunch.creationTimeUtc
            exitObserved=$taskExited; exitCode=$(if ($taskExited) {$taskProcess.ExitCode} else {$null})
            timeout=(-not $taskExited); forcedTerminationByObserver=$false
        } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-exit-observer-result.json') -Encoding utf8
        $taskProcess.Dispose()
    }
} catch {
    $_ | Out-String | Set-Content -LiteralPath (Join-Path $PSScriptRoot "admin-observer-$Phase-error.original.txt") -Encoding utf8
} finally {
    $taskIdentity.Dispose()
}

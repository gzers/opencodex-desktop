$ErrorActionPreference = 'Stop'
$taskProcess = $null
try {
    . (Join-Path $PSScriptRoot 'foreground-contract-live.ps1')
    $taskAuthorization = Assert-BackupForegroundAuthorization -Authorization (
        Get-Content -LiteralPath (Join-Path $PSScriptRoot 'foreground-authorization.json') -Raw -Encoding utf8 | ConvertFrom-Json)
    & (Join-Path $PSScriptRoot 'read-state.ps1') -Phase 'manual-window-followup' -Mode administrator
    [ordered]@{
        atUtc = [datetimeoffset]::UtcNow.ToString('o')
        status = 'read-completed'
        authorization = $taskAuthorization
        authorizationEncoding = 'utf8'
        uiInputInjected = $false
        productLaunched = $false
        userManualOperationRepeated = $false
    } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'reader-pwsh7-result.json') -Encoding utf8
    $taskLaunch = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-launch.json') -Raw -Encoding utf8 | ConvertFrom-Json
    $taskProcess = Get-Process -Id $taskLaunch.pid
    if ($taskProcess.StartTime.ToUniversalTime().Ticks -ne ([datetimeoffset]$taskLaunch.creationTimeUtc).UtcDateTime.Ticks) {
        throw 'PID changed before retaining the followup exit handle'
    }
    $taskHandle = $taskProcess.Handle
    $taskMilliseconds = [int][Math]::Max(1, [Math]::Min(120000,
        [Math]::Floor((([datetimeoffset]$taskAuthorization.expiresAtUtc) - [datetimeoffset]::UtcNow).TotalMilliseconds)))
    [ordered]@{
        atUtc = [datetimeoffset]::UtcNow.ToString('o')
        pid = $taskLaunch.pid
        creationTimeUtc = $taskLaunch.creationTimeUtc
        handleHeld = $true
        timeoutMilliseconds = $taskMilliseconds
        inputInjected = $false
        productLaunched = $false
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'followup-exit-observer-ready.json') -Encoding utf8
    $taskExited = $taskProcess.WaitForExit($taskMilliseconds)
    [ordered]@{
        atUtc = [datetimeoffset]::UtcNow.ToString('o')
        pid = $taskLaunch.pid
        creationTimeUtc = $taskLaunch.creationTimeUtc
        exitObserved = $taskExited
        exitCode = $(if ($taskExited) { $taskProcess.ExitCode } else { $null })
        timeout = (-not $taskExited)
        agentForcedTermination = $false
        inputInjected = $false
        productLaunched = $false
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'followup-exit-observer-result.json') -Encoding utf8
} catch {
    [ordered]@{
        atUtc = [datetimeoffset]::UtcNow.ToString('o')
        status = 'read-or-observer-failed'
        errorType = $_.Exception.GetType().Name
        message = $_.Exception.Message
        automaticRetryAllowed = $false
        uiInputInjected = $false
        productLaunched = $false
    } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'reader-pwsh7-error.json') -Encoding utf8
    throw
} finally {
    if ($null -ne $taskProcess) { $taskProcess.Dispose() }
}

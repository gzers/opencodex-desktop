$ErrorActionPreference = 'Stop'
try {
    . (Join-Path $PSScriptRoot 'foreground-contract-live.ps1')
    $taskAuthorization = Assert-BackupForegroundAuthorization -Authorization (
        Get-Content -LiteralPath (Join-Path $PSScriptRoot 'foreground-authorization.json') -Raw | ConvertFrom-Json)
    & (Join-Path $PSScriptRoot 'read-state.ps1') -Phase 'manual-window-followup' -Mode administrator
    [ordered]@{
        atUtc = [datetimeoffset]::UtcNow.ToString('o')
        status = 'read-completed'
        authorization = $taskAuthorization
        uiInputInjected = $false
        productLaunched = $false
        userManualOperationRepeated = $false
    } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'reader-result.json') -Encoding utf8
} catch {
    [ordered]@{
        atUtc = [datetimeoffset]::UtcNow.ToString('o')
        status = 'read-failed'
        errorType = $_.Exception.GetType().Name
        message = $_.Exception.Message
        automaticRetryAllowed = $false
        uiInputInjected = $false
        productLaunched = $false
    } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'reader-result.json') -Encoding utf8
    throw
}

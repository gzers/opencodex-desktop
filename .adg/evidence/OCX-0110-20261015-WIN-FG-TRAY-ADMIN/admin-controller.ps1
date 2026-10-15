$ErrorActionPreference = 'Stop'
$taskStatusPath = Join-Path $PSScriptRoot 'admin-controller-result.json'
if (Test-Path -LiteralPath $taskStatusPath) { throw 'Administrator controller was already invoked' }
$taskIdentity = [Security.Principal.WindowsIdentity]::GetCurrent()
$taskPrincipal = [Security.Principal.WindowsPrincipal]::new($taskIdentity)
$taskSelf = Get-Process -Id $PID
$taskRecord = [ordered]@{
    atUtc = [DateTime]::UtcNow.ToString('o')
    controllerPid = $PID
    controllerCreatedUtc = $taskSelf.StartTime.ToUniversalTime().ToString('o')
    actualAdministrator = $taskPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    userScope = '继续前台验证; original ordinary/administrator application gate'
    automaticSecurityUiInteraction = $false
    status = 'starting'
}
$taskRecord | ConvertTo-Json | Set-Content -LiteralPath $taskStatusPath -Encoding utf8
try {
    if (!$taskRecord.actualAdministrator) { throw 'Real administrator token required' }
    & (Join-Path $PSScriptRoot 'launch.ps1') -Mode administrator `
        1> (Join-Path $PSScriptRoot 'administrator-launch.stdout.original.txt') `
        2> (Join-Path $PSScriptRoot 'administrator-launch.stderr.original.txt')
    $taskRecord.status = 'application-started-review-required'
} catch {
    $taskRecord.status = 'failed-no-blind-retry'
    $taskRecord.error = $_.Exception.Message
    $_ | Out-String | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'admin-controller-error.original.txt') -Encoding utf8
} finally {
    $taskRecord.finishedAtUtc = [DateTime]::UtcNow.ToString('o')
    $taskRecord | ConvertTo-Json | Set-Content -LiteralPath $taskStatusPath -Encoding utf8
    $taskIdentity.Dispose()
    $taskSelf.Dispose()
}

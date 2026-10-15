$ErrorActionPreference = 'Stop'
$taskRequest = 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261014-WIN-BD2-ETW-RUN\record-request-imagefix.ps1'
$taskStatePath = Join-Path $PSScriptRoot 'launcher-result.json'
if (Test-Path -LiteralPath $taskStatePath) {
    throw 'This approved launcher has already started; inspect its result instead of repeating.'
}
$taskIdentity = [Security.Principal.WindowsIdentity]::GetCurrent()
$taskPrincipal = [Security.Principal.WindowsPrincipal]::new($taskIdentity)
$taskSelf = Get-Process -Id $PID
$taskState = [ordered]@{
    status = 'starting'
    startedAtUtc = [DateTime]::UtcNow.ToString('o')
    controllerPid = $PID
    controllerCreatedUtc = $taskSelf.StartTime.ToUniversalTime().ToString('o')
    controllerImage = $taskSelf.Path
    controllerIsAdministrator = $taskPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    powershellVersion = $PSVersionTable.PSVersion.ToString()
    request = $taskRequest
    requestSha256 = (Get-FileHash -LiteralPath $taskRequest -Algorithm SHA256).Hash.ToLowerInvariant()
    userAuthorization = '批准修正版后续都可以管理员后台 ETW 录制'
    automaticUacInteraction = $false
    originalLibraryMaximumRunCount = 1
    entryReturned = $false
}
$taskState | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskStatePath -Encoding utf8
$taskExit = 1
try {
    if (!$taskState.controllerIsAdministrator) {
        throw 'The controller does not have a real administrator token.'
    }
    $taskState.status = 'recording'
    $taskState | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskStatePath -Encoding utf8
    & $taskRequest -ApprovedByUser 1> (Join-Path $PSScriptRoot 'entry.stdout.original.txt') 2> (Join-Path $PSScriptRoot 'entry.stderr.original.txt')
    $taskState.entryReturned = $true
    $taskState.status = 'entry-returned-review-required'
    $taskExit = 0
}
catch {
    $taskState.status = 'failed-review-required'
    $taskState.error = $_.Exception.Message
    $_ | Out-String | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'launcher-error.original.txt') -Encoding utf8
}
finally {
    $taskState.finishedAtUtc = [DateTime]::UtcNow.ToString('o')
    $taskState.launcherExitCode = $taskExit
    $taskState | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskStatePath -Encoding utf8
    $taskIdentity.Dispose()
    $taskSelf.Dispose()
}
exit $taskExit

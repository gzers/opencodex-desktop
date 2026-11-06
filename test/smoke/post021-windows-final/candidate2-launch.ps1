param([Parameter(Mandatory=$true)][string]$PlanFile)
$ErrorActionPreference='Stop'
$taskRoot='D:\Users\Administrator\WorkSpace\Test\post021-candidate2-targeted-20261105'
if([IO.Path]::GetFullPath($PlanFile) -ine (Join-Path $taskRoot 'plan-attempt2.json')){throw 'Unexpected test plan path'}
$taskIdentity=[Security.Principal.WindowsIdentity]::GetCurrent()
$taskSession=(Get-Process -Id $PID).SessionId
$taskAttempt=Join-Path $taskRoot ('entry-attempts\'+[datetime]::UtcNow.ToString('yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N'))
$null=New-Item -ItemType Directory -Path $taskAttempt
$taskRecord=[ordered]@{
    result='entry-started';atUtc=[datetimeoffset]::UtcNow.ToString('o')
    owner=$taskIdentity.Name;sid=$taskIdentity.User.Value;sessionId=$taskSession
    planSha256=(Get-FileHash -LiteralPath $PlanFile -Algorithm SHA256).Hash.ToLowerInvariant()
    scope='Isolated test entry diagnostics; no credentials or config content logged'
}
$taskEncoding=New-Object System.Text.UTF8Encoding($false)
$taskRecordPath=Join-Path $taskAttempt 'entry-result.json'
function Save-EntryRecord{[IO.File]::WriteAllText($taskRecordPath,($taskRecord|ConvertTo-Json -Depth 4),$taskEncoding)}
Save-EntryRecord
if($taskIdentity.Name -ine 'EZIO-MINI\Administrator' -or $taskIdentity.User.Value -ne 'S-1-5-21-587408064-856152790-2583328096-500' -or $taskSession -ne 1){
    $taskRecord.result='rejected-owner-before-driver';Save-EntryRecord
    Write-Output ('No validation started. Requires Administrator session1; actual '+$taskIdentity.Name+' session'+$taskSession+'. Evidence: '+$taskRecordPath)
    exit 2
}
$taskPriorCodex=[Environment]::GetEnvironmentVariable('CODEX_HOME','Process')
$taskPriorOcx=[Environment]::GetEnvironmentVariable('OPENCODEX_HOME','Process')
$taskExit=1
try{
    $env:CODEX_HOME='C:/Users/Administrator/.codex'
    $env:OPENCODEX_HOME=Join-Path $taskRoot 'sandbox/opencodex-home'
    $taskDriver=Join-Path $PSScriptRoot 'candidate2-targeted-native.mjs'
    $taskRecord.driverSha256=(Get-FileHash -LiteralPath $taskDriver -Algorithm SHA256).Hash.ToLowerInvariant()
    $taskRecord.result='driver-running';Save-EntryRecord
    Write-Output ('Isolated validation running. Console logs: '+$taskAttempt)
    # Windows PowerShell 5.1 must retain native stderr without converting it to
    # a terminating NativeCommandError before the process exit is recorded.
    $ErrorActionPreference='Continue'
    & (Join-Path $taskRoot 'tools/bun.exe') $taskDriver $PlanFile 1> (Join-Path $taskAttempt 'driver.stdout.log') 2> (Join-Path $taskAttempt 'driver.stderr.log')
    $taskExit=$LASTEXITCODE
    $ErrorActionPreference='Stop'
    $taskRecord.result='driver-exited';$taskRecord.exitCode=$taskExit
}catch{
    $taskRecord.result='launcher-error';$taskRecord.error=$_.Exception.Message
}finally{
    [Environment]::SetEnvironmentVariable('CODEX_HOME',$taskPriorCodex,'Process')
    [Environment]::SetEnvironmentVariable('OPENCODEX_HOME',$taskPriorOcx,'Process')
    $taskRecord.finishedAtUtc=[datetimeoffset]::UtcNow.ToString('o');Save-EntryRecord
    Write-Output ('Validation entry finished. Evidence: '+$taskRecordPath)
}
exit $taskExit

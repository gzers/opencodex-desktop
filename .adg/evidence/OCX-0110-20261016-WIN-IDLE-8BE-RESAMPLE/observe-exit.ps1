$ErrorActionPreference='Stop'
$taskLaunch=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'idle-candidate0110-launch.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskTerminal=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'terminal.json') -Raw -Encoding utf8 | ConvertFrom-Json
if(!$taskTerminal.measurementCompleted){throw 'Do not close before the requested observation completes'}
$taskProcess=[Diagnostics.Process]::GetProcessById([int]$taskLaunch.pid)
if($taskProcess.StartTime.ToUniversalTime().Ticks -ne ([datetimeoffset]$taskLaunch.creationTimeUtc).UtcTicks){throw 'Process generation changed'}
if($taskProcess.MainModule.FileName -cne $taskLaunch.executable){throw 'Unexpected executable'}
[void]$taskProcess.SafeHandle
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskProcess.Id
    creationTimeUtc=$taskLaunch.creationTimeUtc;retainedHandle=$true
    uiInputInjected=$false;forceTerminationAllowed=$false
} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'exit-observer-ready.json') -Encoding utf8
$taskExited=$taskProcess.WaitForExit(60000)
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskProcess.Id
    exitObserved=$taskExited;exitCode=if($taskExited){$taskProcess.ExitCode}else{$null}
    retainedHandle=$true;forcedTermination=$false;closeMethod='Computer Use Alt+F4'
} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'candidate0110-exit.json') -Encoding utf8
$taskProcess.Dispose()
if(!$taskExited){throw 'Bound product did not exit during the normal-close observation'}

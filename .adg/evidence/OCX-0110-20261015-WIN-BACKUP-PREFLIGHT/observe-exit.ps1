param([ValidateSet('ordinary-launch.json','ordinary-restart.json')][string]$LaunchRecord='ordinary-launch.json',
      [ValidateSet('first','restart')][string]$Label='first')
$ErrorActionPreference='Stop'
$taskLaunch=Get-Content -LiteralPath (Join-Path $PSScriptRoot $LaunchRecord) -Raw | ConvertFrom-Json
$taskProcess=Get-Process -Id $taskLaunch.pid
if ($taskProcess.StartTime.ToUniversalTime().Ticks -ne ([datetimeoffset]$taskLaunch.creationTimeUtc).UtcDateTime.Ticks) {
    throw 'PID changed before retaining exit observer handle.'
}
$taskHandle=$taskProcess.Handle
$taskReady=Join-Path $PSScriptRoot ("$Label-exit-observer-ready.json")
$taskResult=Join-Path $PSScriptRoot ("$Label-exit-observer-result.json")
if ((Test-Path -LiteralPath $taskReady) -or (Test-Path -LiteralPath $taskResult)) {throw 'Exit observer evidence already exists.'}
[ordered]@{atUtc=[DateTime]::UtcNow.ToString('o');pid=$taskProcess.Id;creationTimeUtc=$taskLaunch.creationTimeUtc;
    handleHeld=$true;timeoutSeconds=300;inputInjected=$false;forcedTermination=$false} |
    ConvertTo-Json | Set-Content -LiteralPath $taskReady -Encoding utf8
$taskExited=$taskProcess.WaitForExit(300000)
[ordered]@{atUtc=[DateTime]::UtcNow.ToString('o');pid=$taskLaunch.pid;creationTimeUtc=$taskLaunch.creationTimeUtc;
    exitObserved=$taskExited;exitCode=$(if ($taskExited) {$taskProcess.ExitCode} else {$null});
    timeout=(-not $taskExited);agentForcedTermination=$false;inputInjected=$false} |
    ConvertTo-Json | Set-Content -LiteralPath $taskResult -Encoding utf8
Get-Content -LiteralPath $taskResult -Raw
$taskProcess.Dispose()

param(
    [Parameter(Mandatory)][ValidateSet('baseline019','candidate0110')][string]$Version,
    [Parameter(Mandatory)][string]$RunDirectory
)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'run-binding.ps1')
$taskRun=Assert-PreparedIdleRun -RunDirectory $RunDirectory
$taskContract=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'contract.json') -Raw | ConvertFrom-Json
$taskHuman=Get-Content -LiteralPath (Join-Path $taskRun 'human-window.json') -Raw | ConvertFrom-Json
if($taskHuman.explicitHumanStart -ne $true -or $taskHuman.automaticContinuation -ne $false -or
   $taskHuman.scope -cne $taskContract.humanWindowScope -or
   (([datetimeoffset]$taskHuman.expiresAtUtc)-([datetimeoffset]$taskHuman.authorizedAtUtc)).TotalSeconds -gt 1800){throw 'Bound human receipt required for cleanup observation'}
$taskLaunchPath=Join-Path $taskRun ('idle-'+$Version+'-launch.json')
$taskLaunch=Get-Content -LiteralPath $taskLaunchPath -Raw | ConvertFrom-Json
$taskPrefix=if($Version -eq 'baseline019'){'baseline'}else{'candidate'}
$taskExpectedHash=$taskContract.($taskPrefix+'ExeSha256')
if($taskLaunch.source -cne $taskContract.($taskPrefix+'Source') -or
   $taskLaunch.exeSha256 -cne $taskExpectedHash -or
   (Get-FileHash -LiteralPath $taskLaunch.executable).Hash.ToLowerInvariant() -cne $taskExpectedHash){throw 'Launch binding differs'}
$taskReady=Join-Path $taskRun ($Version+'-exit-observer-ready.json')
$taskResult=Join-Path $taskRun ($Version+'-exit.json')
if((Test-Path -LiteralPath $taskReady) -or (Test-Path -LiteralPath $taskResult)){throw 'Exit observation already attempted'}
. (Join-Path $PSScriptRoot 'retained-process-times.ps1')
$taskProcess=[Diagnostics.Process]::GetProcessById([int]$taskLaunch.pid)
try {
    [void]$taskProcess.SafeHandle
    $taskCreated=([datetimeoffset]$taskLaunch.creationTimeUtc).UtcDateTime.ToFileTimeUtc()
    if($taskProcess.MainModule.FileName -cne $taskLaunch.executable){throw 'Process image differs'}
    $taskBefore=Get-WindowsRetainedTimesEvidence -Process $taskProcess -ExpectedCreationFileTime $taskCreated
    $taskRemaining=(([datetimeoffset]$taskHuman.expiresAtUtc).AddSeconds(120)-[datetimeoffset]::UtcNow).TotalMilliseconds
    if($taskRemaining -le 0 -or $taskRemaining -gt 1920000){throw 'Cleanup observation outside bounded window'}
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskProcess.Id;creationTimeUtc=$taskLaunch.creationTimeUtc
        retainedHandle=$true;before=$taskBefore;launchSha256=(Get-FileHash -LiteralPath $taskLaunchPath).Hash.ToLowerInvariant()
        observeUntilUtc=([datetimeoffset]$taskHuman.expiresAtUtc).AddSeconds(120).ToString('o')
        uiInputInjected=$false;forcedTerminationAllowed=$false} |
        ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskReady -Encoding utf8
    $taskExited=$taskProcess.WaitForExit([int][Math]::Ceiling($taskRemaining))
    $taskKernel=if($taskExited){Get-WindowsRetainedTimesEvidence -Process $taskProcess -ExpectedCreationFileTime $taskCreated -RequireExited}else{$null}
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskProcess.Id;exitObserved=$taskExited
        exitCode=if($taskExited){$taskProcess.ExitCode}else{$null};retainedHandle=$true;forcedTermination=$false
        kernelTimes=$taskKernel;nativeCount=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count
        closeMethod='Actual close action must be recorded separately';uiInputInjected=$false
        extendedObservationChangesNoAcceptanceDeadline=$true} |
        ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskResult -Encoding utf8
    if(!$taskExited){throw 'Bounded cleanup exit observation expired'}
}finally{$taskProcess.Dispose()}

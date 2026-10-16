param(
    [Parameter(Mandatory)][ValidateSet('baseline019','candidate0110')][string]$Version,
    [Parameter(Mandatory)][string]$RunDirectory
)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'run-binding.ps1')
$taskRun=Assert-PreparedIdleRun -RunDirectory $RunDirectory
. (Join-Path $PSScriptRoot 'retained-process-times.ps1')
$taskRuntime=Join-Path $taskRun $Version
$taskLaunch=Get-Content -LiteralPath (Join-Path $taskRun ('idle-'+$Version+'-launch.json')) -Raw | ConvertFrom-Json
$taskTerminal=Get-Content -LiteralPath (Join-Path $taskRuntime 'terminal.json') -Raw | ConvertFrom-Json
if(!$taskTerminal.measurementCompleted){throw 'Incomplete observation; use a separately recorded interrupted close'}
$taskProcess=[Diagnostics.Process]::GetProcessById([int]$taskLaunch.pid)
try {
    [void]$taskProcess.SafeHandle
    $taskExpectedFileTime=([datetimeoffset]$taskLaunch.creationTimeUtc).UtcDateTime.ToFileTimeUtc()
    $taskBefore=Get-WindowsRetainedTimesEvidence -Process $taskProcess -ExpectedCreationFileTime $taskExpectedFileTime
    if($taskProcess.StartTime.ToUniversalTime().Ticks -ne ([datetimeoffset]$taskLaunch.creationTimeUtc).UtcTicks -or
       $taskProcess.MainModule.FileName -cne $taskLaunch.executable){throw 'Process generation or executable changed'}
    [ordered]@{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskProcess.Id
        creationTimeUtc=$taskLaunch.creationTimeUtc;retainedHandle=$true
        uiInputInjected=$false;forcedTerminationAllowed=$false
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRun ($Version+'-exit-observer-ready.json')) -Encoding utf8
    $taskExited=$taskProcess.WaitForExit(60000)
    $taskKernel=if($taskExited){Get-WindowsRetainedTimesEvidence -Process $taskProcess -ExpectedCreationFileTime $taskExpectedFileTime -RequireExited}else{$null}
    [ordered]@{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskProcess.Id
        exitObserved=$taskExited;exitCode=if($taskExited){$taskProcess.ExitCode}else{$null}
        retainedHandle=$true;forcedTermination=$false;closeMethod='Computer Use Alt+F4'
        kernelTimes=$taskKernel
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRun ($Version+'-exit.json')) -Encoding utf8
    if(!$taskExited){throw 'Normal-close observation timed out'}
}finally{$taskProcess.Dispose()}

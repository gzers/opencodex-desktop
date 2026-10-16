$ErrorActionPreference='Stop'
$taskRun=$PSScriptRoot
$taskLaunch=Get-Content -LiteralPath (Join-Path $taskRun 'idle-candidate0110-launch.json') -Raw | ConvertFrom-Json
if($taskLaunch.source -cne 'b4d487e4e04cd4c80fd5570663e1ed21dd483b19' -or
   $taskLaunch.exeSha256 -cne 'f3fab9d6840c105e426a7dc90efe94fe839e283e47af3929d251d82a6e8675db'){throw 'Bound candidate identity required'}
if(Test-Path -LiteralPath (Join-Path $taskRun 'late-cleanup-observer-ready.json')){throw 'Preserve the cleanup observation'}
. 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261016-WIN-B4-FRONT-PREP\retained-process-times.ps1'
$taskProcess=[Diagnostics.Process]::GetProcessById([int]$taskLaunch.pid)
try {
    [void]$taskProcess.SafeHandle
    if($taskProcess.MainModule.FileName -cne $taskLaunch.executable -or
       (Get-FileHash -LiteralPath $taskLaunch.executable).Hash.ToLowerInvariant() -cne $taskLaunch.exeSha256){throw 'Cleanup image differs'}
    $taskCreated=([datetimeoffset]$taskLaunch.creationTimeUtc).UtcDateTime.ToFileTimeUtc()
    $taskBefore=Get-WindowsRetainedTimesEvidence -Process $taskProcess -ExpectedCreationFileTime $taskCreated
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');before=$taskBefore
        scope='separate read-only cleanup observation; original exit deadline result remains unchanged'
        maximumObservationSeconds=300;foregroundPermissionExtended=$false
        productStartedOrStopped=$false;originalObservationOverwritten=$false;acceptanceClaimed=$false} |
        ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskRun 'late-cleanup-observer-ready.json') -Encoding utf8
    $taskTimer=[Diagnostics.Stopwatch]::StartNew()
    $taskExited=$false
    while($taskTimer.Elapsed.TotalSeconds -lt 300){
        if($taskProcess.WaitForExit(1000)){$taskExited=$true;break}
    }
    $taskKernel=if($taskExited){Get-WindowsRetainedTimesEvidence -Process $taskProcess -ExpectedCreationFileTime $taskCreated -RequireExited}else{$null}
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskProcess.Id
        exitObserved=$taskExited;exitCode=if($taskExited){$taskProcess.ExitCode}else{$null}
        kernelTimes=$taskKernel;retainedHandle=$true;forceTerminated=$false
        nativeCount=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count
        scope='late manual cleanup only';originalExitDeadlineOutcomeUnchanged=$true
        foregroundPermissionExtended=$false;formalPairLifecycleAccepted=$false} |
        ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskRun 'late-cleanup-exit.json') -Encoding utf8
}finally{$taskProcess.Dispose()}

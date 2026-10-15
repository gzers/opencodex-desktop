$ErrorActionPreference='Stop'
$taskOriginal=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'ordinary-unwritable-select-launch.json') -Raw | ConvertFrom-Json
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
. (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/methods/process-user-sid.ps1')
$taskTimer=[Diagnostics.Stopwatch]::StartNew()
$taskMaximum=0
$taskTimeline=[Collections.Generic.List[object]]::new()
$taskReplacement=$null
while($taskTimer.Elapsed.TotalSeconds -lt 240){
    $taskItems=@(Get-Process -Name opencodex-desktop -ErrorAction SilentlyContinue)
    $taskMaximum=[Math]::Max($taskMaximum,$taskItems.Count)
    $taskTimeline.Add(@{atUtc=[datetimeoffset]::UtcNow.ToString('o');pids=@($taskItems.Id)})
    foreach($taskItem in $taskItems){
        if($taskItem.Id -ne $taskOriginal.pid){
            $taskToken=Get-AcceptanceProcessSidEvidence -Process $taskItem -ExpectedExecutable $taskOriginal.executable
            Assert-WindowsProcessMode -Observation $taskToken -Mode ordinary
            if($taskToken.sidSha256 -cne $taskOriginal.applicationUser.sidSha256){throw 'Recovery changed account identity'}
            $taskReplacement=$taskItem
            break
        }
    }
    if($taskReplacement){break}
    Start-Sleep -Milliseconds 50
}
$taskTimeline | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'recovery-process-timeline.original.json') -Encoding utf8
if(!$taskReplacement){throw 'No replacement observed within bounded wait'}
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');phase='recovered';mode='ordinary'
    source=$taskOriginal.source;executable=$taskOriginal.executable;exeSha256=$taskOriginal.exeSha256
    pid=$taskReplacement.Id;creationTimeUtc=$taskToken.creationTimeUtc
    applicationToken=$taskToken;applicationUser=$taskToken
    sandboxUsed=$false;cdpUsed=$false;environmentRootRedirected=$false
    startedBy='product startup recovery';recoveryPid=$taskOriginal.pid
    observedMaxExeCountDuringRecovery=$taskMaximum;businessInitializationInOriginal=$false
} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'recovered-launch.json') -Encoding utf8
try{
    $taskTimer.Restart()
    while(!$taskReplacement.WaitForExit(1000) -and $taskTimer.Elapsed.TotalSeconds -lt 1800){}
    [ordered]@{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');phase='recovered'
        pid=$taskReplacement.Id;creationTimeUtc=$taskToken.creationTimeUtc
        exitObserved=$taskReplacement.HasExited;exitCode=if($taskReplacement.HasExited){$taskReplacement.ExitCode}else{$null}
        retainedHandle=$true;forcedTermination=$false
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'recovered-exit.json') -Encoding utf8
}finally{$taskReplacement.Dispose()}

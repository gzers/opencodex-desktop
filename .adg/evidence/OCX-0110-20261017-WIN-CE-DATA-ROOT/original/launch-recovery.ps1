# One future ordinary-token recovery attempt; UI input is exclusively through Sky.
param([Parameter(Mandatory)][string]$RunDirectory)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'run-binding.ps1')
$taskRun=Assert-PreparedIdleRun -RunDirectory $RunDirectory -RequireCurrentHumanWindow
$taskContract=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'contract.json') -Raw | ConvertFrom-Json
$taskDir=Assert-PreparedRecoveryDirectory -Directory $taskContract.recoveryDirectory
if($taskContract.recoveryExecutable -cne (Join-Path $taskDir 'opencodex-desktop.exe')){throw 'Recovery executable is outside its prepared directory'}
if(!(Test-Path -LiteralPath (Join-Path $taskRun 'recovery-prepared.json'))){throw 'Scoped recovery prerequisites not prepared'}
if(Test-Path -LiteralPath (Join-Path $taskRun 'recovery-launch.json')){throw 'Do not retry or overwrite this recovery attempt'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native instance exists'}
if((Get-FileHash -LiteralPath $taskContract.recoveryExecutable).Hash.ToLowerInvariant() -cne $taskContract.candidateExeSha256){throw 'Recovery image changed'}
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
. $taskContract.sidMethodPath
. (Join-Path $PSScriptRoot 'retained-process-times.ps1')
Assert-WindowsProcessMode -Observation (Get-WindowsHarnessToken) -Mode ordinary
$taskInfo=[Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName=$taskContract.recoveryExecutable
$taskInfo.WorkingDirectory=$taskContract.recoveryDirectory;$taskInfo.UseShellExecute=$false
foreach($taskName in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY','OPENCODEX_WINDOWS_SMOKE_CDP_PORT','OPENCODEX_WINDOWS_SMOKE_SCALE','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','WEBVIEW2_USER_DATA_FOLDER')){
    [void]$taskInfo.Environment.Remove($taskName)
}
$taskOld=[Diagnostics.Process]::Start($taskInfo)
$taskNew=$null;$taskFailure=$null;$taskOldExit=$null;$taskNewTimes=$null;$taskNewToken=$null
$taskMaximum=0;$taskTimeline=[Collections.Generic.List[object]]::new()
try {
    [void]$taskOld.SafeHandle
    $taskToken=Get-AcceptanceProcessSidEvidence -Process $taskOld -ExpectedExecutable $taskContract.recoveryExecutable
    Assert-WindowsProcessMode -Observation $taskToken -Mode ordinary
    if($taskToken.sidSha256 -cne '373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310'){throw 'Recovery account changed'}
    $taskCreated=([datetimeoffset]$taskToken.creationTimeUtc).UtcDateTime.ToFileTimeUtc()
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskContract.candidateSource
        executable=$taskContract.recoveryExecutable;exeSha256=$taskContract.candidateExeSha256
        pid=$taskOld.Id;creationTimeUtc=$taskToken.creationTimeUtc;actualToken=$taskToken
        sandboxUsed=$false;cdpUsed=$false;uiInputInjected=$false} | ConvertTo-Json -Depth 7 |
        Set-Content -LiteralPath (Join-Path $taskRun 'recovery-launch.json') -Encoding utf8
    $taskTimer=[Diagnostics.Stopwatch]::StartNew()
    while($taskTimer.Elapsed.TotalSeconds -lt 240){
        $taskRows=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'")
        $taskMaximum=[Math]::Max($taskMaximum,$taskRows.Count)
        $taskOldTimes=Get-WindowsRetainedTimesEvidence -Process $taskOld -ExpectedCreationFileTime $taskCreated
        if($taskOldTimes.exited){$taskOldExit=$taskOldTimes}
        $taskTimeline.Add(@{atUtc=[datetimeoffset]::UtcNow.ToString('o');pids=@($taskRows.ProcessId);oldKernelExited=$taskOldTimes.exited})
        foreach($taskRow in $taskRows){
            if([int]$taskRow.ProcessId -eq $taskOld.Id){continue}
            if($null -eq $taskNew){
                $taskNew=[Diagnostics.Process]::GetProcessById([int]$taskRow.ProcessId)
                [void]$taskNew.SafeHandle
                $taskNewToken=Get-AcceptanceProcessSidEvidence -Process $taskNew -ExpectedExecutable $taskContract.recoveryExecutable
                Assert-WindowsProcessMode -Observation $taskNewToken -Mode ordinary
                if($taskNewToken.sidSha256 -cne $taskToken.sidSha256){throw 'Replacement token user changed'}
                $taskNewCreated=([datetimeoffset]$taskNewToken.creationTimeUtc).UtcDateTime.ToFileTimeUtc()
                $taskNewTimes=Get-WindowsRetainedTimesEvidence -Process $taskNew -ExpectedCreationFileTime $taskNewCreated
            }elseif([int]$taskRow.ProcessId -ne $taskNew.Id){throw 'Unexpected additional native generation'}
        }
        if($null -ne $taskOldExit -and $null -ne $taskNewTimes){break}
        Start-Sleep -Milliseconds 50
    }
    if($null -eq $taskOldExit -or $null -eq $taskNewTimes){throw 'Bound recovery did not complete within the observation limit'}
}catch{
    $taskFailure=@{exceptionType=$_.Exception.GetType().FullName;hresult=$_.Exception.HResult}
}finally{
    $taskTimeline.ToArray() | ConvertTo-Json -Depth 5 |
        Set-Content -LiteralPath (Join-Path $taskRun 'recovery-census.original.json') -Encoding utf8
    $taskOrder=($null -ne $taskOldExit -and $null -ne $taskNewTimes -and
        [long]$taskNewTimes.creationFileTime -ge [long]$taskOldExit.exitFileTime)
    $taskOldExitCode=if($null -ne $taskOldExit){$taskOld.ExitCode}else{$null}
    [ordered]@{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskContract.candidateSource
        oldPid=$taskOld.Id;newPid=if($taskNew){$taskNew.Id}else{$null}
        oldKernelTimes=$taskOldExit;newKernelTimes=$taskNewTimes;replacementToken=$taskNewToken
        oldExitCode=$taskOldExitCode;observedMaximumNativeCount=$taskMaximum
        exactKernelOrderPassed=$taskOrder;strictRawCountLimitPassed=($taskMaximum -le 1)
        readFailure=$taskFailure
        processSubsetPassed=($taskOrder -and $taskMaximum -le 1 -and $null -eq $taskFailure -and $taskOldExitCode -eq 0)
        nativeRootUiReadbackRequired=$true;replacementNormalCloseRequired=$true
        fullNativeRecoveryGatePassed=$false;productForceTerminated=$false;uiInputInjected=$false
    } | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $taskRun 'recovery-process-order.json') -Encoding utf8
    if($taskNew){$taskNew.Dispose()}
    $taskOld.Dispose()
}
if($taskFailure -or !$taskOrder -or $taskMaximum -gt 1 -or $taskOldExitCode -ne 0){
    throw 'Recovery process subset not admitted; preserve the running state for bound review'
}

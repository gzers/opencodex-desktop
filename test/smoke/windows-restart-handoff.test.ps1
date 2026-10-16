param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
if(!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7){throw 'Windows PowerShell 7 test host required'}
$taskRepo=(Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../..')).Path
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
$taskAllowed=Join-Path $taskRepo 'test'
if(!$taskOutput.StartsWith($taskAllowed+'\',[StringComparison]::OrdinalIgnoreCase) -or
   (Test-Path -LiteralPath $taskOutput)){throw 'Fresh repository-owned test output required'}
if(@(Get-Process -Name opencodex-desktop,ocxd -ErrorAction SilentlyContinue).Count){throw 'Do not overlap a native product instance'}
New-Item -ItemType Directory -Path $taskOutput | Out-Null
$taskSystemShell=Join-Path ([Environment]::GetFolderPath('System')) 'WindowsPowerShell/v1.0/powershell.exe'
$taskMethod=Join-Path $taskRepo 'apps/desktop/tauri/src/infrastructure/windows_restart.ps1'
$taskWaiter=Get-Content -LiteralPath $taskMethod -Raw -Encoding utf8
$taskCases=[Collections.Generic.List[object]]::new()
$taskCleanup=[Collections.Generic.List[object]]::new()
$taskChecks=0
function Check([bool]$Condition,[string]$Message){
    if(!$Condition){throw $Message}
    $script:taskChecks++
}
function WideHex([string]$Value){
    return [Convert]::ToHexString([Text.Encoding]::Unicode.GetBytes($Value)).ToLowerInvariant()
}
function Start-ControlledShell([string[]]$Arguments){
    $taskInfo=[Diagnostics.ProcessStartInfo]::new()
    $taskInfo.FileName=$taskSystemShell
    $taskInfo.UseShellExecute=$false;$taskInfo.CreateNoWindow=$true
    $taskInfo.WindowStyle=[Diagnostics.ProcessWindowStyle]::Hidden
    $taskInfo.RedirectStandardInput=$true;$taskInfo.RedirectStandardOutput=$true
    $taskInfo.RedirectStandardError=$true
    foreach($taskArgument in $Arguments){$taskInfo.ArgumentList.Add($taskArgument)}
    return [Diagnostics.Process]::Start($taskInfo)
}
function Start-Parent {
    $taskProcess=Start-ControlledShell @('-NoLogo','-NoProfile','-NonInteractive','-Command',
        "[Console]::Out.WriteLine('PARENT_READY');[Console]::Out.Flush();if([Console]::In.ReadLine() -cne 'FINISH'){exit 51}")
    $taskRead=$taskProcess.StandardOutput.ReadLineAsync()
    if(!$taskRead.Wait(15000) -or $taskRead.Result -cne 'PARENT_READY'){
        $taskProcess.StandardInput.Close()
        if(!$taskProcess.WaitForExit(5000)){$taskProcess.Kill()}
        $taskProcess.Dispose()
        throw 'Controlled parent did not become ready'
    }
    [void]$taskProcess.SafeHandle
    return $taskProcess
}
function Start-Waiter($Parent,[string]$Marker,[long]$GenerationOffset,[int]$ExitTimeout,[string]$Target){
    $taskCreated=$Parent.StartTime.ToFileTimeUtc()
    # The target is another hidden fixture. Its marker records whether the exact
    # parent generation was still alive, without publishing any other process.
    $taskTargetScript=@"
`$taskMarker=[Text.Encoding]::Unicode.GetString([Convert]::FromBase64String('$([Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($Marker)))'))
`$taskSameParent=`$false
try{`$taskOld=[Diagnostics.Process]::GetProcessById($($Parent.Id));`$taskSameParent=(`$taskOld.StartTime.ToFileTimeUtc() -eq $taskCreated -and !`$taskOld.HasExited);`$taskOld.Dispose()}catch{}
[IO.File]::WriteAllText(`$taskMarker,(@{pid=`$PID;atUtc=[datetimeoffset]::UtcNow.ToString('o');sameParentGenerationAlive=`$taskSameParent}|ConvertTo-Json))
"@
    $taskEncoded=[Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($taskTargetScript))
    $taskTargetArguments='-NoLogo -NoProfile -NonInteractive -WindowStyle Hidden -EncodedCommand '+$taskEncoded
    $taskProcess=Start-ControlledShell @('-NoLogo','-NoProfile','-NonInteractive','-WindowStyle','Hidden','-Command',$taskWaiter)
    foreach($taskLine in @(
        [string]$Parent.Id
        [string]($taskCreated+$GenerationOffset)
        (WideHex $Parent.MainModule.FileName)
        (WideHex $Target)
        (WideHex $taskOutput)
        (WideHex $taskTargetArguments)
        [string]$ExitTimeout
    )){$taskProcess.StandardInput.WriteLine($taskLine)}
    $taskProcess.StandardInput.Flush()
    return $taskProcess
}
function Finish-Parent($Parent){
    $Parent.StandardInput.WriteLine('FINISH');$Parent.StandardInput.Flush()
    Check ($Parent.WaitForExit(10000)) 'Controlled parent did not exit cooperatively'
    Check ($Parent.ExitCode -eq 0) 'Controlled parent returned a failure'
}
function Cleanup-Owned($Process,[string]$Role){
    if($null -eq $Process){return}
    try {
        if(!$Process.HasExited){
            $Process.StandardInput.Close()
            if(!$Process.WaitForExit(3000)){
                $Process.Kill()
                $taskCleanup.Add(@{role=$Role;forcedFixtureTermination=$true})
                [void]$Process.WaitForExit(3000)
            }
        }
    }finally{$Process.Dispose()}
}
try {
    foreach($taskCase in @('commit-after-exit','no-commit','wrong-generation','wrong-target','exit-deadline')){
        $taskParent=$null;$taskHelper=$null
        $taskMarker=Join-Path $taskOutput ($taskCase+'-target.json')
        try {
            $taskParent=Start-Parent
            $taskOffset=if($taskCase -eq 'wrong-generation'){1}else{0}
            $taskTimeout=if($taskCase -eq 'exit-deadline'){300}else{30000}
            $taskTargetPath=if($taskCase -eq 'wrong-target'){Join-Path $taskOutput 'missing-executable.exe'}else{$taskSystemShell}
            $taskHelper=Start-Waiter $taskParent $taskMarker $taskOffset $taskTimeout $taskTargetPath
            $taskRead=$taskHelper.StandardOutput.ReadLineAsync()
            Check ($taskRead.Wait(15000)) 'Waiter handshake was not bounded'
            if($taskCase -in @('wrong-generation','wrong-target')){
                Check ($null -eq $taskRead.Result) 'Invalid generation or target was admitted'
                Check ($taskHelper.WaitForExit(5000)) 'Rejected helper did not exit'
                $taskExpectedCode=if($taskCase -eq 'wrong-generation'){41}else{46}
                Check ($taskHelper.ExitCode -eq $taskExpectedCode) 'Preflight rejection exit code differs'
                Finish-Parent $taskParent
            }else{
                Check ($taskRead.Result -ceq 'OCX_RESTART_READY_V1') 'Waiter did not retain the expected generation'
                if($taskCase -eq 'no-commit'){
                    $taskHelper.StandardInput.Close()
                    Check ($taskHelper.WaitForExit(5000)) 'Aborted helper did not exit'
                    Check ($taskHelper.ExitCode -eq 42) 'EOF was not an abort'
                    Finish-Parent $taskParent
                }else{
                    $taskHelper.StandardInput.WriteLine('OCX_RESTART_COMMIT_V1')
                    $taskHelper.StandardInput.Flush()
                    if($taskCase -eq 'exit-deadline'){
                        Check ($taskHelper.WaitForExit(5000)) 'Parent wait exceeded the test deadline'
                        Check ($taskHelper.ExitCode -eq 43) 'Parent deadline did not stop the launch'
                        Check (!$taskParent.HasExited) 'Deadline fixture parent exited early'
                        Finish-Parent $taskParent
                    }else{
                        Start-Sleep -Milliseconds 150
                        Check (!$taskParent.HasExited -and !(Test-Path -LiteralPath $taskMarker)) 'Target started before parent exit'
                        Finish-Parent $taskParent
                        Check ($taskHelper.WaitForExit(10000)) 'Committed helper did not finish'
                        Check ($taskHelper.ExitCode -eq 0) 'Committed helper failed'
                        $taskTimer=[Diagnostics.Stopwatch]::StartNew()
                        while(!(Test-Path -LiteralPath $taskMarker) -and $taskTimer.Elapsed.TotalSeconds -lt 10){Start-Sleep -Milliseconds 50}
                        Check (Test-Path -LiteralPath $taskMarker) 'Replacement marker was not produced'
                        $taskTarget=Get-Content -LiteralPath $taskMarker -Raw | ConvertFrom-Json
                        Check (!$taskTarget.sameParentGenerationAlive) 'Replacement observed the old generation alive'
                    }
                }
            }
            if($taskCase -ne 'commit-after-exit'){
                Start-Sleep -Milliseconds 150
                Check (!(Test-Path -LiteralPath $taskMarker)) 'Rejected handoff launched a replacement'
            }
            $taskCases.Add(@{case=$taskCase;status='passed';parentPid=$taskParent.Id
                parentCreationFileTime=$taskParent.StartTime.ToFileTimeUtc()
                helperExitCode=$taskHelper.ExitCode;productStarted=$false})
        }finally{
            Cleanup-Owned $taskHelper 'waiter'
            Cleanup-Owned $taskParent 'parent'
        }
    }
    Check ($taskCleanup.Count -eq 0) 'A fixture required forced cleanup'
    Check (@(Get-Process -Name opencodex-desktop,ocxd -ErrorAction SilentlyContinue).Count -eq 0) 'Native product appeared during background tests'
    [ordered]@{
        status='passed';checks=$taskChecks;cases=$taskCases.ToArray()
        waiterSha256=(Get-FileHash -LiteralPath $taskMethod).Hash.ToLowerInvariant()
        controlledParentAndWaiterPairs=5;cleanup=$taskCleanup.ToArray()
        usesSystemWindowsPowerShell=$true;productStarted=$false;foregroundActivated=$false
        dataRootOrRegistryModified=$false;nativeRecoveryGatePassed=$false
        fullPerformanceGatePassed=$false;originalQueryDeadlinesChanged=$false
    } | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $taskOutput 'result.json') -Encoding utf8
    Get-Content -LiteralPath (Join-Path $taskOutput 'result.json') -Raw
}catch{
    [ordered]@{status='failed';checks=$taskChecks;completedCases=$taskCases.ToArray();cleanup=$taskCleanup.ToArray()
        errorType=$_.Exception.GetType().FullName;productStarted=$false;nativeRecoveryGatePassed=$false} |
        ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $taskOutput 'failure.json') -Encoding utf8
    throw
}

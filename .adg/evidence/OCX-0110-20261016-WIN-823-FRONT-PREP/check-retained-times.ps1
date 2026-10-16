param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
if(!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7){throw 'Windows PowerShell 7 required'}
$taskAllowed='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime'
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
if(!$taskOutput.StartsWith($taskAllowed+'\',[StringComparison]::OrdinalIgnoreCase) -or
   (Test-Path -LiteralPath $taskOutput)){throw 'Fresh owned runtime output required'}
if(@(Get-Process -Name opencodex-desktop,ocxd -ErrorAction SilentlyContinue).Count){throw 'Native product exists'}
New-Item -ItemType Directory -Path $taskOutput | Out-Null
. (Join-Path $PSScriptRoot 'retained-process-times.ps1')
$taskShell=Join-Path ([Environment]::GetFolderPath('System')) 'WindowsPowerShell/v1.0/powershell.exe'
$taskProcesses=[Collections.Generic.List[Diagnostics.Process]]::new()
$taskForced=$false
function New-ControlledParent {
    $taskInfo=[Diagnostics.ProcessStartInfo]::new()
    $taskInfo.FileName=$taskShell;$taskInfo.UseShellExecute=$false
    $taskInfo.CreateNoWindow=$true;$taskInfo.WindowStyle='Hidden'
    $taskInfo.RedirectStandardInput=$true;$taskInfo.RedirectStandardOutput=$true
    foreach($taskArgument in @('-NoLogo','-NoProfile','-NonInteractive','-WindowStyle','Hidden','-Command',
        "[Console]::Out.WriteLine('READY');[Console]::Out.Flush();if([Console]::In.ReadLine() -cne 'FINISH'){exit 71}")){
        $taskInfo.ArgumentList.Add($taskArgument)
    }
    $taskProcess=[Diagnostics.Process]::Start($taskInfo)
    $taskProcesses.Add($taskProcess)
    [void]$taskProcess.SafeHandle
    $taskRead=$taskProcess.StandardOutput.ReadLineAsync()
    if(!$taskRead.Wait(10000) -or $taskRead.Result -cne 'READY'){throw 'Controlled process not ready'}
    return $taskProcess
}
try {
    $taskParent=New-ControlledParent
    $taskCreated=$taskParent.StartTime.ToFileTimeUtc()
    $taskLive=Get-WindowsRetainedTimesEvidence -Process $taskParent -ExpectedCreationFileTime $taskCreated
    if($taskLive.exited -or $null -ne $taskLive.exitFileTime){throw 'Live generation claimed terminated'}
    $taskMismatchRejected=$false
    try{Get-WindowsRetainedTimesEvidence -Process $taskParent -ExpectedCreationFileTime ($taskCreated+1) | Out-Null}catch{$taskMismatchRejected=$true}
    if(!$taskMismatchRejected){throw 'Wrong generation was admitted'}
    $taskRequireExitRejected=$false
    try{Get-WindowsRetainedTimesEvidence -Process $taskParent -ExpectedCreationFileTime $taskCreated -RequireExited | Out-Null}catch{$taskRequireExitRejected=$true}
    if(!$taskRequireExitRejected){throw 'Live process accepted as exited'}
    $taskParent.StandardInput.WriteLine('FINISH');$taskParent.StandardInput.Flush()
    if(!$taskParent.WaitForExit(10000) -or $taskParent.ExitCode -ne 0){throw 'Parent did not exit cooperatively'}
    $taskExited=Get-WindowsRetainedTimesEvidence -Process $taskParent -ExpectedCreationFileTime $taskCreated -RequireExited
    if([long]$taskExited.exitFileTime -ne $taskParent.ExitTime.ToFileTimeUtc()){throw 'Kernel exit time and framework readback differ'}
    $taskNext=New-ControlledParent
    $taskNextCreated=$taskNext.StartTime.ToFileTimeUtc()
    $taskNextTimes=Get-WindowsRetainedTimesEvidence -Process $taskNext -ExpectedCreationFileTime $taskNextCreated
    if([long]$taskNextTimes.creationFileTime -lt [long]$taskExited.exitFileTime){throw 'Sequential fixture lifetime ordering failed'}
    $taskNext.StandardInput.WriteLine('FINISH');$taskNext.StandardInput.Flush()
    if(!$taskNext.WaitForExit(10000) -or $taskNext.ExitCode -ne 0){throw 'Next fixture did not exit cooperatively'}
    [ordered]@{
        status='passed';scope='two controlled PowerShell processes only'
        live=$taskLive;exited=$taskExited;next=$taskNextTimes
        wrongGenerationRejected=$taskMismatchRejected;liveExitClaimRejected=$taskRequireExitRejected
        exitTimeMatchesFramework=$true;nextCreatedAtOrAfterParentKernelExit=$true
        forcedFixtureTermination=$false;productStarted=$false;foregroundActivated=$false
        nativeRecoveryGatePassed=$false
    } | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $taskOutput 'result.json') -Encoding utf8
    Get-Content -LiteralPath (Join-Path $taskOutput 'result.json') -Raw
}finally{
    foreach($taskProcess in $taskProcesses){
        if(!$taskProcess.HasExited){
            $taskProcess.StandardInput.Close()
            if(!$taskProcess.WaitForExit(3000)){$taskProcess.Kill();$taskForced=$true;[void]$taskProcess.WaitForExit(3000)}
        }
        $taskProcess.Dispose()
    }
    if($taskForced){throw 'Controlled fixture required forced cleanup; do not admit the result'}
}

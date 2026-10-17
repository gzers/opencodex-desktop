# Deferred until a new direct human window and this tool's complete CI admission.
. (Join-Path $PSScriptRoot 'common.ps1')
[void](Assert-HumanWindow)
Assert-DiagnosticSource
if(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'gpu-prepare-attempt.json')){throw 'One declared preparation attempt only'}
if(([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Ordinary binding controller required'}
$taskRoot=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'root-launch.json') -Raw | ConvertFrom-Json
if($taskRoot.source -cne $taskPlan.productSource -or $taskRoot.exeSha256 -cne $taskPlan.productExeSha256){throw 'Fresh exact product launch required'}
Write-NewJson (Join-Path $PSScriptRoot 'gpu-prepare-attempt.json') @{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');toolSource=$taskPlan.toolSource
    rootLaunchSha256=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'root-launch.json')).Hash.ToLowerInvariant()
    automaticRetry=$false;tracingStarted=$false
}
. (Join-Path $taskRepo 'test/smoke/windows-gpu-resource.ps1')
$taskRoles=Get-WindowsGpuResourceSnapshot -Launch $taskRoot -Mode ordinary
Write-NewJson (Join-Path $PSScriptRoot 'settings-roles.original.json') $taskRoles
$taskGpuRows=@($taskRoles.members | Where-Object role -CEQ 'gpu')
if($taskGpuRows.Count -ne 1){throw 'One actual GPU role required'}
$taskGpu=[Diagnostics.Process]::GetProcessById([int]$taskGpuRows[0].pid)
try{
    [void]$taskGpu.SafeHandle
    if($taskGpu.StartTime.ToUniversalTime().Ticks -ne ([datetimeoffset]$taskGpuRows[0].creationUtc).UtcTicks){throw 'GPU generation changed'}
    Write-NewJson (Join-Path $PSScriptRoot 'gpu-launch.json') @{
        pid=$taskGpu.Id;creationTimeUtc=$taskGpu.StartTime.ToUniversalTime().ToString('o')
        executable=$taskGpu.MainModule.FileName;exeSha256=(Get-FileHash -LiteralPath $taskGpu.MainModule.FileName).Hash.ToLowerInvariant()
        actualRole='gpu';observedAtUtc=[datetimeoffset]::UtcNow.ToString('o')
    }
}finally{$taskGpu.Dispose()}
# A fresh process prevents caller-imported functions from masking CLI scope bugs.
# Both decoded streams are retained even when the entry fails before JSON output.
$taskInfo=[Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName=(Get-Process -Id $PID).Path
$taskInfo.UseShellExecute=$false;$taskInfo.CreateNoWindow=$true
$taskInfo.RedirectStandardOutput=$true;$taskInfo.RedirectStandardError=$true
foreach($taskArgument in @('-NoProfile','-NonInteractive','-File',(Join-Path $taskRepo 'test/smoke/windows-bound-gpu-memory.ps1'),
    '-RootLaunchJson',(Join-Path $PSScriptRoot 'root-launch.json'),'-GpuLaunchJson',(Join-Path $PSScriptRoot 'gpu-launch.json'),
    '-OutputDirectory',(Join-Path $PSScriptRoot 'gpu-capture'),'-Mode','ordinary','-Seconds','30')){
    $taskInfo.ArgumentList.Add($taskArgument)
}
$taskChild=[Diagnostics.Process]::Start($taskInfo)
try{
    [void]$taskChild.SafeHandle
    $taskStdout=$taskChild.StandardOutput.ReadToEndAsync()
    $taskStderr=$taskChild.StandardError.ReadToEndAsync()
    $taskExited=$taskChild.WaitForExit(30000)
    Write-NewJson (Join-Path $PSScriptRoot 'gpu-prepare-child-result.json') @{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskChild.Id
        creationUtc=$taskChild.StartTime.ToUniversalTime().ToString('o')
        exitObserved=$taskExited;exitCode=if($taskExited){$taskChild.ExitCode}else{$null}
        automaticRetry=$false;forcedTermination=$false;tracingRequested=$false
    }
    if(!$taskExited){throw 'Prepare child observation expired; retain child identity and do not kill or retry'}
    $taskOut=$taskStdout.GetAwaiter().GetResult()
    $taskErr=$taskStderr.GetAwaiter().GetResult()
    foreach($taskStream in @(@('gpu-prepare.stdout.decoded.log',$taskOut),@('gpu-prepare.stderr.decoded.log',$taskErr))){
        $taskStreamFile=[IO.FileStream]::new((Join-Path $PSScriptRoot $taskStream[0]),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
        try{$taskBytes=[Text.Encoding]::UTF8.GetBytes($taskStream[1]);$taskStreamFile.Write($taskBytes,0,$taskBytes.Length)}finally{$taskStreamFile.Dispose()}
    }
    if($taskChild.ExitCode -ne 0){throw 'Fresh prepare-only entry failed; both decoded output streams preserved'}
    $taskPrepared=$taskOut | ConvertFrom-Json
    if($taskPrepared.status -cne 'bound-gpu-prepared-only' -or !$taskPrepared.binding.gpuRoleVerified -or
       $taskPrepared.tracingStarted -or $taskPrepared.productStarted -or $taskPrepared.recordingExecutionAdmitted){throw 'Unexpected prepare-only outcome'}
    Write-NewJson (Join-Path $PSScriptRoot 'gpu-prepare-only.json') $taskPrepared
}finally{$taskChild.Dispose()}
[void](Assert-HumanWindow)
Write-NewJson (Join-Path $PSScriptRoot 'record-request.json') @{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');seconds=30;recordApproved=$true
    rootLaunchSha256=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'root-launch.json')).Hash.ToLowerInvariant()
    gpuLaunchSha256=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'gpu-launch.json')).Hash.ToLowerInvariant()
    automaticRetry=$false
}

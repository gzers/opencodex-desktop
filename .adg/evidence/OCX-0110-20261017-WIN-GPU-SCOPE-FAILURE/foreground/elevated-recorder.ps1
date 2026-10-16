. (Join-Path $PSScriptRoot 'common.ps1')
$taskProcess=Get-Process -Id $PID
try{
    $taskHuman=Assert-HumanWindow
    Assert-DiagnosticSource
    $taskAdmin=([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if(!$taskAdmin){throw 'Actual elevated recorder token required'}
    . (Join-Path $taskRepo 'test/smoke/windows-process-token.ps1')
    . (Join-Path $taskDocs '.adg/evidence/OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/methods/process-user-sid.ps1')
    $taskToken=Get-AcceptanceProcessSidEvidence -Process $taskProcess -ExpectedExecutable $taskProcess.Path
    if($taskToken.sidSha256 -cne '373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310'){throw 'Recorder TokenUser changed'}
    Write-NewJson (Join-Path $PSScriptRoot 'elevated-recorder-ready.json') @{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');actualRecorderToken=$taskToken
        controllerPid=$PID;humanWindowSha256=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'human-window.json')).Hash.ToLowerInvariant()
        productStarted=$false;tracingStarted=$false;waitingForFreshPackets=$true
    }
    $taskDeadline=([datetimeoffset]$taskHuman.normalCloseByUtc).AddMinutes(-3)
    while(!(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'record-request.json'))){
        if([datetimeoffset]::UtcNow -ge $taskDeadline){throw 'Bounded wait for fresh launch packets expired'}
        Start-Sleep -Milliseconds 200
    }
    [void](Assert-HumanWindow)
    $taskRequest=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'record-request.json') -Raw | ConvertFrom-Json
    if($taskRequest.seconds -ne 30 -or !$taskRequest.recordApproved){throw 'One approved 30-second recording required'}
    foreach($taskPair in @(@('root-launch.json','rootLaunchSha256'),@('gpu-launch.json','gpuLaunchSha256'))){
        if((Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $taskPair[0])).Hash.ToLowerInvariant() -cne $taskRequest.($taskPair[1])){throw 'Fresh packet changed'}
    }
    & (Join-Path $taskRepo 'test/smoke/windows-bound-gpu-memory.ps1') `
        -RootLaunchJson (Join-Path $PSScriptRoot 'root-launch.json') -GpuLaunchJson (Join-Path $PSScriptRoot 'gpu-launch.json') `
        -OutputDirectory (Join-Path $PSScriptRoot 'gpu-capture') -Mode ordinary -Seconds 30 `
        -DecoderDirectory $taskPlan.decoderDirectory -MethodAdmissionPath (Join-Path $taskPrep 'ci-complete-admission.json') -RecordApproved `
        *> (Join-Path $PSScriptRoot 'gpu-recorder.original.log')
    Write-NewJson (Join-Path $PSScriptRoot 'elevated-recorder-result.json') @{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');completed=$true;exitCode=0;automaticRetry=$false
        productStarted=$false;forcedTermination=$false
    }
    exit 0
}catch{
    Write-NewJson (Join-Path $PSScriptRoot 'elevated-recorder-result.json') @{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');completed=$false;exitCode=1
        exceptionType=$_.Exception.GetType().FullName;message=$_.Exception.Message;scriptStackTrace=$_.ScriptStackTrace
        automaticRetry=$false;productStarted=$false;forcedTermination=$false
    }
    exit 1
}finally{$taskProcess.Dispose()}

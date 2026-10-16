. (Join-Path $PSScriptRoot 'common.ps1')
$taskProcess=$null
try{
    $taskHuman=Assert-HumanWindow
    Assert-DiagnosticSource
    if(([datetimeoffset]$taskHuman.normalCloseByUtc-[datetimeoffset]::UtcNow).TotalSeconds -lt 480){throw 'Need at least eight minutes before normal close target'}
    if(!(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'elevated-recorder-ready.json'))){throw 'Actual elevated recorder must be ready first'}
    if(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'root-launch.json')){throw 'One launch attempt only'}
    if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native0 required'}
    if(([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Ordinary launch controller required'}
    . (Join-Path $taskRepo 'test/smoke/windows-process-token.ps1')
    . (Join-Path $taskDocs '.adg/evidence/OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/methods/process-user-sid.ps1')
    . (Join-Path $taskDocs '.adg/evidence/OCX-0110-20261016-WIN-B4-FRONT-PREP/retained-process-times.ps1')
    $taskInfo=[Diagnostics.ProcessStartInfo]::new()
    $taskInfo.FileName=$taskPlan.productExecutable
    $taskInfo.WorkingDirectory=Split-Path -Parent $taskPlan.productExecutable
    $taskInfo.UseShellExecute=$false
    foreach($taskName in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY',
        'OPENCODEX_WINDOWS_SMOKE_CDP_PORT','OPENCODEX_WINDOWS_SMOKE_SCALE','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS',
        'WEBVIEW2_USER_DATA_FOLDER','__COMPAT_LAYER')){[void]$taskInfo.Environment.Remove($taskName)}
    Write-NewJson (Join-Path $PSScriptRoot 'launch-attempt.json') @{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskPlan.productSource;exeSha256=$taskPlan.productExeSha256
        ordinaryController=$true;automaticRetry=$false;maximumNativeInstances=1
    }
    $taskProcess=[Diagnostics.Process]::Start($taskInfo)
    [void]$taskProcess.SafeHandle
    $taskToken=Get-AcceptanceProcessSidEvidence -Process $taskProcess -ExpectedExecutable $taskPlan.productExecutable
    Assert-WindowsProcessMode -Observation $taskToken -Mode ordinary
    if($taskToken.sidSha256 -cne '373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310'){throw 'Product TokenUser changed'}
    $taskCreated=([datetimeoffset]$taskToken.creationTimeUtc).UtcDateTime.ToFileTimeUtc()
    $taskBefore=Get-WindowsRetainedTimesEvidence -Process $taskProcess -ExpectedCreationFileTime $taskCreated
    Write-NewJson (Join-Path $PSScriptRoot 'root-launch.json') @{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskPlan.productSource;mode='ordinary'
        executable=$taskPlan.productExecutable;exeSha256=$taskPlan.productExeSha256
        pid=$taskProcess.Id;creationTimeUtc=$taskToken.creationTimeUtc;applicationToken=$taskToken
        sandboxUsed=$false;cdpUsed=$false;toolSource=$taskPlan.toolSource
    }
    Write-NewJson (Join-Path $PSScriptRoot 'exit-observer-ready.json') @{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskProcess.Id;before=$taskBefore
        retainedHandle=$true;normalCloseByUtc=$taskHuman.normalCloseByUtc
        observationUntilUtc=([datetimeoffset]$taskHuman.expiresAtUtc).AddSeconds(120).ToString('o')
        uiInputInjected=$false;forcedTerminationAllowed=$false
    }
    $taskCensus=[Collections.Generic.List[object]]::new()
    $taskUntil=([datetimeoffset]$taskHuman.expiresAtUtc).AddSeconds(120)
    while(!$taskProcess.WaitForExit(500)){
        $taskNative=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'")
        $taskCensus.Add(@{atUtc=[datetimeoffset]::UtcNow.ToString('o');count=$taskNative.Count;matchesBoundRoot=($taskNative.Count -eq 1 -and $taskNative[0].ProcessId -eq $taskProcess.Id)})
        if($taskNative.Count -gt 1){throw 'Global single-instance invariant failed'}
        if([datetimeoffset]::UtcNow -ge $taskUntil){break}
    }
    $taskExited=$taskProcess.HasExited
    Write-NewJson (Join-Path $PSScriptRoot 'native-census.original.json') @($taskCensus)
    Write-NewJson (Join-Path $PSScriptRoot 'root-exit.json') @{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskProcess.Id;exitObserved=$taskExited
        exitCode=if($taskExited){$taskProcess.ExitCode}else{$null}
        kernelTimes=if($taskExited){Get-WindowsRetainedTimesEvidence -Process $taskProcess -ExpectedCreationFileTime $taskCreated -RequireExited}else{$null}
        nativeCount=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count
        retainedHandle=$true;forcedTermination=$false;closeActionRecordedSeparately=$true
    }
    if(!$taskExited){throw 'Exit observation expired; do not kill or retry'}
}catch{
    Write-NewJson (Join-Path $PSScriptRoot 'launch-observer-failure.json') @{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');exceptionType=$_.Exception.GetType().FullName
        message=$_.Exception.Message;scriptStackTrace=$_.ScriptStackTrace;automaticRetry=$false;forcedTermination=$false
    }
    throw
}finally{if($taskProcess){$taskProcess.Dispose()}}

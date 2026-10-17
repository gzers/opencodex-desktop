param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
$taskRoot=[IO.Path]::GetFullPath($OutputDirectory)
$taskRepo=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
. (Join-Path $PSScriptRoot 'windows-process-token.ps1')
if(!$taskRoot.StartsWith((Join-Path $taskRepo 'test')+'\',[StringComparison]::OrdinalIgnoreCase) -or
    (Test-Path -LiteralPath $taskRoot)){throw 'Fresh test-owned calibration-check output required'}
New-Item -ItemType Directory -Path $taskRoot|Out-Null
$taskAdmin=Join-Path $PSScriptRoot 'windows-query-calibration-admin.ps1'
$taskOrdinary=Join-Path $PSScriptRoot 'windows-query-calibration.ps1'
$taskDefault=& $taskAdmin|ConvertFrom-Json
if($taskDefault.status -cne 'prepared-only' -or $taskDefault.etwSessionStarted -or
    $taskDefault.workloadStarted -or $taskDefault.productStarted){throw 'Default admin entry must do nothing'}
$taskExpiredStart=[datetimeoffset]::UtcNow.AddHours(-1)
$taskAuth=[ordered]@{origin='human';explicitHumanStart=$true;automaticContinuation=$false
    authorizedAtUtc=$taskExpiredStart.ToString('o')
    expiresAtUtc=$taskExpiredStart.AddMinutes(30).ToString('o')}
$taskWindow=Join-Path $taskRoot 'expired-window.json'
$taskPlan=Join-Path $taskRoot 'expired-plan.json'
$taskAuth|ConvertTo-Json|Set-Content -LiteralPath $taskWindow -Encoding utf8
[ordered]@{authorization=$taskAuth}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath $taskPlan -Encoding utf8
$taskAdminDenied=$false
try{& $taskAdmin -Plan $taskPlan -RecordApproved|Out-Null}
catch{if(!$_.Exception.Message.Contains('Fresh human window and calibration cleanup reservation')){throw};$taskAdminDenied=$true}
$taskOrdinaryDenied=$false
$taskUncreated=Join-Path $taskRoot 'must-not-exist'
try{& $taskOrdinary -OutputDirectory $taskUncreated -NodeExecutable 'C:\Windows\System32\cmd.exe' -HumanWindow $taskWindow -DecoderDirectory $taskRoot -RecordApproved|Out-Null}
catch{if(!$_.Exception.Message.Contains('Fresh human window and calibration cleanup reservation')){throw};$taskOrdinaryDenied=$true}
if(!$taskAdminDenied -or !$taskOrdinaryDenied -or (Test-Path -LiteralPath $taskUncreated)){
    throw 'Expired authorization must reject before creating the workload root or requesting elevation'
}
$taskHarness=Get-WindowsHarnessToken
$taskMechanismExecuted=$false
if(!$taskHarness.elevated){
    $taskNode=(& node --print process.execPath).Trim()
    $taskMechanism=Join-Path $taskRepo ('test\runtime\calibration-check-'+[guid]::NewGuid().ToString('N'))
    & $taskOrdinary -OutputDirectory $taskMechanism -NodeExecutable $taskNode -MechanismOnly|Out-Null
    $taskResult=Get-Content -LiteralPath (Join-Path $taskMechanism 'mechanism-result.json') -Raw|ConvertFrom-Json
    if(!$taskResult.normalExitObserved -or $taskResult.exitCode -ne 0 -or $taskResult.workload.children.Count -ne 20 -or
        $taskResult.etwSessionStarted -or $taskResult.productStarted -or $taskResult.foregroundUsed -or $taskResult.externalNetworkUsed){
        throw 'Ordinary diagnostic mechanism must complete without ETW, product or foreground'
    }
    Copy-Item -LiteralPath (Join-Path $taskMechanism 'mechanism-result.json') -Destination (Join-Path $taskRoot 'mechanism-result.json')
    Copy-Item -LiteralPath (Join-Path $taskMechanism 'probe-result.json') -Destination (Join-Path $taskRoot 'probe-result.json')
    $taskMechanismExecuted=$true
}
[ordered]@{status='passed';defaultAdminEntryPreparedOnly=$true
    expiredAdminAndOrdinaryEntriesRejectedBeforeSideEffects=$true
    ordinaryMechanismExecuted=$taskMechanismExecuted
    ordinaryMechanismSkippedBecauseHarnessElevated=[bool]$taskHarness.elevated
    ownedChildren=$(if($taskMechanismExecuted){20}else{$null})
    ownedLoopbackRequests=$(if($taskMechanismExecuted){4}else{$null})
    ordinaryRootExitCode=$(if($taskMechanismExecuted){0}else{$null})
    etwSessionStarted=$false;productStarted=$false;foregroundUsed=$false
    rawKernelPositiveControlPassed=$false}|ConvertTo-Json|
    Set-Content -LiteralPath (Join-Path $taskRoot 'verification.json') -Encoding utf8
if($taskMechanismExecuted){Write-Output 'Calibration mechanism and expired-window guards passed; elevated kernel recording remains unexecuted.'}
else{Write-Output 'Calibration guards passed; ordinary mechanism explicitly skipped on the elevated harness.'}

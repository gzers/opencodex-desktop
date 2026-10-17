param([string]$Plan,[switch]$RecordApproved)
$ErrorActionPreference='Stop'
if(!$RecordApproved){
    [ordered]@{status='prepared-only';etwSessionStarted=$false;workloadStarted=$false
        productStarted=$false;foregroundAuthorized=$false}|ConvertTo-Json
    return
}
if(!$Plan){throw 'Current calibration plan required'}
. (Join-Path $PSScriptRoot 'windows-startup-pair-state.ps1')
. (Join-Path $PSScriptRoot 'windows-process-token.ps1')
$taskPlanPath=ConvertTo-StartupPairPath $Plan
$taskPlan=Get-Content -LiteralPath $taskPlanPath -Raw -Encoding utf8|ConvertFrom-Json
$taskAuth=$taskPlan.authorization
if($taskAuth.origin -cne 'human' -or $taskAuth.explicitHumanStart -isnot [bool] -or
    !$taskAuth.explicitHumanStart -or $taskAuth.automaticContinuation -isnot [bool] -or
    $taskAuth.automaticContinuation){throw 'Fresh human window required before calibration recording'}
$taskStart=[datetimeoffset]::Parse($taskAuth.authorizedAtUtc)
$taskEnd=[datetimeoffset]::Parse($taskAuth.expiresAtUtc)
$taskNow=[datetimeoffset]::UtcNow
$taskClose=$taskStart.AddMinutes(25)
if($taskEnd -lt $taskClose){$taskClose=$taskEnd}
if($taskEnd -le $taskStart -or ($taskEnd-$taskStart).TotalMinutes -gt 30 -or
    $taskNow -lt $taskStart -or $taskNow -ge $taskEnd -or $taskNow.AddSeconds(75) -ge $taskClose){
    throw 'Fresh human window and calibration cleanup reservation required'
}
$taskRepo=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$taskRoot=ConvertTo-StartupPairPath $taskPlan.runDirectory
if(!$taskRoot.StartsWith((Join-Path $taskRepo 'test/runtime')+'\',[StringComparison]::OrdinalIgnoreCase)){
    throw 'Owned calibration root required'
}
$taskProbe=ConvertTo-StartupPairPath $taskPlan.probeExecutable
if($taskProbe -cne (Join-Path $taskRoot 'owned-calibration-node.exe') -or
    (Get-FileHash -LiteralPath $taskProbe).Hash.ToLowerInvariant() -cne $taskPlan.nodeSha256){
    throw 'Exact owned installed-Node copy required'
}
$taskExpected=@('windows-owned-etw.cs','windows-query-counters.cs','windows-query-counters.ps1',
    'windows-query-calibration-admin.ps1')
if($taskPlan.methods.Count -ne $taskExpected.Count){throw 'Complete fixed observer methods required'}
foreach($taskMethod in $taskPlan.methods){
    if($taskMethod.name -cnotin $taskExpected -or
        (Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $taskMethod.name)).Hash.ToLowerInvariant() -cne $taskMethod.sha256){
        throw 'Calibration observer method differs'
    }
}
if(@($taskPlan.methods.name|Sort-Object -Unique).Count -ne $taskExpected.Count){
    throw 'Each fixed observer method must be bound once'
}
Assert-StartupPairNoProduct
Assert-WindowsProcessMode -Observation (Get-WindowsHarnessToken) -Mode administrator
$taskStop=Join-Path $taskRoot 'stop-etw.flag'
if(Test-Path -LiteralPath $taskStop){throw 'Never reuse a calibration attempt'}
& (Join-Path $PSScriptRoot 'windows-query-counters.ps1') -DecoderDirectory $taskPlan.decoderDirectory|Out-Null
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');controller=(Get-WindowsHarnessToken)
    approvedBackendEtwOnly=$true;productStarted=$false;uiInputInjected=$false}|ConvertTo-Json -Depth 5|
    Set-Content -LiteralPath (Join-Path $taskRoot 'administrator-observer-identity.json') -Encoding utf8
$taskTerminal=[OpenCodex.Diagnostics.ProductQueryCounters]::Observe(
    $taskProbe,(Join-Path $taskRoot 'native-calibration'),$taskStop,$taskClose.UtcDateTime,0)
if(!(Get-Content -LiteralPath $taskTerminal -Raw|ConvertFrom-Json).complete){
    throw 'Calibration event capture incomplete; raw terminal retained'
}
Write-Output $taskTerminal

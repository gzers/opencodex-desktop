param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
if(!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7){throw 'Windows PowerShell 7 required'}
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $taskOutput){throw 'New target-binding output required'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Require native0'}
New-Item -ItemType Directory -Path $taskOutput | Out-Null
. (Join-Path $PSScriptRoot 'windows-memory-target.ps1')
$taskChecks=0
function Assert-Target($Condition,[string]$Message){
    if(!$Condition){throw $Message}
    $script:taskChecks++
}
function Assert-TargetRefusal([scriptblock]$Action,[string]$Expected){
    $taskActual=$null
    try{& $Action | Out-Null}catch{$taskActual=$_.Exception.Message}
    Assert-Target ($null -ne $taskActual -and $taskActual.Contains($Expected)) "Expected refusal: $Expected"
}
$taskSelf=Get-Process -Id $PID
$taskLaunch=[pscustomobject]@{pid=$PID;creationTimeUtc=$taskSelf.StartTime.ToUniversalTime().ToString('o')
    executable=$taskSelf.Path;exeSha256=(Get-FileHash -LiteralPath $taskSelf.Path).Hash.ToLowerInvariant()}
$taskIdentity=[Security.Principal.WindowsIdentity]::GetCurrent()
try{
    $taskSidHash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData(
        [Text.Encoding]::UTF8.GetBytes($taskIdentity.User.Value))).ToLowerInvariant()
}finally{$taskIdentity.Dispose()}
$taskOne=$null
$taskTwo=$null
try{
    $taskOne=Open-WindowsMemoryTargetBinding -Launch $taskLaunch -ControlledSelf
    Assert-Target ($taskOne.targetPid -eq $PID -and $taskOne.rootPid -eq $PID) 'One actual harness generation required'
    Assert-Target ($taskOne.ownerSidSha256 -ceq $taskSidHash) 'TokenUser SID hash differs from the actual current identity'
    Assert-Target ($taskOne.rootToken.status -ceq 'observed' -and $taskOne.rootToken.processId -eq $PID) 'Actual primary token observation required'
    Assert-Target ($taskOne.controlledSelf -and !$taskOne.gpuRoleVerified) 'Controlled proof must not pretend to bind a GPU'
    Assert-Target (!$taskOne.rawSidSaved -and !$taskOne.rawCommandLineSaved -and !$taskOne.tracingStarted -and !$taskOne.productStarted -and !$taskOne.recordingExecutionAdmitted) 'Metadata scope changed'
    $taskReadAgain=Assert-WindowsMemoryTargetBinding -LeaseId $taskOne.leaseId
    Assert-Target ($taskReadAgain.targetCreationUtc -ceq $taskLaunch.creationTimeUtc -and $taskReadAgain.targetExeSha256 -ceq $taskLaunch.exeSha256) 'Retained identity changed'
    $taskTwo=Open-WindowsMemoryTargetBinding -Launch $taskLaunch -ControlledSelf
    Assert-Target ($taskTwo.leaseId -cne $taskOne.leaseId) 'Independent leases require unique capabilities'
    Close-WindowsMemoryTargetBinding -LeaseId $taskOne.leaseId
    Assert-TargetRefusal {Assert-WindowsMemoryTargetBinding -LeaseId $taskOne.leaseId} 'Unknown or closed'
    $taskReadTwo=Assert-WindowsMemoryTargetBinding -LeaseId $taskTwo.leaseId
    Assert-Target ($taskReadTwo.targetPid -eq $PID -and $taskReadTwo.ownerSidSha256 -ceq $taskSidHash) 'Closing a separate lease closed the retained process handle'
    Close-WindowsMemoryTargetBinding -LeaseId $taskTwo.leaseId
    Assert-TargetRefusal {Close-WindowsMemoryTargetBinding -LeaseId $taskTwo.leaseId} 'Unknown or closed'
    $taskBad=$taskLaunch | Select-Object *
    $taskBad.creationTimeUtc=([datetimeoffset]$taskLaunch.creationTimeUtc).AddTicks(1).ToString('o')
    Assert-TargetRefusal {Open-WindowsMemoryTargetBinding -Launch $taskBad -ControlledSelf} 'generation or image differs'
    $taskBad=$taskLaunch | Select-Object *
    $taskBad.exeSha256='0'*64
    Assert-TargetRefusal {Open-WindowsMemoryTargetBinding -Launch $taskBad -ControlledSelf} 'image digest differs'
    $taskBad=$taskLaunch | Select-Object *
    $taskBad.pid=$PID+1
    Assert-TargetRefusal {Open-WindowsMemoryTargetBinding -Launch $taskBad -ControlledSelf} 'this harness only'
    Assert-TargetRefusal {Open-WindowsMemoryTargetBinding -Launch $taskLaunch -RootLaunch $taskLaunch} 'One exact native desktop'
    Assert-Target ($script:OcxMemoryTargetLeases.Count -eq 0) 'Refusals or closes leaked a metadata lease'
    Assert-Target (@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count -eq 0) 'Native set changed'
    $taskOne | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskOutput 'controlled-binding.original.json') -Encoding utf8
    [ordered]@{passed=$true;checks=$taskChecks;actualCurrentTokenUserMatched=$true
        independentRetainedLeasesTested=$true;generationTickMismatchRefused=$true
        productGpuBindingExecuted=$false;tracingStarted=$false;productStarted=$false
        recordingExecutionAdmitted=$false;rawSidSaved=$false;rawCommandLineSaved=$false
        sourceMethodSha256=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'windows-memory-target.ps1')).Hash.ToLowerInvariant()} |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskOutput 'result.json') -Encoding utf8
    Get-Content -LiteralPath (Join-Path $taskOutput 'result.json')
}finally{
    foreach($taskId in @($script:OcxMemoryTargetLeases.Keys)){Close-WindowsMemoryTargetBinding -LeaseId $taskId}
    $taskSelf.Dispose()
}

param([string]$Preparation,[string]$Methods)
$ErrorActionPreference='Stop'
. (Join-Path $Methods 'platform-binding.ps1')
$taskBefore=Assert-RecoveryPlatformRootsAbsent -PreparationDirectory $Preparation
$taskBefore | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $Preparation 'preflight-ready-readback.json') -Encoding utf8
$taskProbe='C:\Users\15119\AppData\Roaming\com.gzers.opencodex.desktop'
if([IO.Path]::GetFullPath($taskProbe) -cne $taskProbe -or (Test-Path -LiteralPath $taskProbe)){throw 'Exact absent manager probe root required'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Require native0'}
$null=New-Item -ItemType Directory -Path $taskProbe
$taskProbeIdentity=Get-RecoveryDirectoryIdentity -Path $taskProbe
$taskRejected=$false
try {$null=Assert-RecoveryPlatformRootsAbsent -PreparationDirectory $Preparation}catch{
    if($_.Exception.Message -cne 'Platform app-data root exists, including an empty root; do not launch recovery'){throw}
    $taskRejected=$true
}finally{
    if(@(Get-ChildItem -LiteralPath $taskProbe -Force).Count){throw 'Probe gained contents; preserve it rather than deleting'}
    $taskNow=Get-RecoveryDirectoryIdentity -Path $taskProbe
    if($taskNow.fileId -cne $taskProbeIdentity.fileId){throw 'Probe directory changed; preserve it'}
    # Nonrecursive deletion of this exact newly created empty application root.
    Remove-Item -LiteralPath $taskProbe
}
if(!$taskRejected){throw 'Empty platform root was incorrectly admitted'}
$taskAfter=Assert-RecoveryPlatformRootsAbsent -PreparationDirectory $Preparation
$taskNoHuman=Join-Path $Preparation 'guard-no-human'
$null=New-Item -ItemType Directory -Path $taskNoHuman
$taskRefusals=@()
foreach($taskName in @('prepare-recovery-v2.ps1','launch-recovery-v2.ps1')){
    $taskRejected=$false
    try {& (Join-Path $Methods $taskName) -RunDirectory $taskNoHuman -PreparationDirectory $Preparation}catch{
        if($_.Exception.Message -notlike '*human-window.json*'){throw}
        $taskRejected=$true
        $taskRefusals+=@{method=$taskName;rejected=$true;reason='human-window receipt missing'}
    }
    if(!$taskRejected){throw 'Missing human window was admitted'}
}
if(@(Get-ChildItem -LiteralPath $taskNoHuman -Force).Count){throw 'No-human refusal produced runtime mutations'}
$taskAfter=Assert-RecoveryPlatformRootsAbsent -PreparationDirectory $Preparation
$taskPointer=(Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop').StartupDataRootV1
$taskPlan=Get-Content -LiteralPath (Join-Path $Preparation 'profile-isolation-plan.json') -Raw | ConvertFrom-Json
$taskPointerSha=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($taskPointer))).ToLowerInvariant()
if($taskPointerSha -cne $taskPlan.startupPointerSha256){throw 'Pointer changed during guard checks'}
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');readyReadback=$taskAfter
    emptyPlatformRootRejected=$taskRejected;emptyProbeRemovedWithoutRecursion=$true
    noHumanWindowRefusals=$taskRefusals;startupPointerUnchangedExact=$true
    productStarted=$false;uiInputInjected=$false;dataProfileContentsDeleted=$false;fullRecoveryGatePassed=$false} |
    ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $Preparation 'guard-validation.original.json') -Encoding utf8

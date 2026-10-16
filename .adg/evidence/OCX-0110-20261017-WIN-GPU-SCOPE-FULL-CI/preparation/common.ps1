$ErrorActionPreference='Stop'
$taskRepo='C:\Users\15119\.codex\worktrees\windows-working-set\opencodex-desktop'
$taskDocs='D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop'
$taskPlan=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'plan.json') -Raw | ConvertFrom-Json
$taskToolAdmissionPath=$taskPlan.toolAdmissionPath
function Write-NewJson([string]$Path,$Value){
    $taskBytes=[Text.Encoding]::UTF8.GetBytes(($Value | ConvertTo-Json -Depth 15)+"`n")
    $taskStream=[IO.FileStream]::new($Path,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
    try{$taskStream.Write($taskBytes,0,$taskBytes.Length)}finally{$taskStream.Dispose()}
}
function Assert-HumanWindow {
    if(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'foreground-released.json')){throw 'This foreground window has already been released'}
    $taskHuman=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'human-window.json') -Raw | ConvertFrom-Json
    $taskNow=[datetimeoffset]::UtcNow
    if(!$taskHuman.explicitHumanStart -or $taskHuman.automaticContinuation -ne $false -or
       $taskHuman.userMessage -cne '开始前台验证' -or $taskHuman.scope -cne 'one-b4-gpu-allocation-diagnostic' -or
       ([datetimeoffset]$taskHuman.authorizedAtUtc) -gt $taskNow -or ([datetimeoffset]$taskHuman.expiresAtUtc) -le $taskNow -or
       (([datetimeoffset]$taskHuman.expiresAtUtc)-([datetimeoffset]$taskHuman.authorizedAtUtc)).TotalSeconds -gt 1800){
        throw 'Current explicit human foreground window required'
    }
    return $taskHuman
}
function Assert-DiagnosticSource {
    . (Join-Path $taskRepo 'test/smoke/windows-memory-target.ps1')
    $taskAdmission=Assert-WindowsMemoryMethodAdmission -Path $taskToolAdmissionPath
    if($taskAdmission.source -cne $taskPlan.toolSource -or $taskAdmission.runId -ne $taskPlan.toolCiRunId){throw 'Wrong exact source or first tool CI'}
    if((Get-FileHash -LiteralPath $taskPlan.productExecutable).Hash.ToLowerInvariant() -cne $taskPlan.productExeSha256){throw 'Exact b4 image changed'}
    $taskPointer=[string](Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop').StartupDataRootV1
    $taskHash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($taskPointer))).ToLowerInvariant()
    if($taskHash -cne $taskPlan.startupPointerSha256){throw 'Isolated anchor pointer changed'}
    $taskPre=Get-Content -LiteralPath $taskPlan.identityPreflightPath -Raw | ConvertFrom-Json
    $taskContract=Get-Content -LiteralPath (Join-Path $taskDocs '.adg/evidence/OCX-0110-20261016-WIN-B4-FRONT-PREP/contract.json') -Raw | ConvertFrom-Json
    if((Get-FileHash -LiteralPath $taskContract.directoryIdentityMethod).Hash.ToLowerInvariant() -cne $taskContract.directoryIdentityMethodSha256){throw 'Frozen directory method changed'}
    . $taskContract.directoryIdentityMethod
    $taskOriginal=Get-RecoveryDirectoryIdentity -Path $taskPre.originalProfile.path
    if($taskOriginal.fileId -cne $taskPre.originalProfile.fileId -or
       $taskOriginal.creationFileTime -cne $taskPre.originalProfile.creationFileTime){throw 'Original profile generation changed'}
    if((Get-FileHash -LiteralPath (Join-Path $taskPre.originalProfile.path 'manager-state/preferences.json')).Hash.ToLowerInvariant() -cne $taskPre.originalPreferencesSha256){throw 'Restored original preferences changed'}
    $taskFixture=Get-RecoveryDirectoryIdentity -Path $taskPlan.isolatedFixture.path
    if($taskFixture.fileId -cne $taskPlan.isolatedFixture.fileId -or $taskFixture.creationFileTime -cne $taskPlan.isolatedFixture.creationFileTime){throw 'Isolated fixture generation changed'}
}

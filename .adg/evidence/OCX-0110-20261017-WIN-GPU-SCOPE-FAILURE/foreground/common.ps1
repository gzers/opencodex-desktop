$ErrorActionPreference='Stop'
$taskRepo='C:\Users\15119\.codex\worktrees\windows-working-set\opencodex-desktop'
$taskDocs='D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop'
$taskPrep=Join-Path $taskRepo 'test/runtime/bound-gpu-memory-prep-20261017T0020'
$taskPlan=Get-Content -LiteralPath (Join-Path $taskPrep 'next-diagnostic-plan.json') -Raw | ConvertFrom-Json
function Write-NewJson([string]$Path,$Value){
    $taskBytes=[Text.Encoding]::UTF8.GetBytes(($Value | ConvertTo-Json -Depth 15)+"`n")
    $taskStream=[IO.FileStream]::new($Path,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
    try{$taskStream.Write($taskBytes,0,$taskBytes.Length)}finally{$taskStream.Dispose()}
}
function Assert-HumanWindow {
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
    [void](Assert-WindowsMemoryMethodAdmission -Path (Join-Path $taskPrep 'ci-complete-admission.json'))
    if((Get-FileHash -LiteralPath $taskPlan.productExecutable).Hash.ToLowerInvariant() -cne $taskPlan.productExeSha256){throw 'Exact b4 image changed'}
    $taskPointer=[string](Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop').StartupDataRootV1
    $taskHash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($taskPointer))).ToLowerInvariant()
    if($taskHash -cne $taskPlan.startupPointerSha256){throw 'Isolated anchor pointer changed'}
    $taskPre=Get-Content -LiteralPath (Join-Path $taskPrep 'diagnostic-preflight.original.json') -Raw | ConvertFrom-Json
    $taskPrefs=Join-Path $taskPre.originalProfile.path 'manager-state/preferences.json'
    if((Get-FileHash -LiteralPath $taskPrefs).Hash.ToLowerInvariant() -cne $taskPre.originalPreferencesSha256){throw 'Restored original preferences changed'}
}

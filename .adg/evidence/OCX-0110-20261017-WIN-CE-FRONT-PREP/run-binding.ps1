# Preparation is not authorization; create a receipt only after a fresh human signal.
function Assert-PreparedIdleRun {
    param([Parameter(Mandatory)][string]$RunDirectory,[switch]$RequireCurrentHumanWindow)
    $taskDiagnostic='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop'
    $taskAllowed=[IO.Path]::GetFullPath((Join-Path $taskDiagnostic 'test/runtime'))
    $taskRun=[IO.Path]::GetFullPath($RunDirectory)
    if(!$taskRun.StartsWith($taskAllowed+'\',[StringComparison]::OrdinalIgnoreCase) -or
       !(Test-Path -LiteralPath $taskRun -PathType Container)){throw 'Existing owned runtime directory required'}
    $taskItem=Get-Item -LiteralPath $taskRun
    while($taskItem.FullName.Length -ge $taskAllowed.Length){
        if($taskItem.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Runtime path alias refused'}
        if($taskItem.FullName -eq $taskAllowed){break}
        $taskItem=$taskItem.Parent
    }
    $taskContract=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'contract.json') -Raw | ConvertFrom-Json
    $taskCandidateVerification=Get-Content -LiteralPath $taskContract.candidateVerificationPath -Raw | ConvertFrom-Json
    if($taskCandidateVerification.source -cne $taskContract.candidateSource -or
       $taskCandidateVerification.status -cne 'verified-exact-ce-candidate' -or
       (Get-FileHash -LiteralPath $taskContract.candidateVerificationPath).Hash.ToLowerInvariant() -cne $taskContract.candidateVerificationSha256 -or
       (Get-FileHash -LiteralPath $taskContract.candidateManifestPath).Hash.ToLowerInvariant() -cne $taskContract.candidateManifestSha256){throw 'Exact product candidate evidence changed'}
    foreach($taskEntry in $taskContract.methodWorkingFileSha256.psobject.Properties){
        if((Get-FileHash -LiteralPath (Join-Path $taskDiagnostic $taskEntry.Name)).Hash.ToLowerInvariant() -cne $taskEntry.Value){throw 'Committed observation method changed'}
    }
    if((Get-FileHash -LiteralPath $taskContract.sidMethodPath).Hash.ToLowerInvariant() -cne $taskContract.sidMethodSha256){throw 'Frozen SID companion method changed'}
    if((Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'retained-process-times.ps1')).Hash.ToLowerInvariant() -cne $taskContract.kernelTimesMethodSha256){throw 'Kernel time method changed'}
    if((Get-FileHash -LiteralPath $taskContract.directoryIdentityMethod).Hash.ToLowerInvariant() -cne $taskContract.directoryIdentityMethodSha256){throw 'Directory identity method changed'}
    if((Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'preparation-readback.json')).Hash.ToLowerInvariant() -cne $taskContract.preparationReadbackSha256){throw 'Fixture preparation binding changed'}
    if($RequireCurrentHumanWindow){
        if(Test-Path -LiteralPath (Join-Path $taskRun 'foreground-released.json')){throw 'Foreground window already released; a new human signal requires a new run'}
        $taskHuman=Get-Content -LiteralPath (Join-Path $taskRun 'human-window.json') -Raw | ConvertFrom-Json
        $taskNow=[datetimeoffset]::UtcNow
        if($taskHuman.explicitHumanStart -ne $true -or
           $taskHuman.scope -cne 'one-single-instance-foreground-idle-pair' -or
           !$taskHuman.userMessage -or $taskHuman.automaticContinuation -ne $false -or
           ([datetimeoffset]$taskHuman.authorizedAtUtc) -gt $taskNow -or
           ([datetimeoffset]$taskHuman.expiresAtUtc) -le $taskNow -or
           (([datetimeoffset]$taskHuman.expiresAtUtc)-([datetimeoffset]$taskHuman.authorizedAtUtc)).TotalMinutes -gt 30){
            throw 'A current explicit human foreground window is required; preparation grants none'
        }
    }
    return $taskRun
}

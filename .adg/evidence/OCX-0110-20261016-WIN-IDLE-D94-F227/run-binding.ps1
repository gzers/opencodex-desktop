# Preparation is not authorization. A new human signal must precede creation of
# the execution receipt. Never fill that receipt from a scheduler wake-up.
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
    foreach($taskEntry in $taskContract.methodWorkingFileSha256.psobject.Properties){
        if((Get-FileHash -LiteralPath (Join-Path $taskDiagnostic $taskEntry.Name)).Hash.ToLowerInvariant() -cne $taskEntry.Value){throw 'Committed observation method changed'}
    }
    if((Get-FileHash -LiteralPath $taskContract.sidMethodPath).Hash.ToLowerInvariant() -cne $taskContract.sidMethodSha256){throw 'Frozen SID companion method changed'}
    if($RequireCurrentHumanWindow){
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

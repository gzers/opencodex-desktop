function Assert-PreparedIdleRun {
    param([string]$RunDirectory,[switch]$RequireCurrentHumanWindow)
    $taskExpected='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop\test\runtime\foreground-99-roots-20261017T172126'
    if([IO.Path]::GetFullPath($RunDirectory) -cne $taskExpected){throw 'Exact owned runtime required'}
    $taskContract=Get-Content -LiteralPath (Join-Path $taskExpected 'contract.json') -Raw | ConvertFrom-Json
    if($taskContract.candidateSource -cne '99a62e4ad35e9404cb08922571ad8b77f89214dd' -or
       (Get-FileHash -LiteralPath $taskContract.recoveryExecutable).Hash.ToLowerInvariant() -cne $taskContract.candidateExeSha256){throw 'Exact installed candidate required'}
    if($RequireCurrentHumanWindow){
        $taskHuman=Get-Content -LiteralPath (Join-Path $taskExpected 'human-window.json') -Raw | ConvertFrom-Json
        if(!$taskHuman.explicitHumanStart -or $taskHuman.automaticContinuation -or
           $taskHuman.scope -cne 'one-single-instance-data-root-recovery-and-retention' -or
           ([datetimeoffset]$taskHuman.expiresAtUtc) -le [datetimeoffset]::UtcNow -or
           (([datetimeoffset]$taskHuman.expiresAtUtc)-([datetimeoffset]$taskHuman.authorizedAtUtc)).TotalMinutes -gt 30){throw 'Current human window required'}
    }
    return $taskExpected
}
function Assert-PreparedRecoveryDirectory {
    param([string]$Directory)
    $taskExpected=Join-Path $PSScriptRoot 'install with spaces'
    if([IO.Path]::GetFullPath($Directory) -cne $taskExpected){throw 'Exact installed directory required'}
    $taskCursor=Get-Item -LiteralPath $Directory
    while($taskCursor){if($taskCursor.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Path alias refused'};$taskCursor=$taskCursor.Parent}
    return $taskExpected
}

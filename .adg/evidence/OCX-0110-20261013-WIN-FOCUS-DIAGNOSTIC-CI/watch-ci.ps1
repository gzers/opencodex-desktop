$ErrorActionPreference = 'Stop'
$taskEvidence = $PSScriptRoot
$taskSource = 'be7395acfce0ec1e36b83300b576db408f8c35bb'
$taskRunId = 38352525092
$taskClock = [Diagnostics.Stopwatch]::StartNew()
$taskPrevious = $null
$taskErrors = 0
$taskTerminalPath = Join-Path $taskEvidence 'watcher-terminal.json'
if (Test-Path -LiteralPath $taskTerminalPath) { throw 'Do not restart or overwrite this observer' }
while ($taskClock.Elapsed.TotalMinutes -lt 20) {
    $taskCredential = @{}
    $taskCredentialLines = $null
    $taskHeaders = $null
    $taskOldPrompt = $env:GIT_TERMINAL_PROMPT
    $env:GIT_TERMINAL_PROMPT = '0'
    try {
        $taskCredentialLines = "protocol=https`nhost=github.com`n`n" | git credential fill 2>$null
        if ($LASTEXITCODE -ne 0) { throw 'Authenticated Git credential unavailable' }
        foreach ($taskLine in $taskCredentialLines) {
            $taskPair = $taskLine -split '=', 2
            if ($taskPair.Length -eq 2) { $taskCredential[$taskPair[0]] = $taskPair[1] }
        }
        if (-not $taskCredential.password) { throw 'Authenticated Git credential missing' }
        $taskHeaders = @{
            Authorization = 'Bearer ' + $taskCredential.password
            Accept = 'application/vnd.github+json'
            'X-GitHub-Api-Version' = '2022-11-28'
        }
        $taskUri = 'https://api.github.com/repos/gzers/opencodex-desktop/actions/runs/' + $taskRunId
        $taskRun = Invoke-RestMethod -Uri $taskUri -Headers $taskHeaders -TimeoutSec 20
        if ($taskRun.head_sha -cne $taskSource -or $taskRun.event -cne 'push' -or
            $taskRun.head_branch -cne 'feature/0.1.10-update-center-focus') {
            throw 'Exact diagnostic source identity differs'
        }
        $taskJobs = Invoke-RestMethod -Uri ($taskUri + '/jobs?per_page=100') -Headers $taskHeaders -TimeoutSec 20
        $taskSnapshot = [ordered]@{
            atUtc = [datetime]::UtcNow.ToString('o'); runId = $taskRunId; sha = $taskSource
            status = $taskRun.status; conclusion = $taskRun.conclusion
            jobs = @($taskJobs.jobs | Select-Object id, name, status, conclusion)
            activeOrFailedWindowsSteps = @(
                ($taskJobs.jobs | Where-Object name -eq 'backend-windows').steps |
                Where-Object { $_.status -eq 'in_progress' -or $_.conclusion -eq 'failure' } |
                Select-Object name, status, conclusion
            )
            watcherPid = $PID; watcherCreationUtc = (Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o')
            elapsedMs = $taskClock.Elapsed.TotalMilliseconds; observationErrors = $taskErrors
            foregroundInputIssued = $false; productRestarted = $false; ciRerunRequested = $false
            mainIntegrated = $false; diagnosticCiAccepted = $false; originalCiFailureCleared = $false
        }
        $taskSnapshot | ConvertTo-Json -Depth 7 -Compress |
            Add-Content -LiteralPath (Join-Path $taskEvidence 'watcher-observations.ndjson') -Encoding utf8
        $taskPhase = (@($taskRun.status, $taskRun.conclusion) + @($taskSnapshot.jobs | ForEach-Object { $_.name + ':' + $_.status + ':' + $_.conclusion }) +
            @($taskSnapshot.activeOrFailedWindowsSteps | ForEach-Object { $_.name + ':' + $_.status })) -join '|'
        if ($taskPhase -cne $taskPrevious) {
            $taskSnapshot | ConvertTo-Json -Depth 7 -Compress | Write-Output
            $taskPrevious = $taskPhase
        }
        if ($taskRun.status -eq 'completed') {
            $taskRun | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $taskEvidence 'run-terminal.json') -Encoding utf8
            $taskJobs | ConvertTo-Json -Depth 14 | Set-Content -LiteralPath (Join-Path $taskEvidence 'jobs-terminal.json') -Encoding utf8
            $taskSnapshot | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskTerminalPath -Encoding utf8
            return
        }
    } catch {
        $taskErrors++
        [ordered]@{
            atUtc = [datetime]::UtcNow.ToString('o'); observationError = $_.Exception.Message
            observationErrors = $taskErrors; ciTerminalClaimed = $false; productRestarted = $false
        } | ConvertTo-Json -Compress | Write-Output
    } finally {
        $taskCredential.Clear(); $taskCredentialLines = $null; $taskHeaders = $null
        $taskLine = $null; $taskPair = $null; $env:GIT_TERMINAL_PROMPT = $taskOldPrompt
    }
    Start-Sleep -Seconds 45
}
[ordered]@{
    atUtc = [datetime]::UtcNow.ToString('o'); runId = $taskRunId; sha = $taskSource
    status = 'bounded-observation-ended'; observationErrors = $taskErrors
    elapsedMs = $taskClock.Elapsed.TotalMilliseconds; ciTerminalClaimed = $false
    productRestarted = $false; ciRerunRequested = $false; foregroundInputIssued = $false
} | ConvertTo-Json | Set-Content -LiteralPath $taskTerminalPath -Encoding utf8

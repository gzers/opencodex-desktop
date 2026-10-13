param([ValidateRange(1, 45)][int]$MaximumMinutes = 45)
$ErrorActionPreference = 'Stop'
$taskEvidence = $PSScriptRoot
$taskAdmission = Get-Content -LiteralPath (Join-Path $taskEvidence 'candidate-admission.json') -Raw | ConvertFrom-Json
$taskSource = 'be7395acfce0ec1e36b83300b576db408f8c35bb'
$taskBranch = 'feature/0.1.10-update-center-focus'
if ($taskAdmission.sourceCommit -cne $taskSource -or -not $taskAdmission.requiredCiJobsPassed -or
    -not $taskAdmission.remoteSourceMatchedAfterDispatch) {
    throw 'Verified candidate admission required'
}
$taskTerminal = Join-Path $taskEvidence 'candidate-observer-terminal.json'
if (Test-Path -LiteralPath $taskTerminal) { throw 'Do not restart a terminal candidate observer' }
$taskClock = [Diagnostics.Stopwatch]::StartNew()
$taskRunId = $taskAdmission.candidateRunId
$taskErrors = 0
$taskPreviousPhase = $null
while ($taskClock.Elapsed.TotalMinutes -lt $MaximumMinutes) {
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
        $taskRepo = 'https://api.github.com/repos/gzers/opencodex-desktop'
        if (-not $taskRunId) {
            $taskRuns = Invoke-RestMethod -Uri ($taskRepo + '/actions/workflows/windows-candidate.yml/runs?head_sha=' + $taskSource + '&per_page=100') `
                -Headers $taskHeaders -TimeoutSec 20
            $taskMatches = @($taskRuns.workflow_runs | Where-Object {
                $_.head_sha -ceq $taskSource -and $_.head_branch -ceq $taskBranch -and $_.event -ceq 'workflow_dispatch'
            })
            if ($taskMatches.Count -gt 1) { throw 'Multiple exact candidate runs; no new dispatch or replacement permitted' }
            if ($taskMatches.Count -eq 0) {
                Write-Output 'Existing dispatch has not appeared yet; do not dispatch again'
                continue
            }
            $taskRunId = $taskMatches[0].id
            $taskMatches[0] | ConvertTo-Json -Depth 14 |
                Set-Content -LiteralPath (Join-Path $taskEvidence 'candidate-run-first-observed.json') -Encoding utf8
        }
        $taskUri = $taskRepo + '/actions/runs/' + $taskRunId
        $taskRun = Invoke-RestMethod -Uri $taskUri -Headers $taskHeaders -TimeoutSec 20
        if ($taskRun.head_sha -cne $taskSource -or $taskRun.head_branch -cne $taskBranch -or
            $taskRun.event -cne 'workflow_dispatch' -or $taskRun.workflow_id -ne $taskAdmission.workflowId) {
            throw 'Actual candidate run source or workflow identity differs'
        }
        $taskJobs = Invoke-RestMethod -Uri ($taskUri + '/jobs?per_page=100') -Headers $taskHeaders -TimeoutSec 20
        if (@($taskJobs.jobs | Where-Object name -cne 'package').Count) { throw 'Unexpected candidate job scope' }
        $taskSnapshot = [ordered]@{
            atUtc = [datetime]::UtcNow.ToString('o'); sourceCommit = $taskSource
            runId = $taskRunId; status = $taskRun.status; conclusion = $taskRun.conclusion
            actualRunSourceVerified = $true; jobs = @($taskJobs.jobs | Select-Object id, name, status, conclusion)
            activeOrFailedSteps = @($taskJobs.jobs.steps | Where-Object {
                $_.status -eq 'in_progress' -or $_.conclusion -eq 'failure'
            } | Select-Object name, status, conclusion)
            observerPid = $PID; observerCreationUtc = (Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o')
            elapsedMs = $taskClock.Elapsed.TotalMilliseconds; observationErrors = $taskErrors
            localInstallerInvoked = $false; localExecutableStarted = $false; foregroundInputIssued = $false
            originalDesktopOrCycleReplaced = $false; ciRerunRequested = $false; redispatched = $false
            mainIntegrated = $false; originalFailedCiFailureCleared = $false; originalFailedCiCauseIsolated = $false
            candidateArtifactAccepted = $false; finalNativeAccepted = $false; formalTaskReceipt = $false
            releasePublished = $false; channelPromoted = $false; releaseAccepted = $false
        }
        $taskSnapshot | ConvertTo-Json -Depth 8 -Compress |
            Add-Content -LiteralPath (Join-Path $taskEvidence 'candidate-observations.ndjson') -Encoding utf8
        $taskPhase = (@($taskRun.status, $taskRun.conclusion) + @($taskSnapshot.activeOrFailedSteps |
            ForEach-Object { $_.name + ':' + $_.status + ':' + $_.conclusion })) -join '|'
        if ($taskPhase -cne $taskPreviousPhase) {
            $taskSnapshot | ConvertTo-Json -Depth 8 -Compress | Write-Output
            $taskPreviousPhase = $taskPhase
        }
        if ($taskRun.status -eq 'completed') {
            $taskRun | ConvertTo-Json -Depth 14 |
                Set-Content -LiteralPath (Join-Path $taskEvidence 'candidate-run-terminal.json') -Encoding utf8
            $taskJobs | ConvertTo-Json -Depth 14 |
                Set-Content -LiteralPath (Join-Path $taskEvidence 'candidate-jobs-terminal.json') -Encoding utf8
            $taskSnapshot | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskTerminal -Encoding utf8
            return
        }
    } catch {
        $taskErrors++
        [ordered]@{
            atUtc = [datetime]::UtcNow.ToString('o'); runId = $taskRunId
            observationError = $_.Exception.Message; observationErrors = $taskErrors
            redispatched = $false; terminalClaimed = $false
        } | ConvertTo-Json -Compress | Write-Output
    } finally {
        $taskCredential.Clear(); $taskCredentialLines = $null; $taskHeaders = $null
        $taskLine = $null; $taskPair = $null; $env:GIT_TERMINAL_PROMPT = $taskOldPrompt
        if (-not $taskRunId) { Start-Sleep -Seconds 45 }
    }
    Start-Sleep -Seconds 45
}
[ordered]@{
    atUtc = [datetime]::UtcNow.ToString('o'); sourceCommit = $taskSource; runId = $taskRunId
    status = 'bounded-observation-ended-without-candidate-terminal-claim'
    elapsedMs = $taskClock.Elapsed.TotalMilliseconds; observationErrors = $taskErrors
    redispatched = $false; localExecutableStarted = $false; localInstallerInvoked = $false
    foregroundInputIssued = $false; finalNativeAccepted = $false; releaseAccepted = $false
} | ConvertTo-Json | Set-Content -LiteralPath $taskTerminal -Encoding utf8

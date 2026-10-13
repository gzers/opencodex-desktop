$ErrorActionPreference = 'Stop'
$taskEvidence = $PSScriptRoot
$taskSource = 'be7395acfce0ec1e36b83300b576db408f8c35bb'
$taskRepo = 'https://api.github.com/repos/gzers/opencodex-desktop'
$taskReadback = Join-Path $taskEvidence 'pr-readback.json'
if (Test-Path -LiteralPath $taskReadback) { throw 'Do not overwrite an earlier PR readback' }
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
    $taskPr = Invoke-RestMethod -Uri ($taskRepo + '/pulls/1') -Headers $taskHeaders -TimeoutSec 20
    if ($taskPr.head.sha -cne $taskSource -or $taskPr.head.ref -cne 'feature/0.1.10-update-center-focus' -or
        $taskPr.base.ref -cne 'main' -or $taskPr.base.sha -cne 'eae85322ebca3b755c034db97cbebf2ea5bf71fc' -or
        -not $taskPr.draft -or $taskPr.state -cne 'open' -or $taskPr.merged -or $taskPr.auto_merge) {
        throw 'PR source, base or expected draft state changed'
    }
    $taskFilesResponse = Invoke-WebRequest -Uri ($taskRepo + '/pulls/1/files?per_page=100') -Headers $taskHeaders -TimeoutSec 20
    $taskFiles = ConvertFrom-Json -InputObject $taskFilesResponse.Content -NoEnumerate
    if ($taskFiles.Count -ne 5 -or @($taskFiles | Where-Object { $_.filename -notmatch '^(apps/|\.github/)' }).Count) {
        throw 'Unexpected code review scope or branch ownership'
    }
    $taskRuns = Invoke-RestMethod -Uri ($taskRepo + '/actions/runs?head_sha=' + $taskSource + '&per_page=100') `
        -Headers $taskHeaders -TimeoutSec 20
    $taskRelevantRuns = @($taskRuns.workflow_runs | Where-Object { $_.head_sha -ceq $taskSource -and $_.name -ceq 'CI' })
    $taskPr | ConvertTo-Json -Depth 18 | Set-Content -LiteralPath (Join-Path $taskEvidence 'draft-pr-current.json') -Encoding utf8
    $taskFiles | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $taskEvidence 'draft-pr-files.json') -Encoding utf8
    $taskRuns | ConvertTo-Json -Depth 16 | Set-Content -LiteralPath (Join-Path $taskEvidence 'head-source-workflow-runs.json') -Encoding utf8
    [ordered]@{
        atUtc = [datetime]::UtcNow.ToString('o'); number = 1; url = $taskPr.html_url; title = $taskPr.title
        sourceCommit = $taskPr.head.sha; baseCommit = $taskPr.base.sha; state = $taskPr.state; draft = $taskPr.draft
        merged = $taskPr.merged; autoMerge = $taskPr.auto_merge; changedFiles = $taskFiles.Count
        sourceCiRuns = @($taskRelevantRuns | Select-Object id, name, event, head_sha, status, conclusion)
        mergeOrReadyForReviewRequested = $false; releaseRequested = $false
        foregroundInputIssued = $false; originalDesktopOrCycleChanged = $false
        localCandidateStarted = $false; nativeFocusFixAccepted = $false; fullNativeAccepted = $false
        originalCiFailureCleared = $false; originalCiCauseIsolated = $false; formalTaskReceipt = $false
    } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskReadback -Encoding utf8
    Get-Content -LiteralPath $taskReadback
} finally {
    $taskCredential.Clear(); $taskCredentialLines = $null; $taskHeaders = $null
    $taskLine = $null; $taskPair = $null; $env:GIT_TERMINAL_PROMPT = $taskOldPrompt
}

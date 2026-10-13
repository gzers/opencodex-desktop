$ErrorActionPreference = 'Stop'
$taskEvidence = $PSScriptRoot
$taskSource = 'be7395acfce0ec1e36b83300b576db408f8c35bb'
$taskBranch = 'feature/0.1.10-update-center-focus'
$taskRepo = 'https://api.github.com/repos/gzers/opencodex-desktop'
$taskTitle = 'fix(updates): 切换详情视图后保留弹窗焦点（待Windows原生复验）'
$taskOutput = Join-Path $taskEvidence 'draft-pr-created.json'
if (Test-Path -LiteralPath $taskOutput) { throw 'Do not recreate or overwrite a draft PR result' }
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
    $taskExistingResponse = Invoke-WebRequest -Uri ($taskRepo + '/pulls?state=all&head=gzers%3Afeature%2F0.1.10-update-center-focus&per_page=100') `
        -Headers $taskHeaders -TimeoutSec 20
    $taskExisting = ConvertFrom-Json -InputObject $taskExistingResponse.Content -NoEnumerate
    if ($taskExisting.Count -ne 0) { throw 'Existing exact-branch PR found; reuse it instead of creating another' }
    $taskFeature = Invoke-RestMethod -Uri ($taskRepo + '/git/ref/heads/' + $taskBranch) -Headers $taskHeaders -TimeoutSec 20
    if ($taskFeature.object.sha -cne $taskSource) { throw 'Remote feature source changed' }
    $taskMain = Invoke-RestMethod -Uri ($taskRepo + '/git/ref/heads/main') -Headers $taskHeaders -TimeoutSec 20
    if ($taskMain.object.sha -cne 'eae85322ebca3b755c034db97cbebf2ea5bf71fc') { throw 'Review base changed; reassess the diff' }
    $taskCi = Invoke-RestMethod -Uri ($taskRepo + '/actions/runs/38352525092') -Headers $taskHeaders -TimeoutSec 20
    $taskCandidate = Invoke-RestMethod -Uri ($taskRepo + '/actions/runs/38355803684') -Headers $taskHeaders -TimeoutSec 20
    foreach ($taskRun in @($taskCi, $taskCandidate)) {
        if ($taskRun.head_sha -cne $taskSource -or $taskRun.status -cne 'completed' -or $taskRun.conclusion -cne 'success') {
            throw 'Exact-source CI or candidate result changed'
        }
    }
    $taskBody = Get-Content -LiteralPath (Join-Path $taskEvidence 'pr-body.md') -Raw -Encoding utf8
    $taskPayload = @{
        title = $taskTitle; body = $taskBody; head = $taskBranch; base = 'main'
        draft = $true; maintainer_can_modify = $false
    } | ConvertTo-Json -Depth 5
    [ordered]@{
        atUtc = [datetime]::UtcNow.ToString('o'); action = 'create-draft-pr'; sourceCommit = $taskSource
        baseCommit = $taskMain.object.sha; title = $taskTitle; existingPrCount = $taskExisting.Count
        connectorAttemptResult = '403 Resource not accessible by integration; no PR created by connector'
        authenticatedLocalGitCredentialAvailable = $true; credentialStoredOrPrinted = $false
        draft = $true; mergeRequested = $false; releaseRequested = $false
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $taskEvidence 'creation-preflight.json') -Encoding utf8
    $taskCreated = Invoke-RestMethod -Method Post -Uri ($taskRepo + '/pulls') -Headers $taskHeaders `
        -ContentType 'application/json; charset=utf-8' -Body ([Text.Encoding]::UTF8.GetBytes($taskPayload)) -TimeoutSec 20
    $taskCreated | ConvertTo-Json -Depth 16 | Set-Content -LiteralPath $taskOutput -Encoding utf8
    if ($taskCreated.head.sha -cne $taskSource -or $taskCreated.base.ref -cne 'main' -or
        -not $taskCreated.draft -or $taskCreated.state -cne 'open' -or $taskCreated.merged) {
        throw 'Created PR requires identity or draft-state audit; do not create again'
    }
    [ordered]@{
        number = $taskCreated.number; title = $taskCreated.title; url = $taskCreated.html_url
        headSha = $taskCreated.head.sha; baseSha = $taskCreated.base.sha; state = $taskCreated.state
        draft = $taskCreated.draft; merged = $taskCreated.merged; autoMerge = $taskCreated.auto_merge
        mergeRequested = $false; releaseRequested = $false
    } | ConvertTo-Json -Depth 5
} finally {
    $taskCredential.Clear(); $taskCredentialLines = $null; $taskHeaders = $null
    $taskLine = $null; $taskPair = $null; $env:GIT_TERMINAL_PROMPT = $taskOldPrompt
}

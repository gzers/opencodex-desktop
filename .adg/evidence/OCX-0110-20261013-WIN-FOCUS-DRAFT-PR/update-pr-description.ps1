$ErrorActionPreference = 'Stop'
$taskEvidence = $PSScriptRoot
$taskSource = 'be7395acfce0ec1e36b83300b576db408f8c35bb'
$taskUri = 'https://api.github.com/repos/gzers/opencodex-desktop/pulls/1'
$taskOutput = Join-Path $taskEvidence 'draft-pr-updated-after-interruption.json'
if (Test-Path -LiteralPath $taskOutput) { throw 'Do not overwrite this PR update receipt' }
$taskCredential = @{}
$taskLines = $null
$taskHeaders = $null
$taskOldPrompt = $env:GIT_TERMINAL_PROMPT
$env:GIT_TERMINAL_PROMPT = '0'
try {
    $taskLines = "protocol=https`nhost=github.com`n`n" | git credential fill 2>$null
    if ($LASTEXITCODE -ne 0) { throw 'Authenticated credential unavailable' }
    foreach ($taskLine in $taskLines) {
        $taskPair = $taskLine -split '=', 2
        if ($taskPair.Length -eq 2) { $taskCredential[$taskPair[0]] = $taskPair[1] }
    }
    if (-not $taskCredential.password) { throw 'Authenticated credential missing' }
    $taskHeaders = @{
        Authorization = 'Bearer ' + $taskCredential.password
        Accept = 'application/vnd.github+json'
        'X-GitHub-Api-Version' = '2022-11-28'
    }
    $taskBefore = Invoke-RestMethod -Uri $taskUri -Headers $taskHeaders -TimeoutSec 20
    if ($taskBefore.head.sha -cne $taskSource -or $taskBefore.state -cne 'open' -or
        -not $taskBefore.draft -or $taskBefore.merged -or $taskBefore.auto_merge) {
        throw 'Expected exact-source unmerged draft changed'
    }
    $taskBody = Get-Content -LiteralPath (Join-Path $taskEvidence 'pr-body-after-interruption.md') -Raw -Encoding utf8
    $taskBodyBytes = [Text.Encoding]::UTF8.GetBytes((@{body=$taskBody} | ConvertTo-Json -Depth 4))
    $taskAfter = Invoke-RestMethod -Method Patch -Uri $taskUri -Headers $taskHeaders `
        -ContentType 'application/json; charset=utf-8' -Body $taskBodyBytes -TimeoutSec 20
    $taskAfter | ConvertTo-Json -Depth 18 | Set-Content -LiteralPath $taskOutput -Encoding utf8
    if ($taskAfter.body -cne $taskBody -or $taskAfter.head.sha -cne $taskSource -or
        -not $taskAfter.draft -or $taskAfter.merged -or $taskAfter.auto_merge) {
        throw 'Updated description or draft state requires audit'
    }
    [ordered]@{
        observedAtUtc=[datetime]::UtcNow.ToString('o');number=1;url=$taskAfter.html_url
        draft=$taskAfter.draft;state=$taskAfter.state;headSha=$taskAfter.head.sha
        descriptionUpdated=$true;mergeRequested=$false;releaseRequested=$false
    } | ConvertTo-Json -Depth 4
} finally {
    $taskCredential.Clear();$taskLines=$null;$taskHeaders=$null
    $taskLine=$null;$taskPair=$null;$env:GIT_TERMINAL_PROMPT=$taskOldPrompt
}

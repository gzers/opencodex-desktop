$ErrorActionPreference = 'Stop'
$taskEvidence = $PSScriptRoot
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskSource = 'be7395acfce0ec1e36b83300b576db408f8c35bb'
$taskRunId = 38352525092
$taskReceiptPath = Join-Path $taskEvidence 'terminal-audit.json'
if (Test-Path -LiteralPath $taskReceiptPath) { throw 'Do not overwrite a terminal audit' }
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
    if ($taskRun.head_sha -cne $taskSource -or $taskRun.head_branch -cne 'feature/0.1.10-update-center-focus' -or
        $taskRun.event -cne 'push' -or $taskRun.status -cne 'completed') {
        throw 'Exact pushed diagnostic CI has not completed'
    }
    $taskJobs = Invoke-RestMethod -Uri ($taskUri + '/jobs?per_page=100') -Headers $taskHeaders -TimeoutSec 20
    $taskRun | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $taskEvidence 'run-audited-terminal.json') -Encoding utf8
    $taskJobs | ConvertTo-Json -Depth 14 | Set-Content -LiteralPath (Join-Path $taskEvidence 'jobs-audited-terminal.json') -Encoding utf8
    $taskWindows = @($taskJobs.jobs | Where-Object name -ceq 'backend-windows')
    if ($taskWindows.Count -ne 1 -or $taskWindows[0].id -ne 115147923845) { throw 'Exact Windows job differs' }
    Invoke-WebRequest -Uri ('https://api.github.com/repos/gzers/opencodex-desktop/actions/jobs/' + $taskWindows[0].id + '/logs') `
        -Headers $taskHeaders -TimeoutSec 40 -OutFile (Join-Path $taskEvidence 'backend-windows-terminal.log')
    $taskArtifacts = Invoke-RestMethod -Uri ($taskUri + '/artifacts?per_page=100') -Headers $taskHeaders -TimeoutSec 20
    $taskArtifacts | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $taskEvidence 'artifacts-terminal.json') -Encoding utf8
    $taskStartupArtifact = @($taskArtifacts.artifacts | Where-Object name -ceq 'windows-startup-evidence')
    if ($taskStartupArtifact.Count -ne 1 -or $taskStartupArtifact[0].expired -or
        $taskStartupArtifact[0].workflow_run.head_sha -cne $taskSource -or
        $taskStartupArtifact[0].digest -notmatch '^sha256:[a-f0-9]{64}$') {
        throw 'One live startup artifact with exact diagnostic source and API digest required'
    }
    $taskArtifact = $taskStartupArtifact[0]
    $taskDownload = Join-Path $taskCode ('test\software\downloads\' + $taskArtifact.id + '.zip')
    if (Test-Path -LiteralPath $taskDownload) { throw 'Do not overwrite an existing artifact ZIP' }
    Invoke-WebRequest -Uri $taskArtifact.archive_download_url -Headers $taskHeaders -TimeoutSec 40 -OutFile $taskDownload
    $taskZipHash = (Get-FileHash -LiteralPath $taskDownload -Algorithm SHA256).Hash.ToLowerInvariant()
    if (('sha256:' + $taskZipHash) -cne $taskArtifact.digest) { throw 'Downloaded artifact digest differs from API' }
    $taskRecord = [ordered]@{
        atUtc = [datetime]::UtcNow.ToString('o'); sourceCommit = $taskSource; runId = $taskRunId
        runConclusion = $taskRun.conclusion; windowsJobId = $taskWindows[0].id
        windowsConclusion = $taskWindows[0].conclusion; jobs = @($taskJobs.jobs | Select-Object id, name, status, conclusion)
        artifactId = $taskArtifact.id; artifactName = $taskArtifact.name; apiDigest = $taskArtifact.digest
        zipPath = $taskDownload; zipSha256 = $taskZipHash; zipDigestMatchesApi = $true
        zipCrcAccepted = $false; artifactContentsAccepted = $false; originalCiFailureCleared = $false
        originalCiCauseIsolated = $false; mainIntegrated = $false; executableStarted = $false
        foregroundInputIssued = $false; productRestarted = $false; ciRerunRequested = $false
        fullNativeAccepted = $false; pairedPerformanceAccepted = $false; realCyclesAccepted = $false
        formalTaskReceipt = $false; releaseAccepted = $false
    }
    $taskRecord | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskReceiptPath -Encoding utf8
    $taskRecord | ConvertTo-Json -Depth 8
} finally {
    $taskCredential.Clear(); $taskCredentialLines = $null; $taskHeaders = $null
    $taskLine = $null; $taskPair = $null; $env:GIT_TERMINAL_PROMPT = $taskOldPrompt
}

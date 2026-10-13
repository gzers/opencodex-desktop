$ErrorActionPreference = 'Stop'
$taskEvidence = $PSScriptRoot
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskSource = 'be7395acfce0ec1e36b83300b576db408f8c35bb'
$taskRunId = 38355803684
$taskRepo = 'https://api.github.com/repos/gzers/opencodex-desktop'
$taskOutput = Join-Path $taskEvidence 'candidate-download-audit.json'
if (Test-Path -LiteralPath $taskOutput) { throw 'Do not overwrite a candidate terminal audit' }
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
    $taskUri = $taskRepo + '/actions/runs/' + $taskRunId
    $taskRun = Invoke-RestMethod -Uri $taskUri -Headers $taskHeaders -TimeoutSec 20
    if ($taskRun.head_sha -cne $taskSource -or $taskRun.event -cne 'workflow_dispatch' -or
        $taskRun.head_branch -cne 'feature/0.1.10-update-center-focus' -or $taskRun.status -cne 'completed') {
        throw 'The exact-source candidate run has not reached terminal state'
    }
    $taskJobs = Invoke-RestMethod -Uri ($taskUri + '/jobs?per_page=100') -Headers $taskHeaders -TimeoutSec 20
    $taskPackage = @($taskJobs.jobs | Where-Object name -ceq 'package')
    if ($taskPackage.Count -ne 1 -or $taskPackage[0].id -ne 115158292064) { throw 'Exact package job differs' }
    $taskRun | ConvertTo-Json -Depth 14 | Set-Content -LiteralPath (Join-Path $taskEvidence 'candidate-run-audited-terminal.json') -Encoding utf8
    $taskJobs | ConvertTo-Json -Depth 14 | Set-Content -LiteralPath (Join-Path $taskEvidence 'candidate-jobs-audited-terminal.json') -Encoding utf8
    Invoke-WebRequest -Uri ($taskRepo + '/actions/jobs/' + $taskPackage[0].id + '/logs') -Headers $taskHeaders `
        -TimeoutSec 40 -OutFile (Join-Path $taskEvidence 'candidate-package-terminal.log')
    $taskArtifacts = Invoke-RestMethod -Uri ($taskUri + '/artifacts?per_page=100') -Headers $taskHeaders -TimeoutSec 20
    $taskArtifacts | ConvertTo-Json -Depth 14 | Set-Content -LiteralPath (Join-Path $taskEvidence 'candidate-artifacts-terminal.json') -Encoding utf8
    $taskRecord = [ordered]@{
        atUtc = [datetime]::UtcNow.ToString('o'); sourceCommit = $taskSource; runId = $taskRunId
        runConclusion = $taskRun.conclusion; packageJobId = $taskPackage[0].id
        packageConclusion = $taskPackage[0].conclusion; artifactDownloads = @()
        artifactContentsVerified = $false; candidateInstallerAccepted = $false
        localInstallerInvoked = $false; localExecutableStarted = $false; foregroundInputIssued = $false
        originalFailedCiFailureCleared = $false; originalFailedCiCauseIsolated = $false
        mainIntegrated = $false; originalDesktopOrCycleReplaced = $false
        finalNativeGateAccepted = $false; pairedPerformanceAccepted = $false; realCyclesAccepted = $false
        formalTaskReceipt = $false; releasePublished = $false; channelPromoted = $false; releaseAccepted = $false
    }
    if ($taskRun.conclusion -ceq 'success') {
        foreach ($taskName in @('windows-candidate-evidence', 'opencodex-desktop-0.1.10-windows-x64-candidate')) {
            if (@($taskArtifacts.artifacts | Where-Object name -ceq $taskName).Count -ne 1) {
                throw 'Successful candidate lacks one exact expected artifact'
            }
        }
    }
    foreach ($taskArtifact in $taskArtifacts.artifacts) {
        if ($taskArtifact.name -cnotin @('windows-candidate-evidence', 'opencodex-desktop-0.1.10-windows-x64-candidate')) {
            throw 'Unexpected candidate artifact name'
        }
        if ($taskArtifact.expired -or $taskArtifact.workflow_run.head_sha -cne $taskSource -or
            $taskArtifact.digest -notmatch '^sha256:[a-f0-9]{64}$') {
            throw 'Candidate artifact source or API digest differs'
        }
        $taskZip = Join-Path $taskCode ('test\software\downloads\' + $taskArtifact.id + '.zip')
        if (Test-Path -LiteralPath $taskZip) { throw 'Do not overwrite an existing candidate artifact ZIP' }
        Invoke-WebRequest -Uri $taskArtifact.archive_download_url -Headers $taskHeaders -TimeoutSec 60 -OutFile $taskZip
        $taskHash = (Get-FileHash -LiteralPath $taskZip -Algorithm SHA256).Hash.ToLowerInvariant()
        if (('sha256:' + $taskHash) -cne $taskArtifact.digest) { throw 'Downloaded candidate ZIP digest differs' }
        $taskRecord.artifactDownloads += [ordered]@{
            artifactId = $taskArtifact.id; name = $taskArtifact.name; zipPath = $taskZip
            zipSha256 = $taskHash; apiDigest = $taskArtifact.digest; digestMatched = $true
        }
    }
    $taskRecord | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $taskOutput -Encoding utf8
    $taskRecord | ConvertTo-Json -Depth 10
} finally {
    $taskCredential.Clear(); $taskCredentialLines = $null; $taskHeaders = $null
    $taskLine = $null; $taskPair = $null; $env:GIT_TERMINAL_PROMPT = $taskOldPrompt
}

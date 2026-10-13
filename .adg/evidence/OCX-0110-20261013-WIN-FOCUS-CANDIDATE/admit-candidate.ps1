$ErrorActionPreference = 'Stop'
$taskEvidence = $PSScriptRoot
$taskSource = 'be7395acfce0ec1e36b83300b576db408f8c35bb'
$taskBranch = 'feature/0.1.10-update-center-focus'
$taskRepo = 'https://api.github.com/repos/gzers/opencodex-desktop'
$taskCiRun = 38352525092
$taskAdmissionPath = Join-Path $taskEvidence 'candidate-admission.json'
if (Test-Path -LiteralPath $taskAdmissionPath) { throw 'Do not overwrite or repeat candidate admission' }
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
    $taskCi = Invoke-RestMethod -Uri ($taskRepo + '/actions/runs/' + $taskCiRun) -Headers $taskHeaders -TimeoutSec 20
    if ($taskCi.head_sha -cne $taskSource -or $taskCi.head_branch -cne $taskBranch -or
        $taskCi.status -cne 'completed' -or $taskCi.conclusion -cne 'success') {
        throw 'Exact candidate source lacks a successful complete CI'
    }
    $taskJobs = Invoke-RestMethod -Uri ($taskCi.url + '/jobs?per_page=100') -Headers $taskHeaders -TimeoutSec 20
    foreach ($taskName in @('frontend', 'backend', 'backend-windows', 'release-tools', 'windows-smoke-evidence')) {
        $taskJob = @($taskJobs.jobs | Where-Object name -ceq $taskName)
        if ($taskJob.Count -ne 1 -or $taskJob[0].status -cne 'completed' -or $taskJob[0].conclusion -cne 'success') {
            throw 'An exact-source required CI job has not passed'
        }
    }
    $taskCi | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $taskEvidence 'ci-admission-run.json') -Encoding utf8
    $taskJobs | ConvertTo-Json -Depth 14 | Set-Content -LiteralPath (Join-Path $taskEvidence 'ci-admission-jobs.json') -Encoding utf8
    $taskRef = Invoke-RestMethod -Uri ($taskRepo + '/git/ref/heads/' + $taskBranch) -Headers $taskHeaders -TimeoutSec 20
    if ($taskRef.object.sha -cne $taskSource -or $taskRef.object.type -cne 'commit') {
        throw 'Remote feature branch has moved from the verified candidate source'
    }
    $taskWorkflow = Invoke-RestMethod -Uri ($taskRepo + '/actions/workflows/windows-candidate.yml') -Headers $taskHeaders -TimeoutSec 20
    if ($taskWorkflow.name -cne 'Windows candidate installer' -or $taskWorkflow.state -cne 'active') {
        throw 'Existing isolated candidate workflow is not active'
    }
    $taskRuns = Invoke-RestMethod -Uri ($taskRepo + '/actions/workflows/windows-candidate.yml/runs?head_sha=' + $taskSource + '&per_page=100') `
        -Headers $taskHeaders -TimeoutSec 20
    $taskExisting = @($taskRuns.workflow_runs | Where-Object { $_.head_sha -ceq $taskSource })
    if ($taskExisting.Count -gt 1) { throw 'Multiple exact-source candidate runs require explicit audit before any dispatch' }
    $taskRecord = [ordered]@{
        atUtc = [datetime]::UtcNow.ToString('o'); sourceCommit = $taskSource; branch = $taskBranch
        exactCiRunId = $taskCiRun; exactCiConclusion = 'success'; requiredCiJobsPassed = $true
        remoteSourceMatchedBeforeDispatch = $true; workflowId = $taskWorkflow.id; workflowName = $taskWorkflow.name
        workflowPath = $taskWorkflow.path; existingExactSourceRuns = $taskExisting.Count
        purpose = 'Build existing NSIS/MSI candidate workflow for later native focus verification; retains full package-lock regression and runner install/uninstall'
        candidateScope = 'isolated GitHub Actions runner; no publication or local installation'
        dispatchRequested = $false; candidateRunId = $null; sourceVerifiedOnActualRun = $false
        existingRunReused = $false; originalFailedCiRunId = 38349316806
        originalFailedCiCauseIsolated = $false; originalFailedCiFailureCleared = $false
        mainIntegrated = $false; releasePublished = $false; channelPromoted = $false
        localExecutableStarted = $false; localInstallerInvoked = $false; foregroundInputIssued = $false
        originalDesktopPid = 37276; originalDesktopSource = 'eae85322ebca3b755c034db97cbebf2ea5bf71fc'
        cycleReplaced = $false; finalNativeGateAccepted = $false; pairedPerformanceAccepted = $false
        realCyclesAccepted = $false; formalTaskReceipt = $false; releaseAccepted = $false
    }
    $taskWorkflow | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskEvidence 'workflow-identity.json') -Encoding utf8
    $taskRef | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskEvidence 'remote-feature-ref-before.json') -Encoding utf8
    $taskRuns | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $taskEvidence 'candidate-runs-before.json') -Encoding utf8
    if ($taskExisting.Count -eq 1) {
        $taskRecord.candidateRunId = $taskExisting[0].id
        $taskRecord.sourceVerifiedOnActualRun = $taskExisting[0].head_sha -ceq $taskSource
        $taskRecord.existingRunReused = $true
        $taskRecord.status = 'existing-exact-source-candidate-reused'
    } else {
        $taskRecord.status = 'dispatch-request-prepared'
        $taskRecord | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskAdmissionPath -Encoding utf8
        $taskPayload = @{ ref = $taskBranch } | ConvertTo-Json -Compress
        $taskResponse = Invoke-WebRequest -Method Post -Uri ($taskRepo + '/actions/workflows/windows-candidate.yml/dispatches') `
            -Headers $taskHeaders -ContentType 'application/json' -Body $taskPayload -TimeoutSec 20
        if ($taskResponse.StatusCode -ne 204) { throw 'Unexpected workflow-dispatch response; do not blindly retry' }
        $taskRecord.dispatchRequested = $true
        $taskRecord.dispatchStatusCode = [int]$taskResponse.StatusCode
        $taskRecord.status = 'dispatch-accepted-awaiting-exact-source-run'
        $taskRecord.dispatchedAtUtc = [datetime]::UtcNow.ToString('o')
    }
    $taskRefAfter = Invoke-RestMethod -Uri ($taskRepo + '/git/ref/heads/' + $taskBranch) -Headers $taskHeaders -TimeoutSec 20
    $taskRecord.remoteSourceMatchedAfterDispatch = $taskRefAfter.object.sha -ceq $taskSource
    $taskRefAfter | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskEvidence 'remote-feature-ref-after.json') -Encoding utf8
    $taskRecord | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskAdmissionPath -Encoding utf8
    $taskRecord | ConvertTo-Json -Depth 8
} finally {
    $taskCredential.Clear(); $taskCredentialLines = $null; $taskHeaders = $null
    $taskLine = $null; $taskPair = $null; $env:GIT_TERMINAL_PROMPT = $taskOldPrompt
}

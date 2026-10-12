param(
    [Parameter(Mandatory=$true)][long]$RunId,
    [Parameter(Mandatory=$true)][string]$ExpectedHead,
    [Parameter(Mandatory=$true)][ValidatePattern('^[a-z0-9-]+$')][string]$Label,
    [string]$EvidenceDirectory = 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-QUERY-PROCESS-TREE'
)
$ErrorActionPreference = 'Stop'
$evidence = [IO.Path]::GetFullPath($EvidenceDirectory)
if (!$evidence.StartsWith('D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\',
    [StringComparison]::OrdinalIgnoreCase)) { throw 'Artifact evidence escaped task root' }
New-Item -ItemType Directory -Path $evidence -Force | Out-Null
$software = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\software\query-job-source'
$credentialInput = "protocol=https`nhost=github.com`n`n"
$credentialLines = $credentialInput | git credential fill 2>$null
if ($LASTEXITCODE -ne 0) { throw 'Authenticated Git credential unavailable' }
$credentialMap = @{}
foreach ($line in $credentialLines) {
    $pair = $line -split '=', 2
    if ($pair.Length -eq 2) { $credentialMap[$pair[0]] = $pair[1] }
}
try {
    if (!$credentialMap.password) { throw 'Authenticated Git credential missing' }
    $headers = @{
        Authorization = 'Bearer ' + $credentialMap.password
        Accept = 'application/vnd.github+json'
        'X-GitHub-Api-Version' = '2022-11-28'
    }
    $runUri = 'https://api.github.com/repos/gzers/opencodex-desktop/actions/runs/' + $RunId
    $run = Invoke-RestMethod -Uri $runUri -Headers $headers
    if ($run.head_sha -cne $ExpectedHead) { throw 'Exact CI source mismatch' }
    $suffix = if ($run.status -eq 'completed') { 'final' } else { 'current' }
    $run | ConvertTo-Json -Depth 10 |
        Set-Content -LiteralPath (Join-Path $evidence "$Label-ci-$suffix.json") -Encoding utf8
    $jobs = Invoke-RestMethod -Uri ($runUri + '/jobs') -Headers $headers
    $jobs | ConvertTo-Json -Depth 12 |
        Set-Content -LiteralPath (Join-Path $evidence "$Label-jobs-$suffix.json") -Encoding utf8
    $windows = @($jobs.jobs | Where-Object name -eq 'backend-windows')
    if ($windows.Count -ne 1) { throw 'Windows job is not unique' }
    if ($windows[0].status -eq 'completed') {
        $logs = 'https://api.github.com/repos/gzers/opencodex-desktop/actions/jobs/' + $windows[0].id + '/logs'
        Invoke-WebRequest -Uri $logs -Headers $headers -OutFile (Join-Path $evidence "$Label-ci-windows.log")
    }
    $artifacts = Invoke-RestMethod -Uri ($runUri + '/artifacts') -Headers $headers
    $artifacts | ConvertTo-Json -Depth 10 |
        Set-Content -LiteralPath (Join-Path $evidence "$Label-artifacts.json") -Encoding utf8
    $selected = @($artifacts.artifacts | Where-Object name -eq 'windows-startup-evidence')
    if ($selected.Count -ne 1) {
        [ordered]@{runId=$RunId;head=$ExpectedHead;status=$run.status;artifact='not yet available'} | ConvertTo-Json
        return
    }
    $artifact = $selected[0]
    if ($artifact.expired) { throw 'Windows test artifact expired' }
    if ($artifact.workflow_run.head_sha -cne $ExpectedHead) { throw 'Artifact source mismatch' }
    $zip = Join-Path $software ("ci-$Label.zip")
    if (!(Test-Path -LiteralPath $zip)) {
        Invoke-WebRequest -Uri ('https://api.github.com/repos/gzers/opencodex-desktop/actions/artifacts/' + $artifact.id + '/zip') `
            -Headers $headers -OutFile $zip
    }
    $digest = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
    if ('sha256:' + $digest -cne $artifact.digest) { throw 'Downloaded artifact hash differs from API digest' }
    [ordered]@{
        runId=$RunId;sourceCommit=$ExpectedHead;artifactId=$artifact.id;sha256=$digest
        apiDigest=$artifact.digest;verified=$true;zip=$zip;label=$Label
        credentialsIncluded=$false
    } | ConvertTo-Json |
        Set-Content -LiteralPath (Join-Path $evidence "$Label-download-verification.json") -Encoding utf8
    [ordered]@{runId=$RunId;head=$ExpectedHead;status=$run.status;conclusion=$run.conclusion;artifactId=$artifact.id;zip=$zip;digestVerified=$true} | ConvertTo-Json
} finally {
    $headers = $null
    $credentialMap.Clear()
    $credentialLines = $null
    $run = $null
    $jobs = $null
    $artifacts = $null
}

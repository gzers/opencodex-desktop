$ErrorActionPreference = 'Stop'
$source = '4852820b89f7f9df7d54bc623e94f3f55d884023'
$runId = 38242585130
$evidence = 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-QUERY-INTEGRATED'
$downloads = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\software\downloads'
New-Item -ItemType Directory -Path $evidence, $downloads -Force | Out-Null
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
    $runUri = 'https://api.github.com/repos/gzers/opencodex-desktop/actions/runs/' + $runId
    $run = Invoke-RestMethod -Uri $runUri -Headers $headers
    if ($run.head_sha -cne $source -or $run.head_branch -ne 'main') { throw 'Candidate source differs from integrated main' }
    if ($run.status -ne 'completed') {
        $run | ConvertTo-Json -Depth 10 |
            Set-Content -LiteralPath (Join-Path $evidence 'candidate-before-download.json') -Encoding utf8
        [ordered]@{runId=$runId;sourceCommit=$source;status=$run.status;downloaded=$false} | ConvertTo-Json
        return
    }
    $run | ConvertTo-Json -Depth 10 |
        Set-Content -LiteralPath (Join-Path $evidence 'candidate-final.json') -Encoding utf8
    if ($run.conclusion -ne 'success') { throw 'Exact integrated candidate is not successful' }
    $artifacts = Invoke-RestMethod -Uri ($runUri + '/artifacts') -Headers $headers
    $artifacts | ConvertTo-Json -Depth 10 |
        Set-Content -LiteralPath (Join-Path $evidence 'candidate-artifacts.json') -Encoding utf8
    $verifications = @()
    foreach ($name in @('opencodex-desktop-0.1.10-windows-x64-candidate', 'windows-candidate-evidence')) {
        $matches = @($artifacts.artifacts | Where-Object name -eq $name)
        if ($matches.Count -ne 1) { throw 'Expected unique named candidate artifact' }
        $artifact = $matches[0]
        if ($artifact.expired -or $artifact.workflow_run.head_sha -cne $source -or
            $artifact.workflow_run.id -ne $runId) { throw 'Artifact identity/retention differs' }
        $archive = Join-Path $downloads ([string]$artifact.id + '.zip')
        if (!(Test-Path -LiteralPath $archive)) {
            Invoke-WebRequest -Uri ('https://api.github.com/repos/gzers/opencodex-desktop/actions/artifacts/' + $artifact.id + '/zip') `
                -Headers $headers -OutFile $archive
        }
        $digest = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
        if ('sha256:' + $digest -cne $artifact.digest) { throw 'Raw ZIP hash differs from authenticated artifact digest' }
        $verifications += [ordered]@{
            id=$artifact.id; name=$artifact.name; sourceCommit=$source
            sha256=$digest; expectedDigest=$artifact.digest; matched=$true
            zipPath=$archive; extracted=(Join-Path $downloads ([string]$artifact.id))
        }
    }
    $verifications | ConvertTo-Json -Depth 7 |
        Set-Content -LiteralPath (Join-Path $evidence 'download-verification.json') -Encoding utf8
    $verifications | ConvertTo-Json -Depth 7
} finally {
    $headers = $null
    $credentialMap.Clear()
    $credentialLines = $null
    $run = $null
    $artifacts = $null
}

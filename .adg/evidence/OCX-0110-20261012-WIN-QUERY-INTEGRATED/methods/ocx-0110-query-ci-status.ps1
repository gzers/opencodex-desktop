param(
    [long[]]$RunIds = @(38237214567, 38237263400, 38237550401),
    [string]$EvidenceDirectory = 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-QUERY-PROCESS-TREE'
)
$ErrorActionPreference = 'Stop'
$allowedRoot = 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\'
$evidence = [IO.Path]::GetFullPath($EvidenceDirectory)
if (!$evidence.StartsWith($allowedRoot, [StringComparison]::OrdinalIgnoreCase)) { throw 'Evidence path escaped task root' }
New-Item -ItemType Directory -Path $evidence -Force | Out-Null
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
    foreach ($runId in $RunIds) {
        $runUri = 'https://api.github.com/repos/gzers/opencodex-desktop/actions/runs/' + $runId
        $run = Invoke-RestMethod -Uri $runUri -Headers $headers
        $jobs = Invoke-RestMethod -Uri ($runUri + '/jobs') -Headers $headers
        $suffix = if ($run.status -eq 'completed') { 'final' } else { 'current' }
        $run | ConvertTo-Json -Depth 10 |
            Set-Content -LiteralPath (Join-Path $evidence ("run-$runId-$suffix.json")) -Encoding utf8
        $jobs | ConvertTo-Json -Depth 12 |
            Set-Content -LiteralPath (Join-Path $evidence ("jobs-$runId-$suffix.json")) -Encoding utf8
        $windows = @($jobs.jobs | Where-Object name -eq 'backend-windows')
        $steps = @($windows.steps | Where-Object { $_.status -eq 'in_progress' -or $_.conclusion -eq 'failure' } |
            Select-Object name, status, conclusion)
        [ordered]@{
            runId = $runId
            head = $run.head_sha
            status = $run.status
            conclusion = $run.conclusion
            windowsJob = $windows | Select-Object id, status, conclusion
            activeOrFailedSteps = $steps
        } | ConvertTo-Json -Depth 6
    }
} finally {
    $headers = $null
    $credentialMap.Clear()
    $credentialLines = $null
    $run = $null
    $jobs = $null
}

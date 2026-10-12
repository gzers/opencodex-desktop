$ErrorActionPreference = 'Stop'
$runId = 38252093444
$expectedHead = 'eae85322ebca3b755c034db97cbebf2ea5bf71fc'
$evidence = 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-VERSION-PROBE-TREE'
$clock = [Diagnostics.Stopwatch]::StartNew()
$lastPhase = $null
while ($clock.Elapsed.TotalMinutes -lt 12) {
    $credentialInput = "protocol=https`nhost=github.com`n`n"
    $credentialLines = $credentialInput | git credential fill 2>$null
    if ($LASTEXITCODE -ne 0) { throw 'Authenticated Git credential unavailable' }
    $credentialMap = @{}
    foreach ($line in $credentialLines) {
        $pair = $line -split '=', 2
        if ($pair.Length -eq 2) { $credentialMap[$pair[0]] = $pair[1] }
    }
    try {
        if (!$credentialMap.password) { throw 'Authenticated credential missing' }
        $headers = @{
            Authorization='Bearer ' + $credentialMap.password
            Accept='application/vnd.github+json'
            'X-GitHub-Api-Version'='2022-11-28'
        }
        $runUri = 'https://api.github.com/repos/gzers/opencodex-desktop/actions/runs/' + $runId
        $run = Invoke-RestMethod -Uri $runUri -Headers $headers -TimeoutSec 20
        if ($run.head_sha -cne $expectedHead) { throw 'Observed CI source changed' }
        $artifacts = Invoke-RestMethod -Uri ($runUri + '/artifacts') -Headers $headers -TimeoutSec 20
        $named = @($artifacts.artifacts | Where-Object name -eq 'windows-startup-evidence')
        $phase = $run.status + '/' + $run.conclusion + '/test-artifact=' + $named.Count
        if ($phase -cne $lastPhase) {
            [ordered]@{
                observedAtUtc=[DateTime]::UtcNow.ToString('o');runId=$runId;sourceCommit=$expectedHead
                status=$run.status;conclusion=$run.conclusion;testArtifactCount=$named.Count
                elapsedWaitMs=$clock.Elapsed.TotalMilliseconds
            } | ConvertTo-Json -Compress | Write-Output
            $lastPhase = $phase
        }
        if ($named.Count -eq 1) {
            if ($named[0].workflow_run.head_sha -cne $expectedHead -or $named[0].expired) {
                throw 'Test artifact source/retention differs'
            }
            [ordered]@{
                observedAtUtc=[DateTime]::UtcNow.ToString('o');runId=$runId;sourceCommit=$expectedHead
                status='test-artifact-available';artifactId=$named[0].id
                elapsedWaitMs=$clock.Elapsed.TotalMilliseconds
                runStatus=$run.status;runConclusion=$run.conclusion
                downloaded=$false;authenticationIncluded=$false
            } | ConvertTo-Json |
                Set-Content -LiteralPath (Join-Path $evidence 'fixed-artifact-wait-terminal.json') -Encoding utf8
            return
        }
        if ($run.status -eq 'completed') {
            [ordered]@{
                observedAtUtc=[DateTime]::UtcNow.ToString('o');runId=$runId;sourceCommit=$expectedHead
                status='ci-terminal-without-expected-test-artifact';conclusion=$run.conclusion
                elapsedWaitMs=$clock.Elapsed.TotalMilliseconds;restarted=$false
            } | ConvertTo-Json |
                Set-Content -LiteralPath (Join-Path $evidence 'fixed-artifact-wait-terminal.json') -Encoding utf8
            return
        }
    } finally {
        $headers = $null
        $credentialMap.Clear()
        $credentialLines = $null
        $run = $null
        $artifacts = $null
    }
    # Background wait, at most one read per minute; no UI or process mutation.
    Start-Sleep -Seconds 60
}
[ordered]@{
    observedAtUtc=[DateTime]::UtcNow.ToString('o');runId=$runId;sourceCommit=$expectedHead
    status='bounded-observation-ended-without-claiming-ci-terminal';elapsedWaitMs=$clock.Elapsed.TotalMilliseconds
    restarted=$false
} | ConvertTo-Json |
    Set-Content -LiteralPath (Join-Path $evidence 'fixed-artifact-wait-terminal.json') -Encoding utf8

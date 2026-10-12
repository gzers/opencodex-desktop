$ErrorActionPreference = 'Stop'
$expectedSource = 'eae85322ebca3b755c034db97cbebf2ea5bf71fc'
$evidence = 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-VERSION-PROBE-INTEGRATED'
$runIds = @(38254821701, 38254885601)
$clock = [Diagnostics.Stopwatch]::StartNew()
$previous = $null
$errorsObserved = 0
while ($clock.Elapsed.TotalMinutes -lt 20) {
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
        $records = @()
        foreach ($runId in $runIds) {
            $uri = 'https://api.github.com/repos/gzers/opencodex-desktop/actions/runs/' + $runId
            $run = Invoke-RestMethod -Uri $uri -Headers $headers -TimeoutSec 20
            if ($run.head_sha -cne $expectedSource -or $run.head_branch -ne 'main') {
                throw 'Exact integrated source changed'
            }
            $records += [ordered]@{
                runId=$runId;name=$run.name;sourceCommit=$run.head_sha
                status=$run.status;conclusion=$run.conclusion
            }
        }
        $phase = ($records | ConvertTo-Json -Depth 5 -Compress)
        if ($phase -cne $previous) {
            [ordered]@{
                observedAtUtc=[DateTime]::UtcNow.ToString('o')
                runs=$records;elapsedWaitMs=$clock.Elapsed.TotalMilliseconds
                observerErrors=$errorsObserved
            } | ConvertTo-Json -Depth 6 -Compress | Write-Output
            $previous = $phase
        }
        $snapshot = [ordered]@{
            observedAtUtc=[DateTime]::UtcNow.ToString('o')
            runs=$records;observerPid=$PID
            observerCreationUtc=(Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o')
            elapsedWaitMs=$clock.Elapsed.TotalMilliseconds
            observerErrors=$errorsObserved
            restartRequested=$false;localInstallerExecuted=$false
            scope='read-only at most one observation per run per minute; no CI/product restart'
        }
        $snapshot | ConvertTo-Json -Depth 7 |
            Set-Content -LiteralPath (Join-Path $evidence 'integrated-ci-live-observation.json') -Encoding utf8
        if (@($records | Where-Object status -ne 'completed').Count -eq 0) {
            $snapshot.terminalStatus = 'both-exact-runs-terminal-requires-artifact-audit'
            $snapshot | ConvertTo-Json -Depth 7 |
                Set-Content -LiteralPath (Join-Path $evidence 'integrated-ci-wait-terminal.json') -Encoding utf8
            return
        }
    } catch {
        $errorsObserved++
        # A read failure is not a terminal CI state. Reobserve these same runs
        # rather than dispatching replacements or changing TLS settings.
        [ordered]@{
            observedAtUtc=[DateTime]::UtcNow.ToString('o')
            observerError=$_.Exception.Message;errorsObserved=$errorsObserved
            restarted=$false;scope='observation error only'
        } | ConvertTo-Json -Compress | Write-Output
    } finally {
        $headers = $null
        $credentialMap.Clear()
        $credentialLines = $null
        $run = $null
    }
    Start-Sleep -Seconds 60
}
[ordered]@{
    observedAtUtc=[DateTime]::UtcNow.ToString('o');runIds=$runIds
    expectedSource=$expectedSource
    status='bounded-observer-ended-without-terminal-ci-claim'
    elapsedWaitMs=$clock.Elapsed.TotalMilliseconds;restartRequested=$false
} | ConvertTo-Json |
    Set-Content -LiteralPath (Join-Path $evidence 'integrated-ci-wait-terminal.json') -Encoding utf8

param([ValidateRange(24, 30)][int]$MaximumHours = 26)
$ErrorActionPreference = 'Stop'
$evidenceRoot = Split-Path -Parent $PSScriptRoot
$taskBase = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\windows0110'
$runRoot = Join-Path $taskBase 'cycles/stable-panel-20261012'
$node = 'C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe'
$query = 'C:\Users\15119\.codex\tmp\ocx-0110-cdp-query.mjs'
$initial = Get-Content -LiteralPath (Join-Path $evidenceRoot 'native-initialization.json') -Raw | ConvertFrom-Json -DateKind String
$baseline = Get-Content -LiteralPath (Join-Path $evidenceRoot 'schedule-baseline.json') -Raw | ConvertFrom-Json -DateKind String
if ($initial.applicationPid -ne 38924 -or
    $initial.sourceCommit -ne 'afff490a430a81f8c233d44928aa3681a23e91e0' -or
    $initial.result.status -ne 'observing-real-stable-and-panel-twenty-four-hour-deadlines') {
    throw 'Real cycle baseline unavailable; do not start a replacement'
}
foreach ($target in @('manager-stable', 'panel-latest')) {
    $deadline = $baseline.entries.$target
    if ($deadline.failures -ne 0 -or $deadline.next_due - $deadline.last_success -ne 86400) {
        throw 'Baseline is not a real successful daily deadline'
    }
}
New-Item -ItemType Directory -Path $runRoot -Force | Out-Null
$ownership = [IO.File]::Open((Join-Path $runRoot 'monitor-owner.lock'),
    [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
$app = $null
$supervisor = $null
$state = [ordered]@{
    status = 'preparing'
    sourceCommit = $initial.sourceCommit
    applicationPid = $initial.applicationPid
    monitorPid = $PID
    monitorCreationUtc = (Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o')
    startedAtUtc = [DateTime]::UtcNow.ToString('o')
    maximumHours = $MaximumHours
    managerDeadlineEpoch = $baseline.entries.'manager-stable'.next_due
    panelDeadlineEpoch = $baseline.entries.'panel-latest'.next_due
    observations = 0
    failedObservations = 0
    consecutiveObservationErrors = 0
    managerNextSuccessfulQueryObserved = $false
    panelNextSuccessfulQueryObserved = $false
    earlierThanRecordedDeadlineTransitionObserved = $false
    fullCycleAccepted = $false
    betaCycleAccepted = $false
    performanceAccepted = $false
    clocksChanged = $false
    timersChanged = $false
    businessResponsesInjected = $false
    originalUpdatePreferencesRestored = $false
    runtimeRoot = $runRoot
    rawProcessOrNetworkCountsClaimed = $false
}
function Save-State {
    $temporary = Join-Path $runRoot 'monitor-state.pending.json'
    $state | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath $temporary -Encoding utf8
    Move-Item -LiteralPath $temporary -Destination (Join-Path $runRoot 'monitor-state.json') -Force
}
function Append-Observation($value) {
    [IO.File]::AppendAllText((Join-Path $runRoot 'observations.ndjson'),
        ($value | ConvertTo-Json -Depth 12 -Compress) + "`n", [Text.UTF8Encoding]::new($false))
}
try {
    $session = Get-Content -LiteralPath (Join-Path $taskBase 'control/current-session.json') -Raw | ConvertFrom-Json -DateKind String
    if ($session.applicationPid -ne 38924 -or $session.supervisorPid -ne 44780 -or
        $session.status -ne 'running' -or $session.mode -ne 'ordinary') { throw 'Current lifecycle differs' }
    $app = [Diagnostics.Process]::GetProcessById(38924)
    $supervisor = [Diagnostics.Process]::GetProcessById(44780)
    if ($app.Path -ne $session.executable -or $app.StartTime.ToUniversalTime().ToString('o') -ne
        $session.applicationToken.creationTimeUtc) { throw 'Application process identity mismatch' }
    # Retain handles so a later reused PID cannot masquerade as this lifecycle.
    [void]$app.Handle
    [void]$supervisor.Handle
    $state.supervisorPid = 44780
    $state.applicationCreationUtc = $app.StartTime.ToUniversalTime().ToString('o')
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $nextObservationMs = 0.0
    $state.status = 'observing-real-deadlines'
    Save-State
    while ($clock.Elapsed.TotalHours -lt $MaximumHours) {
        if ($app.HasExited -or $supervisor.HasExited) {
            $state.status = 'interrupted-original-lifecycle-ended'
            break
        }
        $state.lastRetainedHandleCheckUtc = [DateTime]::UtcNow.ToString('o')
        $state.elapsedMonitorMs = $clock.Elapsed.TotalMilliseconds
        if (Test-Path -LiteralPath (Join-Path $runRoot 'stop.request')) {
            $state.status = 'explicit-monitor-stop-requested'
            break
        }
        if ($clock.Elapsed.TotalMilliseconds -ge $nextObservationMs) {
            try {
                $nativeText = & $node $query (Join-Path $PSScriptRoot 'observe.js')
                if ($LASTEXITCODE -ne 0) { throw 'Native observation helper failed' }
                $native = ($nativeText -join "`n") | ConvertFrom-Json -DateKind String
                if ($native.applicationPid -ne 38924 -or $native.sourceCommit -ne $initial.sourceCommit) {
                    throw 'Native observation identity differs'
                }
                $schedule = Get-Content -LiteralPath (Join-Path $taskBase 'data/manager-state/update-schedule.json') -Raw |
                    ConvertFrom-Json -DateKind String
                $now = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
                $observation = [ordered]@{
                    observedAtUtc = [DateTime]::UtcNow.ToString('o')
                    elapsedMonitorMs = $clock.Elapsed.TotalMilliseconds
                    native = $native
                    schedule = $schedule
                    originalLifecycleLive = $true
                }
                Append-Observation $observation
                $state.observations++
                $state.consecutiveObservationErrors = 0
                $state.lastNativeObservationUtc = $native.collectedAtUtc
                $state.lastForeground = $native.result.foreground
                $state.lastDocumentVisibility = $native.result.documentVisibility
                $state.lastWebviewMonotonicMs = $native.result.atMs
                if ($native.result.preferences.channel -ne 'stable' -or !$native.result.preferences.automaticCheck) {
                    $state.status = 'interrupted-update-preferences-changed'
                    break
                }
                foreach ($target in @('manager-stable', 'panel-latest')) {
                    $prior = $baseline.entries.$target
                    $current = $schedule.entries.$target
                    if ($current.last_success -gt $prior.last_success) {
                        if ($current.last_success -lt $prior.next_due) {
                            $state.earlierThanRecordedDeadlineTransitionObserved = $true
                        }
                        if ($current.failures -eq 0 -and $current.next_due - $current.last_success -eq 86400) {
                            if ($target -eq 'manager-stable') {
                                $state.managerNextSuccessfulQueryObserved = $true
                                $state.managerObservedNextSuccessEpoch = $current.last_success
                            } else {
                                $state.panelNextSuccessfulQueryObserved = $true
                                $state.panelObservedNextSuccessEpoch = $current.last_success
                            }
                        }
                    }
                }
                if ($state.managerNextSuccessfulQueryObserved -and $state.panelNextSuccessfulQueryObserved) {
                    $state.status = 'observed-real-next-successful-deadlines-requires-final-audit'
                    # Observing next persisted success does not prove raw request counts,
                    # original full-cycle matrix or performance acceptance.
                    break
                }
                if ($now -ge [Math]::Min($state.managerDeadlineEpoch, $state.panelDeadlineEpoch)) {
                    $state.status = if ($state.lastForeground) {
                        'deadline-elapsed-awaiting-real-terminal-query'
                    } else {
                        'deadline-elapsed-awaiting-eligible-foreground'
                    }
                }
                $nearest = [Math]::Min($state.managerDeadlineEpoch, $state.panelDeadlineEpoch)
                $intervalMs = if ([Math]::Abs($nearest - $now) -lt 60) { 5000.0 } else { 300000.0 }
                $nextObservationMs = $clock.Elapsed.TotalMilliseconds + $intervalMs
            } catch {
                $state.failedObservations++
                $state.consecutiveObservationErrors++
                $state.lastObservationError = $_.Exception.Message
                $state.lastObservationErrorUtc = [DateTime]::UtcNow.ToString('o')
                # Observation failure is not terminal and never restarts the product.
                $nextObservationMs = $clock.Elapsed.TotalMilliseconds + 30000.0
            }
            Save-State
            if ($state.observations -eq 1) { Write-Output ($state | ConvertTo-Json -Depth 7 -Compress) }
        }
        Save-State
        Start-Sleep -Seconds 5
    }
    if ($clock.Elapsed.TotalHours -ge $MaximumHours) { $state.status = 'maximum-observation-duration-elapsed-not-accepted' }
} catch {
    $state.status = 'monitor-preparation-failed'
    $state.lastObservationError = $_.Exception.Message
    throw
} finally {
    if ($app -and !$app.HasExited) {
        try {
            $cleanupText = & $node $query (Join-Path $PSScriptRoot 'cleanup.js')
            if ($LASTEXITCODE -ne 0) { throw 'Native cleanup helper failed' }
            $cleanup = ($cleanupText -join "`n") | ConvertFrom-Json -DateKind String
            $cleanup | ConvertTo-Json -Depth 12 |
                Set-Content -LiteralPath (Join-Path $runRoot 'native-cleanup.json') -Encoding utf8
            $state.originalUpdatePreferencesRestored = $cleanup.result.originalUpdatePreferencesRestored
        } catch {
            $state.cleanupError = $_.Exception.Message
        }
    }
    $state.finishedAtUtc = [DateTime]::UtcNow.ToString('o')
    Save-State
    if ($app) { $app.Dispose() }
    if ($supervisor) { $supervisor.Dispose() }
    $ownership.Dispose()
    Write-Output ($state | ConvertTo-Json -Depth 7 -Compress)
}

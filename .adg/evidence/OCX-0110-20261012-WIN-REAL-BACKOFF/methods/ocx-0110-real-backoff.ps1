$ErrorActionPreference='Stop'
$taskSource='afff490a430a81f8c233d44928aa3681a23e91e0'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskBase=Join-Path $taskCode 'test/runtime/windows0110'
$taskData=Join-Path $taskBase 'data'
$taskState=Join-Path $taskData 'manager-state/update-schedule.json'
$taskEvidence='D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-REAL-BACKOFF'
$taskNode='C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe'
$taskQuery='C:\Users\15119\.codex\tmp\ocx-0110-cdp-query.mjs'
New-Item -ItemType Directory -Force -Path $taskEvidence | Out-Null
$taskGuard=[IO.File]::Open((Join-Path $taskBase 'control/backoff-monitor.lock'),
    [IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
$taskBytes=[IO.File]::ReadAllBytes($taskState)
$taskBeforeHash=(Get-FileHash -LiteralPath $taskState).Hash.ToLower()
$taskSession=Get-Content -LiteralPath (Join-Path $taskBase 'control/current-session.json') -Raw | ConvertFrom-Json -DateKind String
if ($taskSession.sourceCommit -ne $taskSource -or $taskSession.status -ne 'running') { throw 'Exact merged instance is required' }
$taskApp=Get-Process -Id $taskSession.applicationPid -ErrorAction Stop
if ($taskApp.Path -cne $taskSession.executable -or
    $taskApp.StartTime.ToUniversalTime().ToString('o') -cne $taskSession.applicationToken.creationTimeUtc) {
    throw 'Retained app process identity differs'
}
$taskReport=[ordered]@{
    schemaVersion=1;sourceCommit=$taskSource;applicationPid=$taskApp.Id
    applicationCreationTimeUtc=$taskSession.applicationToken.creationTimeUtc
    monitorPid=$PID;monitorCreationTimeUtc=(Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o')
    startedAtUtc=[datetime]::UtcNow.ToString('o');status='preparing';requiredFloorMs=1800000
    systemClockChanged=$false;timeAccelerated=$false;scheduleBeforeSha256=$taskBeforeHash
    fixture='schema99 in isolated update-schedule; restore exact original bytes after real failure'
    completePerformanceAcceptance=$false;real24h6hAcceptance=$false;releaseAcceptance=$false
}
function Save-Report {
    $taskReport | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskEvidence 'monitor-state.json') -Encoding utf8
}
function Invoke-Phase([string]$Script,[string]$Name) {
    & $taskNode $taskQuery (Join-Path 'C:\Users\15119\.codex\tmp' $Script) *> (Join-Path $taskEvidence $Name)
    if ($LASTEXITCODE -ne 0) {throw "Native backoff phase failed: $Script"}
    Get-Content -LiteralPath (Join-Path $taskEvidence $Name) -Raw | ConvertFrom-Json
}
$taskFixtureRestored=$false
$taskProbeArmed=$false
Save-Report
try {
    try {
        [IO.File]::WriteAllText($taskState,'{"schema":99,"entries":{}}')
        $taskStart=Invoke-Phase 'ocx-0110-error-backoff-start.js' 'native-error-start.json'
        $taskProbeArmed=$true
    } finally {
        [IO.File]::WriteAllBytes($taskState,$taskBytes)
        if ((Get-FileHash -LiteralPath $taskState).Hash.ToLower() -cne $taskBeforeHash) {throw 'Original test schedule restoration failed'}
        $taskFixtureRestored=$true
        $taskReport.scheduleRestoredSha256=$taskBeforeHash
    }
    if ($taskStart.result.planCalls.Count -ne 1 -or $taskStart.result.planCalls[0].outcome -ne 'rejected') {
        throw 'A single real native schedule failure was not captured'
    }
    $taskReport.firstFailureAtMs=$taskStart.result.firstFailureAtMs
    $taskReport.firstFailureEvidence='native-error-start.json'
    $taskReport.status='waiting-real-floor'
    $taskWake=Invoke-Phase 'ocx-0110-error-backoff-wakes.js' 'source-wakes.json'
    $taskReport.sourceEventsGenerated=$taskWake.result.sourceEventsGenerated
    Save-Report
    Write-Output ($taskReport | ConvertTo-Json -Depth 6 -Compress)
    $taskDeadline=[datetime]::UtcNow.AddMinutes(35)
    $taskIndex=0
    while ([datetime]::UtcNow -lt $taskDeadline) {
        $taskCurrent=Get-Process -Id $taskSession.applicationPid -ErrorAction Stop
        if ($taskCurrent.Path -cne $taskSession.executable -or
            $taskCurrent.StartTime.ToUniversalTime().ToString('o') -cne $taskSession.applicationToken.creationTimeUtc) {
            throw 'Application identity changed during elapsed-time observation'
        }
        $taskObservation=Invoke-Phase 'ocx-0110-error-backoff-observe.js' ('observation-'+$taskIndex.ToString('D4')+'.json')
        $taskIndex++
        $taskValue=$taskObservation.result
        $taskReport.observationCount=$taskIndex
        $taskReport.lastObservedAtUtc=$taskValue.observedAtUtc
        $taskReport.elapsedSinceFailureMs=$taskValue.elapsedSinceFailureMs
        $taskReport.planCallCount=$taskValue.planCalls.Count
        $taskReport.queryCount=$taskValue.queries.Count
        $taskReport.lastForeground=$taskValue.foreground
        $taskReport.lastVisibility=$taskValue.visibility
        $taskRetry=@($taskValue.planCalls | Select-Object -Skip 1)
        foreach($taskCall in $taskRetry) {
            if ($taskCall.calledAtMs - $taskValue.firstFailureAtMs -lt 1800000) {
                $taskReport.status='failed-early-retry'
                throw 'Production plan retried before the real 30-minute floor'
            }
        }
        if ($taskValue.queries.Count) {throw 'Unexpected update queries occurred during the cache/floor observation'}
        if ($taskRetry.Count -and $taskRetry[0].outcome -eq 'fulfilled') {
            $taskReport.status='passed-recorded-real-floor'
            $taskReport.firstRetryAfterFailureMs=$taskRetry[0].calledAtMs - $taskValue.firstFailureAtMs
            $taskReport.firstRetryTargetCount=$taskRetry[0].targetCount
            break
        }
        Save-Report
        Start-Sleep -Seconds 30
    }
    if ($taskReport.status -eq 'waiting-real-floor') {$taskReport.status='incomplete-retry-not-observed'}
} catch {
    if ($taskReport.status -ne 'failed-early-retry') {$taskReport.status='failed-or-interrupted'}
    $taskReport.error=$_.Exception.Message
    throw
} finally {
    if (!$taskFixtureRestored) {
        [IO.File]::WriteAllBytes($taskState,$taskBytes)
        $taskReport.scheduleFixtureRestored=$true
    }
    try {
        $taskCleanup=Invoke-Phase 'ocx-0110-error-backoff-cleanup.js' 'native-cleanup.json'
        $taskReport.cleanupSucceeded=$taskCleanup.result.cleaned
        $taskReport.actualAutoCheckAfterCleanup=$taskCleanup.result.actualAutoCheck
    } catch {
        $taskReport.cleanupSucceeded=$false;$taskReport.cleanupError=$_.Exception.Message
        if ($taskReport.status -eq 'passed-recorded-real-floor') {$taskReport.status='incomplete-cleanup'}
    }
    $taskReport.finishedAtUtc=[datetime]::UtcNow.ToString('o')
    $taskReport.scheduleAfterSha256=(Get-FileHash -LiteralPath $taskState).Hash.ToLower()
    Save-Report
    $taskGuard.Dispose();$taskApp.Dispose()
}
Write-Output ($taskReport | ConvertTo-Json -Depth 8)

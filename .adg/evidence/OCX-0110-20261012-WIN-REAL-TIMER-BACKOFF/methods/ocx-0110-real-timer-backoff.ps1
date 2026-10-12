$ErrorActionPreference='Stop'
$taskSource='afff490a430a81f8c233d44928aa3681a23e91e0'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskBase=Join-Path $taskCode 'test/runtime/windows0110'
$taskStatePath=Join-Path $taskBase 'data/manager-state/update-schedule.json'
$taskEvidence='D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-REAL-TIMER-BACKOFF'
$taskNode='C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe'
$taskQuery='C:\Users\15119\.codex\tmp\ocx-0110-cdp-query.mjs'
New-Item -ItemType Directory -Force -Path $taskEvidence | Out-Null
$taskLock=[IO.File]::Open((Join-Path $taskBase 'control/real-timer-backoff.lock'),
    [IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
$taskStateBytes=[IO.File]::ReadAllBytes($taskStatePath)
$taskBeforeHash=(Get-FileHash -LiteralPath $taskStatePath).Hash.ToLower()
$taskSession=Get-Content -LiteralPath (Join-Path $taskBase 'control/current-session.json') -Raw | ConvertFrom-Json -DateKind String
$taskApp=Get-Process -Id $taskSession.applicationPid -ErrorAction Stop
if ($taskSession.sourceCommit -ne $taskSource -or $taskApp.Path -cne $taskSession.executable -or
    $taskApp.StartTime.ToUniversalTime().ToString('o') -cne $taskSession.applicationToken.creationTimeUtc) {
    throw 'Exact retained merged process required'
}
$taskReport=[ordered]@{
    schemaVersion=2;sourceCommit=$taskSource;applicationPid=$taskApp.Id
    applicationCreationTimeUtc=$taskSession.applicationToken.creationTimeUtc
    monitorPid=$PID;monitorCreationTimeUtc=(Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o')
    startedAtUtc=[datetime]::UtcNow.ToString('o');status='preparing'
    requiredFloorMs=1800000;previousInternalFloor='retained, start unobserved; not reset'
    originalScheduleSha256=$taskBeforeHash;timerDelaysChanged=$false;systemClockChanged=$false
    nativeBridgeChanged=$false;responseInjected=$false;scriptsExecuted=$false
    performanceAcceptance=$false;real24h6hCycleAcceptance=$false;formalTaskReceipt=$false
}
function Save-MonitorReport {
    $taskReport | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $taskEvidence 'monitor-state.json') -Encoding utf8
}
function Invoke-TimerPhase([string]$Script,[string]$EvidenceName) {
    & $taskNode $taskQuery (Join-Path 'C:\Users\15119\.codex\tmp' $Script) *> (Join-Path $taskEvidence $EvidenceName)
    if ($LASTEXITCODE -ne 0) {throw "Real timer phase failed: $Script"}
    Get-Content -LiteralPath (Join-Path $taskEvidence $EvidenceName) -Raw | ConvertFrom-Json
}
$taskRecorderInstalled=$false
$taskFixtureChanged=$false
$taskScheduleRestored=$false
$taskAnchorObserved=$false
Save-MonitorReport
try {
    $taskInstalled=Invoke-TimerPhase 'ocx-0110-timer-trace-install.js' 'recorder-installation.json'
    if (!$taskInstalled.result.recorderInstalled -or !$taskInstalled.result.nativeBridgeUnchanged) {
        throw 'Transparent timer recorder installation was not independently confirmed'
    }
    $taskRecorderInstalled=$true
    [IO.File]::WriteAllText($taskStatePath,'{"schema":99,"entries":{}}')
    $taskFixtureChanged=$true
    $taskEnabled=Invoke-TimerPhase 'ocx-0110-timer-trace-enable.js' 'scheduler-enabled.json'
    $taskReport.status='waiting-fresh-error-floor'
    $taskReport.recorderVerifiedInstalled=$true
    Save-MonitorReport
    Write-Output ($taskReport | ConvertTo-Json -Depth 6 -Compress)
    $taskDeadline=[datetime]::UtcNow.AddMinutes(65)
    $taskIndex=0
    while ([datetime]::UtcNow -lt $taskDeadline) {
        $taskLive=Get-Process -Id $taskSession.applicationPid -ErrorAction Stop
        if ($taskLive.Path -cne $taskSession.executable -or
            $taskLive.StartTime.ToUniversalTime().ToString('o') -cne $taskSession.applicationToken.creationTimeUtc) {
            throw 'Product process identity changed; do not restart the observation'
        }
        $taskObservation=Invoke-TimerPhase 'ocx-0110-timer-trace-observe.js' ('observation-'+$taskIndex.ToString('D4')+'.json')
        $taskIndex++
        $taskValue=$taskObservation.result
        $taskReport.observationCount=$taskIndex
        $taskReport.lastObservedAtUtc=$taskValue.observedAtUtc
        $taskReport.lastForeground=$taskValue.foreground
        $taskReport.lastVisibility=$taskValue.visibility
        $taskReport.recordCount=$taskValue.recordCount
        if ($taskValue.anchor -and !$taskAnchorObserved) {
            if ($taskValue.anchor.delayMs -ne 1800000 -or !$taskValue.sourceWitness) {
                throw 'Fresh error timer must have exact real thirty-minute delay and source attribution'
            }
            $taskAnchorObserved=$true
            $taskReport.status='observing-real-thirty-minute-floor'
            $taskReport.anchorAtMs=$taskValue.anchor.scheduledAtMs
            $taskReport.anchorAtUtc=$taskValue.anchor.scheduledAtUtc
            $taskReport.sourceWitness=$taskValue.sourceWitness
            [IO.File]::WriteAllBytes($taskStatePath,$taskStateBytes)
            if ((Get-FileHash -LiteralPath $taskStatePath).Hash.ToLower() -cne $taskBeforeHash) {
                throw 'Original schedule restoration failed'
            }
            $taskScheduleRestored=$true
            $taskReport.scheduleRestoredSha256=$taskBeforeHash
            $taskWakes=Invoke-TimerPhase 'ocx-0110-timer-trace-wakes.js' 'route-online-wakes.json'
            $taskReport.generatedSourceEvents=$taskWakes.result.sourceEvents
            Save-MonitorReport
            Write-Output ("Fresh real floor observed at " + $taskReport.anchorAtUtc + "; original schedule restored, generated source wakes preserved the timer.")
        }
        if ($taskAnchorObserved) {
            $taskReport.elapsedSinceAnchorMs=$taskValue.elapsedSinceAnchorMs
            $taskReport.associatedArmCount=$taskValue.associatedArmCount
            $taskReport.activeTimerCount=$taskValue.activeTimerCount
            if ($taskValue.activeTimerCount -gt 1) {throw 'Multiple live update timers observed'}
            if ($taskValue.fired.Count) {
                $taskFirstFire=@($taskValue.fired | Sort-Object firedAtMs)[0]
                $taskElapsed=$taskFirstFire.firedAtMs - $taskReport.anchorAtMs
                $taskReport.actualFirstFireAfterAnchorMs=$taskElapsed
                $taskReport.firstFire=$taskFirstFire
                if ($taskElapsed -lt 1800000) {
                    $taskReport.status='inconclusive-observed-before-anchor-budget'
                    throw 'Observed firing is earlier than strict recorded anchor budget; do not reduce threshold or infer a product failure from measurement offset'
                }
                $taskReport.status='passed-recorded-real-timer-floor'
                break
            }
        }
        Save-MonitorReport
        Start-Sleep -Seconds 30
    }
    if ($taskReport.status -in @('waiting-fresh-error-floor','observing-real-thirty-minute-floor')) {
        $taskReport.status='incomplete-window-or-floor-not-observed'
    }
} catch {
    if ($taskReport.status -ne 'inconclusive-observed-before-anchor-budget') {$taskReport.status='failed-or-interrupted'}
    $taskReport.error=$_.Exception.Message
    throw
} finally {
    if ($taskFixtureChanged -and !$taskScheduleRestored) {
        [IO.File]::WriteAllBytes($taskStatePath,$taskStateBytes)
        $taskScheduleRestored=(Get-FileHash -LiteralPath $taskStatePath).Hash.ToLower() -ceq $taskBeforeHash
    }
    $taskReport.scheduleRestored=$taskScheduleRestored -or !$taskFixtureChanged
    if ($taskRecorderInstalled) {
        try {
            $taskCleanup=Invoke-TimerPhase 'ocx-0110-timer-trace-cleanup.js' 'native-cleanup.json'
            $taskReport.cleanupSucceeded=$taskCleanup.result.cleaned
            $taskReport.actualAutoCheckAfterCleanup=$taskCleanup.result.actualAutoCheck
        } catch {
            $taskReport.cleanupSucceeded=$false;$taskReport.cleanupError=$_.Exception.Message
            if ($taskReport.status -eq 'passed-recorded-real-timer-floor') {$taskReport.status='incomplete-cleanup'}
        }
    }
    $taskReport.finishedAtUtc=[datetime]::UtcNow.ToString('o')
    $taskReport.finalScheduleSha256=(Get-FileHash -LiteralPath $taskStatePath).Hash.ToLower()
    Save-MonitorReport
    $taskLock.Dispose();$taskApp.Dispose()
}
Write-Output ($taskReport | ConvertTo-Json -Depth 8)

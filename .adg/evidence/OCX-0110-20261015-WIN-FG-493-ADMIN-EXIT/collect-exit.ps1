$ErrorActionPreference = 'Stop'
$taskResultPath = Join-Path $PSScriptRoot 'exit-postcheck.original.json'
if (Test-Path -LiteralPath $taskResultPath) { throw 'Do not overwrite an exit checkpoint' }
$taskEvidence = Split-Path -Parent $PSScriptRoot
$taskPrior = Join-Path $taskEvidence 'OCX-0110-20261015-WIN-FG-493-CONTINUE'
$taskFollowup = Join-Path $taskEvidence 'OCX-0110-20261015-WIN-FG-493-MANUAL-WINDOW'
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskAdmin = Get-Content -LiteralPath (Join-Path $taskPrior 'administrator-launch.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskOrdinary = Get-Content -LiteralPath (Join-Path $taskPrior 'ordinary-launch.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskLastLive = Get-Content -LiteralPath (Join-Path $taskEvidence 'OCX-0110-20261015-WIN-TREE-BROWSER/checkpoint.original.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskController = Get-Content -LiteralPath (Join-Path $taskPrior 'administrator-controller-request.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskReader = Get-Content -LiteralPath (Join-Path $taskFollowup 'reader-pwsh7-request.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskNative = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'" |
    Select-Object ProcessId, CreationDate, ExecutablePath)
$taskAbsentObserved = [datetimeoffset]::UtcNow.ToString('o')
if ($taskNative.Count) { throw 'Native application still exists; do not claim exit' }
$taskFormerPidPresent = @(Get-CimInstance Win32_Process -Filter ('ProcessId=' + $taskAdmin.pid)).Count -gt 0
$taskControllerPidPresent = @(Get-CimInstance Win32_Process -Filter ('ProcessId=' + $taskController.controllerPid)).Count -gt 0
$taskReaderPidPresent = @(Get-CimInstance Win32_Process -Filter ('ProcessId=' + $taskReader.readerPid)).Count -gt 0
if ($taskFormerPidPresent) { throw 'Former PID is present; identity requires review' }
if ($taskLastLive.nativeCount -ne 1 -or $taskLastLive.nativeProcesses[0].ProcessId -ne $taskAdmin.pid) {
    throw 'Last live checkpoint identity mismatch'
}
$taskHead = (& git -C $taskCode rev-parse HEAD).Trim()
if ($taskHead -cne $taskAdmin.source -or @(& git -C $taskCode status --porcelain).Count) {
    throw 'Candidate source changed or worktree not clean'
}
$taskCandidate = (Get-FileHash -LiteralPath $taskAdmin.executable).Hash.ToLowerInvariant()
if ($taskCandidate -cne $taskAdmin.sha256) { throw 'Candidate image changed' }
$taskDaily = (Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLowerInvariant()
if ($taskDaily -cne $taskOrdinary.dailyExeSha256) { throw 'Daily image changed' }
$taskProtected = [ordered]@{}
foreach ($taskProperty in $taskOrdinary.protectedSandboxHashes.PSObject.Properties) {
    $taskHash = (Get-FileHash -LiteralPath (Join-Path $taskCode ('test/runtime/windows0110/data/manager-state/' + $taskProperty.Name))).Hash.ToLowerInvariant()
    if ($taskHash -cne $taskProperty.Value) { throw 'Protected sandbox changed' }
    $taskProtected[$taskProperty.Name] = $taskHash
}
$taskTaskPath = 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\work\objects\task\TASK-OPENCODEX-DESKTOP-182.json'
$taskTaskHash = (Get-FileHash -LiteralPath $taskTaskPath).Hash.ToLowerInvariant()
if ($taskTaskHash -cne 'bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93') {
    throw 'Original TASK changed'
}
$taskTask = Get-Content -LiteralPath $taskTaskPath -Raw -Encoding utf8 | ConvertFrom-Json
$taskObservers = @()
foreach ($taskSource in @(
    [pscustomobject]@{Root=$taskPrior;Name='administrator-exit-observer-result.json'},
    [pscustomobject]@{Root=$taskFollowup;Name='followup-exit-observer-result.json'}
)) {
    $taskObserverPath = Join-Path $taskSource.Root $taskSource.Name
    $taskObserver = Get-Content -LiteralPath $taskObserverPath -Raw -Encoding utf8 | ConvertFrom-Json
    if ($taskObserver.pid -ne $taskAdmin.pid -or
        ([datetimeoffset]$taskObserver.creationTimeUtc).UtcTicks -ne ([datetimeoffset]$taskAdmin.creationTimeUtc).UtcTicks -or
        !$taskObserver.timeout -or $taskObserver.exitObserved -or $null -ne $taskObserver.exitCode) {
        throw 'Unexpected historical exit observer'
    }
    $taskCopyPath = Join-Path $PSScriptRoot $taskSource.Name
    if (Test-Path -LiteralPath $taskCopyPath) { throw 'Observer copy already exists' }
    Copy-Item -LiteralPath $taskObserverPath -Destination $taskCopyPath
    $taskObservers += [ordered]@{
        file=$taskSource.Name;atUtc=$taskObserver.atUtc;timeout=$true;exitObserved=$false;exitCode=$null
        sha256=(Get-FileHash -LiteralPath $taskObserverPath).Hash.ToLowerInvariant()
    }
}
$taskLogPath = Join-Path $PSScriptRoot 'app-log-after-exit.original.txt'
if (Test-Path -LiteralPath $taskLogPath) { throw 'Log checkpoint already exists' }
[IO.File]::WriteAllBytes($taskLogPath,[IO.File]::ReadAllBytes((Join-Path $taskAdmin.dataRoot 'logs/app.log')))
$taskNativeAfter = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'" |
    Select-Object ProcessId, CreationDate, ExecutablePath)
if ($taskNativeAfter.Count) { throw 'A new native application appeared during exit checks' }
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o')
    nativeAbsentObservedAtUtc=$taskAbsentObserved
    userSignalOrigin='human';userSignal='我已退出';userReportedExit=$true
    source=$taskHead;codeWorktreeClean=$true
    formerPid=$taskAdmin.pid;formerCreationTimeUtc=$taskAdmin.creationTimeUtc;formerPidPresent=$false
    controllerPid=$taskController.controllerPid;controllerPidPresent=$taskControllerPidPresent
    followupReaderPid=$taskReader.readerPid;followupReaderPidPresent=$taskReaderPidPresent
    nativeCount=$taskNative.Count;nativeProcesses=$taskNative
    nativeCountAfterProtectionChecks=$taskNativeAfter.Count
    priorLiveCheckpointAtUtc=$taskLastLive.atUtc;priorLiveCheckpointUnchanged=$true
    historicalObservers=$taskObservers;actualExitTimeUtc=$null;actualExitCode=$null
    candidateExeSha256=$taskCandidate;dailyExeSha256=$taskDaily;dailyExeUnchanged=$true
    protectedSandboxHashes=$taskProtected;protectedSandboxUnchanged=$true
    originalTaskSha256=$taskTaskHash;originalTaskBytes=(Get-Item -LiteralPath $taskTaskPath).Length
    originalTaskRevision=$taskTask.revision;originalTaskState=$taskTask.state;originalTaskResult=$taskTask.result
    originalTaskRequiredChecks=$taskTask.required_checks
    productLaunched=$false;uiInputInjected=$false;foregroundWindowExtended=$false
    agentForcedTermination=$false;installerExecuted=$false;accountOrProfileCreated=$false
    macNativeHandoffPerformed=$false;fullVersionGatePassed=$false;taskClosed=$false;releaseRequested=$false
} | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $taskResultPath -Encoding utf8

$ErrorActionPreference='Stop'
$taskResultPath=Join-Path $PSScriptRoot 'exit-postcheck.original.json'
if (Test-Path -LiteralPath $taskResultPath) { throw 'Do not overwrite prior exit evidence' }
$taskPriorRoot=Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-FG-493-UPDATES-TREE'
$taskPrior=Get-Content -LiteralPath (Join-Path $taskPriorRoot 'foreground-postcheck.json') -Raw | ConvertFrom-Json
$taskRestart=Get-Content -LiteralPath (Join-Path $taskPriorRoot 'ordinary-restart.json') -Raw | ConvertFrom-Json
$taskObserver=Get-Content -LiteralPath (Join-Path $taskPriorRoot 'restart-exit-observer-result.json') -Raw | ConvertFrom-Json
if ($taskPrior.source -cne '493da80675e6508e46e0fd83d86ee7216abf96fe' -or $taskPrior.nativeCount -ne 1 -or
    $taskRestart.pid -ne 77760 -or $taskObserver.exitObserved -or !$taskObserver.timeout -or
    $null -ne $taskObserver.exitCode) { throw 'Historical identity or observer boundary changed' }
$taskNative=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'" |
    Select-Object ProcessId,CreationDate,ExecutablePath)
if ($taskNative.Count) { throw 'A native product still exists; do not report zero' }
$taskAbsentAt=[datetimeoffset]::UtcNow.ToString('o')
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskDaily=(Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLowerInvariant()
if ($taskDaily -cne $taskPrior.dailyExeSha256) { throw 'Daily EXE changed' }
$taskExeHash=(Get-FileHash -LiteralPath $taskRestart.executable).Hash.ToLowerInvariant()
if ($taskExeHash -cne $taskPrior.exeSha256) { throw 'Candidate EXE changed' }
$taskProtected=[ordered]@{}
foreach ($taskProperty in $taskPrior.protectedSandboxHashes.PSObject.Properties) {
    $taskName=$taskProperty.Name
    $taskPath=Join-Path $taskCode ('test/runtime/windows0110/data/manager-state/'+$taskName)
    $taskProtected[$taskName]=(Get-FileHash -LiteralPath $taskPath).Hash.ToLowerInvariant()
    if ($taskProtected[$taskName] -cne $taskProperty.Value) { throw 'Earlier protected data changed' }
}
$taskTaskPath='D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\work\objects\task\TASK-OPENCODEX-DESKTOP-182.json'
$taskTaskHash=(Get-FileHash -LiteralPath $taskTaskPath).Hash.ToLowerInvariant()
if ($taskTaskHash -cne $taskPrior.originalTaskSha256) { throw 'Original TASK changed' }
$taskSource=(& git -C $taskCode rev-parse HEAD).Trim()
if ($taskSource -cne $taskPrior.source -or @(& git -C $taskCode status --porcelain).Count) {
    throw 'Code worktree identity or clean state changed'
}
$taskSnapshotPath=Join-Path $taskPriorRoot 'owned-state-after-restart/snapshot.json'
$taskSnapshot=Get-Content -LiteralPath $taskSnapshotPath -Raw | ConvertFrom-Json
$taskReread=Join-Path $PSScriptRoot 'reread'
if (Test-Path -LiteralPath $taskReread) { throw 'Prior reread exists; preserve its bytes' }
$taskData=[IO.Path]::GetFullPath($taskRestart.dataRoot).TrimEnd('\')
if ($taskData -cne (Join-Path $taskCode 'test\runtime\foreground-updates-493-20261015T1130\data')) {
    throw 'Unexpected isolated data root'
}
$taskFiles=@()
foreach ($taskRow in $taskSnapshot.files) {
    $taskRelative=[string]$taskRow.path
    if ($taskRelative -notlike 'backups/*' -and $taskRelative -notlike 'cache/updates/*' -and
        $taskRelative -notin @('manager-state/preferences.json','manager-state/update-schedule.json')) { continue }
    if ($taskRelative.Contains('..') -or [IO.Path]::IsPathRooted($taskRelative)) { throw 'Invalid relative path' }
    $taskPath=[IO.Path]::GetFullPath((Join-Path $taskData $taskRelative))
    $taskOutput=[IO.Path]::GetFullPath((Join-Path $taskReread $taskRelative))
    if (!$taskPath.StartsWith($taskData+'\',[StringComparison]::OrdinalIgnoreCase) -or
        !$taskOutput.StartsWith([IO.Path]::GetFullPath($taskReread)+'\',[StringComparison]::OrdinalIgnoreCase)) {
        throw 'Reread escaped owned boundaries'
    }
    $taskBytes=[IO.File]::ReadAllBytes($taskPath)
    $taskHash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($taskBytes)).ToLowerInvariant()
    if ($taskBytes.Length -ne $taskRow.bytes -or $taskHash -cne $taskRow.sha256) {
        throw 'Isolated persistent file differs from the post-restart snapshot'
    }
    [IO.Directory]::CreateDirectory((Split-Path -Parent $taskOutput)) | Out-Null
    [IO.File]::WriteAllBytes($taskOutput,$taskBytes)
    $taskFiles += [ordered]@{path=$taskRelative;bytes=$taskBytes.Length;sha256=$taskHash;matchesAfterRestartSnapshot=$true}
}
if ($taskFiles.Count -ne 5) { throw 'Expected five unchanged persistent files' }
$taskNativeAfter=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'" |
    Select-Object ProcessId,CreationDate,ExecutablePath)
if ($taskNativeAfter.Count) { throw 'A product appeared during the checkpoint' }
$taskResult=[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');nativeAbsentObservedAtUtc=$taskAbsentAt
    userSignal='已退出';userSignalOrigin='human';source=$taskSource;candidateExeSha256=$taskExeHash
    formerPid=$taskRestart.pid;formerCreationTimeUtc=$taskRestart.creationTimeUtc
    nativeCount=0;nativeProcesses=@();nativeCountAfterProtectionChecks=0
    userReportedExit=$true;actualExitTimeUtc=$null;actualExitCode=$null
    originalObserverTimedOut=$true;originalObserverExitObserved=$false;originalObserverExitCode=$null
    priorLiveCheckpointAtUtc=$taskPrior.atUtc;priorLiveCheckpointChanged=$false
    dailyExeUnchanged=$true;dailyExeSha256=$taskDaily
    protectedSandboxUnchanged=$true;protectedSandboxHashes=$taskProtected
    originalTaskSha256=$taskTaskHash;originalTaskBytes=(Get-Item -LiteralPath $taskTaskPath).Length
    codeWorktreeClean=$true;unchangedIsolatedPersistentFiles=$taskFiles
    applicationStarted=$false;agentForcedTermination=$false;uiInputIssued=$false
    installerExecuted=$false;thirdPartyLifecycleScriptsExecuted=$false
    foregroundWindowExtended=$false;macNativeHandoffPerformed=$false
    fullVersionGatePassed=$false;taskClosed=$false;releaseRequested=$false
}
[IO.File]::WriteAllText($taskResultPath,($taskResult|ConvertTo-Json -Depth 10)+"`n",[Text.UTF8Encoding]::new($false))
$taskResult|ConvertTo-Json -Depth 10

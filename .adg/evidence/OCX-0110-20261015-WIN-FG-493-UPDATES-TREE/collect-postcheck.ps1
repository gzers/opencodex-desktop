$ErrorActionPreference='Stop'
$taskResultPath=Join-Path $PSScriptRoot 'foreground-postcheck.json'
$taskLogPath=Join-Path $PSScriptRoot 'foreground-app-log.original.txt'
if ((Test-Path -LiteralPath $taskResultPath) -or (Test-Path -LiteralPath $taskLogPath)) {
    throw 'Do not overwrite an earlier checkpoint'
}
$taskExitPath=Join-Path $PSScriptRoot 'restart-exit-observer-result.json'
if (!(Test-Path -LiteralPath $taskExitPath)) { throw 'Observer still waiting; read its actual result later' }
$taskExit=Get-Content -LiteralPath $taskExitPath -Raw | ConvertFrom-Json
$taskLaunch=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'ordinary-restart.json') -Raw | ConvertFrom-Json
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskNative=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'" |
    Select-Object ProcessId,CreationDate,ExecutablePath)
if ($taskNative.Count -gt 1) { throw 'Unexpected multiple native products; investigate identity before proceeding' }
if ($taskNative.Count -eq 1) {
    if ($taskNative[0].ProcessId -ne $taskLaunch.pid -or
        $taskNative[0].ExecutablePath -cne $taskLaunch.executable) {
        throw 'Unexpected native identity'
    }
    $taskProcess=Get-Process -Id $taskLaunch.pid
    . (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
    $taskToken=Get-WindowsProcessToken -Process $taskProcess -ExpectedExecutable $taskLaunch.executable
    if (([datetimeoffset]$taskToken.creationTimeUtc).UtcDateTime.Ticks -ne
        ([datetimeoffset]$taskLaunch.creationTimeUtc).UtcDateTime.Ticks) { throw 'PID was reused' }
} else { $taskToken=$null }
$taskExpected=[ordered]@{
    'preferences.json'='8498f7cf8984c74a555202027800027fa7fd1366e0e5ce51c455aaf3ba1fdb7a'
    'data-root.json'='8ae75370e9f9966182caddd12ef094f16449c88119e66012a8a4a99a0c1f0e25'
    'backup-cleanup-policy.json'='85884ab649b82b6b4ab880fe24492c6c400a266254277e29e78c8080b20d2e11'
    'update-schedule.json'='045991fb0a7a71604632e5c9255c3fe008e8e5f21c7e28feabe0d4178bc4ffd2'
}
$taskProtected=[ordered]@{}
foreach ($taskName in $taskExpected.Keys) {
    $taskProtected[$taskName]=(Get-FileHash -LiteralPath (Join-Path $taskCode ('test/runtime/windows0110/data/manager-state/'+$taskName))).Hash.ToLowerInvariant()
    if ($taskProtected[$taskName] -cne $taskExpected[$taskName]) { throw 'Earlier protected data changed' }
}
$taskDaily=(Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLowerInvariant()
if ($taskDaily -cne '43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67') { throw 'Daily EXE changed' }
$taskTask=(Get-FileHash -LiteralPath 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\work\objects\task\TASK-OPENCODEX-DESKTOP-182.json').Hash.ToLowerInvariant()
if ($taskTask -cne 'bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93') { throw 'Original TASK changed' }
$taskExeHash=(Get-FileHash -LiteralPath $taskLaunch.executable).Hash.ToLowerInvariant()
if ($taskExeHash -cne $taskLaunch.sha256) { throw 'Candidate bytes changed' }
$taskSource=(& git -C $taskCode rev-parse HEAD).Trim()
if ($taskSource -cne $taskLaunch.source) { throw 'Code source changed' }
$taskClean=(@(& git -C $taskCode status --porcelain).Count -eq 0)
if (!$taskClean) { throw 'Code worktree no longer clean' }
$taskLog=Join-Path $taskLaunch.dataRoot 'logs/app.log'
[IO.File]::WriteAllBytes($taskLogPath,[IO.File]::ReadAllBytes($taskLog))
$taskPost=[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskSource;exeSha256=$taskExeHash
    nativeCount=$taskNative.Count;nativeProcesses=$taskNative;retainedRestartTokenAtCheckpoint=$taskToken
    firstExitCode=0;restartExitObserved=$taskExit.exitObserved;restartExitCode=$taskExit.exitCode
    restartObserverTimedOut=$taskExit.timeout
    uiInputStoppedAfterCaptureRecoveryFailure=$true;agentForcedTermination=$false
    lastUiInputAtUtc='2026-10-15T03:44:46.997Z'
    foregroundWindowExpiresUtc='2026-10-15T04:00:22.8066504Z'
    codeWorktreeClean=$taskClean;dailyExeUnchanged=$true;dailyExeSha256=$taskDaily
    protectedSandboxUnchanged=$true;protectedSandboxHashes=$taskProtected;originalTaskSha256=$taskTask
    cdpUsedForUiInput=$false;hostInstallerExecuted=$false;thirdPartyLifecycleScriptsExecuted=$false
    systemDpiChanged=$false;systemTransparencyChanged=$false
    fullVersionGatePassed=$false;releaseRequested=$false;macNativeHandoffPerformed=$false
}
[IO.File]::WriteAllText($taskResultPath,($taskPost|ConvertTo-Json -Depth 10)+"`n",[Text.UTF8Encoding]::new($false))
$taskPost|ConvertTo-Json -Depth 10

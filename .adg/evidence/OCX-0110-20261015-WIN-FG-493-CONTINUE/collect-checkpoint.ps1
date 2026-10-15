$ErrorActionPreference='Stop'
$taskPostPath=Join-Path $PSScriptRoot 'checkpoint.original.json'
$taskLogPath=Join-Path $PSScriptRoot 'app-log-at-checkpoint.original.txt'
if((Test-Path -LiteralPath $taskPostPath) -or (Test-Path -LiteralPath $taskLogPath)){throw 'Do not overwrite checkpoint evidence'}
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskSource='493da80675e6508e46e0fd83d86ee7216abf96fe'
$taskOrdinary=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'ordinary-launch.json') -Raw | ConvertFrom-Json
$taskAdmin=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-launch.json') -Raw | ConvertFrom-Json
$taskSamplerResult=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-window-sampler-result.json') -Raw | ConvertFrom-Json
if($taskSamplerResult.pid -ne $taskAdmin.pid){throw 'Sampler identity mismatch'}
if((Get-FileHash -LiteralPath $taskAdmin.executable).Hash.ToLowerInvariant() -cne $taskAdmin.sha256){throw 'Candidate image changed'}
if((& git -C $taskCode rev-parse HEAD).Trim() -cne $taskSource -or @(& git -C $taskCode status --porcelain).Count){throw 'Source/worktree changed'}
$taskDaily=(Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLowerInvariant()
if($taskDaily -cne $taskOrdinary.dailyExeSha256){throw 'Daily image changed'}
$taskProtected=[ordered]@{}
foreach($taskProperty in $taskOrdinary.protectedSandboxHashes.PSObject.Properties) {
    $taskHash=(Get-FileHash -LiteralPath (Join-Path $taskCode ('test/runtime/windows0110/data/manager-state/'+$taskProperty.Name))).Hash.ToLowerInvariant()
    if($taskHash -cne $taskProperty.Value){throw 'Protected sandbox changed'}
    $taskProtected[$taskProperty.Name]=$taskHash
}
$taskTaskHash=(Get-FileHash -LiteralPath 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\work\objects\task\TASK-OPENCODEX-DESKTOP-182.json').Hash.ToLowerInvariant()
if($taskTaskHash -cne 'bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93'){throw 'Original TASK changed'}
$taskNative=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'" |
    Select-Object ProcessId,CreationDate,ExecutablePath)
if($taskNative.Count -gt 1 -or ($taskNative.Count -eq 1 -and $taskNative[0].ProcessId -ne $taskAdmin.pid)){throw 'Unexpected native identity'}
$taskControllerRequest=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-controller-request.json') -Raw | ConvertFrom-Json
$taskController=@(Get-CimInstance Win32_Process -Filter ("ProcessId="+$taskControllerRequest.controllerPid) |
    Select-Object ProcessId,CreationDate,ExecutablePath)
$taskExitResultPath=Join-Path $PSScriptRoot 'administrator-exit-observer-result.json'
$taskExitAvailable=Test-Path -LiteralPath $taskExitResultPath
$taskExit=$null
if($taskExitAvailable) {
    $taskExit=Get-Content -LiteralPath $taskExitResultPath -Raw | ConvertFrom-Json
    if($taskExit.pid -ne $taskAdmin.pid){throw 'Exit result identity mismatch'}
    [IO.File]::WriteAllBytes((Join-Path $PSScriptRoot 'administrator-exit-at-checkpoint.original.json'),[IO.File]::ReadAllBytes($taskExitResultPath))
}
[IO.File]::WriteAllBytes($taskLogPath,[IO.File]::ReadAllBytes((Join-Path $taskAdmin.dataRoot 'logs/app.log')))
$taskPost=[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskSource;candidateExeSha256=$taskAdmin.sha256
    nativeCount=$taskNative.Count;nativeProcesses=$taskNative
    controllerProcessesObserved=$taskController;controllerPid=$taskControllerRequest.controllerPid
    currentOrdinaryCimMayNotReadElevatedExecutable=$true
    actualProductTokenEvidence='administrator-launch.json and administrator-window-sampler-ready.json'
    administratorObserverResultAvailable=$taskExitAvailable
    administratorExitObserved=$(if($taskExitAvailable){$taskExit.exitObserved}else{$false})
    administratorExitCode=$(if($taskExitAvailable){$taskExit.exitCode}else{$null})
    controllerLiveAtCheckpoint=($taskController.Count -eq 1)
    administratorSamplerTerminal=$true;administratorSamplerSamples=$taskSamplerResult.samples
    dailyExeUnchanged=$true;dailyExeSha256=$taskDaily
    protectedSandboxUnchanged=$true;protectedSandboxHashes=$taskProtected;originalTaskSha256=$taskTaskHash
    codeWorktreeClean=$true;agentForcedTermination=$false;installerExecuted=$false
    cdpUsedForUiInput=$false;systemDpiChanged=$false;systemSecurityOrGlobalEnvironmentChanged=$false
    thirdPartyLifecycleScriptsExecuted=$false;macNativeHandoffPerformed=$false
    newAccountOrProfileCreated=$false;fullVersionGatePassed=$false;taskClosed=$false;releaseRequested=$false
}
[IO.File]::WriteAllText($taskPostPath,($taskPost|ConvertTo-Json -Depth 10)+"`n",[Text.UTF8Encoding]::new($false))
$taskPost|ConvertTo-Json -Depth 10

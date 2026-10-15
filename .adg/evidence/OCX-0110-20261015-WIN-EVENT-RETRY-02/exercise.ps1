$ErrorActionPreference='Stop'
$taskRoot='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\foreground-02-current-20261015/install-a/data'
$taskPointer=(Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop').StartupDataRootV1 | ConvertFrom-Json
if($taskPointer.anchor -notlike '*foreground-02-current-20261015\install-a\data'){throw 'Unexpected test startup pointer'}
$taskIdle=Get-Content -LiteralPath (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-IDLE-02-019-FINAL/terminal.json') -Raw | ConvertFrom-Json
if(!$taskIdle.measurementCompleted){throw 'Pair collection has not finished; never mutate its scenario'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Another product instance exists'}
$taskPrevious=Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-FG-02-ROOT'
$taskShell=[Diagnostics.Process]::GetCurrentProcess().MainModule.FileName
$taskNative=Join-Path $PSScriptRoot 'native.ps1'
foreach($taskMethod in @('native.ps1','launch.ps1')){
    if(Test-Path -LiteralPath (Join-Path $PSScriptRoot $taskMethod)){throw 'Event test method already copied; inspect previous attempt'}
    $taskMethodSource=if($taskMethod -eq 'native.ps1'){
        Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-IDLE-02-019-FINAL/native.ps1'
    }else{Join-Path $taskPrevious $taskMethod}
    Copy-Item -LiteralPath $taskMethodSource -Destination (Join-Path $PSScriptRoot $taskMethod)
}
$taskPhase='event-retry'
Start-Process -FilePath $taskShell -WindowStyle Hidden -ArgumentList @(
    '-NoLogo','-NoProfile','-NonInteractive','-ExecutionPolicy','RemoteSigned','-File',
    (Join-Path $PSScriptRoot 'launch.ps1'),'-Mode','ordinary','-Phase',$taskPhase,'-Location','install-b'
) -RedirectStandardOutput (Join-Path $PSScriptRoot 'launch-controller.stdout.txt') -RedirectStandardError (Join-Path $PSScriptRoot 'launch-controller.stderr.txt')
Start-Sleep -Seconds 3
& $taskShell -NoLogo -NoProfile -NonInteractive -ExecutionPolicy RemoteSigned -File $taskNative -Launch $taskPhase -Phase event-retry-ready *> (Join-Path $PSScriptRoot 'ready.log')
if($LASTEXITCODE -ne 0){throw 'Event fixture product not ready'}
$taskPreferences=Join-Path $taskRoot 'manager-state/preferences.json'
$taskNotifications=Join-Path $taskRoot 'manager-state/notifications.json'
$taskItem=Get-Item -LiteralPath $taskPreferences -Force
if($taskItem.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Preference alias refused'}
$taskAttributes=$taskItem.Attributes
$taskBeforeHash=(Get-FileHash -LiteralPath $taskPreferences).Hash.ToLowerInvariant()
Copy-Item -LiteralPath $taskPreferences -Destination (Join-Path $PSScriptRoot 'before-preferences.original.json')
Copy-Item -LiteralPath $taskNotifications -Destination (Join-Path $PSScriptRoot 'before-notifications.original.json')
$taskOld=Get-Content -LiteralPath $taskPreferences -Raw | ConvertFrom-Json
if($taskOld.appearance.theme -cne 'light'){throw 'Exact retry fixture requires light before requesting dark'}
try{
    [IO.File]::SetAttributes($taskPreferences,($taskAttributes -bor [IO.FileAttributes]::ReadOnly))
    & $taskShell -NoLogo -NoProfile -NonInteractive -ExecutionPolicy RemoteSigned -File $taskNative -Launch $taskPhase -Phase event-save-readonly-failed -Action invoke -Name '深色' *> (Join-Path $PSScriptRoot 'failed-action.log')
    if($LASTEXITCODE -ne 0){throw 'Native failed-save action not observed'}
    Start-Sleep -Milliseconds 700
    Copy-Item -LiteralPath $taskPreferences -Destination (Join-Path $PSScriptRoot 'failed-preferences.original.json')
    Copy-Item -LiteralPath $taskNotifications -Destination (Join-Path $PSScriptRoot 'failed-notifications.original.json')
    $taskFailureStore=Get-Content -LiteralPath $taskNotifications -Raw | ConvertFrom-Json
    $taskFailures=@($taskFailureStore.items | Where-Object {$_.notification_id -like 'preferences-save-failed:*' -and !$_.resolved})
    if((Get-FileHash -LiteralPath $taskPreferences).Hash.ToLowerInvariant() -cne $taskBeforeHash -or $taskFailures.Count -ne 1){
        throw 'Readonly fixture did not establish one real unchanged-preference failure; no success claim'
    }
}finally{
    [IO.File]::SetAttributes($taskPreferences,$taskAttributes)
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');originalAttributes=[int]$taskAttributes;attributesRestored=([int](Get-Item -LiteralPath $taskPreferences -Force).Attributes -eq [int]$taskAttributes);fixture='Readonly flag on the owned test preferences only';aclChanged=$false} |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'attributes-restored.json') -Encoding utf8
}
& $taskShell -NoLogo -NoProfile -NonInteractive -ExecutionPolicy RemoteSigned -File $taskNative -Launch $taskPhase -Phase event-save-exact-retry -Action invoke -Name '深色' *> (Join-Path $PSScriptRoot 'retry-action.log')
if($LASTEXITCODE -ne 0){throw 'Native exact retry action not observed'}
Start-Sleep -Milliseconds 700
Copy-Item -LiteralPath $taskPreferences -Destination (Join-Path $PSScriptRoot 'retry-preferences.original.json')
Copy-Item -LiteralPath $taskNotifications -Destination (Join-Path $PSScriptRoot 'retry-notifications.original.json')
$taskRetryStore=Get-Content -LiteralPath $taskNotifications -Raw | ConvertFrom-Json
$taskResolved=@($taskRetryStore.items | Where-Object {$_.notification_id -ceq $taskFailures[0].notification_id})
$taskSaved=Get-Content -LiteralPath $taskPreferences -Raw | ConvertFrom-Json
if($taskSaved.appearance.theme -cne 'dark' -or $taskResolved.Count -ne 1 -or !$taskResolved[0].resolved -or !$taskResolved[0].resolved_at){throw 'Real exact retry did not resolve its original failure'}
foreach($taskOther in @($taskFailureStore.items | Where-Object {$_.notification_id -cne $taskFailures[0].notification_id -and !$_.resolved})){
    $taskAfter=@($taskRetryStore.items | Where-Object notification_id -CEQ $taskOther.notification_id)
    if($taskAfter.Count -ne 1 -or $taskAfter[0].resolved){throw 'Unrelated unresolved event was cleared'}
}
& $taskShell -NoLogo -NoProfile -NonInteractive -ExecutionPolicy RemoteSigned -File $taskNative -Launch $taskPhase -Phase event-retry-close -Action close *> (Join-Path $PSScriptRoot 'close.log')
Start-Sleep -Seconds 1
$taskExit=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'event-retry-exit.json') -Raw | ConvertFrom-Json
if(!$taskExit.exitObserved -or $taskExit.exitCode -ne 0){throw 'Event test did not exit normally'}
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');failureId=$taskFailures[0].notification_id
    identityBefore=$taskFailures[0].event_identity;identityAfter=$taskResolved[0].event_identity
    resolvedAt=$taskResolved[0].resolved_at;realPreferenceWriteSucceeded=$true
    originalFailureResolved=$true;unrelatedFailuresRetained=$true
    normalExit=$true;physicalDpiGateClaimed=$false;fullEventGatePassed=$false;fullVersionGatePassed=$false
} | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'result.json') -Encoding utf8

$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskSource='493da80675e6508e46e0fd83d86ee7216abf96fe'
$taskSha='9ca21e30c9c2eee68000e1e8a03c58a42918e46d97387f96a102b7f11afb326a'
$taskPriorRoot=Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-FG-493-UPDATES-TREE'
$taskExitRoot=Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-FG-493-EXIT'
$taskPrior=Get-Content -LiteralPath (Join-Path $taskPriorRoot 'ordinary-restart.json') -Raw | ConvertFrom-Json
$taskExit=Get-Content -LiteralPath (Join-Path $taskExitRoot 'exit-postcheck.original.json') -Raw | ConvertFrom-Json
. (Join-Path $PSScriptRoot 'foreground-contract-live.ps1')
$taskAuth=Assert-BackupForegroundAuthorization -Authorization (
    Get-Content -LiteralPath (Join-Path $PSScriptRoot 'foreground-authorization.json') -Raw | ConvertFrom-Json)
if ($taskExit.nativeCount -ne 0 -or !$taskExit.userReportedExit -or $taskExit.formerPid -ne $taskPrior.pid) {
    throw 'Prior instance exit and zero checkpoint required'
}
if ((& git -C $taskCode rev-parse HEAD).Trim() -cne $taskSource -or @(& git -C $taskCode status --porcelain).Count) {
    throw 'Exact clean code source required'
}
if ((Get-FileHash -LiteralPath $taskPrior.executable).Hash.ToLowerInvariant() -cne $taskSha) { throw 'Candidate digest changed' }
$taskNative=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'")
if ($taskNative.Count) { throw 'A native product exists; do not start a second instance' }
if ((Test-Path -LiteralPath (Join-Path $PSScriptRoot 'ordinary-launch.json')) -or
    (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'ordinary-launch.identity-pending.json'))) { throw 'Launch already attempted' }
$taskProtected=[ordered]@{}
foreach ($taskProperty in $taskExit.protectedSandboxHashes.PSObject.Properties) {
    $taskPath=Join-Path $taskCode ('test/runtime/windows0110/data/manager-state/'+$taskProperty.Name)
    $taskProtected[$taskProperty.Name]=(Get-FileHash -LiteralPath $taskPath).Hash.ToLowerInvariant()
    if ($taskProtected[$taskProperty.Name] -cne $taskProperty.Value) { throw 'Protected sandbox changed' }
}
foreach ($taskFile in $taskExit.unchangedIsolatedPersistentFiles) {
    $taskPath=Join-Path $taskPrior.dataRoot $taskFile.path
    if ((Get-FileHash -LiteralPath $taskPath).Hash.ToLowerInvariant() -cne $taskFile.sha256) {
        throw 'Isolated data changed since verified exit'
    }
}
$taskDaily=(Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLowerInvariant()
if ($taskDaily -cne $taskExit.dailyExeSha256) { throw 'Daily EXE changed' }
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
$taskHarness=Get-WindowsHarnessToken
Assert-WindowsProcessMode -Observation $taskHarness -Mode ordinary
$taskInfo=[Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName=$taskPrior.executable;$taskInfo.WorkingDirectory=Split-Path -Parent $taskPrior.executable
$taskInfo.UseShellExecute=$false
$taskInfo.Environment['OPENCODEX_SANDBOX']='1'
$taskInfo.Environment['OPENCODEX_SANDBOX_ROOT']=$taskPrior.dataRoot
$taskInfo.Environment['OPENCODEX_SANDBOX_BOUNDARY']=$taskPrior.runtime
$taskSocket=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0)
$taskSocket.Start();$taskPort=$taskSocket.LocalEndpoint.Port;$taskSocket.Stop()
$taskInfo.Environment['OPENCODEX_WINDOWS_SMOKE_CDP_PORT']=[string]$taskPort
foreach ($taskName in @('OPENCODEX_WINDOWS_SMOKE_SCALE','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','WEBVIEW2_USER_DATA_FOLDER')) {
    [void]$taskInfo.Environment.Remove($taskName)
}
$taskProcess=[Diagnostics.Process]::Start($taskInfo)
$taskRecord=[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskSource;sha256=$taskSha;executable=$taskPrior.executable
    runtime=$taskPrior.runtime;dataRoot=$taskPrior.dataRoot;pid=$taskProcess.Id
    creationTimeUtc=$taskProcess.StartTime.ToUniversalTime().ToString('o')
    authorization=$taskAuth;harnessToken=$taskHarness;nativeCountBefore=0;priorIsolatedPersistentFilesVerified=5
    preferencesChangedByLauncher=$false;backupsChangedByLauncher=$false;port=$taskPort
    dailyExeUnchanged=$true;dailyExeSha256=$taskDaily
    protectedSandboxUnchanged=$true;protectedSandboxHashes=$taskProtected
    hostInstallerExecuted=$false;forcedScaleUsed=$false;cdpUsedForUiInput=$false
    identityReviewRequired=$true;agentForcedTermination=$false
}
$taskRecord | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'ordinary-launch.identity-pending.json') -Encoding utf8
$taskToken=Get-WindowsProcessToken -Process $taskProcess -ExpectedExecutable $taskPrior.executable
Assert-WindowsProcessMode -Observation $taskToken -Mode ordinary
$taskRecord.applicationToken=$taskToken;$taskRecord.creationTimeUtc=$taskToken.creationTimeUtc;$taskRecord.identityReviewRequired=$false
$taskRecord | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'ordinary-launch.json') -Encoding utf8
$taskRecord | ConvertTo-Json -Depth 10

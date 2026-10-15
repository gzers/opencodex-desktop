$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'foreground-contract-live.ps1')
$taskAuth=Assert-BackupForegroundAuthorization -Authorization (
    Get-Content -LiteralPath (Join-Path $PSScriptRoot 'foreground-authorization.json') -Raw | ConvertFrom-Json)
$taskPrior=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'ordinary-launch.json') -Raw | ConvertFrom-Json
$taskExit=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'first-exit-postcheck.json') -Raw | ConvertFrom-Json
if ($taskExit.nativeCount -ne 0 -or !$taskExit.exitObserved -or $taskExit.exitCode -ne 0) {
    throw 'Prior instance must exit normally and all native instances become zero.'
}
if (@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count) {
    throw 'Do not start a second native instance.'
}
if (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'ordinary-restart.json')) {throw 'Restart already recorded; inspect it.'}
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
if ((& git -C $taskCode rev-parse HEAD).Trim() -cne $taskPrior.source) {throw 'Source changed.'}
if ((Get-FileHash -LiteralPath $taskPrior.executable).Hash.ToLowerInvariant() -cne $taskPrior.sha256) {throw 'Candidate bytes changed.'}
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
Assert-WindowsProcessMode -Observation (Get-WindowsHarnessToken) -Mode ordinary
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
    atUtc=[DateTime]::UtcNow.ToString('o');source=$taskPrior.source;executable=$taskPrior.executable;sha256=$taskPrior.sha256
    runtime=$taskPrior.runtime;dataRoot=$taskPrior.dataRoot;pid=$taskProcess.Id
    creationTimeUtc=$taskProcess.StartTime.ToUniversalTime().ToString('o');priorPid=$taskPrior.pid
    nativeCountBefore=0;preferencesChangedByLauncher=$false;backupsChangedByLauncher=$false
    authorization=$taskAuth;hostInstallerExecuted=$false;forcedScaleUsed=$false;cdpUsedForUiInput=$false
}
$taskRecord | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'ordinary-restart.identity-pending.json') -Encoding utf8
$taskToken=Get-WindowsProcessToken -Process $taskProcess -ExpectedExecutable $taskPrior.executable
Assert-WindowsProcessMode -Observation $taskToken -Mode ordinary
$taskRecord.applicationToken=$taskToken;$taskRecord.creationTimeUtc=$taskToken.creationTimeUtc
$taskRecord | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'ordinary-restart.json') -Encoding utf8
$taskRecord | ConvertTo-Json -Depth 10

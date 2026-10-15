param([switch]$StartAfterForegroundSignal,[string]$ForegroundAuthorizationFile='')
$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskSource='493da80675e6508e46e0fd83d86ee7216abf96fe'
$taskSha='9ca21e30c9c2eee68000e1e8a03c58a42918e46d97387f96a102b7f11afb326a'
$taskExe=Join-Path $taskCode 'test/software/candidate-493da80-38673288645/msi-readonly/opencodex-desktop.exe'
$taskRuntime=Join-Path $taskCode 'test/runtime/foreground-updates-493-20261015T1130'
$taskData=Join-Path $taskRuntime 'data'
$taskEvidence=Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-493-CANDIDATE'
. (Join-Path $PSScriptRoot 'foreground-contract-live.ps1')
if ((& git -C $taskCode rev-parse HEAD).Trim() -cne $taskSource) { throw 'Source changed; re-admit instead of silently switching it.' }
& git -C $taskCode diff --quiet HEAD -- apps/desktop/tauri/config/preferences.defaults.json test/smoke/windows-process-token.ps1
if ($LASTEXITCODE -ne 0) { throw 'Preference defaults or token reader has unreviewed changes.' }
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
if ((Get-FileHash -LiteralPath $taskExe).Hash.ToLowerInvariant() -cne $taskSha) { throw 'Candidate bytes changed.' }
$taskCandidate=Get-Content -LiteralPath (Join-Path $taskEvidence 'archive-verification.json') -Raw | ConvertFrom-Json
if ($taskCandidate.archiveIntegrity -ne 'passed' -or $taskCandidate.source -cne $taskSource -or
    $taskCandidate.applicationSha256 -cne $taskSha) { throw 'Exact-source candidate receipt required.' }
$taskNative=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'" |
    Select-Object ProcessId,ExecutablePath,CreationDate)
if ($taskNative.Count) { throw 'Native application already exists; no second instance.' }
if (Test-Path -LiteralPath $taskRuntime) { throw 'Runtime already exists; do not overwrite or blindly restart it.' }
if (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'ordinary-launch.json')) { throw 'This launch already has a receipt.' }
$taskHarness=Get-WindowsHarnessToken
Assert-WindowsProcessMode -Observation $taskHarness -Mode ordinary
$taskPrior=Get-Content -LiteralPath (Join-Path $taskEvidence 'candidate-local-postcheck.json') -Raw | ConvertFrom-Json
$taskDaily=(Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLowerInvariant()
if ($taskDaily -cne $taskPrior.dailyExeSha256) { throw 'Daily installation identity changed; review first.' }
$taskProtected=[ordered]@{}
foreach ($taskName in @('preferences.json','data-root.json','backup-cleanup-policy.json','update-schedule.json')) {
    $taskProtected[$taskName]=(Get-FileHash -LiteralPath (Join-Path $taskCode ('test/runtime/windows0110/data/manager-state/'+$taskName))).Hash.ToLowerInvariant()
    if ($taskProtected[$taskName] -cne $taskPrior.protectedSandboxHashes.$taskName) { throw 'Earlier protected data changed.' }
}
$taskPlan=[ordered]@{
    atUtc=[DateTime]::UtcNow.ToString('o'); source=$taskSource; executable=$taskExe; sha256=$taskSha
    runtime=$taskRuntime; dataRoot=$taskData; nativeCountBefore=$taskNative.Count
    harnessToken=$taskHarness; mode='ordinary'; permissionMode='review-only-unless-new-human-signal'
    defaultReviewStartsApplication=$false; installerExecuted=$false; foregroundInputIssued=$false
    runtimeCreated=$false; dailyExeUnchanged=$true; dailyExeSha256=$taskDaily
    protectedSandboxUnchanged=$true; protectedSandboxAfter=$taskProtected
    dpiChangePlanned=$false; forcedScalePlanned=$false
    webviewProfile='sandbox data/EBWebView via existing loopback smoke configuration'
    cdpUsedForUiInput=$false; uiInputMethod='Sky only, in the new foreground window'
    initialTheme='dark'; autoUpdate=$false; autoPanel=$false; startupCleanup=$false
    keepOnClose=$false; backupsPreparedOrAgeAdjusted=$false
}
if (!$StartAfterForegroundSignal) { $taskPlan | ConvertTo-Json -Depth 8; return }
if ([string]::IsNullOrWhiteSpace($ForegroundAuthorizationFile)) { throw 'A new recorded human foreground signal is required before launch.' }
$taskAuthPath=(Resolve-Path -LiteralPath $ForegroundAuthorizationFile).Path
$taskEvidenceRoot=[IO.Path]::GetFullPath($PSScriptRoot).TrimEnd('\')
if (!$taskAuthPath.StartsWith($taskEvidenceRoot+'\',[StringComparison]::OrdinalIgnoreCase)) {
    throw 'Authorization record must remain under this owned evidence directory.'
}
$taskAuth=Assert-BackupForegroundAuthorization -Authorization (
    Get-Content -LiteralPath $taskAuthPath -Raw | ConvertFrom-Json)
New-Item -ItemType Directory -Path (Join-Path $taskData 'manager-state') -Force | Out-Null
$taskPrefs=Get-Content -LiteralPath (Join-Path $taskCode 'apps/desktop/tauri/config/preferences.defaults.json') -Raw | ConvertFrom-Json
$taskPrefs.theme='dark';$taskPrefs.auto_panel=$false;$taskPrefs.launch_with_codex=$false
$taskPrefs.app_update_auto_check=$false;$taskPrefs.startup_cleanup=$false;$taskPrefs.keep_proxy_on_close=$false
$taskPrefs | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $taskData 'manager-state/preferences.json') -Encoding utf8
$taskSocket=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0)
$taskSocket.Start();$taskPort=$taskSocket.LocalEndpoint.Port;$taskSocket.Stop()
$taskInfo=[Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName=$taskExe;$taskInfo.WorkingDirectory=Split-Path -Parent $taskExe;$taskInfo.UseShellExecute=$false
$taskInfo.Environment['OPENCODEX_SANDBOX']='1'
$taskInfo.Environment['OPENCODEX_SANDBOX_ROOT']=$taskData
$taskInfo.Environment['OPENCODEX_SANDBOX_BOUNDARY']=$taskRuntime
$taskInfo.Environment['OPENCODEX_WINDOWS_SMOKE_CDP_PORT']=[string]$taskPort
foreach ($taskName in @('OPENCODEX_WINDOWS_SMOKE_SCALE','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','WEBVIEW2_USER_DATA_FOLDER')) {
    [void]$taskInfo.Environment.Remove($taskName)
}
$taskProcess=[Diagnostics.Process]::Start($taskInfo)
$taskRecord=[ordered]@{
    atUtc=[DateTime]::UtcNow.ToString('o'); source=$taskSource; sha256=$taskSha; executable=$taskExe
    runtime=$taskRuntime; dataRoot=$taskData; pid=$taskProcess.Id
    creationTimeUtc=$taskProcess.StartTime.ToUniversalTime().ToString('o')
    authorization=$taskAuth; harnessToken=$taskHarness
    port=$taskPort; hostInstallerExecuted=$false; forcedScaleUsed=$false; cdpUsedForUiInput=$false
    identityReviewRequired=$true; agentForcedTermination=$false
}
# Preserve the new PID even if token observation fails, so no blind second
# instance can be mistaken for a retry of a launch that never occurred.
$taskRecord | ConvertTo-Json -Depth 10 |
    Set-Content -LiteralPath (Join-Path $PSScriptRoot 'ordinary-launch.identity-pending.json') -Encoding utf8
$taskToken=Get-WindowsProcessToken -Process $taskProcess -ExpectedExecutable $taskExe
Assert-WindowsProcessMode -Observation $taskToken -Mode ordinary
$taskRecord.applicationToken=$taskToken
$taskRecord.creationTimeUtc=$taskToken.creationTimeUtc
$taskRecord.identityReviewRequired=$false
$taskRecord | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'ordinary-launch.json') -Encoding utf8
$taskRecord | ConvertTo-Json -Depth 10

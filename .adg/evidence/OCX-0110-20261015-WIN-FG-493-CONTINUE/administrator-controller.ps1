$ErrorActionPreference='Stop'
try {
    $taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
    . (Join-Path $PSScriptRoot 'foreground-contract-live.ps1')
    $taskAuth=Assert-BackupForegroundAuthorization -Authorization (
        Get-Content -LiteralPath (Join-Path $PSScriptRoot 'foreground-authorization.json') -Raw | ConvertFrom-Json)
    $taskPrior=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'ordinary-launch.json') -Raw | ConvertFrom-Json
    $taskExit=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'first-exit-observer-result.json') -Raw | ConvertFrom-Json
    if (!$taskExit.exitObserved -or $taskExit.exitCode -ne 0 -or $taskExit.pid -ne $taskPrior.pid) {
        throw 'Ordinary instance must exit normally before administrator launch'
    }
    if (@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count) {
        throw 'Native product already exists'
    }
    if ((Test-Path -LiteralPath (Join-Path $PSScriptRoot 'administrator-launch.json')) -or
        (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'administrator-launch.identity-pending.json'))) {
        throw 'Administrator launch already attempted'
    }
    if ((& git -C $taskCode rev-parse HEAD).Trim() -cne $taskPrior.source -or
        @(& git -C $taskCode status --porcelain).Count) { throw 'Exact clean source required' }
    if ((Get-FileHash -LiteralPath $taskPrior.executable).Hash.ToLowerInvariant() -cne $taskPrior.sha256) {
        throw 'Candidate image changed'
    }
    . (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
    $taskHarness=Get-WindowsHarnessToken
    Assert-WindowsProcessMode -Observation $taskHarness -Mode administrator
    $taskInfo=[Diagnostics.ProcessStartInfo]::new()
    $taskInfo.FileName=$taskPrior.executable
    $taskInfo.WorkingDirectory=Split-Path -Parent $taskPrior.executable
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
        atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskPrior.source;sha256=$taskPrior.sha256
        executable=$taskPrior.executable;runtime=$taskPrior.runtime;dataRoot=$taskPrior.dataRoot
        pid=$taskProcess.Id;creationTimeUtc=$taskProcess.StartTime.ToUniversalTime().ToString('o')
        authorization=$taskAuth;harnessToken=$taskHarness;nativeCountBefore=0;port=$taskPort
        preferencesChangedByLauncher=$false;backupsChangedByLauncher=$false
        hostInstallerExecuted=$false;forcedScaleUsed=$false;cdpUsedForUiInput=$false
        identityReviewRequired=$true;agentForcedTermination=$false
    }
    $taskRecord | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-launch.identity-pending.json') -Encoding utf8
    $taskToken=Get-WindowsProcessToken -Process $taskProcess -ExpectedExecutable $taskPrior.executable
    Assert-WindowsProcessMode -Observation $taskToken -Mode administrator
    $taskRecord.applicationToken=$taskToken
    $taskRecord.creationTimeUtc=$taskToken.creationTimeUtc
    $taskRecord.identityReviewRequired=$false
    $taskRecord | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-launch.json') -Encoding utf8
    $taskDuration=[int][Math]::Max(1,[Math]::Min(900000,[Math]::Floor($taskAuth.remainingSeconds*1000)))
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskProcess.Id
        creationTimeUtc=$taskToken.creationTimeUtc;handleHeld=$true;timeoutMilliseconds=$taskDuration
        agentForcedTermination=$false;inputInjected=$false} | ConvertTo-Json |
        Set-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-exit-observer-ready.json') -Encoding utf8
    $taskExited=$taskProcess.WaitForExit($taskDuration)
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskProcess.Id
        creationTimeUtc=$taskToken.creationTimeUtc;exitObserved=$taskExited
        exitCode=$(if($taskExited){$taskProcess.ExitCode}else{$null})
        timeout=(-not $taskExited);agentForcedTermination=$false;inputInjected=$false} |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-exit-observer-result.json') -Encoding utf8
    $taskProcess.Dispose()
} catch {
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');errorType=$_.Exception.GetType().Name
        message=$_.Exception.Message;automaticRetryAllowed=$false;agentForcedTermination=$false} |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-controller-error.json') -Encoding utf8
    throw
}

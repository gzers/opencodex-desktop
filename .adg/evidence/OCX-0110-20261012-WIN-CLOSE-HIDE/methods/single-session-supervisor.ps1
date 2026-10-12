param(
    [Parameter(Mandatory=$true)][string]$Executable,
    [Parameter(Mandatory=$true)][string]$ExpectedSha256,
    [Parameter(Mandatory=$true)][string]$ExpectedCommit,
    [ValidateSet('ordinary','administrator')][string]$Mode='ordinary'
)
$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskBase=Join-Path $taskCode 'test/runtime/windows0110'
$taskAnchor=Join-Path $taskBase 'data'
$taskControl=Join-Path $taskBase 'control'
$taskRun=Join-Path $taskBase ('evidence/run-'+[datetime]::UtcNow.ToString('yyyyMMddTHHmmssZ'))
$taskExe=(Resolve-Path -LiteralPath $Executable).Path
if (!$taskExe.StartsWith((Join-Path $taskCode 'test/software')+'\',[StringComparison]::OrdinalIgnoreCase)) {
    throw 'Application must be an extracted candidate in this task software directory'
}
if ($ExpectedCommit -notmatch '^[0-9a-f]{40}$' -or $ExpectedSha256 -notmatch '^[0-9a-f]{64}$') {
    throw 'Exact source and executable identities are mandatory'
}
if ((Get-FileHash -LiteralPath $taskExe).Hash.ToLower() -cne $ExpectedSha256) { throw 'Candidate EXE identity mismatch' }
New-Item -ItemType Directory -Force -Path $taskBase,$taskAnchor,$taskControl,$taskRun | Out-Null
$taskOwnership=[IO.File]::Open(
    (Join-Path $taskControl 'session-owner.lock'),
    [IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None
)
$taskProcess=$null
$taskStdout=$null
$taskStderr=$null
$taskReport=[ordered]@{
    schemaVersion=1;sourceCommit=$ExpectedCommit;executable=$taskExe
    executableSha256=$ExpectedSha256;mode=$Mode;base=$taskBase;anchor=$taskAnchor
    supervisorPid=$PID;supervisorCreationTimeUtc=(Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o')
    startedAtUtc=[datetime]::UtcNow.ToString('o');status='preparing'
    installerExecuted=$false;preservesDailyInstallation=$true
    scope='one retained application process; copied EXE and explicit sandbox, not installer/default-directory acceptance'
}
function Save-SessionReport {
    $taskReport | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $taskRun 'session.json') -Encoding utf8
    $taskReport | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $taskControl 'current-session.json') -Encoding utf8
}
try {
    $taskExisting=@(Get-CimInstance Win32_Process | Where-Object {
        $_.Name -eq 'opencodex-desktop.exe' -and $_.ExecutablePath -and
        $_.ExecutablePath.StartsWith($taskCode+'\',[StringComparison]::OrdinalIgnoreCase)
    })
    if ($taskExisting.Count) { throw 'An isolated application already exists; refusing to open another' }
    . (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
    $taskReport.harnessToken=Get-WindowsHarnessToken
    Assert-WindowsProcessMode -Observation $taskReport.harnessToken -Mode $Mode
    $taskHome=Join-Path $taskAnchor 'home'
    $taskTemp=Join-Path $taskBase 'temp'
    New-Item -ItemType Directory -Force -Path $taskHome,$taskTemp,
        (Join-Path $taskHome 'AppData/Roaming'),(Join-Path $taskHome 'AppData/Local'),
        (Join-Path $taskAnchor 'manager-state') | Out-Null
    $taskPreferencesPath=Join-Path $taskAnchor 'manager-state/preferences.json'
    if (!(Test-Path -LiteralPath $taskPreferencesPath)) {
        $taskPreferences=Get-Content -LiteralPath (Join-Path $taskCode 'apps/desktop/tauri/config/preferences.defaults.json') -Raw | ConvertFrom-Json
        $taskPreferences.schema_version=2
        $taskPreferences.auto_panel=$false
        $taskPreferences.launch_with_codex=$false
        $taskPreferences.app_update_auto_check=$false
        $taskPreferences | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskPreferencesPath -Encoding utf8
        $taskReport.preferencesSeed='supported flat schema-2 defaults with startup panel and automatic checks disabled during initial gate setup'
    } else { $taskReport.preferencesSeed='existing isolated preferences retained' }
    $taskStop=Join-Path $taskControl 'stop.request'
    if (Test-Path -LiteralPath $taskStop) { Remove-Item -LiteralPath $taskStop }
    $taskSocket=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0)
    $taskSocket.Start();$taskPort=$taskSocket.LocalEndpoint.Port;$taskSocket.Stop()
    $taskInfo=[Diagnostics.ProcessStartInfo]::new()
    $taskInfo.FileName=$taskExe
    $taskInfo.WorkingDirectory=Split-Path -Parent $taskExe
    $taskInfo.UseShellExecute=$false;$taskInfo.CreateNoWindow=$true
    $taskInfo.RedirectStandardError=$true;$taskInfo.RedirectStandardOutput=$true
    $taskInfo.Environment['OPENCODEX_SANDBOX']='1'
    $taskInfo.Environment['OPENCODEX_SANDBOX_ROOT']=$taskAnchor
    $taskInfo.Environment['OPENCODEX_SANDBOX_BOUNDARY']=$taskBase
    $taskInfo.Environment['OPENCODEX_WINDOWS_SMOKE_CDP_PORT']=[string]$taskPort
    $taskInfo.Environment['HOME']=$taskHome
    $taskInfo.Environment['USERPROFILE']=$taskHome
    $taskInfo.Environment['APPDATA']=Join-Path $taskHome 'AppData/Roaming'
    $taskInfo.Environment['LOCALAPPDATA']=Join-Path $taskHome 'AppData/Local'
    $taskInfo.Environment['TEMP']=$taskTemp;$taskInfo.Environment['TMP']=$taskTemp
    foreach($taskKey in @(
        'OPENCODEX_WINDOWS_SMOKE_SCALE','WEBVIEW2_USER_DATA_FOLDER',
        'WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','NVM_SYMLINK','NVM_HOME','NODE_HOME'
    )) { [void]$taskInfo.Environment.Remove($taskKey) }
    $taskProcess=[Diagnostics.Process]::Start($taskInfo)
    $taskApplicationCreatedAt=$taskProcess.StartTime.ToUniversalTime()
    $taskStdout=$taskProcess.StandardOutput.ReadToEndAsync()
    $taskStderr=$taskProcess.StandardError.ReadToEndAsync()
    $taskReport.applicationToken=Get-WindowsProcessToken -Process $taskProcess -ExpectedExecutable $taskExe
    Assert-WindowsProcessMode -Observation $taskReport.applicationToken -Mode $Mode
    $taskReport.applicationPid=$taskProcess.Id
    $taskReport.cdpPort=$taskPort
    $taskReport.endpoint="http://127.0.0.1:$taskPort"
    $taskDeadline=[datetime]::UtcNow.AddSeconds(30)
    $taskLog=Join-Path $taskAnchor 'logs/app.log'
    $taskVerified=$false
    while ([datetime]::UtcNow -lt $taskDeadline) {
        if ($taskProcess.HasExited) { throw 'Isolated application exited during initialization' }
        if ((Test-Path -LiteralPath $taskLog) -and
            (Get-Item -LiteralPath $taskLog).LastWriteTimeUtc -ge $taskApplicationCreatedAt -and
            (Get-Content -LiteralPath $taskLog -Raw).Contains("commit=$ExpectedCommit")) {
            $taskVerified=$true;break
        }
        Start-Sleep -Milliseconds 200
    }
    if (!$taskVerified) { throw 'Application startup log did not prove exact source identity' }
    $taskReport.sourceCommitMatched=$true
    $taskReport.startupLogWrittenDuringThisProcess=$true
    $taskReport.status='running'
    Save-SessionReport
    Write-Output ($taskReport | ConvertTo-Json -Depth 7 -Compress)
    $taskClock=[Diagnostics.Stopwatch]::StartNew()
    $taskNextObservation=0.0
    while (!$taskProcess.HasExited -and !(Test-Path -LiteralPath $taskStop) -and $taskClock.Elapsed.TotalHours -lt 48) {
        if ($taskClock.Elapsed.TotalSeconds -ge $taskNextObservation) {
            $taskReport.lastLiveAtUtc=[datetime]::UtcNow.ToString('o')
            $taskReport.elapsedSeconds=$taskClock.Elapsed.TotalSeconds
            $taskReport.applicationToken=Get-WindowsProcessToken -Process $taskProcess -ExpectedExecutable $taskExe
            Assert-WindowsProcessMode -Observation $taskReport.applicationToken -Mode $Mode
            Save-SessionReport
            $taskNextObservation=$taskClock.Elapsed.TotalSeconds+10
        }
        Start-Sleep -Milliseconds 500
    }
    $taskReport.status=if($taskProcess.HasExited){'application-exited'}else{'stop-requested'}
    if ($taskProcess.HasExited) {$taskReport.exitCode=$taskProcess.ExitCode}
} catch {
    $taskReport.status='failed';$taskReport.error=$_.Exception.Message
    throw
} finally {
    # Cleanup is restricted to the process retained by this supervisor.
    if ($taskProcess) {
        if (!$taskProcess.HasExited) {$taskProcess.Kill($true);[void]$taskProcess.WaitForExit(10000)}
        if ($taskStdout -and $taskStdout.Wait(5000)) {$taskStdout.Result | Set-Content -LiteralPath (Join-Path $taskRun 'stdout.log') -Encoding utf8}
        if ($taskStderr -and $taskStderr.Wait(5000)) {$taskStderr.Result | Set-Content -LiteralPath (Join-Path $taskRun 'stderr.log') -Encoding utf8}
        $taskProcess.Dispose()
    }
    $taskReport.finishedAtUtc=[datetime]::UtcNow.ToString('o')
    Save-SessionReport
    $taskOwnership.Dispose()
}

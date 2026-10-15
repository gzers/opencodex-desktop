$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskExe=Join-Path $taskCode 'test/software/candidate-02a0d4-38737628201/msi-readonly/opencodex-desktop.exe'
$taskRuntime=Join-Path $taskCode 'test/runtime/real-state-floor-02-20261015'
if((Test-Path -LiteralPath $taskRuntime) -or (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'state-floor-launch.json'))){throw 'Real state-floor run already exists'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Another native instance exists'}
if((Get-FileHash -LiteralPath $taskExe).Hash.ToLowerInvariant() -cne '7383c275f58fed7e25678420a6d3f5c739f9b23d7373acc40fab1e7cd98db6d9'){throw 'Candidate changed'}
$taskRoot=Join-Path $taskRuntime 'data'
New-Item -ItemType Directory -Path (Join-Path $taskRoot 'manager-state') -Force | Out-Null
$taskPrefs=Get-Content -LiteralPath (Join-Path $taskCode 'apps/desktop/tauri/config/preferences.defaults.json') -Raw | ConvertFrom-Json
$taskPrefs.auto_panel=$false;$taskPrefs.launch_with_codex=$false;$taskPrefs.keep_proxy_on_close=$false
$taskPrefs.startup_cleanup=$false;$taskPrefs.app_update_auto_check=$true
$taskPrefs.theme='light';$taskPrefs.visual_effects='mid'
$taskPrefs | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskRoot 'manager-state/preferences.json') -Encoding utf8
[IO.File]::WriteAllText((Join-Path $taskRoot 'manager-state/update-schedule.json'),'{real-owned-malformed-schedule',[Text.UTF8Encoding]::new($false))
$taskSocket=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0)
$taskSocket.Start();$taskPort=$taskSocket.LocalEndpoint.Port;$taskSocket.Stop()
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
. (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/methods/process-user-sid.ps1')
$taskInfo=[Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName=$taskExe;$taskInfo.WorkingDirectory=Split-Path -Parent $taskExe;$taskInfo.UseShellExecute=$false
$taskInfo.Environment['OPENCODEX_SANDBOX']='1'
$taskInfo.Environment['OPENCODEX_SANDBOX_ROOT']=$taskRoot
$taskInfo.Environment['OPENCODEX_SANDBOX_BOUNDARY']=$taskRuntime
$taskInfo.Environment['OPENCODEX_WINDOWS_SMOKE_CDP_PORT']=[string]$taskPort
foreach($taskName in @('OPENCODEX_WINDOWS_SMOKE_SCALE','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','WEBVIEW2_USER_DATA_FOLDER')){[void]$taskInfo.Environment.Remove($taskName)}
$taskProcess=[Diagnostics.Process]::Start($taskInfo)
try{
    $taskUser=Get-AcceptanceProcessSidEvidence -Process $taskProcess -ExpectedExecutable $taskExe
    Assert-WindowsProcessMode -Observation $taskUser -Mode ordinary
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');phase='state-floor';mode='ordinary'
        humanForegroundAuthorization='现在开始前台验证持续完成后续任务；我给你前台'
        source='02a0d4a8cbae8ef1450908d86b2c75fcad65f04f';executable=$taskExe
        exeSha256='7383c275f58fed7e25678420a6d3f5c739f9b23d7373acc40fab1e7cd98db6d9'
        pid=$taskProcess.Id;creationTimeUtc=$taskUser.creationTimeUtc
        applicationToken=$taskUser;applicationUser=$taskUser;runtime=$taskRuntime;dataRoot=$taskRoot;port=$taskPort
        sandboxUsed=$true;cdpUsed=$true;realMalformedState=$true;systemClockChanged=$false
        frontendOrNativeCommandStubbed=$false;productionFilesChanged=$false
    } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'state-floor-launch.json') -Encoding utf8
    $taskTimer=[Diagnostics.Stopwatch]::StartNew()
    while(!$taskProcess.WaitForExit(1000) -and $taskTimer.Elapsed.TotalSeconds -lt 2400){}
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskProcess.Id
        creationTimeUtc=$taskUser.creationTimeUtc;exitObserved=$taskProcess.HasExited
        exitCode=if($taskProcess.HasExited){$taskProcess.ExitCode}else{$null}
        retainedHandle=$true;forcedTermination=$false
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'state-floor-exit.json') -Encoding utf8
}finally{$taskProcess.Dispose()}

$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskDescriptor=Get-Content -LiteralPath (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-8BE-CANDIDATE/candidate-descriptor.json') -Raw | ConvertFrom-Json
$taskRuntime=Join-Path $taskCode 'test/runtime/native-depth-8be-20261016'
if($taskDescriptor.source -cne '8be87e2d54a895d4fe02e31300100a1ffed90121' -or !$taskDescriptor.sameShaCiAndArtifactVerified){throw 'New source candidate not admitted'}
if((Get-FileHash -LiteralPath $taskDescriptor.executable).Hash.ToLowerInvariant() -cne $taskDescriptor.sha256){throw 'New source executable changed'}
if((Test-Path -LiteralPath $taskRuntime) -or (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'native-depth-launch.json'))){throw 'New source native depth run already exists'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native instance exists'}
$taskRoot=Join-Path $taskRuntime 'data'
New-Item -ItemType Directory -Path (Join-Path $taskRoot 'manager-state') -Force | Out-Null
$taskPrefs=Get-Content -LiteralPath (Join-Path $taskCode 'apps/desktop/tauri/config/preferences.defaults.json') -Raw | ConvertFrom-Json
$taskPrefs.auto_panel=$false;$taskPrefs.launch_with_codex=$false
$taskPrefs.keep_proxy_on_close=$false;$taskPrefs.app_update_auto_check=$false
$taskPrefs.startup_cleanup=$false;$taskPrefs.theme='light';$taskPrefs.visual_effects='mid'
$taskPrefs | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskRoot 'manager-state/preferences.json') -Encoding utf8
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
. (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/methods/process-user-sid.ps1')
$taskSocket=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0)
$taskSocket.Start();$taskPort=$taskSocket.LocalEndpoint.Port;$taskSocket.Stop()
$taskInfo=[Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName=$taskDescriptor.executable;$taskInfo.WorkingDirectory=Split-Path -Parent $taskDescriptor.executable;$taskInfo.UseShellExecute=$false
$taskInfo.Environment['OPENCODEX_SANDBOX']='1'
$taskInfo.Environment['OPENCODEX_SANDBOX_ROOT']=$taskRoot
$taskInfo.Environment['OPENCODEX_SANDBOX_BOUNDARY']=$taskRuntime
$taskInfo.Environment['OPENCODEX_WINDOWS_SMOKE_CDP_PORT']=[string]$taskPort
foreach($taskName in @('OPENCODEX_WINDOWS_SMOKE_SCALE','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','WEBVIEW2_USER_DATA_FOLDER')){[void]$taskInfo.Environment.Remove($taskName)}
$taskProcess=[Diagnostics.Process]::Start($taskInfo)
try{
    $taskUser=Get-AcceptanceProcessSidEvidence -Process $taskProcess -ExpectedExecutable $taskDescriptor.executable
    Assert-WindowsProcessMode -Observation $taskUser -Mode ordinary
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');phase='native-depth';mode='ordinary'
        source=$taskDescriptor.source;executable=$taskDescriptor.executable;exeSha256=$taskDescriptor.sha256
        pid=$taskProcess.Id;creationTimeUtc=$taskUser.creationTimeUtc
        applicationToken=$taskUser;applicationUser=$taskUser;dataRoot=$taskRoot;port=$taskPort
        sandboxUsed=$true;cdpUsed=$true;forcedDpiUsed=$false;productionCodePatchedAtRuntime=$false
    } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'native-depth-launch.json') -Encoding utf8
    $taskTimer=[Diagnostics.Stopwatch]::StartNew()
    while(!$taskProcess.WaitForExit(1000) -and $taskTimer.Elapsed.TotalSeconds -lt 600){}
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskProcess.Id;creationTimeUtc=$taskUser.creationTimeUtc
        exitObserved=$taskProcess.HasExited;exitCode=if($taskProcess.HasExited){$taskProcess.ExitCode}else{$null}
        retainedHandle=$true;forcedTermination=$false
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'native-depth-exit.json') -Encoding utf8
}finally{$taskProcess.Dispose()}

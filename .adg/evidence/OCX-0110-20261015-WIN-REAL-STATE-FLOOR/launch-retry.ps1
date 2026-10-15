$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskOld=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'state-floor-launch.json') -Raw | ConvertFrom-Json
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Existing native product'}
if(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'state-floor-retry-launch.json')){throw 'Retry already launched'}
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
. (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/methods/process-user-sid.ps1')
$taskInfo=[Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName=$taskOld.executable;$taskInfo.WorkingDirectory=Split-Path -Parent $taskOld.executable;$taskInfo.UseShellExecute=$false
$taskInfo.Environment['OPENCODEX_SANDBOX']='1'
$taskInfo.Environment['OPENCODEX_SANDBOX_ROOT']=$taskOld.dataRoot
$taskInfo.Environment['OPENCODEX_SANDBOX_BOUNDARY']=$taskOld.runtime
$taskInfo.Environment['OPENCODEX_WINDOWS_SMOKE_CDP_PORT']=[string]$taskOld.port
foreach($taskName in @('OPENCODEX_WINDOWS_SMOKE_SCALE','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','WEBVIEW2_USER_DATA_FOLDER')){[void]$taskInfo.Environment.Remove($taskName)}
$taskProcess=[Diagnostics.Process]::Start($taskInfo)
try{
    $taskUser=Get-AcceptanceProcessSidEvidence -Process $taskProcess -ExpectedExecutable $taskOld.executable
    Assert-WindowsProcessMode -Observation $taskUser -Mode ordinary
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');phase='state-floor-retry';mode='ordinary'
        source=$taskOld.source;executable=$taskOld.executable;exeSha256=$taskOld.exeSha256
        pid=$taskProcess.Id;creationTimeUtc=$taskUser.creationTimeUtc
        applicationToken=$taskUser;applicationUser=$taskUser;runtime=$taskOld.runtime;dataRoot=$taskOld.dataRoot;port=$taskOld.port
        sandboxUsed=$true;cdpUsed=$true;realMalformedState=$true;systemClockChanged=$false
        reason='Initial attempt could not instrument immutable native bridge; retry uses passive IPC network events'
    } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'state-floor-retry-launch.json') -Encoding utf8
    $taskTimer=[Diagnostics.Stopwatch]::StartNew()
    while(!$taskProcess.WaitForExit(1000) -and $taskTimer.Elapsed.TotalSeconds -lt 2400){}
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskProcess.Id;creationTimeUtc=$taskUser.creationTimeUtc
        exitObserved=$taskProcess.HasExited;exitCode=if($taskProcess.HasExited){$taskProcess.ExitCode}else{$null}
        retainedHandle=$true;forcedTermination=$false
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'state-floor-retry-exit.json') -Encoding utf8
}finally{$taskProcess.Dispose()}

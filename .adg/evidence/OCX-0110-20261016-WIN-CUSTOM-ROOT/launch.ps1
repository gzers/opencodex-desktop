param(
    [ValidateSet('ordinary','administrator')][string]$Mode='ordinary',
    [Parameter(Mandatory=$true)][ValidatePattern('^[a-z][a-z0-9-]+$')][string]$Phase,
    [ValidateSet('install-a','install-b','install-c')][string]$Location='install-a',
    [switch]$ScriptedRootChecks
)
$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskRuntime=Join-Path $taskCode 'test/runtime/foreground-02-current-20261015'
$taskExe=Join-Path $taskRuntime ($Location+'/opencodex-desktop.exe')
$taskRecordPath=Join-Path $PSScriptRoot ($Phase+'-launch.json')
if(Test-Path -LiteralPath $taskRecordPath){throw 'Launch already recorded'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native instance exists'}
if((Get-FileHash -LiteralPath $taskExe).Hash.ToLowerInvariant() -cne '7383c275f58fed7e25678420a6d3f5c739f9b23d7373acc40fab1e7cd98db6d9'){throw 'Candidate image changed'}
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
. (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/methods/process-user-sid.ps1')
$taskHarness=Get-WindowsHarnessToken
Assert-WindowsProcessMode -Observation $taskHarness -Mode $Mode
$taskInfo=[Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName=$taskExe
$taskInfo.WorkingDirectory=Split-Path -Parent $taskExe
$taskInfo.UseShellExecute=$false
foreach($taskVariable in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY','OPENCODEX_WINDOWS_SMOKE_CDP_PORT','OPENCODEX_WINDOWS_SMOKE_SCALE','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','WEBVIEW2_USER_DATA_FOLDER')){
    [void]$taskInfo.Environment.Remove($taskVariable)
}
$taskProcess=[Diagnostics.Process]::Start($taskInfo)
try{
    $taskToken=Get-WindowsProcessToken -Process $taskProcess -ExpectedExecutable $taskExe
    Assert-WindowsProcessMode -Observation $taskToken -Mode $Mode
    $taskUser=Get-AcceptanceProcessSidEvidence -Process $taskProcess -ExpectedExecutable $taskExe
    [ordered]@{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');phase=$Phase;mode=$Mode
        source='02a0d4a8cbae8ef1450908d86b2c75fcad65f04f'
        executable=$taskExe;exeSha256='7383c275f58fed7e25678420a6d3f5c739f9b23d7373acc40fab1e7cd98db6d9'
        pid=$taskProcess.Id;creationTimeUtc=$taskToken.creationTimeUtc
        applicationToken=$taskToken;applicationUser=$taskUser;harnessToken=$taskHarness
        sandboxUsed=$false;cdpUsed=$false;environmentRootRedirected=$false
    } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskRecordPath -Encoding utf8
    if($ScriptedRootChecks){
        $taskShell=[Diagnostics.Process]::GetCurrentProcess().MainModule.FileName
        Start-Sleep -Seconds 6
        foreach($taskStep in @(
            @('-Phase',($Phase+'-first-screen')),
            @('-Phase',($Phase+'-settings'),'-Action','invoke','-Name','设置'),
            @('-Phase',($Phase+'-install-settings'),'-Action','invoke','-Name','安装配置'),
            @('-Phase',($Phase+'-close'),'-Action','close')
        )){
            & $taskShell -NoLogo -NoProfile -NonInteractive -ExecutionPolicy RemoteSigned -File (Join-Path $PSScriptRoot 'native.ps1') -Launch $Phase @taskStep
            if($LASTEXITCODE -ne 0){throw 'Scripted native step failed; retain product for inspection'}
        }
    }
    $taskTimer=[Diagnostics.Stopwatch]::StartNew()
    while(!$taskProcess.WaitForExit(1000) -and $taskTimer.Elapsed.TotalSeconds -lt 1800){}
    $taskExitObserved=$taskProcess.HasExited
    [ordered]@{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');phase=$Phase
        pid=$taskProcess.Id;creationTimeUtc=$taskToken.creationTimeUtc
        exitObserved=$taskExitObserved;exitCode=if($taskExitObserved){$taskProcess.ExitCode}else{$null}
        waitedSeconds=$taskTimer.Elapsed.TotalSeconds;forcedTermination=$false
        retainedHandle=$true
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot ($Phase+'-exit.json')) -Encoding utf8
}finally{$taskProcess.Dispose()}

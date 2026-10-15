param([Parameter(Mandatory=$true)][string]$CandidateDescriptor)
$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskRuntime=Join-Path $taskCode 'test/runtime/startup-pair-8be-20261015'
$taskCandidate=Get-Content -LiteralPath $CandidateDescriptor -Raw | ConvertFrom-Json
if($taskCandidate.source -cne '8be87e2d54a895d4fe02e31300100a1ffed90121' -or
    !$taskCandidate.sameShaCiAndArtifactVerified -or $taskCandidate.installerExecuted){throw 'Exact new candidate must be admitted independently'}
$taskMethod=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'method.json') -Raw | ConvertFrom-Json
if(Test-Path -LiteralPath $taskRuntime){throw 'Startup pair already exists; do not silently rerun'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native instance exists before startup trial'}
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
. (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/methods/process-user-sid.ps1')
Add-Type -AssemblyName UIAutomationClient,UIAutomationTypes
Add-Type -TypeDefinition @'
using System; using System.Runtime.InteropServices;
public static class StartupPairNative {
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr w,out uint p);
 [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
 [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint first,uint second,bool attach);
 [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr w);
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr w);
 [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr w,uint m,IntPtr a,IntPtr b);
 public static bool Activate(IntPtr w){
  uint ignored,current=GetCurrentThreadId(),foreground=GetWindowThreadProcessId(GetForegroundWindow(),out ignored),target=GetWindowThreadProcessId(w,out ignored);
  bool a=foreground!=0&&foreground!=current&&AttachThreadInput(current,foreground,true);
  bool b=target!=0&&target!=current&&target!=foreground&&AttachThreadInput(current,target,true);
  try{BringWindowToTop(w);return SetForegroundWindow(w);}
  finally{if(b)AttachThreadInput(current,target,false);if(a)AttachThreadInput(current,foreground,false);}
 }
}
'@
$taskLocalRoot=[IO.Path]::GetFullPath((Join-Path $env:LOCALAPPDATA 'com.gzers.opencodex.desktop'))
$taskProfile=[IO.Path]::GetFullPath((Join-Path $taskLocalRoot 'EBWebView'))
if($taskProfile -cne ($taskLocalRoot+'\EBWebView')){throw 'Unexpected application WebView endpoint'}
New-Item -ItemType Directory -Path (Join-Path $taskRuntime 'cache-history') -Force | Out-Null
$taskRoots=@{
    baseline019=(Join-Path $env:APPDATA 'com.gzers.opencodex.desktop')
    candidate0110=(Join-Path $taskCode 'test/runtime/foreground-02-current-20261015/install-a/data')
}
$taskVersions=@{
    baseline019=@{executable=(Join-Path $taskCode 'test/software/acceptance-kit-02a0d4-20261015/bin/baseline019/opencodex-desktop.exe');sha256='626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608';source='de6862d40389d527527b9db8ef52e186a58b625f'}
    candidate0110=@{executable=$taskCandidate.executable;sha256=$taskCandidate.sha256;source=$taskCandidate.source}
}
foreach($taskVersion in $taskVersions.Keys){
    $taskDescriptor=$taskVersions[$taskVersion]
    if((Get-FileHash -LiteralPath $taskDescriptor.executable).Hash.ToLowerInvariant() -cne $taskDescriptor.sha256){throw 'Startup image changed'}
    $taskPreferences=Join-Path $taskRoots[$taskVersion] 'manager-state/preferences.json'
    $taskOriginal=Get-Content -LiteralPath $taskPreferences -Raw | ConvertFrom-Json
    if($taskOriginal.schema_version -ne 2){throw 'Expected known migrated manager preference fixture'}
    Copy-Item -LiteralPath $taskPreferences -Destination (Join-Path $PSScriptRoot ($taskVersion+'-before-preferences.original.json'))
    $taskOriginal.appearance.theme='light';$taskOriginal.appearance.visual_effects='high'
    $taskOriginal.appearance.glow_render='mesh';$taskOriginal.appearance.interface_scale=100
    $taskOriginal.shell.auto_panel=$false;$taskOriginal.shell.launch_with_codex=$false
    $taskOriginal.shell.keep_proxy_on_close=$false;$taskOriginal.shell.launch_main=$true
    $taskOriginal.updates.app_update_auto_check=$false;$taskOriginal.maintenance.startup_cleanup=$false
    $taskOriginal | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $taskPreferences -Encoding utf8
    Copy-Item -LiteralPath $taskPreferences -Destination (Join-Path $PSScriptRoot ($taskVersion+'-seeded-preferences.json'))
}
$taskSettingsCondition=[Windows.Automation.AndCondition]::new(
    [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::NameProperty,'设置'),
    [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ControlTypeProperty,[Windows.Automation.ControlType]::Button))
$taskRouteCondition=[Windows.Automation.AndCondition]::new(
    [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::NameProperty,'桌面壳偏好'),
    [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ControlTypeProperty,[Windows.Automation.ControlType]::Text))
function Wait-StartupWebViewQuiescence {
    $taskWait=[Diagnostics.Stopwatch]::StartNew()
    do {
        if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native instance remains between startup samples'}
        $taskMatching=@(Get-CimInstance Win32_Process -Filter "Name='msedgewebview2.exe'" |
            Where-Object {$_.CommandLine -and $_.CommandLine.IndexOf($taskProfile,[StringComparison]::OrdinalIgnoreCase) -ge 0})
        if(!$taskMatching.Count){return}
        Start-Sleep -Milliseconds 200
    }while($taskWait.Elapsed.TotalSeconds -lt 15)
    throw 'Owned application WebView processes still alive; do not reset a live profile'
}
$taskRecords=[Collections.Generic.List[object]]::new()
try{
    for($taskRound=1;$taskRound -le 20;$taskRound++){
        # Alternate version order per round instead of measuring all of one
        # version first; each cold launch is followed by its own warm launch.
        $taskOrder=if($taskRound % 2){@('baseline019','candidate0110')}else{@('candidate0110','baseline019')}
        foreach($taskVersion in $taskOrder){
            $taskDescriptor=$taskVersions[$taskVersion]
            foreach($taskKind in @('cache-cold','process-warm')){
                Wait-StartupWebViewQuiescence
                if($taskKind -eq 'cache-cold' -and (Test-Path -LiteralPath $taskProfile)){
                    $taskItem=Get-Item -LiteralPath $taskProfile -Force
                    if(!$taskItem.PSIsContainer -or $taskItem.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'WebView root alias refused'}
                    $taskSaved=[IO.Path]::GetFullPath((Join-Path $taskRuntime ('cache-history/'+$taskRound+'-'+$taskVersion)))
                    if(!$taskSaved.StartsWith($taskRuntime+'\cache-history\',[StringComparison]::OrdinalIgnoreCase) -or
                        (Test-Path -LiteralPath $taskSaved)){throw 'Cache sidecar endpoint invalid'}
                    Move-Item -LiteralPath $taskProfile -Destination $taskSaved
                }
                if($taskKind -eq 'cache-cold' -and (Test-Path -LiteralPath $taskProfile)){throw 'Cold WebView profile still exists'}
                if($taskKind -eq 'process-warm' -and !(Test-Path -LiteralPath $taskProfile)){throw 'Warm WebView profile missing'}
                $taskInfo=[Diagnostics.ProcessStartInfo]::new()
                $taskInfo.FileName=$taskDescriptor.executable
                $taskInfo.WorkingDirectory=Split-Path -Parent $taskDescriptor.executable
                $taskInfo.UseShellExecute=$false
                foreach($taskName in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY','OPENCODEX_WINDOWS_SMOKE_CDP_PORT','OPENCODEX_WINDOWS_SMOKE_SCALE','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','WEBVIEW2_USER_DATA_FOLDER')){
                    [void]$taskInfo.Environment.Remove($taskName)
                }
                $taskStartedUtc=[datetimeoffset]::UtcNow.ToString('o')
                $taskTimer=[Diagnostics.Stopwatch]::StartNew()
                $taskProduct=[Diagnostics.Process]::Start($taskInfo)
                [void]$taskProduct.SafeHandle
                $taskWindow=[IntPtr]::Zero
                $taskButton=$null;$taskPolls=0
                while($taskTimer.Elapsed.TotalSeconds -lt 30){
                    if($taskProduct.HasExited){throw 'Product exited before startup interaction'}
                    $taskProduct.Refresh();$taskWindow=$taskProduct.MainWindowHandle
                    if($taskWindow -ne [IntPtr]::Zero){
                        try{
                            $taskElement=[Windows.Automation.AutomationElement]::FromHandle($taskWindow)
                            $taskButton=$taskElement.FindFirst([Windows.Automation.TreeScope]::Descendants,$taskSettingsCondition)
                            if($taskButton -and $taskButton.Current.IsEnabled){break}
                        }catch{$taskButton=$null}
                    }
                    $taskPolls++;Start-Sleep -Milliseconds 10
                }
                if(!$taskButton){throw 'No enabled Settings button before startup bound'}
                $taskButtonObservedMs=$taskTimer.Elapsed.TotalMilliseconds
                [void][StartupPairNative]::Activate($taskWindow)
                if([StartupPairNative]::GetForegroundWindow() -ne $taskWindow){throw 'Startup interaction could not obtain foreground'}
                $taskInvoke=$taskButton.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern)
                $taskInvokedAtMs=$taskTimer.Elapsed.TotalMilliseconds
                $taskInvoke.Invoke()
                $taskRoute=$null
                while($taskTimer.Elapsed.TotalSeconds -lt 30){
                    try{$taskRoute=$taskElement.FindFirst([Windows.Automation.TreeScope]::Descendants,$taskRouteCondition)}catch{$taskRoute=$null}
                    if($taskRoute){break}
                    $taskPolls++;Start-Sleep -Milliseconds 10
                }
                if(!$taskRoute){throw 'Actual Settings interaction did not render its route before startup bound'}
                $taskInteractiveMs=$taskTimer.Elapsed.TotalMilliseconds
                $taskUser=Get-AcceptanceProcessSidEvidence -Process $taskProduct -ExpectedExecutable $taskDescriptor.executable
                Assert-WindowsProcessMode -Observation $taskUser -Mode ordinary
                if($taskUser.sidSha256 -cne '373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310'){throw 'Startup user changed'}
                $taskBrowsers=@(Get-CimInstance Win32_Process -Filter ("Name='msedgewebview2.exe' AND ParentProcessId="+$taskProduct.Id) |
                    Where-Object {$_.CommandLine -and $_.CommandLine.IndexOf($taskProfile,[StringComparison]::OrdinalIgnoreCase) -ge 0})
                if(!(Test-Path -LiteralPath $taskProfile) -or !$taskBrowsers.Count){throw 'Actual product WebView did not use the inspected cold/warm profile'}
                $taskRecord=[ordered]@{
                    round=$taskRound;version=$taskVersion;kind=$taskKind;source=$taskDescriptor.source;exeSha256=$taskDescriptor.sha256
                    startedAtUtc=$taskStartedUtc;pid=$taskProduct.Id;creationTimeUtc=$taskUser.creationTimeUtc
                    buttonObservedMs=$taskButtonObservedMs;invokedAtMs=$taskInvokedAtMs;interactiveMs=$taskInteractiveMs
                    pollingIterations=$taskPolls;nominalPollDelayMs=10;actualSettingsRouteObserved=$true
                    foregroundAtInteraction=([StartupPairNative]::GetForegroundWindow() -eq $taskWindow)
                    applicationUser=$taskUser;webviewProfileExistsAfterStart=(Test-Path -LiteralPath $taskProfile)
                    actualDirectChildWebViewProfileMatched=$true;browserProcessIds=@($taskBrowsers.ProcessId);browserCommandLinesPublished=$false
                    osCachesPurged=$false;sandboxUsed=$false;cdpUsed=$false
                }
                [void][StartupPairNative]::PostMessageW($taskWindow,0x10,[IntPtr]::Zero,[IntPtr]::Zero)
                if(!$taskProduct.WaitForExit(10000)){throw 'Startup sample failed to exit normally; do not start another'}
                $taskRecord.exitObserved=$true;$taskRecord.exitCode=$taskProduct.ExitCode;$taskRecord.forcedTermination=$false
                if($taskProduct.ExitCode -ne 0){throw 'Startup sample exited nonzero'}
                $taskRecords.Add($taskRecord)
                $taskRecord | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $PSScriptRoot ($taskRound.ToString('00')+'-'+$taskVersion+'-'+$taskKind+'.json')) -Encoding utf8
                [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');completed=$taskRecords.Count;totalPlanned=80;lastRound=$taskRound;lastVersion=$taskVersion;lastKind=$taskKind;lastInteractiveMs=$taskInteractiveMs} |
                    ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'progress.json') -Encoding utf8
                $taskProduct.Dispose()
            }
        }
    }
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');samples=$taskRecords.Count;completed=$true;formalPerformanceGatePassed=$false;osColdStartupClaimed=$false} |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'terminal.json') -Encoding utf8
}catch{
    $_ | Out-String | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'failure.original.txt') -Encoding utf8
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');samples=$taskRecords.Count;completed=$false;formalPerformanceGatePassed=$false;forcedTermination=$false} |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'terminal.json') -Encoding utf8
    throw
}

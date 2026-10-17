param([string]$Plan,[switch]$Execute)
$ErrorActionPreference='Stop'
if(!$Execute){
    [ordered]@{status='prepared-only';productStarted=$false;foregroundUsed=$false
        profilesMoved=$false;etwSessionStarted=$false}|ConvertTo-Json
    return
}
if(!$Plan){throw 'Explicit prepared native cache plan required'}
. (Join-Path $PSScriptRoot 'windows-startup-pair-state.ps1')
. (Join-Path $PSScriptRoot 'windows-cache-query-state.ps1')
. (Join-Path $PSScriptRoot 'windows-process-token.ps1')
. (Join-Path $PSScriptRoot 'windows-observation-binding.ps1')
$taskPlan=Get-Content -LiteralPath (ConvertTo-StartupPairPath $Plan) -Raw -Encoding utf8|ConvertFrom-Json
$taskRun=ConvertTo-StartupPairPath $taskPlan.runDirectory
$taskRepo=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
if(!$taskRun.StartsWith((Join-Path $taskRepo 'test/runtime')+'\',[StringComparison]::OrdinalIgnoreCase)){
    throw 'Owned private run required'
}
if(!(Test-Path -LiteralPath $taskRun -PathType Container)){throw 'Prepared private run directory required'}
if((Test-Path -LiteralPath (Join-Path $taskRun 'samples')) -or
    (Test-Path -LiteralPath (Join-Path $taskRun 'native-run-terminal.json')) -or
    (Test-Path -LiteralPath (Join-Path $taskRun 'failure.original.json'))){
    throw 'Do not replay or overwrite an existing native attempt; recover it through its preparation owner'
}
$taskHuman=Get-Content -LiteralPath $taskPlan.humanWindow -Raw -Encoding utf8|ConvertFrom-Json
function Assert-CacheForegroundWindow([int]$ReserveSeconds=0){
    $taskNow=[datetimeoffset]::UtcNow
    $taskStart=[datetimeoffset]::Parse($taskHuman.authorizedAtUtc)
    $taskEnd=[datetimeoffset]::Parse($taskHuman.expiresAtUtc)
    $taskClose=$taskStart.AddMinutes(25)
    if($taskEnd -lt $taskClose){$taskClose=$taskEnd}
    if($taskHuman.origin -cne 'human' -or $taskHuman.explicitHumanStart -isnot [bool] -or
        !$taskHuman.explicitHumanStart -or $taskHuman.automaticContinuation -isnot [bool] -or
        $taskHuman.automaticContinuation -or $taskEnd -le $taskStart -or
        ($taskEnd-$taskStart).TotalMinutes -gt 30 -or $taskNow -lt $taskStart -or
        $taskNow.AddSeconds($ReserveSeconds) -ge $taskClose){
        throw 'Fresh human window or remaining normal-cleanup reservation missing'
    }
    $taskClose
}
function Write-CacheJson([string]$Path,$Value){
    $temporary=$Path+'.'+[guid]::NewGuid().ToString('N')+'.tmp'
    [IO.File]::WriteAllText($temporary,($Value|ConvertTo-Json -Depth 12),[Text.UTF8Encoding]::new($false))
    try{[IO.File]::Move($temporary,$Path,$true)}
    finally{if(Test-Path -LiteralPath $temporary){Remove-Item -LiteralPath $temporary}}
}
$taskProduct=$null
$taskObserver=$null
$taskAdministrator=$null
$taskObserverOut=$null
$taskObserverError=$null
$taskRows=[Collections.Generic.List[object]]::new()
$taskFailure=$null
$taskRestored=$false
$taskEtwReady=$false
$taskObserverForced=$false
$taskRestorationAdmitted=$false
$taskCounterStop=Join-Path $taskRun 'stop-native-counters.flag'
try{
    $taskClose=Assert-CacheForegroundWindow -ReserveSeconds 120
    if($taskPlan.nativeUiaDriverExplicitlyApproved -isnot [bool] -or !$taskPlan.nativeUiaDriverExplicitlyApproved){
        throw 'The native UIA sampler must be explicitly approved'
    }
    $taskFirst=if($null -ne $taskPlan.generationStart){[int]$taskPlan.generationStart}else{1}
    $taskCount=if($null -ne $taskPlan.generationCount){[int]$taskPlan.generationCount}else{10}
    if($taskFirst -lt 1 -or $taskCount -lt 1 -or $taskFirst+$taskCount-1 -gt 10){
        throw 'Only previously missing generations within the original ten may be requested'
    }
    Assert-WindowsProcessMode -Observation (Get-WindowsHarnessToken) -Mode ordinary
    Assert-StartupPairNoProduct
    if($taskPlan.source -cne '8e69ff6db69ea13b8cb29feb9f7a1abacd202a7b' -or
        (Get-FileHash -LiteralPath $taskPlan.executable).Hash.ToLowerInvariant() -cne
        '870cb30c62c0cf563edacf7fe30a35d3b66cbffbb574beeb6b354ed2c7fab15a'){
        throw 'Exact admitted8e MSI image required'
    }
    foreach($taskMethod in @($taskPlan.directoryIdentityMethod,$taskPlan.userIdentityMethod,$taskPlan.restoreMethod)){
        if((Get-FileHash -LiteralPath $taskMethod.path).Hash.ToLowerInvariant() -cne $taskMethod.sha256){
            throw 'Pinned native identity/restoration method differs'
        }
    }
    $taskRestorePath=ConvertTo-StartupPairPath $taskPlan.restoreMethod.path
    if(!$taskRestorePath.StartsWith((Join-Path $taskRepo 'test/runtime')+'\',[StringComparison]::OrdinalIgnoreCase) -or
        [IO.Path]::GetFileName($taskRestorePath) -cne 'restore-cache-query-fixtures.ps1'){
        throw 'Exact owned cache-fixture restoration adapter required'
    }
    . $taskPlan.directoryIdentityMethod.path
    . $taskPlan.userIdentityMethod.path
    if($taskPlan.savedOriginalDirectories.Count -ne 2){throw 'Both originals must be preserved'}
    foreach($taskSaved in $taskPlan.savedOriginalDirectories){
        $taskSavedPath=ConvertTo-StartupPairPath $taskSaved.path
        if(!$taskSavedPath.StartsWith($taskRun+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned saved originals required'}
        $taskIdentity=Get-RecoveryDirectoryIdentity -Path $taskSavedPath
        if($taskIdentity.fileId -cne $taskSaved.fileId -or $taskIdentity.creationFileTime -cne $taskSaved.creationFileTime){
            throw 'Preserved original directory identity differs'
        }
    }
    $taskCurrentIdentity=Get-RecoveryDirectoryIdentity -Path (Join-Path $env:APPDATA 'com.gzers.opencodex.desktop')
    if($taskCurrentIdentity.fileId -cin $taskPlan.savedOriginalDirectories.fileId){
        throw 'Original roaming directory is active; never use it as the fixture'
    }
    $taskRestorationAdmitted=$true
    $taskRoot=ConvertTo-StartupPairPath $taskPlan.candidateRoot
    if(!$taskRoot.StartsWith($taskRun+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned cache fixture required'}
    $taskReadiness=Get-CacheQueryFixtureReadiness -Root $taskRoot -Now ([datetimeoffset]::UtcNow) -Through $taskClose -RequirePreparedPreferences
    $taskCalibration=Get-Content -LiteralPath $taskPlan.calibrationResult -Raw -Encoding utf8|ConvertFrom-Json
    if(!$taskCalibration.rawKernelPositiveControlPassed -or !$taskCalibration.nativePositiveControl.passed -or
        !$taskCalibration.normalExitObserved -or $taskCalibration.exitCode -ne 0){
        throw 'Completed ordinary-target/kernel positive control required'
    }
    if($taskCalibration.observerMethods.Count -ne 4){throw 'Positive control must bind the current observer methods'}
    foreach($taskMethod in $taskCalibration.observerMethods){
        if($taskMethod.name -cnotin @('windows-owned-etw.cs','windows-query-counters.cs',
            'windows-query-counters.ps1','windows-query-calibration-admin.ps1') -or
            (Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $taskMethod.name)).Hash.ToLowerInvariant() -cne $taskMethod.sha256){
            throw 'Positive control observer source differs'
        }
    }
    $taskPointer=(Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop' -Name StartupDataRootV1).StartupDataRootV1|ConvertFrom-Json
    if($taskPointer.schema -ne 1 -or $taskPointer.anchor -cnotin @($taskRoot,('\\?\'+$taskRoot))){
        throw 'Actual startup pointer does not bind the private fixture'
    }
    if((Test-Path -LiteralPath (Join-Path $taskRun 'samples')) -or (Test-Path -LiteralPath $taskCounterStop)){
        throw 'Never overwrite or repeat a native sample attempt'
    }
    New-Item -ItemType Directory -Path (Join-Path $taskRun 'samples')|Out-Null
    Add-Type -AssemblyName UIAutomationClient,UIAutomationTypes
    Add-Type -TypeDefinition @'
using System; using System.Text; using System.Diagnostics; using System.Runtime.InteropServices;
public static class CacheQueryNative {
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr w,out uint p);
 [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
 [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a,uint b,bool attach);
 [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr w);
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr w);
 [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr w,uint m,IntPtr a,IntPtr b);
 public static bool OwnsWindow(IntPtr w,int expected){uint owner;GetWindowThreadProcessId(w,out owner);return owner==(uint)expected;}
 [DllImport("user32.dll")] private static extern IntPtr OpenInputDesktop(uint flags,bool inherit,uint access);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] private static extern bool GetUserObjectInformationW(IntPtr h,int n,StringBuilder s,uint size,out uint needed);
 [DllImport("user32.dll")] private static extern bool CloseDesktop(IntPtr h);
 [DllImport("wtsapi32.dll")] private static extern bool WTSQuerySessionInformationW(IntPtr server,int session,int kind,out IntPtr data,out int size);
 [DllImport("wtsapi32.dll")] private static extern void WTSFreeMemory(IntPtr data);
 public static bool Interactive(){
  IntPtr d=OpenInputDesktop(0,false,1);
  if(d==IntPtr.Zero)return false;
  try{var name=new StringBuilder(256);uint needed;
   if(!GetUserObjectInformationW(d,2,name,512,out needed)||name.ToString()!="Default")return false;
  }finally{CloseDesktop(d);}
  IntPtr value;int bytes;
  const int WTSConnectState=8, WTSActive=0;
  if(!WTSQuerySessionInformationW(IntPtr.Zero,Process.GetCurrentProcess().SessionId,WTSConnectState,out value,out bytes))return false;
  try{return bytes==sizeof(int)&&Marshal.ReadInt32(value)==WTSActive;}finally{WTSFreeMemory(value);}
 }
 public static bool Activate(IntPtr w){
  if(!Interactive())return false;
  uint ignored,current=GetCurrentThreadId(),front=GetWindowThreadProcessId(GetForegroundWindow(),out ignored),target=GetWindowThreadProcessId(w,out ignored);
  bool a=front!=0&&front!=current&&AttachThreadInput(current,front,true);
  bool b=target!=0&&target!=current&&target!=front&&AttachThreadInput(current,target,true);
  try{BringWindowToTop(w);return SetForegroundWindow(w);}
  finally{if(b)AttachThreadInput(current,target,false);if(a)AttachThreadInput(current,front,false);}
 }
}
'@
    $taskProfile=Join-Path $env:LOCALAPPDATA 'com.gzers.opencodex.desktop\EBWebView'
    function Wait-CacheWebViewExit{
        $watch=[Diagnostics.Stopwatch]::StartNew()
        do{
            Assert-StartupPairNoProduct
            $children=@(Get-CimInstance Win32_Process -Filter "Name='msedgewebview2.exe'"|
                Where-Object {$_.CommandLine -and $_.CommandLine.IndexOf($taskProfile,[StringComparison]::OrdinalIgnoreCase) -ge 0})
            if(!$children.Count){return}
            Start-Sleep -Milliseconds 100
        }while($watch.Elapsed.TotalSeconds -lt 15)
        throw 'Owned WebView remains alive between generations'
    }
    $taskCounterPlan=Join-Path $taskRun 'native-counter-plan.json'
    Write-CacheJson $taskCounterPlan ([ordered]@{authorization=$taskHuman;runDirectory=$taskRun
        executable=$taskPlan.executable;source=$taskPlan.source
        exeSha256='870cb30c62c0cf563edacf7fe30a35d3b66cbffbb574beeb6b354ed2c7fab15a'
        debugPort=[int]$taskPlan.debugPort;stopFile=$taskCounterStop})
    $taskAdminScript=Join-Path $PSScriptRoot 'windows-query-counters.ps1'
    $taskAdministrator=Start-Process -FilePath (Join-Path $PSHOME 'pwsh.exe') -Verb RunAs -WindowStyle Hidden -PassThru -ArgumentList @(
        '-NoProfile','-NonInteractive','-File',('"'+$taskAdminScript+'"'),
        '-Plan',('"'+$taskCounterPlan+'"'),'-DecoderDirectory',('"'+$taskPlan.decoderDirectory+'"'),'-RecordApproved')
    $taskWait=[Diagnostics.Stopwatch]::StartNew()
    while(!(Test-Path -LiteralPath (Join-Path $taskRun 'native-counters\ready.json'))){
        if($taskAdministrator.HasExited -or $taskWait.Elapsed.TotalSeconds -ge 25){throw 'Owned ETW observer not ready'}
        Start-Sleep -Milliseconds 100
    }
    $taskEtwReady=$true
    for($taskGeneration=$taskFirst;$taskGeneration -lt $taskFirst+$taskCount;$taskGeneration++){
        $null=Assert-CacheForegroundWindow -ReserveSeconds 115
        Wait-CacheWebViewExit
        $null=Get-CacheQueryFixtureReadiness -Root $taskRoot -Now ([datetimeoffset]::UtcNow) -Through $taskClose -RequirePreparedPreferences
        $taskGenerationRoot=Join-Path $taskRun ('samples\'+$taskGeneration.ToString('00'))
        New-Item -ItemType Directory -Path $taskGenerationRoot|Out-Null
        $taskInfo=[Diagnostics.ProcessStartInfo]::new()
        $taskInfo.FileName=$taskPlan.executable
        $taskInfo.WorkingDirectory=Split-Path -Parent $taskPlan.executable
        $taskInfo.UseShellExecute=$false
        foreach($taskName in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY',
            'OPENCODEX_WINDOWS_SMOKE_CDP_PORT','OPENCODEX_WINDOWS_SMOKE_SCALE','WEBVIEW2_USER_DATA_FOLDER')){
            [void]$taskInfo.Environment.Remove($taskName)
        }
        # The application smoke variable requires a sandbox. Use the SDK's
        # ordinary-process diagnostic argument without changing its data root.
        $taskInfo.Environment['WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS']=
            '--remote-debugging-address=127.0.0.1 --remote-debugging-port='+$taskPlan.debugPort
        $taskStarted=[datetimeoffset]::UtcNow.ToString('o')
        $taskTimer=[Diagnostics.Stopwatch]::StartNew()
        $taskProduct=[Diagnostics.Process]::Start($taskInfo)
        $null=$taskProduct.SafeHandle
        $taskUser=Get-AcceptanceProcessSidEvidence -Process $taskProduct -ExpectedExecutable $taskPlan.executable
        Assert-WindowsProcessMode -Observation $taskUser -Mode ordinary
        if($taskUser.sidSha256 -cne '373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310'){
            throw 'Actual ordinary user differs'
        }
        do{
            if($taskProduct.HasExited){throw 'Product exited before native window'}
            $taskProduct.Refresh()
            $taskWindow=$taskProduct.MainWindowHandle
            if($taskWindow -ne [IntPtr]::Zero){break}
            if($taskTimer.Elapsed.TotalSeconds -ge 30){throw 'No native product window'}
            Start-Sleep -Milliseconds 50
        }while($true)
        if(![CacheQueryNative]::Activate($taskWindow) -or [CacheQueryNative]::GetForegroundWindow() -ne $taskWindow){
            throw 'Interactive foreground unavailable; no UI input sent'
        }
        $taskWait=[Diagnostics.Stopwatch]::StartNew()
        while(!@(Get-NetTCPConnection -State Listen -LocalPort $taskPlan.debugPort -ErrorAction SilentlyContinue).Count){
            if($taskProduct.HasExited -or $taskWait.Elapsed.TotalSeconds -ge 10){throw 'Actual ordinary WebView debugger listener unavailable'}
            Start-Sleep -Milliseconds 100
        }
        $taskBinding=Save-WindowsObservationBinding -Process $taskProduct -ExpectedExecutable $taskPlan.executable `
            -DebugPort $taskPlan.debugPort -OutputPath (Join-Path $taskGenerationRoot 'native-binding.json')
        $taskObserverRoot=Join-Path $taskGenerationRoot 'observer'
        $taskObserverStop=Join-Path $taskGenerationRoot 'stop-observer.flag'
        $taskObservationPlan=Join-Path $taskGenerationRoot 'observation-plan.json'
        $taskManager=Get-Content -LiteralPath (Join-Path $taskRoot 'cache\updates\manager-stable.json') -Raw|ConvertFrom-Json
        $taskPanel=Get-Content -LiteralPath (Join-Path $taskRoot 'cache\updates\panel-latest.json') -Raw|ConvertFrom-Json
        Write-CacheJson $taskObservationPlan ([ordered]@{authorization=$taskHuman;source=$taskPlan.source
            outputDirectory=$taskObserverRoot;endpoint=('http://127.0.0.1:'+$taskPlan.debugPort)
            nativeBinding=$taskBinding;expectedRoot=$taskRoot;expectedManagerCheckedAt=$taskManager.last_checked_at
            expectedPanelVersion=$taskPanel.version;stopFile=$taskObserverStop})
        $taskNodeInfo=[Diagnostics.ProcessStartInfo]::new()
        $taskNodeInfo.FileName=$taskPlan.nodeExecutable
        $taskNodeInfo.WorkingDirectory=$taskGenerationRoot
        $taskNodeInfo.UseShellExecute=$false
        $taskNodeInfo.CreateNoWindow=$true
        $taskNodeInfo.RedirectStandardOutput=$true
        $taskNodeInfo.RedirectStandardError=$true
        $taskNodeInfo.StandardOutputEncoding=[Text.UTF8Encoding]::new($false)
        $taskNodeInfo.StandardErrorEncoding=[Text.UTF8Encoding]::new($false)
        [void]$taskNodeInfo.Environment.Remove('NODE_OPTIONS')
        foreach($taskArg in @((Join-Path $PSScriptRoot 'windows-cache-query-observe.mjs'),'--observe',$taskObservationPlan)){
            $taskNodeInfo.ArgumentList.Add($taskArg)
        }
        $taskObserver=[Diagnostics.Process]::Start($taskNodeInfo)
        $taskObserverOut=$taskObserver.StandardOutput.ReadToEndAsync()
        $taskObserverError=$taskObserver.StandardError.ReadToEndAsync()
        $taskWait=[Diagnostics.Stopwatch]::StartNew()
        $taskReadyFile=Join-Path $taskObserverRoot 'observer-ready.json'
        while(!(Test-Path -LiteralPath $taskReadyFile)){
            if($taskObserver.HasExited -or $taskWait.Elapsed.TotalSeconds -ge 20){throw 'Passive observer did not become ready'}
            Start-Sleep -Milliseconds 100
        }
        $taskReady=Get-Content -LiteralPath $taskReadyFile -Raw|ConvertFrom-Json
        $taskRoute=$taskReady.metadata.initialRoute
        for($taskAction=1;$taskAction -le 10;$taskAction++){
            $null=Assert-CacheForegroundWindow
            if($taskProduct.HasExited -or ![CacheQueryNative]::OwnsWindow($taskWindow,$taskProduct.Id)){
                throw 'Retained product/window ownership changed before input'
            }
            if(![CacheQueryNative]::Activate($taskWindow)){throw 'Interactive desktop/foreground lost'}
            $taskTarget=if($taskRoute -ceq 'overview'){'settings'}else{'overview'}
            $taskLabel=if($taskTarget -ceq 'settings'){'设置'}else{'概览'}
            $taskCondition=[Windows.Automation.AndCondition]::new(
                [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::NameProperty,$taskLabel),
                [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ControlTypeProperty,[Windows.Automation.ControlType]::Button))
            $taskElement=[Windows.Automation.AutomationElement]::FromHandle($taskWindow)
            $taskButton=$taskElement.FindFirst([Windows.Automation.TreeScope]::Descendants,$taskCondition)
            if(!$taskButton -or !$taskButton.Current.IsEnabled){throw 'Observed native navigation button unavailable'}
            if($taskProduct.HasExited -or ![CacheQueryNative]::OwnsWindow($taskWindow,$taskProduct.Id) -or
                [CacheQueryNative]::GetForegroundWindow() -ne $taskWindow){throw 'Foreground/owner changed before native input'}
            $taskInvoke=$taskButton.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern)
            $taskInvoke.Invoke()
            Write-CacheJson (Join-Path $taskObserverRoot ('route-'+$taskAction+'-request.json')) `
                ([ordered]@{index=$taskAction;expectedRoute=$taskTarget;atUtc=[datetimeoffset]::UtcNow.ToString('o')})
            $taskAck=Join-Path $taskObserverRoot ('route-'+$taskAction+'-confirmed.json')
            $taskWait=[Diagnostics.Stopwatch]::StartNew()
            while(!(Test-Path -LiteralPath $taskAck)){
                if($taskObserver.HasExited -or $taskWait.Elapsed.TotalSeconds -ge 5){throw 'Actual route transition not confirmed'}
                Start-Sleep -Milliseconds 50
            }
            $taskRoute=$taskTarget
        }
        while(!$taskObserver.WaitForExit(500)){
            $null=Assert-CacheForegroundWindow
            if($taskProduct.HasExited -or $taskTimer.Elapsed.TotalSeconds -ge 90){throw 'Incomplete original startup evaluation before cleanup bound'}
        }
        [IO.File]::WriteAllText((Join-Path $taskGenerationRoot 'observer.stdout.original.log'),$taskObserverOut.GetAwaiter().GetResult())
        [IO.File]::WriteAllText((Join-Path $taskGenerationRoot 'observer.stderr.original.log'),$taskObserverError.GetAwaiter().GetResult())
        if($taskObserver.ExitCode -ne 0){throw 'Passive observer failed; preserve this generation'}
        $taskSample=Get-Content -LiteralPath (Join-Path $taskObserverRoot 'generation-result.json') -Raw|ConvertFrom-Json
        if($taskSample.routeTransitions -ne 10 -or !$taskSample.realStartupPlannerCompleted -or $taskSample.observedQueryCalls -ne 0){
            throw 'Actual valid-cache generation incomplete'
        }
        $taskObserver.Dispose();$taskObserver=$null
        if(!$taskProduct.HasExited -and [CacheQueryNative]::OwnsWindow($taskWindow,$taskProduct.Id)){
            [void][CacheQueryNative]::PostMessageW($taskWindow,0x10,[IntPtr]::Zero,[IntPtr]::Zero)
        }
        if(!$taskProduct.WaitForExit(10000) -or $taskProduct.ExitCode -ne 0){throw 'Normal product exit0 not confirmed'}
        $taskRows.Add([ordered]@{generation=$taskGeneration;startedAtUtc=$taskStarted;pid=$taskProduct.Id
            user=$taskUser;routeTransitions=10;originalPlannerCompleted=$true;normalExitCode=$taskProduct.ExitCode
            endedAtUtc=[datetimeoffset]::UtcNow.ToString('o');forcedTermination=$false})
        $taskProduct.Dispose();$taskProduct=$null
        Wait-CacheWebViewExit
        Write-CacheJson (Join-Path $taskRun 'progress.json') `
            ([ordered]@{completedGenerations=$taskRows.Count;routeTransitions=$taskRows.Count*10
                plannedGenerations=$taskCount;firstGeneration=$taskFirst;originalRequiredGenerations=10})
    }
}catch{
    $taskFailure=$_.Exception.ToString()
    Write-CacheJson (Join-Path $taskRun 'failure.original.json') `
        ([ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');error=$taskFailure;completedGenerations=$taskRows.Count})
}finally{
    $taskCleanupErrors=[Collections.Generic.List[string]]::new()
    if($taskObserver){
        if(!$taskObserver.HasExited){
            [IO.File]::WriteAllText($taskObserverStop,'stop')
            if(!$taskObserver.WaitForExit(5000)){$taskObserver.Kill();$taskObserver.WaitForExit(5000)|Out-Null;$taskObserverForced=$true}
        }
        if($taskObserverOut -and $taskObserverOut.Wait(1000) -and
            !(Test-Path -LiteralPath (Join-Path $taskGenerationRoot 'observer.stdout.original.log'))){
            [IO.File]::WriteAllText((Join-Path $taskGenerationRoot 'observer.stdout.original.log'),$taskObserverOut.Result)
        }
        if($taskObserverError -and $taskObserverError.Wait(1000) -and
            !(Test-Path -LiteralPath (Join-Path $taskGenerationRoot 'observer.stderr.original.log'))){
            [IO.File]::WriteAllText((Join-Path $taskGenerationRoot 'observer.stderr.original.log'),$taskObserverError.Result)
        }
        $taskObserver.Dispose()
    }
    if($taskProduct){
        if(!$taskProduct.HasExited){
            $taskProduct.Refresh()
            if($taskProduct.MainWindowHandle -ne [IntPtr]::Zero){
                [void][CacheQueryNative]::PostMessageW($taskProduct.MainWindowHandle,0x10,[IntPtr]::Zero,[IntPtr]::Zero)
            }
            if(!$taskProduct.WaitForExit(10000)){$taskCleanupErrors.Add('Product normal exit still pending; never force terminate')}
        }
        $taskProduct.Dispose()
    }
    if($taskAdministrator){
        [IO.File]::WriteAllText($taskCounterStop,'stop')
        if(!$taskAdministrator.WaitForExit(20000)){$taskCleanupErrors.Add('Owned ETW observer exit unknown')}
        elseif($taskAdministrator.ExitCode -ne 0){$taskCleanupErrors.Add('Owned ETW observer exited nonzero')}
        $taskAdministrator.Dispose()
    }
    try{
        Assert-StartupPairNoProduct
        if(Get-Command Wait-CacheWebViewExit -ErrorAction SilentlyContinue){Wait-CacheWebViewExit}
        if(!$taskRestorationAdmitted){throw 'No verified saved-original transaction admitted; preparation owner must recover it'}
        if((Get-FileHash -LiteralPath $taskPlan.restoreMethod.path).Hash.ToLowerInvariant() -cne $taskPlan.restoreMethod.sha256){
            throw 'Restoration method changed'
        }
        & $taskPlan.restoreMethod.path -RunDirectory $taskRun
        if(!$?){throw 'Original configuration restoration failed'}
        $taskRestored=$true
    }catch{$taskCleanupErrors.Add($_.Exception.ToString())}
    Write-CacheJson (Join-Path $taskRun 'native-run-terminal.json') `
        ([ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');rows=$taskRows.ToArray()
            completedGenerations=$taskRows.Count;routeTransitions=$taskRows.Count*10
            failure=$taskFailure;cleanupErrors=$taskCleanupErrors.ToArray();originalConfigurationRestored=$taskRestored
            productForcedTermination=$false;observerForcedTermination=$taskObserverForced
            nativeEtwReadyObserved=$taskEtwReady;fullPerformanceGatePassed=$false;fullWindowsGatePassed=$false})
}
if($taskFailure -or !$taskRestored -or $taskCleanupErrors.Count){throw 'Native cache attempt incomplete; original evidence retained'}
Write-Output 'Requested sequential native generations completed; aggregate original10/100 and raw counter analysis are still required.'

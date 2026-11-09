param(
 [Parameter(Mandatory=$true)][string]$RunDirectory,
 [Parameter(Mandatory=$true)][string]$ProgramDirectory,
 [Parameter(Mandatory=$true)][string]$ProvisionTool,
 [Parameter(Mandatory=$true)][ValidateSet('console','rdp')][string]$Role,
 [Parameter(Mandatory=$true)][string]$ExpectedSid,
 [Parameter(Mandatory=$true)][string]$ExpectedSource,
 [Parameter(Mandatory=$true)][string]$ExpectedBuildRun,
 [Parameter(Mandatory=$true)][string]$ExpectedAppsTree,
 [Parameter(Mandatory=$true)][int]$ExpectedSessionId
)
$ErrorActionPreference='Stop'
if(!$IsWindows){throw 'Actual Windows session required'}
. (Join-Path $PSScriptRoot 'windows023-two-user-native-helper.ps1')
. (Join-Path $PSScriptRoot 'windows-process-token.ps1')
$taskSid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$taskSelf=[Diagnostics.Process]::GetCurrentProcess()
$taskToken=Get-WindowsHarnessToken
if($taskSid -cne $ExpectedSid -or $taskSelf.SessionId -ne $ExpectedSessionId){throw 'Actual SID/session differs from sealed launch request'}
$taskMode=if($Role -eq 'console'){'administrator'}else{'ordinary'}
Assert-WindowsProcessMode $taskToken $taskMode
$taskRoot=Assert-TwoUserLocalPath $RunDirectory
$taskProgram=Assert-TwoUserLocalPath $ProgramDirectory
Assert-TwoUserNoReparseAncestors $taskRoot
Assert-TwoUserNoReparseAncestors $taskProgram
$taskOutput=Join-Path $taskRoot $Role
if(Test-Path -LiteralPath $taskOutput){throw 'Never overwrite earlier session evidence'}
New-Item -ItemType Directory -Path $taskOutput | Out-Null
$taskPrivate=Join-Path $taskOutput 'private'
$taskFixture=Join-Path $taskPrivate 'sandbox'
New-Item -ItemType Directory -Path $taskFixture | Out-Null
& icacls.exe $taskPrivate /setowner ('*'+$taskSid) /T /C *> (Join-Path $taskOutput 'fixture-owner.log')
if($LASTEXITCODE -ne 0){throw 'New fixture owner setup failed'}
& icacls.exe $taskPrivate /inheritance:r /grant:r ('*'+$taskSid+':(OI)(CI)F') '*S-1-5-18:(OI)(CI)F' *> (Join-Path $taskOutput 'fixture-acl.log')
if($LASTEXITCODE -ne 0){throw 'New fixture ACL setup failed'}
$taskInstall=Join-Path $taskFixture 'program'
New-Item -ItemType Directory -Path $taskInstall | Out-Null
foreach($entry in Get-ChildItem -LiteralPath $taskProgram -Force){Copy-Item -LiteralPath $entry.FullName -Destination $taskInstall -Recurse}
$taskBuild=Get-Content -LiteralPath (Join-Path $taskProgram 'core-build.json') -Raw | ConvertFrom-Json
if($taskBuild.source -cne $ExpectedSource -or $taskBuild.run -cne $ExpectedBuildRun -or $taskBuild.appsTree -cne $ExpectedAppsTree -or @($taskBuild.files).Count -ne 9 -or $taskBuild.accepted -ne $false){throw 'Exact unaccepted source build required'}
foreach($entry in $taskBuild.files){
 $path=Join-Path $taskInstall $entry.path
 $null=Assert-TwoUserUnder $taskInstall $path
 Assert-TwoUserNoReparseAncestors $path
 if((Get-Item -LiteralPath $path).Length -ne $entry.bytes -or (Get-FileHash -LiteralPath $path).Hash -ine $entry.sha256){throw 'Sealed candidate member differs'}
}
$taskDescriptor=Get-Content -LiteralPath (Join-Path $taskInstall 'runtime-bundle/runtime-bundle.manifest.json') -Raw | ConvertFrom-Json
if($taskDescriptor.archiveName -notmatch '^[A-Za-z0-9_.-]+$' -or $taskDescriptor.runtimeVersion -cne '2.97.0-desktop023.lifecycle.3' -or $taskDescriptor.runtimeArchiveSha256 -cne 'ec14baaee354c6729d1d2545ab9165d207b14a3a4239f354662163087eeb0739'){throw 'Adopted runtime identity differs'}
$taskArchive=Join-Path $taskInstall ('runtime-bundle/'+$taskDescriptor.archiveName)
if((Get-FileHash -LiteralPath $taskArchive).Hash -ine $taskDescriptor.archiveSha256){throw 'Pinned resource archive differs'}
# The fixture tool invokes the unchanged, digest-pinned offline provisioner.
# It refuses every existing I/runtime; product recovery is tested separately.
$taskProvision=(Resolve-Path -LiteralPath $ProvisionTool).ProviderPath
$taskProvisionBuild=Get-Content -LiteralPath (Join-Path (Split-Path -Parent $taskProvision) 'fixture-build.json') -Raw | ConvertFrom-Json
if(!$taskProvisionBuild.fixtureOnly -or $taskProvisionBuild.source -cne $ExpectedSource -or $taskProvisionBuild.appsTree -cne $taskBuild.appsTree -or $taskProvisionBuild.descriptorSha256 -cne 'aefa7bee63a1c32d96d912a5c919249c46bae86656e2275e1036146a6cee3576' -or (Get-FileHash -LiteralPath $taskProvision).Hash -ine $taskProvisionBuild.sha256){throw 'Fixture binary/source/pin differs'}
New-Item -ItemType Directory -Path (Join-Path $taskFixture 'home') | Out-Null
& icacls.exe $taskFixture /setowner ('*'+$taskSid) /T /C *> (Join-Path $taskOutput 'new-fixture-owner.log')
if($LASTEXITCODE -ne 0){throw 'New Sandbox ownership setup failed'}
$taskNode=Join-Path $taskInstall 'runtime/node/node.exe'
$taskExe=Join-Path $taskInstall 'opencodex-desktop.exe'
$taskCode=Join-Path $taskPrivate 'code';New-Item -ItemType Directory -Path $taskCode | Out-Null
foreach($name in @('windows-cdp.mjs','windows023-current-session-rpc.mjs')){Copy-Item -LiteralPath (Join-Path $PSScriptRoot $name) -Destination $taskCode}
$taskDriver=Get-TwoUserDriverText
$from=$taskDriver.IndexOf('    await diagnosticReadiness();',[StringComparison]::Ordinal)
$to=$taskDriver.IndexOf('    const confirmedLate=',$from,[StringComparison]::Ordinal)
if($from -lt 0 -or $to -le $from){throw 'Reviewed failure diagnostic boundary changed'}
$taskDriver=$taskDriver.Remove($from,$to-$from)
$capture=@'
const sessionInvoke=runtimeInvoke,sessionTrace=[];
runtimeInvoke=async(command,args={},timeout=10000)=>{
 const began=Date.now();sessionTrace.push({stage:'begin',command,action:args.request?.action??null,atUtc:new Date().toISOString()});
 try{
  let value;
  if(command==='process_action'){
   const expression='window.__TAURI_INTERNALS__.invoke('+JSON.stringify(command)+','+JSON.stringify(args)+').then(value=>({raw:JSON.stringify(value),value}),error=>({rejected:JSON.stringify(error)}))';
   const answer=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true},timeout);
   assert(!answer.exceptionDetails,'Actual product action failed');
   const response=answer.result?.value;
   await writeFile(join(output,'action-envelope.json'),JSON.stringify(response,null,2)+'\n',{flag:'wx'});
   if(response?.raw!==undefined)await writeFile(join(output,'action.raw.json'),response.raw,{flag:'wx'});
   assert(!response?.rejected,'Actual product action rejected');value=response.value;
  }else value=await sessionInvoke(command,args,command==='get_status_snapshot'?5000:timeout);
  sessionTrace.push({stage:'returned',command,action:args.request?.action??null,elapsedMilliseconds:Date.now()-began});return value;
 }catch(error){sessionTrace.push({stage:'failed',command,error:error.message,elapsedMilliseconds:Date.now()-began});throw error}
 finally{await writeFile(join(output,'invoke-trace.json'),JSON.stringify(sessionTrace,null,2)+'\n')}
};
'@
$taskDriver=$taskDriver.Replace("if(endpoint==='--runtime-self-test')",$capture+[Environment]::NewLine+"if(endpoint==='--runtime-self-test')")
$taskReplacements=@(
 @("join(process.env.USERPROFILE,'.codex-desktop/opencodex/home')","join(c.fixtureRoot,'data/opencodex/home')"),
 @("join(process.env.USERPROFILE,'.codex-desktop/desktop')","join(c.fixtureRoot,'data/desktop')"),
 @("[process.env.USERPROFILE,join(process.env.USERPROFILE,'.codex-desktop'),join(process.env.USERPROFILE,'.codex-desktop/opencodex'),c.home]","[c.fixtureRoot,join(c.fixtureRoot,'data'),join(c.fixtureRoot,'data/opencodex'),c.home]")
)
foreach($replacement in $taskReplacements){if(!$taskDriver.Contains($replacement[0])){throw 'Reviewed R33 adaptation no longer matches'};$taskDriver=$taskDriver.Replace($replacement[0],$replacement[1])}
$taskIdentityWait=@'

// Fixture transport only: keep this driver generation alive until its retained
// handle/SID/FILETIME admission has completed. Product timers are unchanged.
if(process.env.OCXD_DRIVER_IDENTITY_ACK){
 const {access}=await import('node:fs/promises');const until=Date.now()+10000;
 while(true){try{await access(process.env.OCXD_DRIVER_IDENTITY_ACK);break}catch{if(Date.now()>until)throw Error('Driver identity admission missing');await new Promise(r=>setTimeout(r,20))}}
}
'@
$taskDriver+=$taskIdentityWait
[IO.File]::AppendAllText((Join-Path $taskCode 'windows023-current-session-rpc.mjs'),$taskIdentityWait)
[IO.File]::WriteAllText((Join-Path $taskCode 'two-user-driver.mjs'),$taskDriver)
$taskLedger=[Collections.Generic.Dictionary[int,object]]::new()
$taskManager=$null;$taskFacts=$null;$taskRuntimeFact=$null
$taskSummary=[ordered]@{accepted=$false;diagnosticOnly=$true;role=$Role;sid=$taskSid;sessionId=$taskSelf.SessionId;harnessToken=$taskToken;source=$taskBuild.source;sandbox=$true;productionUserRootVerified=$false;dailyClientDirectoriesUsed=$false;cleanup='pending'}
function Get-SessionDailyHashes {
 $profile=[Environment]::GetFolderPath([Environment+SpecialFolder]::UserProfile)
 @(foreach($relative in @('.codex/config.toml','.codex/auth.json','.claude/settings.json','.claude.json','.codex-desktop/desktop/manager-location.json','.codex-desktop/desktop/manager-state/preferences.json')){
  $path=Join-Path $profile $relative
  @{path=$path;exists=(Test-Path -LiteralPath $path -PathType Leaf);sha256=$(if(Test-Path -LiteralPath $path -PathType Leaf){(Get-FileHash -LiteralPath $path).Hash.ToLowerInvariant()}else{$null})}
 })
}
$taskDailyBefore=Get-SessionDailyHashes
Write-TwoUserJson (Join-Path $taskOutput 'daily-config-before.json') $taskDailyBefore
function Add-SessionDescendants {
 $null=Assert-TwoUserRootForDescendantObservation $taskFacts
 $snapshot=@(Get-CimInstance Win32_Process)
 $parents=[Collections.Generic.HashSet[int]]::new();[void]$parents.Add([int]$taskFacts.pid)
 do{
  $changed=$false
  foreach($row in $snapshot){
   if(!$parents.Contains([int]$row.ParentProcessId) -or $parents.Contains([int]$row.ProcessId) -or $row.CreationDate.ToUniversalTime().Ticks -lt ([long]$taskFacts.creationTicks-9)){continue}
   $child=Get-Process -Id $row.ProcessId -ErrorAction SilentlyContinue
   if(!$child){continue}
   try{
    try{[void]$child.Handle;$fact=Get-TwoUserProcessFact $child $taskSid}
    catch{
     # A retained OS handle may prove this exact short-lived generation exited.
     # Never reopen a PID to substitute for that proof or admit a live unknown.
     if($child.WaitForExit(250)){
      @{snapshotPid=$row.ProcessId;retainedOriginalExited=$true;admitted=$false;accepted=$false} | ConvertTo-Json -Compress | Add-Content -LiteralPath (Join-Path $taskOutput 'exited-before-cim.jsonl')
      continue
     }
     throw
    }
    $taskLedger[[int]$fact.pid]=$fact;[void]$parents.Add([int]$fact.pid);$changed=$true
   }finally{$child.Dispose()}
  }
 }while($changed)
}
function Invoke-SessionDriver([string]$Mode,[int]$Port=0,[int]$ExpectedPid=0){
 $dir=Join-Path $taskOutput ($Mode+'-'+[guid]::NewGuid().ToString('N').Substring(0,8));New-Item -ItemType Directory -Path $dir | Out-Null
 $inputPath=Join-Path $dir 'command.json'
 Write-TwoUserJson $inputPath @{mode=$Mode;fixtureRoot=$taskFixture;root=(Join-Path $taskFixture 'data/desktop');home=(Join-Path $taskFixture 'data/opencodex/home');port=$Port;expectedPid=$ExpectedPid}
 $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$taskNode;$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
 $driverName=if($Mode -in @('prepare','inspect')){'windows023-current-session-rpc.mjs'}else{'two-user-driver.mjs'}
 foreach($arg in @((Join-Path $taskCode $driverName),('http://127.0.0.1:'+$taskPort),$dir,$inputPath)){$info.ArgumentList.Add($arg)}
 $info.Environment['USERPROFILE']=Join-Path $taskFixture 'home';$info.Environment['HOME']=Join-Path $taskFixture 'home'
 $identityAck=Join-Path $dir 'driver-identity-admitted'
 $info.Environment['OCXD_DRIVER_IDENTITY_ACK']=$identityAck
 $driver=[Diagnostics.Process]::Start($info)
 try{
  $fact=Get-TwoUserProcessFact $driver $taskSid;$taskLedger[[int]$fact.pid]=$fact
  [IO.File]::WriteAllText($identityAck,'retained handle admitted')
  $stdout=$driver.StandardOutput.ReadToEndAsync();$stderr=$driver.StandardError.ReadToEndAsync()
  $deadline=[datetime]::UtcNow.AddSeconds($(if($Mode -eq 'prepare'){360}else{120}))
  while(!$driver.HasExited){
   Add-SessionDescendants
   if([datetime]::UtcNow -gt $deadline){throw 'Driver envelope elapsed; original lifecycle budgets remain failed'}
   Start-Sleep -Milliseconds 300
  }
  $file=Join-Path $dir $(if($driverName -eq 'two-user-driver.mjs'){'runtime-step.json'}else{'rpc-result.json'})
  if(!(Test-Path -LiteralPath $file)){throw 'No controlled actual-app receipt'}
  $value=Get-Content -LiteralPath $file -Raw | ConvertFrom-Json
  Write-TwoUserJson (Join-Path $dir 'driver-exit.json') @{exitCode=$driver.ExitCode;handleExited=$driver.HasExited;identity=$fact}
  [pscustomobject]@{exitCode=$driver.ExitCode;receipt=$value;directory=$dir}
 }finally{$driver.Dispose()}
}
try{
 $listener=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0);$listener.Start();$taskPort=$listener.LocalEndpoint.Port;$listener.Stop()
 $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$taskExe;$info.WorkingDirectory=$taskInstall;$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
 $info.Environment['OPENCODEX_SANDBOX']='1';$info.Environment['OPENCODEX_SANDBOX_ROOT']=$taskFixture;$info.Environment['OPENCODEX_WINDOWS_SMOKE_CDP_PORT']=[string]$taskPort
 foreach($key in @('HOME','OPENCODEX_HOME','CODEX_HOME','CLAUDE_CONFIG_DIR','WEBVIEW2_USER_DATA_FOLDER','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS')){[void]$info.Environment.Remove($key)}
 $taskManager=[Diagnostics.Process]::Start($info);$taskStdout=$taskManager.StandardOutput.ReadToEndAsync();$taskStderr=$taskManager.StandardError.ReadToEndAsync()
 $taskFacts=Get-TwoUserProcessFact $taskManager $taskSid;$taskLedger[[int]$taskFacts.pid]=$taskFacts
 $applicationToken=Get-WindowsProcessToken $taskManager $taskExe
 Assert-WindowsProcessMode $applicationToken $taskMode
 # Let the real product create its recognized v2 structure BEFORE supply writes D.
 $bootstrapDeadline=[datetime]::UtcNow.AddSeconds(20)
 do{
  Add-SessionDescendants
  $bootstrapWindow=$null;try{$bootstrapWindow=Get-TwoUserWindow $taskFacts}catch{}
  $bootstrapReady=$bootstrapWindow -and (Test-Path -LiteralPath (Join-Path $taskFixture 'data/desktop/manager-state/data-root.json'))
  if(!$bootstrapReady){Start-Sleep -Milliseconds 250}
 }while(!$bootstrapReady -and [datetime]::UtcNow -lt $bootstrapDeadline)
 if(!$bootstrapReady -or (Test-Path -LiteralPath (Join-Path $taskFixture 'data/opencodex/home/runtime-port.json'))){throw 'Actual empty-product initialization unproved; no supply permitted'}
 foreach($fact in @($taskLedger.Values | Sort-Object {[long]$_.creationTicks} -Descending)){Stop-TwoUserTracked $fact}
 if(!$taskManager.WaitForExit(10000)){throw 'Original initialization generation did not exit'}
 Write-TwoUserJson (Join-Path $taskOutput 'initialization-only.json') @{identity=$taskFacts;token=$applicationToken;window=$bootstrapWindow;originalHandleExited=$true;runtimeStarted=$false;normalExitTest=$false;method='exact-owned fixture preparation cleanup';accepted=$false}
 $taskManager.Dispose();$taskManager=$null;$taskFacts=$null
 $provisionInfo=[Diagnostics.ProcessStartInfo]::new();$provisionInfo.FileName=$taskProvision;$provisionInfo.ArgumentList.Add($taskFixture);$provisionInfo.UseShellExecute=$false;$provisionInfo.CreateNoWindow=$true;$provisionInfo.RedirectStandardOutput=$true;$provisionInfo.RedirectStandardError=$true
 $provision=[Diagnostics.Process]::Start($provisionInfo)
 try{
  $provisionFact=Get-TwoUserProcessFact $provision $taskSid;$taskLedger[[int]$provisionFact.pid]=$provisionFact
  $provisionOut=$provision.StandardOutput.ReadToEndAsync();$provisionErr=$provision.StandardError.ReadToEndAsync()
  if(!$provision.WaitForExit(300000)){throw 'Fixture provision deadline; no runtime start attempted'}
  Write-TwoUserJson (Join-Path $taskOutput 'offline-provision.json') @{identity=$provisionFact;exitCode=$provision.ExitCode;stdout=$provisionOut.GetAwaiter().GetResult();stderr=$provisionErr.GetAwaiter().GetResult();accepted=$false}
  if($provision.ExitCode -ne 0){throw 'Actual offline provisioner refused fresh fixture'}
 }finally{$provision.Dispose()}
 $taskManager=[Diagnostics.Process]::Start($info);$taskStdout=$taskManager.StandardOutput.ReadToEndAsync();$taskStderr=$taskManager.StandardError.ReadToEndAsync()
 $taskFacts=Get-TwoUserProcessFact $taskManager $taskSid;$taskLedger[[int]$taskFacts.pid]=$taskFacts
 $applicationToken=Get-WindowsProcessToken $taskManager $taskExe
 Assert-WindowsProcessMode $applicationToken $taskMode
 $taskSummary.manager=$taskFacts;$taskSummary.applicationToken=$applicationToken
 Write-TwoUserJson (Join-Path $taskOutput 'identity.json') $taskSummary
 $prepared=Invoke-SessionDriver 'prepare';Write-TwoUserJson (Join-Path $taskOutput 'prepared.json') $prepared
 if($prepared.exitCode -ne 0){throw 'Actual Sandbox preparation failed; retain controlled receipt'}
 Write-TwoUserJson (Join-Path $taskOutput 'ready.json') @{sid=$taskSid;sessionId=$taskSelf.SessionId;manager=$taskFacts;applicationToken=$applicationToken;port=$taskPort;sandbox=$true;accepted=$false}
 $sequence=0;$until=[datetime]::UtcNow.AddMinutes(35)
 while([datetime]::UtcNow -lt $until){
  $path=Join-Path $taskOutput ('command-'+(++$sequence)+'.json')
  while(!(Test-Path -LiteralPath $path)){if([datetime]::UtcNow -gt $until){throw 'Controller command deadline'};if($taskManager.HasExited){throw 'Manager exited before exit check'};Start-Sleep -Milliseconds 250}
  $command=Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
  if($command.mode -notin @('start','conflict','observe','stop','inspect','await-tray-exit','finish')){throw 'Unadmitted controller command'}
  if($command.mode -eq 'await-tray-exit'){
   Write-TwoUserJson (Join-Path $taskOutput 'awaiting-native-tray-exit.json') @{sid=$taskSid;sessionId=$taskSelf.SessionId;manager=$taskFacts;accepted=$false}
   $exited=$taskManager.WaitForExit(30000)
   Write-TwoUserJson (Join-Path $taskOutput ('reply-'+$sequence+'.json')) @{mode=$command.mode;originalHandleExited=$exited;exitCode=$(if($exited){$taskManager.ExitCode}else{$null});accepted=$false}
   $taskSummary.normalManagerExitVerified=$exited
   if(!$exited){throw 'Actual native tray exit not observed within thirty seconds'}
   break
  }
  if($command.mode -eq 'finish'){break}
  $runtimeToken=$null;$runtimeIdentity=$null
  $before=Get-TwoUserWindow $taskFacts
  if($command.mode -eq 'stop' -and $taskRuntimeFact){$null=Assert-TwoUserLive $taskRuntimeFact}
  $value=Invoke-SessionDriver $command.mode ([int]$command.port) ([int]$command.expectedPid)
  $after=Get-TwoUserWindow $taskFacts
  $public=$value.receipt.record
  if($public){
   if(!$taskLedger.ContainsKey([int]$public.pid)){throw 'Runtime PID not retained from own launch family'}
   $process=Get-Process -Id $public.pid
   try{$runtimeIdentity=Get-TwoUserProcessFact $process $taskSid;$runtimeToken=Get-WindowsProcessToken $process $runtimeIdentity.path;Assert-WindowsProcessMode $runtimeToken $taskMode}finally{$process.Dispose()}
   $listeners=@(Get-NetTCPConnection -State Listen -LocalPort $public.port -ErrorAction SilentlyContinue)
   if(!$listeners.Count -or @($listeners | Where-Object {$_.OwningProcess -ne $runtimeIdentity.pid -or $_.LocalAddress -notin @('127.0.0.1','::1')}).Count){throw 'Own listener/identity differs'}
   $taskRuntimeFact=$runtimeIdentity
  }
  $restore=$null
  if($command.mode -eq 'stop' -and $value.receipt.result -eq 'pass-runtime-step'){
   $client=Join-Path $taskFixture 'home/.codex/config.toml'
   $bytes=if(Test-Path -LiteralPath $client){[IO.File]::ReadAllText($client)}else{''}
   $restore=@{originalServiceGenerationExited=($taskRuntimeFact -and (Test-TwoUserOriginalExited $taskRuntimeFact));listenerAbsent=(@(Get-NetTCPConnection -State Listen -LocalPort ([int]$command.port) -ErrorAction SilentlyContinue).Count -eq 0);ownRecordAbsent=!(Test-Path -LiteralPath (Join-Path $taskFixture 'data/opencodex/home/runtime-port.json'));clientRouteRemoved=(!$bytes.Contains('127.0.0.1:'+([string]$command.port)))}
   $restore.originalServiceExit=Get-TwoUserOriginalExit $taskRuntimeFact
   $restore.passed=($restore.originalServiceExit.originalHandleExited -and $restore.originalServiceExit.exitCode -eq 0 -and $restore.originalServiceGenerationExited -and $restore.listenerAbsent -and $restore.ownRecordAbsent -and $restore.clientRouteRemoved)
  }
  Write-TwoUserJson (Join-Path $taskOutput ('reply-'+$sequence+'.json')) @{sid=$taskSid;sessionId=$taskSelf.SessionId;mode=$command.mode;value=$value;managerBefore=$before;managerAfter=$after;runtimeIdentity=$runtimeIdentity;runtimeToken=$runtimeToken;restore=$restore;accepted=$false}
 }
 $taskSummary.result='diagnostic-complete'
}catch{
 $taskSummary.result='fail';$taskSummary.error=$_.Exception.Message;$taskSummary.errorLine=$_.InvocationInfo.ScriptLineNumber
 Write-TwoUserJson (Join-Path $taskOutput 'failure.json') $taskSummary
}finally{
 $cleanup=@();$taskCleanupFailed=$false
 $exits=@(foreach($fact in $taskLedger.Values){
  try{@{identity=$fact;exit=(Get-TwoUserOriginalExit $fact)}}
  catch{@{identity=$fact;exitUnknown=$true}}
 })
 Write-TwoUserJson (Join-Path $taskOutput 'original-exits-before-cleanup.json') $exits
 if($taskManager -and !$taskManager.HasExited -and $taskFacts){
  try{Add-SessionDescendants}catch{$taskCleanupFailed=$true;$cleanup+=@{result='descendant-identity-unproved'}}
 }
 foreach($fact in @($taskLedger.Values | Sort-Object {[long]$_.creationTicks} -Descending)){
  try{$alreadyExited=Test-TwoUserOriginalExited $fact;Stop-TwoUserTracked $fact;$cleanup+=@{identity=$fact;result=$(if($alreadyExited){'original-exited'}else{'exact-owned-cleanup'})}}catch{$taskCleanupFailed=$true;$cleanup+=@{identity=$fact;result='retained-unproved'}}
 }
 $executed=Join-Path $taskOutput 'executed-code';New-Item -ItemType Directory -Path $executed|Out-Null
 if(Test-Path -LiteralPath $taskCode){foreach($entry in Get-ChildItem -LiteralPath $taskCode -File){Copy-Item -LiteralPath $entry.FullName -Destination $executed}}
 foreach($n in @('windows023-current-session.ps1','windows023-two-user-native-helper.ps1','windows-process-token.ps1')){Copy-Item -LiteralPath (Join-Path $PSScriptRoot $n) -Destination $executed}
 $taskLog=Join-Path $taskFixture 'data/desktop/logs/app.log'
 if(Test-Path -LiteralPath $taskLog){Copy-Item -LiteralPath $taskLog -Destination (Join-Path $taskOutput 'app.log')}
 $taskDailyAfter=Get-SessionDailyHashes
 Write-TwoUserJson (Join-Path $taskOutput 'daily-config-after.json') $taskDailyAfter
 $taskSummary.dailyConfigBytesUnchanged=([string](ConvertTo-Json -InputObject $taskDailyBefore -Compress) -ceq [string](ConvertTo-Json -InputObject $taskDailyAfter -Compress))
 if(!$taskSummary.dailyConfigBytesUnchanged){$taskCleanupFailed=$true}
 $taskSummary.cleanup=if($taskCleanupFailed){'failed'}else{'tracked-generations-exited'}
 $taskSummary.cleanupReceipts=$cleanup
 if($taskManager -and $taskManager.HasExited){
  $taskSummary.managerExitCode=$taskManager.ExitCode
  if($taskStdout){[IO.File]::WriteAllText((Join-Path $taskOutput 'manager-stdout.log'),$taskStdout.GetAwaiter().GetResult())}
  if($taskStderr){[IO.File]::WriteAllText((Join-Path $taskOutput 'manager-stderr.log'),$taskStderr.GetAwaiter().GetResult())}
 }
 if($taskManager){$taskManager.Dispose()};Close-TwoUserProcessHandles
 if(!$taskCleanupFailed){
  $null=Assert-TwoUserUnder $taskOutput $taskPrivate
  Assert-TwoUserNoReparseAncestors $taskPrivate
  Remove-Item -LiteralPath $taskPrivate -Recurse -Force
  $taskSummary.privateFixtureRemoved=!(Test-Path -LiteralPath $taskPrivate)
 }
 Write-TwoUserJson (Join-Path $taskOutput 'finished.json') $taskSummary
}

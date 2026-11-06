# Limited two-user helpers; dot-sourcing never provisions accounts or launches.
param([ValidateSet('Library','Child')][string]$Mode='Library',[string]$ConfigPath)
function Get-TwoUserReuseParts([string]$OrdinaryScript) {
 $tokens=$null;$errors=$null
 $ast=[Management.Automation.Language.Parser]::ParseFile($OrdinaryScript,[ref]$tokens,[ref]$errors)
 if($errors.Count){throw 'Ordinary startup source no longer parses'}
 $strings=@($ast.FindAll({param($n) $n -is [Management.Automation.Language.StringConstantExpressionAst]},$true))
 $profileNodes=@($strings|Where-Object {$_.Value.Contains('public static class OcxNativeProfileProvision')})
 $desktop=@($strings|Where-Object {$_.Value.Contains('public sealed class OcxTemporaryDesktopAccess')})
 $child=@($strings|Where-Object {$_.Value.StartsWith('param([string]$InputFile)') -and $_.Value.Contains('OcxFirstLogonFolders')})
 if($profileNodes.Count -ne 1 -or $desktop.Count -ne 1 -or $child.Count -ne 1){throw 'Ordinary AST reuse points changed; review upstream'}
 $cut=$child[0].Value.IndexOf(' if($c.profileOnly){exit 0}',[StringComparison]::Ordinal)
 if($cut -lt 0){throw 'Ordinary Profile bootstrap boundary changed'}
 $bootstrap=$child[0].Value.Substring(0,$cut)+@'
 & $c.nativeHelper -Mode Child -ConfigPath $InputFile
 exit $LASTEXITCODE
}catch{[Console]::Error.WriteLine('Profile bootstrap failed: '+$_.Exception.GetType().Name);exit 1}
'@
 [pscustomobject]@{ProfileType=$profileNodes[0].Value;DesktopType=$desktop[0].Value;Bootstrap=$bootstrap}
}
function Assert-TwoUserChildGuard {
 if(!$IsWindows -or $env:GITHUB_ACTIONS -cne 'true' -or [string]::IsNullOrWhiteSpace($env:RUNNER_TEMP)){throw 'Disposable Windows Actions runner required'}
}
function Assert-TwoUserLocalPath([string]$Path) {
 if($Path -notmatch '^[a-zA-Z]:[\/]' -or $Path -match '(^|[\/])\.\.?([\/]|$)' -or $Path.Substring(2).Contains(':')){throw 'Local absolute non-device path required'}
 $resolved=[IO.Path]::GetFullPath($Path).TrimEnd('\')
 if($resolved.Length -le 3){throw 'Drive root is not a fixture boundary'}
 $resolved
}
function Assert-TwoUserNoReparseAncestors([string]$Path) {
 $current=[IO.Path]::GetFullPath($Path)
 while($current){
  if(Test-Path -LiteralPath $current){
   $item=Get-Item -LiteralPath $current -Force
   if($item.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Reparse ancestor is not a native fixture boundary'}
  }
  $parent=[IO.Path]::GetDirectoryName($current)
  if($parent -ceq $current){break};$current=$parent
 }
}
function Assert-TwoUserUnder([string]$Parent,[string]$Path) {
 $a=Assert-TwoUserLocalPath $Parent;$b=Assert-TwoUserLocalPath $Path
 if(!$b.StartsWith($a+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Path escaped tracked fixture'}
 $b
}
function Test-TwoUserSamePath([string]$Left,[string]$Right) {
 # Comparison only: retain the real backend/native paths in all evidence/actions.
 $a=$Left.Replace('/','\');$b=$Right.Replace('/','\')
 foreach($name in @('a','b')){
  $v=Get-Variable -Name $name -ValueOnly
  if($v -match '^\\\\\?\\[a-zA-Z]:\\'){$v=$v.Substring(4)}
  Set-Variable -Name $name -Value $v.TrimEnd('\')
 }
 [StringComparer]::OrdinalIgnoreCase.Equals($a,$b)
}
function Get-TwoUserTreeHash([string]$Directory) {
 $rootItem=Get-Item -LiteralPath $Directory -Force
 if(!$rootItem.PSIsContainer -or ($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Tree root must be a real directory'}
 $base=(Resolve-Path -LiteralPath $Directory).ProviderPath
 $entries=[Collections.Generic.List[object]]::new();$count=0
 $queue=[Collections.Generic.Queue[string]]::new();$queue.Enqueue($base)
 while($queue.Count){foreach($item in Get-ChildItem -LiteralPath $queue.Dequeue() -Force){
  if($item.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Reparse point in bounded tree'}
  if(++$count -gt 150000){throw 'Candidate inventory limit'}
  $relative=[IO.Path]::GetRelativePath($base,$item.FullName)
  if($item.PSIsContainer){$entries.Add([ordered]@{path=$relative;kind='directory'});$queue.Enqueue($item.FullName)}
  else{$entries.Add([ordered]@{path=$relative;kind='file';bytes=$item.Length;sha256=(Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash.ToLowerInvariant()})}
 }}
 $sorted=@($entries|Sort-Object {$_.path});$json=ConvertTo-Json -InputObject $sorted -Depth 5 -Compress
 [pscustomobject]@{sha256=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($json))).ToLowerInvariant();entries=$sorted}
}
function Get-TwoUserProcessFact([Diagnostics.Process]$Process,[string]$ExpectedSid) {
 [void]$Process.Handle
 if($Process.HasExited){throw 'Tracked process exited'}
 $row=Get-CimInstance Win32_Process -Filter ('ProcessId='+$Process.Id) -ErrorAction Stop
 $owner=Invoke-CimMethod -InputObject $row -MethodName GetOwnerSid -ErrorAction Stop
 if($owner.ReturnValue -ne 0 -or $owner.Sid -cne $ExpectedSid){throw 'Tracked SID mismatch'}
 $ticks=$Process.StartTime.ToUniversalTime().Ticks;$delta=$ticks-$row.CreationDate.ToUniversalTime().Ticks
 if($delta -lt 0 -or $delta -gt 9 -or $Process.Path -ine $row.ExecutablePath){throw 'Tracked generation/image mismatch'}
 [pscustomobject]@{pid=$Process.Id;sid=$owner.Sid;path=$Process.Path;creationTicks=[string]$ticks;sessionId=$row.SessionId}
}
function Assert-TwoUserLive($Fact) {
 $p=Get-Process -Id $Fact.pid -ErrorAction Stop
 try{$now=Get-TwoUserProcessFact $p $Fact.sid
  if($now.creationTicks -cne $Fact.creationTicks -or $now.path -ine $Fact.path){throw 'PID reuse/image change'}
  $now
 }finally{$p.Dispose()}
}
function Add-TwoUserDescendants($Root,[Collections.Generic.Dictionary[int,object]]$Ledger) {
 $null=Assert-TwoUserLive $Root;$snapshot=@(Get-CimInstance Win32_Process)
 $parents=[Collections.Generic.HashSet[int]]::new();[void]$parents.Add([int]$Root.pid);$Ledger[[int]$Root.pid]=$Root
 do{$changed=$false;foreach($row in $snapshot){
  if($parents.Contains([int]$row.ParentProcessId) -and !$parents.Contains([int]$row.ProcessId) -and $row.CreationDate.ToUniversalTime().Ticks -ge ([long]$Root.creationTicks-9)){
   try{$p=Get-Process -Id $row.ProcessId -ErrorAction Stop
    try{$fact=Get-TwoUserProcessFact $p $Root.sid}finally{$p.Dispose()}
    $Ledger[[int]$fact.pid]=$fact;[void]$parents.Add([int]$fact.pid);$changed=$true
   }catch{if(Get-Process -Id $row.ProcessId -ErrorAction SilentlyContinue){throw 'Descendant ownership unproven'}}
  }
 }}while($changed)
}
function Stop-TwoUserTracked($Fact) {
 $p=Get-Process -Id $Fact.pid -ErrorAction SilentlyContinue
 if(!$p){return}
 try{$now=Get-TwoUserProcessFact $p $Fact.sid
  if($now.creationTicks -cne $Fact.creationTicks -or $now.path -ine $Fact.path){throw 'Refusing changed PID/SID/image/creation cleanup'}
  $p.Kill();if(!$p.WaitForExit(10000)){throw 'Tracked child did not exit'}
 }finally{$p.Dispose()}
}
function Initialize-TwoUserWindowProbe {
 if(!('OcxTwoUserWindow' -as [type])){Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class OcxTwoUserWindow {
 [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr window);
 [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window,out uint pid);
 [DllImport("kernel32.dll",SetLastError=true)] public static extern bool GetNamedPipeServerProcessId(IntPtr pipe,out uint pid);
 public static bool OwnedVisible(IntPtr window,int expected){uint pid;return window!=IntPtr.Zero&&IsWindowVisible(window)&&GetWindowThreadProcessId(window,out pid)!=0&&pid==expected;}
}
'@}
}
function Get-TwoUserWindow($Fact) {
 Initialize-TwoUserWindowProbe;$null=Assert-TwoUserLive $Fact;$p=Get-Process -Id $Fact.pid
 try{$p.Refresh();$handle=$p.MainWindowHandle
  if(![OcxTwoUserWindow]::OwnedVisible($handle,$p.Id)){throw 'Owned visible native window unavailable'}
  [ordered]@{pid=$p.Id;handle=[string]$handle.ToInt64();ownedVisible=$true;observedUtc=[datetime]::UtcNow.ToString('o')}
 }finally{$p.Dispose()}
}
function Get-TwoUserPipeName([string]$Sid) {
 if($Sid -notmatch '^S-1-5-21-\d+-\d+-\d+-\d+$'){throw 'Real local SID required'}
 $bytes=[Text.Encoding]::UTF8.GetBytes('windows-user-sid-v2'+[char]0+$Sid)
 'OpenCodeXDesktop-'+[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)[0..11]).ToLowerInvariant()
}
function Invoke-TwoUserCli([string]$Cli,[string[]]$Arguments) {
 $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$Cli;$info.UseShellExecute=$false
 $info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
 foreach($arg in $Arguments){$info.ArgumentList.Add($arg)}
 $p=[Diagnostics.Process]::Start($info)
 try{$stdout=$p.StandardOutput.ReadToEndAsync();$stderr=$p.StandardError.ReadToEndAsync()
  if(!$p.WaitForExit(20000)){
   Stop-TwoUserTracked (Get-TwoUserProcessFact $p ([Security.Principal.WindowsIdentity]::GetCurrent().User.Value));throw 'CLI deadline'
  }
  if($p.ExitCode -ne 0){throw 'Candidate CLI did not confirm same-user status'}
  $json=$stdout.GetAwaiter().GetResult()|ConvertFrom-Json
  if($json.ok -ne $true -or $json.schema_version -ne 1){throw 'Actual CLI JSON failed'}
  $json
 }finally{$p.Dispose()}
}
function Write-TwoUserJson([string]$Path,$Value) {
 $temp=$Path+'.writing-'+[guid]::NewGuid().ToString('N')
 $Value|ConvertTo-Json -Depth 15|Set-Content -LiteralPath $temp -Encoding utf8
 Move-Item -LiteralPath $temp -Destination $Path
}
function Get-TwoUserDriverText {
@'
import assert from 'node:assert/strict';
import {writeFile} from 'node:fs/promises';
import {join} from 'node:path';
import {discoverMainTarget} from './windows-cdp.mjs';
const [endpoint,output]=process.argv.slice(2);
let ws;let seq=0;const pending=new Map();
function call(method,params={}){const id=++seq;return new Promise((resolve,reject)=>{const timer=setTimeout(()=>{pending.delete(id);reject(Error('CDP deadline'));},10000);pending.set(id,{resolve:v=>{clearTimeout(timer);resolve(v);},reject});ws.send(JSON.stringify({id,method,params}));});}
try{
 const target=await discoverMainTarget(endpoint,{timeoutMs:60000});
 const url=new URL(target.webSocketDebuggerUrl);assert(['127.0.0.1','localhost','[::1]'].includes(url.hostname));assert.equal(url.port,new URL(endpoint).port);
 ws=new WebSocket(target.webSocketDebuggerUrl);await new Promise((r,j)=>{ws.addEventListener('open',r,{once:true});ws.addEventListener('error',()=>j(Error('CDP unavailable')),{once:true});setTimeout(()=>j(Error('CDP open deadline')),10000).unref();});
 ws.addEventListener('message',e=>{const v=JSON.parse(e.data);const p=pending.get(v.id);if(p){pending.delete(v.id);v.error?p.reject(Error('CDP rejected')):p.resolve(v.result);}});
 let facts;
 for(let i=0;i<120;i++){
  const r=await call('Runtime.evaluate',{expression:'(async()=>{if(!window.__TAURI_INTERNALS__?.invoke||!document.querySelector("#app")?.children.length)return null;const i=window.__TAURI_INTERNALS__.invoke;return {mounted:true,textLength:document.body.innerText.length,about:await i("app_about"),paths:await i("model_paths_v2_snapshot"),config:await i("get_data_root_config"),preferences:await i("get_preferences"),runtime:await i("runtime_source")};})()',awaitPromise:true,returnByValue:true});
  if(!r.exceptionDetails&&r.result?.value?.mounted){facts=r.result.value;if(facts.paths.program.consistency==='consistent')break;}
  await new Promise(r=>setTimeout(r,1000));
 }
 assert(facts?.mounted&&facts.textLength>30,'Actual first screen unavailable');assert.equal(facts.about.platform,'Windows');
 const image=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,'first-screen.png'),Buffer.from(image.data,'base64'));
 await writeFile(join(output,'first-screen.json'),JSON.stringify({mounted:true,textLength:facts.textLength,about:facts.about,paths:facts.paths,config:facts.config,cliEnabled:facts.preferences.cliEnabled,runtime:facts.runtime},null,2)+'\n');
 assert.equal(facts.paths.program.consistency,'consistent','NotReady: staged runtime unverified');
 assert.equal(facts.paths.runtimeEntry.status,'valid');assert.equal(facts.paths.runtimePrefix.status,'valid');
}finally{ws?.close();}
'@
}
function Invoke-TwoUserChild([string]$InputFile) {
 Assert-TwoUserChildGuard
 $c=Get-Content -LiteralPath $InputFile -Raw|ConvertFrom-Json
 . (Join-Path $c.code 'windows-process-token.ps1')
 $sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
 if($sid -cne $c.accountSid){throw 'Actual child SID differs'}
 if($c.ephemeralGuard.passed -ne $true -or $c.ephemeralGuard.githubActions -cne 'true'){throw 'Controller ephemeralGuard missing'}
 $principal=[Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
 if($principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Temporary account is administrator'}
 $nativeProfile=Assert-TwoUserLocalPath $env:USERPROFILE
 if($nativeProfile -ine $c.profile -or (Split-Path -Leaf $nativeProfile) -notlike 'ocxacc*'){throw 'Registered temporary Profile differs'}
 foreach($key in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_WINDOWS_SMOKE_CDP_PORT','HOME','OPENCODEX_HOME','CODEX_HOME','WEBVIEW2_USER_DATA_FOLDER','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS')){[Environment]::SetEnvironmentVariable($key,$null,'Process')}
 $out=$c.evidence;$code=$c.code;$installed=Join-Path $env:LOCALAPPDATA 'Programs/OpenCodeX Desktop'
 $null=Assert-TwoUserUnder $nativeProfile $installed
 if(Test-Path -LiteralPath $installed){throw 'Fresh per-user installation required'}
 New-Item -ItemType Directory -Path $installed|Out-Null
 $original=Get-TwoUserTreeHash $c.candidateDirectory
 foreach($entry in Get-ChildItem -LiteralPath $c.candidateDirectory -Force){Copy-Item -LiteralPath $entry.FullName -Destination $installed -Recurse}
 $installationOwner=(Get-Acl -LiteralPath $installed).GetOwner([Security.Principal.SecurityIdentifier]).Value
 if($installationOwner -cne $sid){throw 'Private program copy owner differs from real account'}
 $copy=Get-TwoUserTreeHash $installed
 if($copy.sha256 -cne $original.sha256){throw 'Full private installation copy differs'}
 Write-TwoUserJson (Join-Path $out 'installation-copy.json') $copy
 $exe=Join-Path $installed $c.executableName;$cli=Join-Path $installed 'ocxd.exe'
 $root=Join-Path $nativeProfile '.codex-desktop/desktop';$nativeHome=Join-Path $nativeProfile '.codex-desktop/opencodex/home'
 $ledger=[Collections.Generic.Dictionary[int,object]]::new();$manager=$null
 $result=[ordered]@{ephemeralGuard=$c.ephemeralGuard;sid=$sid;profile=$nativeProfile;processHome=$env:USERPROFILE;localAppData=$env:LOCALAPPDATA;desktopRoot=$root;opencodexHome=$nativeHome;installation=$installed;result='fail';runtimeProxyVerification='not-verified';sandbox=$false}
 $start={
  $listener=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0);$listener.Start();$port=$listener.LocalEndpoint.Port;$listener.Stop()
  $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$exe;$info.WorkingDirectory=$installed;$info.UseShellExecute=$false
  # Standard WebView2 diagnostics only, NOT the sandbox-only product smoke hook.
  $info.Environment['WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS']="--remote-debugging-address=127.0.0.1 --remote-debugging-port=$port"
  $p=[Diagnostics.Process]::Start($info);$fact=Get-TwoUserProcessFact $p $sid;$ledger[[int]$fact.pid]=$fact
  [pscustomobject]@{process=$p;fact=$fact;port=$port}
 }
 $screen={param($app,[string]$phase)
  $evidence=Join-Path $out $phase;New-Item -ItemType Directory -Path $evidence|Out-Null
  $cdpDeadline=[datetime]::UtcNow.AddSeconds(40);$listeners=@()
  do{Add-TwoUserDescendants $app.fact $ledger
   $listeners=@(Get-NetTCPConnection -State Listen -LocalPort $app.port -ErrorAction SilentlyContinue)
   if(!$listeners.Count){Start-Sleep -Milliseconds 250}
  }while(!$listeners.Count -and [datetime]::UtcNow -lt $cdpDeadline)
  if(!$listeners.Count -or @($listeners|Where-Object {!$ledger.ContainsKey([int]$_.OwningProcess) -or $_.LocalAddress -notin @('127.0.0.1','::1')}).Count){throw 'NotReady: CDP listener is not owned by this Manager tree'}
  foreach($listenerFact in $listeners){$null=Assert-TwoUserLive $ledger[[int]$listenerFact.OwningProcess]}
  $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$c.node;$info.UseShellExecute=$false;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
  foreach($a in @((Join-Path $code 'two-user-driver.mjs'),"http://127.0.0.1:$($app.port)",$evidence)){$info.ArgumentList.Add($a)}
  $driver=[Diagnostics.Process]::Start($info);$driverFact=Get-TwoUserProcessFact $driver $sid;$ledger[[int]$driverFact.pid]=$driverFact
  $stderr=$driver.StandardError.ReadToEndAsync();$stdout=$driver.StandardOutput.ReadToEndAsync();$deadline=[datetime]::UtcNow.AddSeconds(200)
  while(!$driver.HasExited -and [datetime]::UtcNow -lt $deadline){if(Test-Path -LiteralPath (Join-Path $out 'stop')){throw 'Controller requested stop'};Add-TwoUserDescendants $app.fact $ledger;Start-Sleep -Milliseconds 300}
  if(!$driver.HasExited){throw 'NotReady: first screen deadline'}
  $stderr.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $evidence 'driver-stderr.log') -Encoding utf8
  if($driver.ExitCode -ne 0){throw 'NotReady: first screen/complete staged resources unavailable'}
  $driver.Dispose();$facts=Get-Content -LiteralPath (Join-Path $evidence 'first-screen.json') -Raw|ConvertFrom-Json
  $token=Get-WindowsProcessToken -Process $app.process -ExpectedExecutable $exe
  Assert-WindowsProcessMode -Observation $token -Mode ordinary
  $window=Get-TwoUserWindow $app.fact
  if(!(Test-TwoUserSamePath $facts.config.currentDataRoot $root) -or !(Test-TwoUserSamePath $facts.config.currentOpencodexHome $nativeHome)){throw 'Actual D/H escaped Profile'}
  if(!(Test-TwoUserSamePath $facts.paths.desktopRoot.path $root) -or !(Test-TwoUserSamePath $facts.paths.opencodexHome.path $nativeHome)){throw 'Snapshot D/H mismatch'}
  if(!(Test-TwoUserSamePath $facts.paths.installationRoot.path $installed) -or !(Test-TwoUserSamePath $facts.paths.managerExecutable.path $exe)){throw 'Product installation context escaped private copy'}
  $null=Assert-TwoUserUnder $installed (Assert-TwoUserLocalPath (($facts.paths.runtimePrefix.path -replace '^\\\\\?\\','')))
  if($facts.runtime.kind -cne 'managed' -or !(Test-TwoUserSamePath $facts.runtime.path $facts.paths.runtimeEntry.path)){throw 'NotReady: actual runtime source is not the own managed entry'}
  $appLog=Join-Path $root 'logs/app.log'
  if(!(Test-Path -LiteralPath $appLog) -or !(Get-Content -LiteralPath $appLog -Raw).Contains('commit='+$c.commit)){throw 'NotReady: EXE source commit provenance missing'}
  [pscustomobject]@{facts=$facts;window=$window;directory=$evidence;token=$token}
 }
 try{
  # Let the real product initialize structure2 before editing isolated CLI prefs.
  $manager=& $start;$initial=& $screen $manager 'initial'
  if($initial.facts.cliEnabled -ne $false){throw 'Default CLI was enabled'}
  Add-TwoUserDescendants $manager.fact $ledger
  foreach($fact in @($ledger.Values|Sort-Object {[long]$_.creationTicks} -Descending)){Stop-TwoUserTracked $fact};$ledger.Clear();$manager.process.Dispose()
  $prefs=Join-Path $root 'manager-state/preferences.json'
  $document=if(Test-Path -LiteralPath $prefs){Get-Content -LiteralPath $prefs -Raw|ConvertFrom-Json -AsHashtable}else{@{schema_version=2}}
  $document.cli=@{cli_enabled=$true};$document.shell=@{auto_panel=$false;launch_main=$true;launch_with_codex=$false};$document.updates=@{app_update_auto_check=$false}
  $document|ConvertTo-Json -Depth 12|Set-Content -LiteralPath $prefs -Encoding utf8
  $manager=& $start;$live=& $screen $manager 'live'
  # Windows product UI deliberately keeps CLI disabled. This isolated raw
  # preference enables only the backend pipe fixture; the actual ocxd calls
  # below prove it is effective without claiming a supported UI CLI toggle.
  if($live.facts.cliEnabled -ne $false){throw 'Windows UI CLI capability boundary changed'}
  $pointer=Join-Path $root 'manager-location.json';$metadata=Join-Path $root 'manager-state/data-root.json'
  $location=Get-Content -LiteralPath $pointer -Raw|ConvertFrom-Json;$meta=Get-Content -LiteralPath $metadata -Raw|ConvertFrom-Json
  if($location.schemaVersion -ne 1 -or !(Test-TwoUserSamePath $location.activeRoot (Join-Path $nativeProfile '.codex-desktop')) -or $meta.structure_version -cne '2'){throw 'Actual fixed pointer/structure2 missing'}
  $same=Invoke-TwoUserCli $cli @('status','--json');Write-TwoUserJson (Join-Path $out 'same-user-status.json') $same
  $data=Invoke-TwoUserCli $cli @('data-root','show','--json')
  if(!(Test-TwoUserSamePath $data.data.active_data_root $root) -or !(Test-TwoUserSamePath $data.data.opencodex_home $nativeHome)){throw 'Candidate CLI uses another D/H'}
  Write-TwoUserJson (Join-Path $out 'same-user-data-root.json') $data
  $pipe=Get-TwoUserPipeName $sid;$own=[IO.Pipes.NamedPipeClientStream]::new('.',$pipe,[IO.Pipes.PipeDirection]::InOut)
  try{$own.Connect(5000);[uint32]$serverId=0
   if(![OcxTwoUserWindow]::GetNamedPipeServerProcessId($own.SafePipeHandle.DangerousGetHandle(),[ref]$serverId) -or $serverId -ne $manager.fact.pid){throw 'Actual product pipe server PID differs'}
  }finally{$own.Dispose()}
  Add-Type -AssemblyName System.Security.Cryptography.ProtectedData
  $plain=[Security.Cryptography.RandomNumberGenerator]::GetBytes(32)
  try{
   $cipher=[Security.Cryptography.ProtectedData]::Protect($plain,$null,[Security.Cryptography.DataProtectionScope]::CurrentUser)
   $ownPlain=[Security.Cryptography.ProtectedData]::Unprotect($cipher,$null,[Security.Cryptography.DataProtectionScope]::CurrentUser)
   if([Convert]::ToBase64String($plain) -cne [Convert]::ToBase64String($ownPlain)){throw 'Own DPAPI readback failed'}
   [Array]::Clear($ownPlain,0,$ownPlain.Length);[IO.File]::WriteAllBytes((Join-Path $out 'synthetic-dpapi.bin'),$cipher)
  }finally{[Array]::Clear($plain,0,$plain.Length)}
  $result.manager=$manager.fact;$result.window=$live.window;$result.applicationToken=$live.token;$result.pipe=$pipe;$result.defaultCliDisabled=$true;$result.sameUserCli=$true
  Write-TwoUserJson (Join-Path $out 'ready.json') $result
  $peerFile=Join-Path $out 'peer.json';$deadline=[datetime]::UtcNow.AddSeconds(720)
  while(!(Test-Path -LiteralPath $peerFile)){
   if(Test-Path -LiteralPath (Join-Path $out 'stop')){throw 'Peer startup incomplete'}
   if([datetime]::UtcNow -gt $deadline){throw 'Peer deadline'}
   Add-TwoUserDescendants $manager.fact $ledger;Start-Sleep -Milliseconds 250
  }
  $peer=Get-Content -LiteralPath $peerFile -Raw|ConvertFrom-Json
  if($peer.sid -ceq $sid -or $peer.pipe -cne (Get-TwoUserPipeName $peer.sid)){throw 'Peer identity differs'}
  $peerDataDenied=$false
  try{$peerFileHandle=[IO.File]::Open($peer.privateFile,'Open','Read','ReadWrite');$peerFileHandle.Dispose()}catch [UnauthorizedAccessException]{$peerDataDenied=$true}
  if(!$peerDataDenied){throw 'Other account actual business data was readable'}
  $result.peerBusinessData='access-denied'
  $peerStream=[IO.Pipes.NamedPipeClientStream]::new('.',$peer.pipe,[IO.Pipes.PipeDirection]::InOut);$denied=$false
  try{$peerStream.Connect(3000)}catch [UnauthorizedAccessException]{$denied=$true}
  catch [IO.IOException]{if(($_.Exception.HResult -band 0xffff) -eq 5){$denied=$true}else{throw 'NotReady: peer pipe denial unproven'}}
  finally{$peerStream.Dispose()}
  if(!$denied){throw 'Other SID pipe accepted or denial unproven'}
  $result.crossPipe='access-denied-real-token';$result.crossPipeServer=$peer.manager.pid
  $dpapiDenied=$false;$peerBytes=[IO.File]::ReadAllBytes($peer.cipher)
  try{$unexpected=[Security.Cryptography.ProtectedData]::Unprotect($peerBytes,$null,[Security.Cryptography.DataProtectionScope]::CurrentUser);[Array]::Clear($unexpected,0,$unexpected.Length)}catch [Security.Cryptography.CryptographicException]{$dpapiDenied=$true}
  if(!$dpapiDenied){throw 'Other SID synthetic DPAPI accepted'}
  $result.dpapi='own-roundtrip-and-peer-denied-synthetic-subscope-only'
  $before=@{};foreach($f in @($pointer,$metadata)){$before[$f]=(Get-FileHash -LiteralPath $f).Hash}
  $lockFile=Join-Path $root 'manager-state/app.lock';$lockBefore=Get-Item -LiteralPath $lockFile
  $duplicateInfo=[Diagnostics.ProcessStartInfo]::new();$duplicateInfo.FileName=$exe;$duplicateInfo.WorkingDirectory=$installed;$duplicateInfo.UseShellExecute=$false
  $duplicateProcess=[Diagnostics.Process]::Start($duplicateInfo)
  try{[void]$duplicateProcess.Handle
   if(!$duplicateProcess.WaitForExit(20000)){$dupFact=Get-TwoUserProcessFact $duplicateProcess $sid;$ledger[[int]$dupFact.pid]=$dupFact;throw 'Same-SID second Manager remained alive'}
   if($duplicateProcess.ExitCode -eq 0){throw 'Second Manager did not report rejection'}
   $duplicatePid=$duplicateProcess.Id;$duplicateExit=$duplicateProcess.ExitCode
  }finally{$duplicateProcess.Dispose()}
  $null=Assert-TwoUserLive $manager.fact;$after=Get-TwoUserWindow $manager.fact
  if($after.handle -cne $live.window.handle){throw 'Original window changed'}
  foreach($f in @($pointer,$metadata)){if((Get-FileHash -LiteralPath $f).Hash -cne $before[$f]){throw 'Duplicate changed pointer/metadata'}}
  $lockAfter=Get-Item -LiteralPath $lockFile
  if($lockBefore.Length -ne $lockAfter.Length -or $lockBefore.LastWriteTimeUtc -ne $lockAfter.LastWriteTimeUtc){throw 'Duplicate changed lock metadata'}
  $result.duplicate=[ordered]@{rejected=$true;secondPid=$duplicatePid;secondExitCode=$duplicateExit;originalPid=$manager.fact.pid;originalWindowUnchanged=$true;pointerMetadataHashUnchanged=$true;lockSizeAndWriteTimeUnchanged=$true;lockContentHash='unavailable-while-byte-range-lock-held'}
  $result.result='pass-user-isolation';Write-TwoUserJson (Join-Path $out 'checked.json') $result
  while(!(Test-Path -LiteralPath (Join-Path $out 'stop'))){if([datetime]::UtcNow -gt $deadline){throw 'Controller stop deadline'};Add-TwoUserDescendants $manager.fact $ledger;Start-Sleep -Milliseconds 250}
 }catch{
  $result.failureKind=$_.Exception.GetType().FullName;$result.failureLine=$_.InvocationInfo.ScriptLineNumber
  $result.failureCode=if($_.Exception.Message.StartsWith('NotReady:')){$_.Exception.Message}else{'Native identity/UI/IPC/cleanup assertion failed; inspect stage and retained evidence'}
  $result.result=if($_.Exception.Message.StartsWith('NotReady:')){'not-ready'}else{'fail'}
  Write-TwoUserJson (Join-Path $out 'failure.json') $result
 }finally{
  if($manager){try{Add-TwoUserDescendants $manager.fact $ledger}catch{}}
  $cleanup=@();foreach($fact in @($ledger.Values|Sort-Object {[long]$_.creationTicks} -Descending)){
   try{Stop-TwoUserTracked $fact;$cleanup+=@{process=$fact;result='stopped-or-exited'}}catch{$cleanup+=@{process=$fact;result='retained-unproven'};$result.result='cleanup-failed'}
  }
  $result.cleanup=$cleanup
  $logDirectory=Join-Path $root 'logs'
  if(Test-Path -LiteralPath $logDirectory){
   $retainedLogs=Join-Path $out 'application-logs';New-Item -ItemType Directory -Path $retainedLogs|Out-Null
   foreach($file in @(Get-ChildItem -LiteralPath $logDirectory -File -Filter '*.log'|Select-Object -First 32)){
    if($file.Length -le 8MB -and !($file.Attributes -band [IO.FileAttributes]::ReparsePoint)){Copy-Item -LiteralPath $file.FullName -Destination (Join-Path $retainedLogs $file.Name)}
   }
  }
  try{$result.finalInventory=Get-TwoUserTreeHash (Join-Path $nativeProfile '.codex-desktop');$result.inventory='hashed'}catch{$result.inventory='unavailable';if($result.result -ceq 'pass-user-isolation'){$result.result='evidence-incomplete'}}
  Write-TwoUserJson (Join-Path $out 'finished.json') $result
  if($manager){$manager.process.Dispose()}
 }
 if($result.result -cne 'pass-user-isolation'){exit 1};exit 0
}
if($Mode -ceq 'Child'){Invoke-TwoUserChild $ConfigPath}

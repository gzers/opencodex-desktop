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
function Read-TwoUserRuntimePolicy([string]$Path) {
 # npm lockfiles intentionally use packages[''] for the root package. A
 # PSCustomObject cannot represent that legal key; preserve the complete map.
 $policy=Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json -AsHashtable
 if($policy.schemaVersion -ne 1 -or $policy.node.version -notmatch '^\d+\.\d+\.\d+$'){throw 'Source-pinned runtime policy is invalid'}
 $policy
}
function Test-TwoUserLocalSpelling([string]$Path) {
 if([string]::IsNullOrWhiteSpace($Path)){return $false}
 # Normalize separators for the lexical test only. In a .NET regex [\/]
 # escapes slash and does not accept a Windows backslash. Never use OS
 # GetFullPath to admit relative/device/traversal input before this check.
 $spelling=$Path.Replace([char]92,[char]47)
 if($spelling -cnotmatch '^[A-Za-z]:/' -or $spelling.Substring(2).Contains(':') -or $spelling -match '[\x00-\x1f\x7f]'){return $false}
 foreach($part in $spelling.Substring(3).Split('/')){if($part -in @('.','..')){return $false}}
 return $spelling.TrimEnd('/').Length -gt 2
}
function Assert-TwoUserLocalPath([string]$Path) {
 if(!(Test-TwoUserLocalSpelling $Path)){throw 'Local absolute non-device path required'}
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
function Save-TwoUserProcessHandle($Process,[string]$CreationTicks) {
 if(!('OcxRetainedProcess' -as [type])){Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;
public sealed class OcxRetainedProcess : IDisposable {
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool DuplicateHandle(IntPtr sourceProcess,IntPtr source,IntPtr targetProcess,out SafeWaitHandle duplicate,uint access,bool inherit,uint options);
 [DllImport("kernel32.dll",SetLastError=true)] static extern uint WaitForSingleObject(SafeWaitHandle handle,uint milliseconds);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetExitCodeProcess(SafeWaitHandle handle,out uint code);
 SafeWaitHandle handle;
 public OcxRetainedProcess(IntPtr source){if(!DuplicateHandle(new IntPtr(-1),source,new IntPtr(-1),out handle,0,false,2))throw new Win32Exception();}
 public bool Exited(){uint result=WaitForSingleObject(handle,0);if(result==0)return true;if(result==258)return false;throw new Win32Exception();}
 public uint ExitCode(){if(!Exited())throw new InvalidOperationException("Original process is still live");uint code;if(!GetExitCodeProcess(handle,out code))throw new Win32Exception();return code;}
 public void Dispose(){handle.Dispose();}
}
'@}
 if(!$script:twoUserRetainedHandles){$script:twoUserRetainedHandles=@{}}
 $key=[string]$Process.Id+'/'+$CreationTicks
 if(!$script:twoUserRetainedHandles.ContainsKey($key)){$script:twoUserRetainedHandles[$key]=[OcxRetainedProcess]::new($Process.Handle)}
}
function Test-TwoUserOriginalExited($Fact) {
 $key=[string]$Fact.pid+'/'+[string]$Fact.creationTicks
 if($script:twoUserRetainedHandles -and $script:twoUserRetainedHandles.ContainsKey($key)){return $script:twoUserRetainedHandles[$key].Exited()}
 return $false
}
function Get-TwoUserOriginalExit($Fact) {
 $key=[string]$Fact.pid+'/'+[string]$Fact.creationTicks
 if(!$script:twoUserRetainedHandles -or !$script:twoUserRetainedHandles.ContainsKey($key)){throw 'Original retained handle unavailable; do not infer exit code'}
 $original=$script:twoUserRetainedHandles[$key];$exited=$original.Exited()
 [pscustomobject]@{originalHandleExited=$exited;exitCode=$(if($exited){$original.ExitCode()}else{$null})}
}
function Close-TwoUserProcessHandles {
 if($script:twoUserRetainedHandles){foreach($handle in $script:twoUserRetainedHandles.Values){$handle.Dispose()};$script:twoUserRetainedHandles.Clear()}
}
function Get-TwoUserProcessObservation([Diagnostics.Process]$Process) {
 if(!('OcxNativeProcessIdentity' -as [type])){Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;
public sealed class OcxNativeProcessIdentity {
 public uint ProcessId { get; set; }
 public string Executable { get; set; }
 public DateTime CreationTimeUtc { get; set; }
 [DllImport("kernel32.dll",SetLastError=true)] static extern uint GetProcessId(IntPtr process);
 [DllImport("kernel32.dll",SetLastError=true)] static extern uint WaitForSingleObject(IntPtr process,uint milliseconds);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,ExactSpelling=true,SetLastError=true)] static extern bool QueryFullProcessImageNameW(IntPtr process,uint flags,StringBuilder name,ref uint size);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetProcessTimes(IntPtr process,out long created,out long exited,out long kernel,out long user);
 static void RequireLive(IntPtr handle) {
  uint result=WaitForSingleObject(handle,0);
  if(result==0xffffffff)throw new Win32Exception(Marshal.GetLastWin32Error());
  if(result!=258)throw new InvalidOperationException("Target process exited");
 }
 public static OcxNativeProcessIdentity Observe(Process process) {
  SafeProcessHandle retained=process.SafeHandle;bool referenced=false;
  try {
   retained.DangerousAddRef(ref referenced);IntPtr handle=retained.DangerousGetHandle();RequireLive(handle);
   uint pid=GetProcessId(handle);if(pid==0)throw new Win32Exception(Marshal.GetLastWin32Error());
   var name=new StringBuilder(32768);uint length=(uint)name.Capacity;
   if(!QueryFullProcessImageNameW(handle,0,name,ref length))throw new Win32Exception(Marshal.GetLastWin32Error());
   long created,exited,kernel,user;
   if(!GetProcessTimes(handle,out created,out exited,out kernel,out user))throw new Win32Exception(Marshal.GetLastWin32Error());
   RequireLive(handle);
   return new OcxNativeProcessIdentity{ProcessId=pid,Executable=name.ToString(),CreationTimeUtc=DateTime.FromFileTimeUtc(created)};
  } finally {if(referenced)retained.DangerousRelease();}
 }
}
'@}
 # Process ownership is still independently checked through GetOwnerSid below.
 # Only explicit Manager admission reads elevation. A renderer's restricted
 # token need not grant token-query access to prove its handle/image/generation.
 $observed=[OcxNativeProcessIdentity]::Observe($Process)
 if($observed.ProcessId -ne $Process.Id -or [string]::IsNullOrWhiteSpace($observed.Executable)){throw 'Retained native PID/image unavailable'}
 [ordered]@{status='observed';processId=$observed.ProcessId;executable=$observed.Executable;creationTimeUtc=$observed.CreationTimeUtc.ToString('o')}
}
function Assert-TwoUserNativeIdentity($Observed,[int]$ExpectedPid,[string]$ExpectedPath,[string]$ExpectedTicks) {
 if($Observed.status -cne 'observed' -or $Observed.processId -ne $ExpectedPid -or $Observed.executable -ine $ExpectedPath){throw 'Retained native PID/image mismatch'}
 $created=[datetime]::Parse($Observed.creationTimeUtc,[Globalization.CultureInfo]::InvariantCulture,[Globalization.DateTimeStyles]::RoundtripKind).ToUniversalTime().Ticks
 if([string]$created -cne $ExpectedTicks){throw 'Retained native creation generation mismatch'}
}
function Assert-TwoUserCimIdentity($Row,$Owner,$Native,[string]$ExpectedSid) {
 if(!$Row -or !$Owner -or $Row.ProcessId -ne $Native.processId -or $Owner.ReturnValue -ne 0 -or $Owner.Sid -cne $ExpectedSid){throw 'Tracked SID/PID mismatch'}
 # CIM can omit ExecutablePath for an ordinary user's sandboxed child.
 # It has no image authority when missing. The SAME retained native handle is
 # still required to report exact PID/image/FILETIME both before and after CIM.
 if(![string]::IsNullOrWhiteSpace($Row.ExecutablePath) -and $Native.executable -ine $Row.ExecutablePath){throw 'Tracked supplied-CIM/native image mismatch'}
}
function Get-TwoUserProcessFact([Diagnostics.Process]$Process,[string]$ExpectedSid) {
 [void]$Process.Handle
 if($Process.HasExited){throw 'Tracked process exited'}
 $before=Get-TwoUserProcessObservation $Process
 $ticks=[string]$Process.StartTime.ToUniversalTime().Ticks;$image=$before.executable
 # Native observation brackets CIM ownership with the SAME retained process
 # handle and exact FILETIME. CIM's microsecond CreationDate is not the process
 # generation authority. No timestamp tolerance or PID-only fallback is used.
 Assert-TwoUserNativeIdentity $before $Process.Id $image $ticks
 $row=Get-CimInstance Win32_Process -Filter ('ProcessId='+$Process.Id) -ErrorAction Stop
 $owner=Invoke-CimMethod -InputObject $row -MethodName GetOwnerSid -ErrorAction Stop
 $after=Get-TwoUserProcessObservation $Process
 Assert-TwoUserNativeIdentity $after $Process.Id $image $ticks
 Assert-TwoUserCimIdentity $row $owner $after $ExpectedSid
 Save-TwoUserProcessHandle $Process $ticks
 [pscustomobject]@{pid=$Process.Id;sid=$owner.Sid;path=$image;creationTicks=$ticks;sessionId=$row.SessionId;generationSource='retained-handle-native-filetime';cimImageSupplied=(![string]::IsNullOrWhiteSpace($row.ExecutablePath))}
}
function Assert-TwoUserLive($Fact) {
 $p=Get-Process -Id $Fact.pid -ErrorAction Stop
 try{$now=Get-TwoUserProcessFact $p $Fact.sid
  if($now.creationTicks -cne $Fact.creationTicks -or $now.path -ine $Fact.path){throw 'PID reuse/image change'}
  $now
 }finally{$p.Dispose()}
}
function Assert-TwoUserRootForDescendantObservation($Root) {
 # Original native handle exit is a positive OS fact. The Process object may
 # finish between a parent HasExited check and this call; do not re-admit an
 # exited root by PID or convert a genuine identity failure into absence.
 if(Test-TwoUserOriginalExited $Root){return 'original-exited'}
 try{$null=Assert-TwoUserLive $Root;return 'original-live'}
 catch{if(Test-TwoUserOriginalExited $Root){return 'original-exited'};throw}
}
function Add-TwoUserDescendants($Root,[Collections.Generic.Dictionary[int,object]]$Ledger) {
 $rootObservation=Assert-TwoUserRootForDescendantObservation $Root
 # The original native handle remains retained, including after exit. Continue
 # the same parent-chain snapshot and exact SID/image/creation checks for every
 # discovered descendant; an unreadable live descendant still refuses cleanup.
 $snapshot=@(Get-CimInstance Win32_Process -ErrorAction Stop)
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
 if(Test-TwoUserOriginalExited $Fact){return}
 $p=Get-Process -Id $Fact.pid -ErrorAction SilentlyContinue
 if(!$p){return}
 try{
  # A retained Process object can observe termination between enumeration and
  # identity query. An exited object needs no kill; a still-live unknown object
  # remains a refusal, including a reused PID or another SID.
  if($p.HasExited){return}
  try{$now=Get-TwoUserProcessFact $p $Fact.sid}catch{if($p.HasExited -or (Test-TwoUserOriginalExited $Fact)){return};throw}
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
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] public static extern bool WaitNamedPipe(string name,uint timeout);
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
 $budget=[Diagnostics.Stopwatch]::StartNew()
 if($c.diagnosticSweep){Wait-TwoUserDiagnosticPipe $manager.fact (Get-TwoUserPipeName $sid) 20000}
 $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$Cli;$info.UseShellExecute=$false
 $info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
 foreach($arg in $Arguments){$info.ArgumentList.Add($arg)}
 $p=[Diagnostics.Process]::Start($info)
 try{$stdout=$p.StandardOutput.ReadToEndAsync();$stderr=$p.StandardError.ReadToEndAsync()
  $remaining=[Math]::Max(1,20000-[int]$budget.ElapsedMilliseconds)
  if(!$p.WaitForExit($remaining)){
   Stop-TwoUserTracked (Get-TwoUserProcessFact $p ([Security.Principal.WindowsIdentity]::GetCurrent().User.Value));throw 'CLI deadline'
  }
  if($c.diagnosticSweep){
   $commandName=($Arguments -join '-').Replace('/','_').Replace([string][char]92,'_')
   Write-TwoUserJson (Join-Path $out ('cli-'+$commandName+'.json')) @{exitCode=$p.ExitCode;stdout=$stdout.GetAwaiter().GetResult();stderr=$stderr.GetAwaiter().GetResult();pid=$p.Id;accepted=$false;command=$Arguments}
  }
  if($p.ExitCode -ne 0){throw 'Candidate CLI did not confirm same-user status'}
  $json=$stdout.GetAwaiter().GetResult()|ConvertFrom-Json
  if($json.ok -ne $true -or $json.schema_version -ne 1){throw 'Actual CLI JSON failed'}
  $json
 }finally{$p.Dispose()}
}
function Wait-TwoUserDiagnosticPipe($Fact,[string]$Name,[int]$TimeoutMilliseconds=20000) {
 Initialize-TwoUserWindowProbe
 $deadline=[datetime]::UtcNow.AddMilliseconds($TimeoutMilliseconds)
 do {
  $null=Assert-TwoUserLive $Fact
  if([OcxTwoUserWindow]::WaitNamedPipe(('\\.\pipe\'+$Name),1)){return}
  $errorCode=[Runtime.InteropServices.Marshal]::GetLastWin32Error()
  if($errorCode -notin @(2,121,231)){throw ('Actual same-user pipe wait failed: '+$errorCode)}
  Start-Sleep -Milliseconds 100
 }while([datetime]::UtcNow -lt $deadline)
 throw 'Actual same-user pipe not available within existing IPC budget'
}
function Read-TwoUserBoundedPipeBytes($Stream,[byte[]]$Buffer) {
 $cancel=[Threading.CancellationTokenSource]::new(5000)
 try{
  $pendingRead=$Stream.ReadExactlyAsync($Buffer,0,$Buffer.Length,$cancel.Token)
  # Pipe ValueTask uses IValueTaskSource: GetResult does not block while pending.
  # Convert once to Task before waiting, retaining cancellation and exact reads.
  $readTask=if($pendingRead -is [Threading.Tasks.Task]){$pendingRead}else{$pendingRead.AsTask()}
  [void]$readTask.GetAwaiter().GetResult()
 }finally{$cancel.Dispose()}
}
function Complete-TwoUserReadonlyPipeFrame($Stream) {
 $requestId='req_'+[guid]::NewGuid().ToString()
 $body=[Text.Encoding]::UTF8.GetBytes((@{request_id=$requestId;command='data_root_show';args=@{};confirm=$false;contract_version=1}|ConvertTo-Json -Compress))
 $header=[BitConverter]::GetBytes([uint64]$body.Length)
 if([BitConverter]::IsLittleEndian){[Array]::Reverse($header)}
 $Stream.Write($header,0,8);$Stream.Write($body,0,$body.Length);$Stream.Flush()
 $read=[byte[]]::new(8);Read-TwoUserBoundedPipeBytes $Stream $read
 if([BitConverter]::IsLittleEndian){[Array]::Reverse($read)}
 $length=[BitConverter]::ToUInt64($read,0)
 if($length -lt 1 -or $length -gt 1048576){throw 'Actual pipe response frame outside bounded contract'}
 $response=[byte[]]::new([int]$length);Read-TwoUserBoundedPipeBytes $Stream $response
 $value=[Text.Encoding]::UTF8.GetString($response)|ConvertFrom-Json
 if($value.request_id -cne $requestId -or $value.ok -ne $true){throw 'Actual readonly owned-pipe request did not succeed'}
 $value
}
function Write-TwoUserJson([string]$Path,$Value) {
 $temp=$Path+'.writing-'+[guid]::NewGuid().ToString('N')
 ConvertTo-Json -InputObject $Value -Depth 15|Set-Content -LiteralPath $temp -Encoding utf8
 [IO.File]::Move($temp,$Path,$true)
}
function Get-TwoUserRuntimeDriverText {
@'
let runtimeStage='command';let runtimePublic={};
function runtimeFailureCode(error){
 const codes=new Map([['Actual service not healthy within production start budget','runtime_start_not_healthy'],['Start confirmation exceeded original 30-second budget','runtime_start_confirmation_timeout'],['B claimed running on peer port','peer_port_claimed_running'],['B published an owned service record on A port','peer_port_owned_record'],['No explicit production port-conflict diagnostic','port_conflict_unproven'],['Manager did not confirm stop','manager_stop_not_confirmed'],['Stop did not clear own service record/state','runtime_stop_incomplete'],['Production attestation failed','runtime_attestation_failed'],['Runtime record changed','runtime_record_changed'],['Public manager command failed','public_manager_command_rejected']]);
 return codes.get(error?.message)||'runtime_binding_or_transport_failed';
}
const runtimeSleep=ms=>new Promise(r=>setTimeout(r,ms));
function freshNativeClientTextRestored(text){
 return typeof text==='string'
  && !/^[ \t]*(?:openai_base_url|experimental_realtime_ws_base_url|model_catalog_json)\s*=/m.test(text)
  && !/^[ \t]*model_provider\s*=\s*["']opencodex["']/m.test(text);
}

function fixtureConfig(port){
 assert(Number.isInteger(port)&&(port===0||(port>=1024&&port<=65535)));
 const config={hostname:'127.0.0.1',openaiProviderTierVersion:2,defaultProvider:'sweep-fixture',defaultModelAliases:false,providers:{'sweep-fixture':{adapter:'openai-chat',baseUrl:'http://127.0.0.1:1/v1',allowPrivateNetwork:true,apiKey:'synthetic-unused',models:['sweep-fixture-model'],selectedModels:['sweep-fixture-model'],liveModels:false,defaultAliases:false}}};
 if(port!==0)config.port=port;return config;
}
function publicRecord(record){
 assert(Number.isInteger(record.pid)&&record.pid>0&&Number.isInteger(record.port)&&record.port>=1024&&record.port<=65535,'Runtime public identity invalid');
 return {pid:record.pid,port:record.port};
}
function proofMatches(secret,challenge,pid,port,proof){
 if(typeof secret!=='string'||secret.length!==43||typeof proof!=='string')return false;
 const expected=createHmac('sha256',secret).update('opencodex-local-management-v1\n'+challenge+'\n'+pid+'\n'+port).digest();
 const received=Buffer.from(proof,'base64url');return received.length===expected.length&&timingSafeEqual(received,expected);
}
function assertConflictAction(action){
 assert(action?.failureCode==='port_conflict'&&action.result===null&&action.lifecycleState==='starting_failed'&&action.canStop===false&&action.canRestart===false,'Actual manager conflict rejection not proved');
}
function runtimeSelfTest(){
 const secret=Buffer.alloc(32,7).toString('base64url'),challenge='fixture';
 const proof=createHmac('sha256',secret).update('opencodex-local-management-v1\nfixture\n42\n54321').digest('base64url');
 assert(proofMatches(secret,challenge,42,54321,proof));assert(!proofMatches(secret,challenge,43,54321,proof));
 assert(!proofMatches(secret,challenge,42,54322,proof));assert(!proofMatches(secret,challenge,42,54321,'bad'));
 assert.deepEqual(publicRecord({pid:42,port:54321,attestationSecret:secret}),{pid:42,port:54321});
 assert(!JSON.stringify(publicRecord({pid:42,port:54321,attestationSecret:secret})).includes(secret));
 assert.equal(fixtureConfig(54321).port,54321);assert(!Object.hasOwn(fixtureConfig(0),'port'));
 assert.equal(fixtureConfig(0).defaultProvider,'sweep-fixture');assert.equal(Object.keys(fixtureConfig(0).providers).length,1);assert.equal(fixtureConfig(0).providers['sweep-fixture'].liveModels,false);assert.throws(()=>fixtureConfig(-1));
 assert.notEqual(Buffer.from(JSON.stringify(fixtureConfig(54321)),'utf8')[0],0xef);
 const conflict={failureCode:'port_conflict',result:null,lifecycleState:'starting_failed',canStop:false,canRestart:false};
 assertConflictAction(conflict);
 for(const changed of [{failureCode:null},{result:'started'},{lifecycleState:'running'},{canStop:true},{canRestart:true}])assert.throws(()=>assertConflictAction({...conflict,...changed}));
 assert(freshNativeClientTextRestored(''));assert(freshNativeClientTextRestored('model_provider = "openai"\n'));
 for(const key of ['openai_base_url','experimental_realtime_ws_base_url','model_catalog_json'])assert(!freshNativeClientTextRestored(key+' = "stale"\n'));
 assert(!freshNativeClientTextRestored('model_provider = "opencodex"\n'));
 console.log('pass-runtime-pure-fixtures-only');
}
async function runtimeInvoke(command,args={},timeout=10000){
 const expression='window.__TAURI_INTERNALS__.invoke('+JSON.stringify(command)+','+JSON.stringify(args)+')';
 const r=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true},timeout);
 assert(!r.exceptionDetails,'Public manager command failed');return r.result?.value;
}
async function runtimeCommand(c){
 assert(['start','conflict','observe','stop'].includes(c.mode));
 runtimePublic={mode:c.mode,port:c.port};
 const normalized=p=>{let v=resolve(p);if(v.charCodeAt(0)===92&&v.charCodeAt(1)===92&&v.charCodeAt(2)===63&&v.charCodeAt(3)===92)v=v.slice(4);return v.toLowerCase();};
 const same=(a,b)=>typeof a==='string'&&typeof b==='string'&&normalized(a)===normalized(b);
 assert(same(c.home,join(process.env.USERPROFILE,'.codex-desktop/opencodex/home')),'Own real H required');
 assert(same(c.root,join(process.env.USERPROFILE,'.codex-desktop/desktop')),'Own real D required');
 const bound=async()=>{
  const config=await runtimeInvoke('get_data_root_config');const paths=await runtimeInvoke('model_paths_v2_snapshot');
  assert(same(config.currentOpencodexHome,c.home)&&same(config.currentDataRoot,c.root),'Manager H/D binding changed');
  assert(same(paths.opencodexHome.path,c.home)&&same(paths.desktopRoot.path,c.root),'Snapshot H/D binding changed');
  assert(paths.program.consistency==='consistent'&&paths.runtimeEntry.status==='valid'&&paths.runtimePrefix.status==='valid','Actual managed runtime not valid');
  return paths;
 };
 const snapshot=async()=>{
  const s=await runtimeInvoke('get_status_snapshot');
  // Official status may expose paths.config (a FILE) as opencodex_home and
  // omit dataRoot. Verify supplied facts exactly; missing facts gain no authority.
  await bound();assert(s.facts,'Status facts absent');
  assert(!s.facts.opencodex_home||same(s.facts.opencodex_home,c.home)||same(s.facts.opencodex_home,join(c.home,'config.json')),'Status H binding differs');
  assert(!s.facts.data_root||same(s.facts.data_root,c.root)||same(s.facts.data_root,c.home),'Status root binding differs');
  const value={runtime:s.matrix.runtime,health:s.facts.health,source:s.source,pid:s.pid,port:s.port,protection:s.facts.protection,startupStatus:s.facts.startup_status,servicePresent:s.facts.service_present,rebootSafe:s.facts.reboot_safe};
  runtimePublic.status=value;return value;
 };
 const recordPath=join(c.home,'runtime-port.json');
 const record=async()=>{
  let st;try{st=await lstat(recordPath);}catch(e){if(e.code==='ENOENT')return null;throw e;}
  assert(st.isFile()&&!st.isSymbolicLink()&&st.size<=4096,'Unsafe runtime record');
  const r=JSON.parse(await readFile(recordPath,'utf8'));runtimePublic.record=publicRecord(r);return r;
 };
 const requestedPort=c.port;
 const health=async()=>{
  const r=await record();
  if(c.mode==='start'&&requestedPort===0&&c.port===0&&r){c.port=r.port;runtimePublic.automaticPort={requested:'omitted',selected:r.port,source:'actual-opencodex-runtime-record'};runtimePublic.port=r.port;}
  assert(r&&r.port===c.port&&(!c.expectedPid||r.pid===c.expectedPid),'Runtime record changed');
  const challenge=randomBytes(32).toString('base64url');
  // Literal loopback, no proxies/redirects, no admin key or inference request.
  const response=await new Promise((accept,reject)=>{
   const req=loopbackRequest({hostname:'127.0.0.1',port:r.port,path:'/healthz',method:'GET',headers:{'x-opencodex-attestation-challenge':challenge}},res=>{
    let bytes=0;const chunks=[];res.on('data',chunk=>{bytes+=chunk.length;if(bytes>65536){res.destroy();reject(Error('Bounded health required'));}else chunks.push(chunk);});
    res.on('error',reject);res.on('end',()=>accept({status:res.statusCode,proof:res.headers['x-opencodex-attestation-proof'],body:Buffer.concat(chunks)}));
   });req.on('error',reject);req.setTimeout(5000,()=>req.destroy(Error('Loopback health deadline')));req.end();
  });
  runtimePublic.health={status:response.status,proofPresent:typeof response.proof==='string',attestationVerified:proofMatches(r.attestationSecret,challenge,r.pid,r.port,response.proof)};
  assert(response.status===200&&runtimePublic.health.attestationVerified,'Production attestation failed');
  const h=JSON.parse(response.body.toString('utf8'));assert(h.service==='opencodex'&&h.pid===r.pid&&h.port===r.port,'Health identity changed');
  const again=await record();assert(again&&again.pid===r.pid&&again.port===r.port&&again.attestationSecret===r.attestationSecret,'Runtime binding changed');
  return {record:publicRecord(r),attestation:'verified-production-challenge',healthService:h.service,version:typeof h.version==='string'?h.version:null};
 };
 const diagnosticStatus=async()=>{
  const paths=await bound(),install=resolve(paths.runtimePrefix.path,'../..'),node=join(install,'runtime/node/node.exe');
  const packageRoot=join(paths.runtimePrefix.path,'node_modules/@bitkyc08/opencodex');
  const declaration=JSON.parse(await readFile(join(packageRoot,'package.json'),'utf8'));
  const relative=typeof declaration.bin==='string'?declaration.bin:declaration.bin?.ocx;
  assert(typeof relative==='string'&&!relative.includes('..')&&!relative.startsWith('/'),'Diagnostic actual package bin required');
  const script=join(packageRoot,relative);
  const readStatus=async(program,args,budget)=>new Promise(resolveResult=>{
   const env={HOME:process.env.USERPROFILE,USERPROFILE:process.env.USERPROFILE,OPENCODEX_HOME:c.home,CODEX_HOME:join(c.home,'diagnostic-unused-codex'),PATH:join(install,'runtime/node'),CI:'1',OCX_NO_DESKTOP_HANDOFF:'1'};
   for(const key of ['SystemRoot','WINDIR','COMSPEC','TEMP','TMP','APPDATA','LOCALAPPDATA'])if(process.env[key])env[key]=process.env[key];
   const child=diagnosticSpawn(program,args,{env,stdio:['ignore','pipe','ignore'],cwd:c.home,windowsHide:true});
   let output='',overflow=false;const started=Date.now();
   const timer=setTimeout(()=>child.kill(),budget);
   child.stdout.on('data',chunk=>{if(Buffer.byteLength(output)+chunk.length>1024*1024){overflow=true;child.kill();}else output+=chunk.toString('utf8');});
   child.on('error',()=>{clearTimeout(timer);resolveResult({result:'spawn-failed',accepted:false});});
   child.on('close',(code,signal)=>{clearTimeout(timer);let status;try{status=JSON.parse(output);}catch{}
    resolveResult({exitCode:code,signal,elapsedMilliseconds:Date.now()-started,outputOverflow:overflow,parsed:!!status,proxy:status?.proxy?{running:status.proxy.running,pid:status.proxy.pid,port:status.proxy.port}:null,startup:status?.startup?{status:status.startup.status}:null,accepted:false});});
  });
  const result=await readStatus(node,[script,'status','--json'],5000);
  await writeFile(join(output,'actual-readonly-status-diagnostic.json'),JSON.stringify(result)+'\n');
  // Readonly comparisons only AFTER the unchanged production gate has failed.
  // Direct bundled Bun helps distinguish launcher/engine startup from status
  // collection itself; neither this nor the late result can admit a service.
  const bun=join(install,'runtime/bun/bun.exe'),entry=join(packageRoot,'src/cli/index.ts');
  const direct=await readStatus(bun,['--no-env-file',entry,'status','--json'],5000);
  await writeFile(join(output,'direct-bun-status-diagnostic.json'),JSON.stringify(direct)+'\n');
  const late=await readStatus(node,[script,'status','--json'],15000);
  await writeFile(join(output,'late-node-status-diagnostic.json'),JSON.stringify({...late,originalBudgetPassed:false})+'\n');
 };
 const diagnosticReadiness=async()=>{
  // The real thirty-second attempt already failed. These bounded readonly
  // observations cannot accept it or authorize the next user service.
  const samples=[];const until=Date.now()+15000;
  do{
   try{
    const verified=await health();
    const response=await new Promise((accept,reject)=>{
     const req=loopbackRequest({hostname:'127.0.0.1',port:c.port,path:'/readyz',method:'GET'},res=>{let bytes=0;const chunks=[];res.on('data',chunk=>{bytes+=chunk.length;if(bytes>65536){res.destroy();reject(Error('bounded readiness'))}else chunks.push(chunk)});res.on('error',reject);res.on('end',()=>accept({code:res.statusCode,body:Buffer.concat(chunks)}))});
     req.on('error',reject);req.setTimeout(1000,()=>req.destroy(Error('readonly probe timeout')));req.end();
    });
    const body=JSON.parse(response.body.toString('utf8'));
    samples.push({at:Date.now(),code:response.code,identity:body.service==='opencodex'&&body.pid===verified.record.pid&&body.port===c.port,status:['ready','pending','failed'].includes(body.status)?body.status:'invalid'});
   }catch{samples.push({at:Date.now(),status:'observation-failed'})}
   await runtimeSleep(2000);
  }while(Date.now()<until);
  const files={};
  for(const [name,path] of [['codex',join(process.env.USERPROFILE,'.codex')],['claude',join(process.env.USERPROFILE,'.claude')],['runtimeHome',c.home]]){
   try{files[name]=(await readdir(path)).filter(n=>/^[a-zA-Z0-9_.-]{1,80}$/.test(n)).slice(0,64)}catch{files[name]=[]}
  }
  await writeFile(join(output,'late-readiness-diagnostic.json'),JSON.stringify({samples,files,accepted:false,originalBudgetPassed:false})+'\n');
 };
 const action=async action=>{
  await bound();const r=await runtimeInvoke('process_action',{request:{action,confirm:true}},action==='stop'?15000:35000);
  const value={lifecycleState:r.lifecycleState,result:r.result,failureCode:r.failure?.code??null,canStop:r.canStop,canRestart:r.canRestart};runtimePublic.action=value;return value;
 };
 runtimeStage='binding';await bound();
 let result={result:'pass-runtime-step',mode:c.mode,home:c.home,root:c.root,port:c.port};
 if(c.mode==='start'||c.mode==='conflict'){
  runtimeStage='own-config';assert(!(await record()),'Fresh service record required');
  // Default native integration requires an initialized Codex home directory.
  // Each new SID owns this EMPTY fixture; the real runtime must create its own
  // config/catalog and become ready. Never seed artifacts or disable integration.
  const clientHome=join(process.env.USERPROFILE,'.codex');
  try{const st=await lstat(clientHome);assert(st.isDirectory()&&!st.isSymbolicLink(),'Unsafe own Codex fixture')}
  catch(error){if(error.code!=='ENOENT')throw error;await mkdir(clientHome,{mode:0o700})}
  result.clientHomeFixture={emptyDirectoryInitialized:true,configSeeded:false,catalogSeeded:false,integrationDisabled:false};
  for(const path of [process.env.USERPROFILE,join(process.env.USERPROFILE,'.codex-desktop'),join(process.env.USERPROFILE,'.codex-desktop/opencodex'),c.home]){
   const st=await lstat(path);assert(st.isDirectory()&&!st.isSymbolicLink(),'Own H has a link');
  }
  const configPath=join(c.home,'config.json'),temp=join(c.home,'config.json.two-user-'+randomBytes(12).toString('hex'));
  let old;try{const st=await lstat(configPath);assert(st.isFile()&&!st.isSymbolicLink()&&st.size<512*1024);old=JSON.parse(await readFile(configPath,'utf8'));}catch(e){if(e.code!=='ENOENT')throw e;}
  assert(!old||!old.providers||Object.keys(old.providers).length===0||(Object.keys(old.providers).length===1&&JSON.stringify(old.providers)===JSON.stringify(fixtureConfig(0).providers)),'Fixture H contains unexpected providers');
  const handle=await open(temp,'wx',0o600);
  try{await handle.writeFile(JSON.stringify(fixtureConfig(c.port))+'\n','utf8');await handle.sync();}finally{await handle.close();}
  try{await rename(temp,configPath);}catch(e){await unlink(temp).catch(()=>{});throw e;}
  result.config={path:configPath,providers:1,syntheticOnly:true,liveModels:false,utf8Bom:false,atomicRename:true};
  runtimeStage='manager-start';const startBegan=Date.now();result.action=await action('start');
  const until=startBegan+30000;let last;
  if(c.mode==='conflict'){
   // The actual public action has already rejected a fixed peer-port bind.
   // With no owned endpoint, full CLI status cold-start is a separate diagnostic;
   // it cannot replace the action/record/peer OS identity proof or grant success.
   assertConflictAction(result.action);
   do{assert(!(await record()),'B published an owned service record on A port');await runtimeSleep(250);}while(Date.now()<until);
   assert(!(await record()),'B published an owned service record on A port');
   result.conflict={rejected:true,ownedRecordAbsent:true,diagnosticMarker:true,source:'actual-process-action-port_conflict',canStop:false,canRestart:false};
   result.status={source:'actual-conflict-action-and-absent-owned-record',lifecycleState:result.action.lifecycleState,claimedRunning:false};
  }else{
   runtimeStage='wait-owned-health';let observed;
   const samples=[];
   do{try{observed=await health();last=await snapshot();samples.push({at:Date.now(),elapsedMilliseconds:Date.now()-startBegan,health:runtimePublic.health,status:last});if(last.runtime==='running'&&last.health==='healthy'&&Number(last.pid)===observed.record.pid&&last.port===c.port)break;}
    catch(error){samples.push({at:Date.now(),health:runtimePublic.health??null,error:runtimeFailureCode(error)});}
    await runtimeSleep(250);
   }while(Date.now()<until);
   await writeFile(join(output,'owned-health-observations.json'),JSON.stringify({samples:samples.slice(0,64),originalBudgetMilliseconds:30000,startAction:result.action,totalMilliseconds:Date.now()-startBegan,accepted:false})+'\n');
   const startElapsedMilliseconds=Date.now()-startBegan;
   runtimePublic.startBudget={budgetMilliseconds:30000,elapsedMilliseconds:startElapsedMilliseconds};
   const startBudgetPassed=result.action.result==='started'&&startElapsedMilliseconds<=30000;
   if(!(startBudgetPassed&&observed&&last?.runtime==='running'&&last.health==='healthy'&&Number(last.pid)===observed.record.pid&&last.port===c.port)){
    // Additional readonly diagnosis follows the already failed original budget;
    // its result cannot make this attempt pass or authorize the next service.
    await writeFile(join(output,'production-attempt-failed.json'),JSON.stringify({accepted:false,originalBudgetPassed:false,budgetMilliseconds:30000})+'\n');
    await diagnosticReadiness();
    // Independent cleanup diagnosis after the ORIGINAL start already failed.
    // Fresh protected-record/HMAC and exact H binding are mandatory. Stop still
    // uses the public product command with its unchanged10s budget. Supplemental
    // data never makes pass-runtime-step or authorizes a peer protocol.
    const originalStartAction=runtimePublic.action;
    const diagnosticStop={accepted:false,diagnosticOnly:true,originalStartAccepted:false,originalStopBudgetMilliseconds:10000,managerReportedStopped:false};
    const stopBegan=Date.now();
    try{
     await bound();const own=await health();
     diagnosticStop.verifiedRecord=own.record;
     diagnosticStop.ownershipBasis='fresh-own-record-and-HMAC-before-official-manager-OS-owner-check';
     diagnosticStop.action=await action('stop');
     diagnosticStop.managerReportedStopped=diagnosticStop.action.result==='stopped';
    }catch(error){diagnosticStop.errorCode=runtimeFailureCode(error);}
    finally{
     diagnosticStop.observationMilliseconds=Date.now()-stopBegan;
     runtimePublic.action=originalStartAction;
     await writeFile(join(output,'supplemental-stop-after-failed-start.json'),JSON.stringify(diagnosticStop,null,2)+'\n',{flag:'wx'});
    }
    if(!diagnosticStop.managerReportedStopped){
     try{await diagnosticStatus();}catch{await writeFile(join(output,'actual-readonly-status-diagnostic.json'),JSON.stringify({result:'diagnostic-failed',accepted:false})+'\n');}
    }
    const confirmedLate=startElapsedMilliseconds>30000&&result.action.result==='started'&&observed&&last?.runtime==='running'&&last.health==='healthy'&&Number(last.pid)===observed.record.pid&&last.port===c.port;
    throw Error(confirmedLate?'Start confirmation exceeded original 30-second budget':'Actual service not healthy within production start budget');
   }
   result={...result,...observed,status:last,port:c.port,startBudget:{budgetMilliseconds:30000,elapsedMilliseconds:startElapsedMilliseconds},automaticPort:requestedPort===0?runtimePublic.automaticPort:null};
   const clientConfig=join(process.env.USERPROFILE,'.codex/config.toml');
   const clientStat=await lstat(clientConfig);assert(clientStat.isFile()&&!clientStat.isSymbolicLink()&&clientStat.size>0&&clientStat.size<=512*1024,'Actual Codex config injection missing');
   const injected=await readFile(clientConfig,'utf8');assert(injected.includes('openai_base_url')&&injected.includes(':'+c.port),'Codex routing did not use actual assigned port');
   result.clientHomeFixture={...result.clientHomeFixture,actualConfigWritten:true,actualAssignedPortBound:true};
  }
 }else if(c.mode==='observe'){
  runtimeStage='observe';result={...result,...await health(),status:await snapshot()};
  assert(result.status.runtime==='running'&&result.status.health==='healthy'&&Number(result.status.pid)===result.record.pid&&result.status.port===c.port,'Actual running state lost');
 }else{
  runtimeStage='before-stop';result.before=await health();
  const stopBegan=Date.now();runtimeStage='manager-stop';result.action=await action('stop');assert(result.action.result==='stopped','Manager did not confirm stop');
  const until=stopBegan+10000;let last;
  do{last=await snapshot();if(!(await record())&&last.runtime!=='running')break;await runtimeSleep(250);}while(Date.now()<until);
  assert(!(await record())&&last.runtime!=='running','Stop did not clear own service record/state');result.status=last;result.stopped=true;
  const nativeConfig=join(process.env.USERPROFILE,'.codex/config.toml');let nativeText='';
  try{const st=await lstat(nativeConfig);assert(st.isFile()&&!st.isSymbolicLink()&&st.size<=512*1024,'Unsafe fresh native client readback');nativeText=await readFile(nativeConfig,'utf8');}catch(error){if(error.code!=='ENOENT')throw error;}
  assert(freshNativeClientTextRestored(nativeText),'Native client routing/catalog fields survived official Stop');
  result.nativeClientRestoredReadback=true;result.stopBudget={budgetMilliseconds:10000,elapsedMilliseconds:Date.now()-stopBegan};
  assert(result.stopBudget.elapsedMilliseconds<=10000,'Stop confirmation exceeded original shared budget');

 }
 runtimeStage='final-binding';await bound();
 await writeFile(join(output,'runtime-step.json'),JSON.stringify(result,null,2)+'\n');
}
'@
}
function Get-TwoUserDriverText {
$text=@'
import assert from 'node:assert/strict';
import {writeFile,readFile,open,rename,lstat,readdir,unlink,mkdir} from 'node:fs/promises';
import {createHmac,randomBytes,timingSafeEqual} from 'node:crypto';
import {request as loopbackRequest} from 'node:http';
import {spawn as diagnosticSpawn} from 'node:child_process';
import {join,resolve} from 'node:path';
import {discoverMainTarget} from './windows-cdp.mjs';
const [endpoint,output,commandFile]=process.argv.slice(2);
let ws;let seq=0;const pending=new Map();
function call(method,params={},timeout=10000){const id=++seq;return new Promise((resolve,reject)=>{const timer=setTimeout(()=>{pending.delete(id);reject(Error('CDP deadline'));},timeout);pending.set(id,{resolve:v=>{clearTimeout(timer);resolve(v);},reject});ws.send(JSON.stringify({id,method,params}));});}
/*RUNTIME_CONTROL*/
if(endpoint==='--runtime-self-test'){runtimeSelfTest();process.exit(0);}
try{
 const target=await discoverMainTarget(endpoint,{timeoutMs:60000});
 const url=new URL(target.webSocketDebuggerUrl);assert(['127.0.0.1','localhost','[::1]'].includes(url.hostname));assert.equal(url.port,new URL(endpoint).port);
 ws=new WebSocket(target.webSocketDebuggerUrl);await new Promise((r,j)=>{ws.addEventListener('open',r,{once:true});ws.addEventListener('error',()=>j(Error('CDP unavailable')),{once:true});setTimeout(()=>j(Error('CDP open deadline')),10000).unref();});
 ws.addEventListener('message',e=>{const v=JSON.parse(e.data);const p=pending.get(v.id);if(p){pending.delete(v.id);v.error?p.reject(Error('CDP rejected')):p.resolve(v.result);}});
 if(commandFile&&commandFile!=='--diagnostic-sweep'){await runtimeCommand(JSON.parse(await readFile(commandFile,'utf8')));}
 else {
 let facts;
 for(let i=0;i<120;i++){
  const sampleStarted=Date.now();
  let r;
  try{r=await call('Runtime.evaluate',{expression:'(async()=>{if(!window.__TAURI_INTERNALS__?.invoke||!document.querySelector("#app")?.children.length)return null;const i=window.__TAURI_INTERNALS__.invoke;const start=performance.now();const paths=await i("model_paths_v2_snapshot");return {mounted:true,textLength:document.body.innerText.length,about:await i("app_about"),paths,pathSnapshotMilliseconds:performance.now()-start,config:await i("get_data_root_config"),preferences:await i("get_preferences"),runtime:await i("runtime_source")};})()',awaitPromise:true,returnByValue:true});}
  catch(error){await writeFile(join(output,'snapshot-call-failure.json'),JSON.stringify({elapsedMilliseconds:Date.now()-sampleStarted,budgetMilliseconds:10000,error:String(error),accepted:false})+'\n');throw error;}
  await writeFile(join(output,'snapshot-call-latency.json'),JSON.stringify({elapsedMilliseconds:Date.now()-sampleStarted,pathSnapshotMilliseconds:r.result?.value?.pathSnapshotMilliseconds??null,budgetMilliseconds:10000,accepted:false})+'\n');
  if(!r.exceptionDetails&&r.result?.value?.mounted){facts=r.result.value;if(facts.paths.program.consistency==='consistent')break;}
  await new Promise(r=>setTimeout(r,1000));
 }
 assert(facts?.mounted&&facts.textLength>30,'Actual first screen unavailable');assert.equal(facts.about.platform,'Windows');
 if(process.argv.includes('--diagnostic-sweep')){
  const environmentStarted=Date.now();
  try{
   const environment=await call('Runtime.evaluate',{expression:"(async()=>{const started=performance.now();window.__ocxdSweepEnvironment={pending:true};try{const value=await window.__TAURI_INTERNALS__.invoke('discover_environment',{request:null});return window.__ocxdSweepEnvironment={pending:false,elapsedMilliseconds:performance.now()-started,value}}catch(error){return window.__ocxdSweepEnvironment={pending:false,elapsedMilliseconds:performance.now()-started,error:{code:error?.code,message:error?.message}}}})()",awaitPromise:true,returnByValue:true});
   facts.environment=environment.exceptionDetails?{error:{message:'Actual environment IPC raised an exception'}}:environment.result.value;
  }catch(error){
   facts.environment={error:{message:'Actual environment observation failed within unchanged 10000ms budget: '+String(error)}};
   // The original observation remains failed. Retain a late result only to
   // diagnose its owned processes; this never authorizes readiness or ports.
   const lateUntil=Date.now()+30000;let late;
   do{const readback=await call('Runtime.evaluate',{expression:'window.__ocxdSweepEnvironment??null',returnByValue:true});late=readback.result?.value;if(late?.pending===false)break;await new Promise(r=>setTimeout(r,100));}while(Date.now()<lateUntil);
   await writeFile(join(output,'environment-late-result.json'),JSON.stringify({accepted:false,originalBudgetPassed:false,diagnosticOnly:true,result:late??null})+'\n');
  }
  await writeFile(join(output,'environment-call.json'),JSON.stringify({elapsedMilliseconds:Date.now()-environmentStarted,budgetMilliseconds:10000,...facts.environment,accepted:false})+'\n');
 }

 const image=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,'first-screen.png'),Buffer.from(image.data,'base64'));
 await writeFile(join(output,'first-screen.json'),JSON.stringify({mounted:true,textLength:facts.textLength,about:facts.about,paths:facts.paths,config:facts.config,cliEnabled:facts.preferences.cliEnabled,runtime:facts.runtime,environment:facts.environment},null,2)+'\n');
 if(!process.argv.includes('--diagnostic-sweep'))assert.equal(facts.paths.program.consistency,'consistent','NotReady: staged runtime unverified');
 if(!process.argv.includes('--diagnostic-sweep')){assert.equal(facts.paths.runtimeEntry.status,'valid');assert.equal(facts.paths.runtimePrefix.status,'valid');}
 }
}catch(error){
 if(commandFile&&commandFile!=='--diagnostic-sweep'){await writeFile(join(output,'runtime-step.json'),JSON.stringify({result:'fail',failureCode:runtimeFailureCode(error),stage:runtimeStage,publicFacts:runtimePublic})+'\n');process.exitCode=1;}else{throw error;}
}finally{ws?.close();}
'@
 $text.Replace('/*RUNTIME_CONTROL*/',(Get-TwoUserRuntimeDriverText))
}
function Test-TwoUserRuntimeFullGate([bool]$Requested,[bool]$IsolationPassed,[string]$PortProof,[bool]$TerminalProof) {
 $Requested -and $IsolationPassed -and $PortProof -ceq 'verified-runtime-ports' -and $TerminalProof
}
function Send-TwoUserRuntimeCommand($Account,[string]$Step,[int]$Port=0,[int]$PeerPid=0,[int]$PeerPort=0) {
 $Account.runtimeSequence++
 Write-TwoUserJson (Join-Path $Account.evidence ('runtime-command-'+$Account.runtimeSequence+'.json')) @{step=$Step;port=$Port;peerPid=$PeerPid;peerPort=$PeerPort}
}
function Wait-TwoUserRuntimeReply($Account,[string]$Step,$Ledger) {
 $path=Join-Path $Account.evidence ('runtime-reply-'+$Account.runtimeSequence+'.json');$until=[datetime]::UtcNow.AddSeconds(120)
 while(!(Test-Path -LiteralPath $path)){
  if($Account.process.HasExited -or (Test-Path -LiteralPath (Join-Path $Account.evidence 'failure.json'))){throw 'Native runtime child failed; preserve controlled failure receipts'}
  Add-TwoUserDescendants $Account.fact $Ledger
  if([datetime]::UtcNow -gt $until){throw 'Native runtime controlled-reply deadline'}
  Start-Sleep -Milliseconds 100
 }
 $r=Get-Content -LiteralPath $path -Raw|ConvertFrom-Json
 if($r.result -cne 'pass-runtime-step' -or $r.step -cne $Step -or $r.sequence -ne $Account.runtimeSequence -or $r.sid -cne $Account.sid -or $r.verifyRuntimePorts -ne $true -or !(Test-TwoUserSamePath $r.home (Join-Path $Account.profile '.codex-desktop/opencodex/home'))){throw 'Runtime step receipt binding differs'}
 $r
}
function Assert-TwoUserRuntimeUnchanged($Service,$Expected) {
 $now=Assert-TwoUserLive $Service.process
 if($now.pid -ne $Expected.pid -or $now.creationTicks -cne $Expected.creationTicks -or $now.path -ine $Expected.path -or $now.sid -cne $Expected.sid){throw 'Peer action changed original service generation'}
 $listeners=@(Get-NetTCPConnection -State Listen -LocalPort $Service.port -ErrorAction SilentlyContinue)
 if(!$listeners.Count -or @($listeners|Where-Object {$_.OwningProcess -ne $Expected.pid}).Count){throw 'Peer action changed original listener'}
}
function Invoke-TwoUserRuntimeProtocol($Accounts,$Ledger) {
 $a=$Accounts[0];$b=$Accounts[1]
 Send-TwoUserRuntimeCommand $a 'start';$startedA=Wait-TwoUserRuntimeReply $a 'start' $Ledger
 $serviceA=$startedA.service
 if($serviceA.process.sid -cne $a.sid){throw 'A runtime SID differs'}
 Send-TwoUserRuntimeCommand $b 'conflict' $serviceA.port $serviceA.process.pid
 $conflict=Wait-TwoUserRuntimeReply $b 'conflict' $Ledger
 if(!$conflict.conflict.conflict.rejected -or !$conflict.peerListenerUnchanged){throw 'Explicit B conflict rejection missing'}
 Assert-TwoUserRuntimeUnchanged $serviceA $serviceA.process
 Send-TwoUserRuntimeCommand $a 'after-conflict';$afterConflict=Wait-TwoUserRuntimeReply $a 'after-conflict' $Ledger
 Assert-TwoUserRuntimeUnchanged $afterConflict.service $serviceA.process
 Send-TwoUserRuntimeCommand $b 'start' 0 0 $serviceA.port
 $startedB=Wait-TwoUserRuntimeReply $b 'start' $Ledger;$serviceB=$startedB.service
 if($serviceB.process.sid -cne $b.sid -or $serviceB.process.sid -ceq $serviceA.process.sid -or $serviceB.port -eq $serviceA.port -or $serviceB.process.pid -eq $serviceA.process.pid -or (Test-TwoUserSamePath $serviceA.home $serviceB.home)){throw 'Two actual service identities/ports/H collide'}
 Send-TwoUserRuntimeCommand $a 'overlap';Send-TwoUserRuntimeCommand $b 'overlap'
 $start=[datetime]::UtcNow
 foreach($n in 1..6){
  Assert-TwoUserRuntimeUnchanged $serviceA $serviceA.process;Assert-TwoUserRuntimeUnchanged $serviceB $serviceB.process
  $null=Get-TwoUserWindow (Get-Content -LiteralPath (Join-Path $a.evidence 'ready.json') -Raw|ConvertFrom-Json).manager
  $null=Get-TwoUserWindow (Get-Content -LiteralPath (Join-Path $b.evidence 'ready.json') -Raw|ConvertFrom-Json).manager
  Start-Sleep -Seconds 1
 }
 $overlapA=Wait-TwoUserRuntimeReply $a 'overlap' $Ledger;$overlapB=Wait-TwoUserRuntimeReply $b 'overlap' $Ledger
 if($overlapA.observedSeconds -lt 6 -or $overlapB.observedSeconds -lt 6){throw 'Actual service health overlap incomplete'}
 $overlap=[ordered]@{minimumSeconds=6;observedSeconds=([datetime]::UtcNow-$start).TotalSeconds;fromUtc=$start.ToString('o');toUtc=[datetime]::UtcNow.ToString('o');a=$overlapA;b=$overlapB}
 Send-TwoUserRuntimeCommand $b 'stop';$stopB=Wait-TwoUserRuntimeReply $b 'stop' $Ledger
 if(!$stopB.originalGenerationExited){throw 'B exact own service stop missing'}
 Assert-TwoUserRuntimeUnchanged $serviceA $serviceA.process
 Send-TwoUserRuntimeCommand $a 'after-peer-stop';$afterStop=Wait-TwoUserRuntimeReply $a 'after-peer-stop' $Ledger
 Assert-TwoUserRuntimeUnchanged $afterStop.service $serviceA.process
 Send-TwoUserRuntimeCommand $a 'stop';$stopA=Wait-TwoUserRuntimeReply $a 'stop' $Ledger
 if(!$stopA.originalGenerationExited){throw 'A exact own service stop missing'}
 foreach($account in $Accounts){
  Send-TwoUserRuntimeCommand $account 'complete';$null=Wait-TwoUserRuntimeReply $account 'complete' $Ledger
  $done=Get-Content -LiteralPath (Join-Path $account.evidence 'runtime-complete.json') -Raw|ConvertFrom-Json
  if($done.result -cne 'verified-runtime-ports' -or $done.sid -cne $account.sid -or $done.verifyRuntimePorts -ne $true -or !$done.servicesStopped){throw 'Runtime terminal per-account proof missing'}
 }
 [ordered]@{result='verified-runtime-ports';verifyRuntimePorts=$true;a=$startedA;bConflict=$conflict;aAfterConflict=$afterConflict;b=$startedB;overlap=$overlap;bStop=$stopB;aAfterBStop=$afterStop;aStop=$stopA;servicesStopped=$true}
}
# Fixed per-account protocol. No controller-supplied paths, actions or executable.
function Get-TwoUserRuntimeSteps([int]$Role) {
 if($Role -eq 1){return @('start','after-conflict','overlap','after-peer-stop','stop','complete')}
 if($Role -eq 2){return @('conflict','start','overlap','stop','complete')}
 throw 'Invalid runtime fixture role'
}
function Invoke-TwoUserRuntimeDriver($Context,$App,$Ledger,[string]$Mode,[int]$Port,[int]$ExpectedPid,[string]$Label) {
 Assert-TwoUserChildGuard
 Assert-TwoUserNoReparseAncestors (Join-Path $Context.profile '.codex-desktop/opencodex/home')
 Assert-TwoUserNoReparseAncestors (Join-Path $Context.profile '.codex-desktop/desktop')
 $listeners=@(Get-NetTCPConnection -State Listen -LocalPort $App.port -ErrorAction SilentlyContinue)
 Add-TwoUserDescendants $App.fact $Ledger
 if(!$listeners.Count -or @($listeners|Where-Object {!$Ledger.ContainsKey([int]$_.OwningProcess) -or $_.LocalAddress -notin @('127.0.0.1','::1')}).Count){throw 'Runtime driver CDP ownership changed'}
 foreach($listener in $listeners){$null=Assert-TwoUserLive $Ledger[[int]$listener.OwningProcess]}
 $dir=Join-Path $Context.evidence ('runtime-'+$Label);New-Item -ItemType Directory -Path $dir|Out-Null
 $inputPath=Join-Path $dir 'command.json'
 Write-TwoUserJson $inputPath @{mode=$Mode;home=(Join-Path $Context.profile '.codex-desktop/opencodex/home');root=(Join-Path $Context.profile '.codex-desktop/desktop');port=$Port;expectedPid=$ExpectedPid}
 $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$Context.node;$info.UseShellExecute=$false;$info.RedirectStandardError=$true;$info.RedirectStandardOutput=$true
 foreach($a in @((Join-Path $Context.code 'two-user-driver.mjs'),('http://127.0.0.1:'+$App.port),$dir,$inputPath)){$info.ArgumentList.Add($a)}
 $driver=[Diagnostics.Process]::Start($info)
 try{
  $fact=Get-TwoUserProcessFact $driver $Context.accountSid;$Ledger[[int]$fact.pid]=$fact
  $stdout=$driver.StandardOutput.ReadToEndAsync();$stderr=$driver.StandardError.ReadToEndAsync()
  # Transport envelope only; the driver keeps production start=30s/stop=10s.
  $deadline=[datetime]::UtcNow.AddSeconds(100)
  $lastCommandSample=$null
  while(!$driver.HasExited){
   Add-TwoUserDescendants $App.fact $Ledger
   Write-TwoUserJson (Join-Path $Context.evidence 'runtime-tracked.json') @($Ledger.Values)
   if((Test-Path -LiteralPath (Join-Path $dir 'production-attempt-failed.json')) -and (!$lastCommandSample -or ([datetime]::UtcNow-$lastCommandSample).TotalSeconds -ge 2)){
    $lastCommandSample=[datetime]::UtcNow
    $classifications=@();foreach($row in @(Get-CimInstance Win32_Process)){
     if(!$Ledger.ContainsKey([int]$row.ProcessId)){continue}
     $fact=$Ledger[[int]$row.ProcessId]
     if($row.CreationDate.ToUniversalTime().Ticks -lt ([long]$fact.creationTicks-9) -or $row.CreationDate.ToUniversalTime().Ticks -gt [long]$fact.creationTicks){continue}
     $text=[string]$row.CommandLine
     $classifications+=@{pid=$row.ProcessId;image=[IO.Path]::GetFileName($fact.path);category=$(if($text -match 'icacls'){'private-acl'}elseif($text -match 'debug.*models|bundled-models'){'codex-catalog'}elseif($text -match '--version'){'version-probe'}elseif($text -match 'Get-AppxPackage|Get-CimInstance|Win32_Service|schtasks|Get-Service'){'windows-discovery'}elseif($text -match 'status.*--json'){'full-status'}else{'other'});accepted=$false}
    }
    @{atUtc=$lastCommandSample.ToString('o');processes=$classifications;accepted=$false}|ConvertTo-Json -Depth 5 -Compress|Add-Content -LiteralPath (Join-Path $dir 'owned-process-work-samples.log')
   }
   if([datetime]::UtcNow -gt $deadline){throw 'Runtime driver transport deadline'}
   Start-Sleep -Milliseconds 100
  }
  # Raw errors can contain runtime-record material. Retain controlled stage only.
  $reply=Join-Path $dir 'runtime-step.json'
  if(!(Test-Path -LiteralPath $reply)){throw 'Runtime driver returned no controlled receipt'}
  $value=Get-Content -LiteralPath $reply -Raw|ConvertFrom-Json
  if($driver.ExitCode -ne 0 -or $value.result -cne 'pass-runtime-step'){
   # The unchanged production attempt has already failed. Preserve actual
   # backend timing/error separately; this observation never changes the gate.
   $installation=Split-Path -Parent $App.fact.path
   $observer=Join-Path $installation 'sweep-state.exe'
   if(Test-Path -LiteralPath $observer){
    $probeInfo=[Diagnostics.ProcessStartInfo]::new();$probeInfo.FileName=$observer;$probeInfo.UseShellExecute=$false;$probeInfo.RedirectStandardOutput=$true;$probeInfo.RedirectStandardError=$true
    foreach($arg in @('--runtime-observe',$installation,(Join-Path $Context.profile '.codex-desktop/desktop'),(Join-Path $Context.profile '.codex-desktop/opencodex/home'))){$probeInfo.ArgumentList.Add($arg)}
    $probe=[Diagnostics.Process]::Start($probeInfo)
    try{
     $probeFact=Get-TwoUserProcessFact $probe $Context.accountSid;$Ledger[[int]$probeFact.pid]=$probeFact
     $probeOutput=$probe.StandardOutput.ReadToEndAsync();$probeError=$probe.StandardError.ReadToEndAsync()
     if(!$probe.WaitForExit(12000)){Stop-TwoUserTracked $probeFact;Write-TwoUserJson (Join-Path $dir 'actual-backend-status-observer.json') @{timedOut=$true;accepted=$false}}
     else{$probeOutput.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $dir 'actual-backend-status-observer.log')}
    }finally{$probe.Dispose()}
   }
   throw ('Runtime step failed: '+$Label)
  }
  $value
 }finally{$driver.Dispose()}
}
function Get-TwoUserOwnedRuntime($Context,$App,$Ledger,$PublicRecord,[string]$RuntimePrefix,$Expected) {
 $null=Assert-TwoUserLive $App.fact
 if(!$Ledger.ContainsKey([int]$PublicRecord.pid)){throw 'Runtime PID was not observed in this fixture launch tree'}
 $p=Get-Process -Id $PublicRecord.pid -ErrorAction Stop
 try{
  $fact=Get-TwoUserProcessFact $p $Context.accountSid
  $known=$Ledger[[int]$fact.pid]
  if($fact.creationTicks -cne $known.creationTicks -or $fact.path -ine $known.path){throw 'Runtime generation differs from tracked launch'}
  if($Expected -and ($fact.pid -ne $Expected.pid -or $fact.creationTicks -cne $Expected.creationTicks -or $fact.path -ine $Expected.path -or $fact.sid -cne $Expected.sid)){throw 'Original service generation changed'}
  # Actual layout: package prefix I/runtime/opencodex; engines are siblings.
  $runtimeRoot=Split-Path -Parent $RuntimePrefix
  $expectedImages=@((Join-Path $runtimeRoot 'node/node.exe'),(Join-Path $runtimeRoot 'bun/bun.exe'),(Join-Path $RuntimePrefix 'node_modules/bun/bin/bun.exe'),(Join-Path $RuntimePrefix 'node_modules/@oven/bun-windows-x64/bin/bun.exe'),(Join-Path $RuntimePrefix 'node_modules/@oven/bun-windows-x64-baseline/bin/bun.exe'))
  if(!@($expectedImages|Where-Object {Test-TwoUserSamePath $_ $fact.path}).Count){throw 'Runtime image is not the actual bundled runtime'}
  $null=Assert-TwoUserUnder $runtimeRoot $fact.path
  $token=Get-WindowsProcessToken -Process $p -ExpectedExecutable $fact.path
  Assert-WindowsProcessMode -Observation $token -Mode ordinary
  $listeners=@(Get-NetTCPConnection -State Listen -LocalPort $PublicRecord.port -ErrorAction SilentlyContinue)
  if(!$listeners.Count -or @($listeners|Where-Object {$_.OwningProcess -ne $fact.pid -or $_.LocalAddress -notin @('127.0.0.1','::1')}).Count){throw 'Service listener does not belong exclusively to tracked real SID/PID'}
  [ordered]@{process=$fact;token=$token;imageSha256=(Get-FileHash -LiteralPath $fact.path -Algorithm SHA256).Hash.ToLowerInvariant();port=[int]$PublicRecord.port;listeners=@($listeners|ForEach-Object {@{address=$_.LocalAddress;port=$_.LocalPort;pid=$_.OwningProcess}});home=(Join-Path $Context.profile '.codex-desktop/opencodex/home');observedUtc=[datetime]::UtcNow.ToString('o')}
 }finally{$p.Dispose()}
}
function Invoke-TwoUserRuntimeSession($Context,$App,$Ledger,[string]$RuntimePrefix,$State) {
 Assert-TwoUserChildGuard
 if($Context.verifyRuntimePorts -ne $true){throw 'Runtime ports require explicit controller and child flag'}
 $steps=Get-TwoUserRuntimeSteps $Context.runtimeRole
 $index=0;$until=[datetime]::UtcNow.AddSeconds(720)
 try{
  foreach($step in $steps){
   $index++;$path=Join-Path $Context.evidence ('runtime-command-'+$index+'.json')
   while(!(Test-Path -LiteralPath $path)){
    if(Test-Path -LiteralPath (Join-Path $Context.evidence 'stop')){throw 'Runtime protocol interrupted by controller'}
    if([datetime]::UtcNow -gt $until){throw 'Runtime controlled-step deadline'}
    Add-TwoUserDescendants $App.fact $Ledger;Start-Sleep -Milliseconds 100
   }
   $command=Get-Content -LiteralPath $path -Raw|ConvertFrom-Json
   if($command.step -cne $step){throw 'Runtime controlled-step order changed'}
   $reply=[ordered]@{result='pass-runtime-step';step=$step;sequence=$index;sid=$Context.accountSid;verifyRuntimePorts=$true;home=(Join-Path $Context.profile '.codex-desktop/opencodex/home')}
   if($step -in @('start','conflict')){
    if($State.service){throw 'Fixture already owns a runtime'}
    $port=if($step -ceq 'conflict'){
     if($Context.runtimeRole -ne 2 -or [int]$command.port -lt 1024 -or [int]$command.port -gt 65535 -or [int]$command.peerPid -le 0){throw 'Explicit peer port identity required'}
     [int]$command.port
    }else{0} # No chosen port in the file: the adopted runtime must allocate it.
    $observed=Invoke-TwoUserRuntimeDriver $Context $App $Ledger $step $port 0 ([string]$index)
    if($step -ceq 'conflict'){
     $listeners=@(Get-NetTCPConnection -State Listen -LocalPort $port -ErrorAction SilentlyContinue)
     if(!$observed.conflict.rejected -or !$listeners.Count -or @($listeners|Where-Object {$_.OwningProcess -ne [int]$command.peerPid}).Count){throw 'Conflict changed or replaced peer listener'}
     $reply.conflict=$observed;$reply.peerListenerUnchanged=$true
    }else{
     $State.service=Get-TwoUserOwnedRuntime $Context $App $Ledger $observed.record $RuntimePrefix $null
     if($observed.automaticPort.requested -cne 'omitted' -or $observed.automaticPort.selected -ne $State.service.port){throw 'Actual omitted-port automatic allocation not proved'}
     $reply.service=$State.service;$reply.health=$observed;$reply.automaticPort=$observed.automaticPort
    }
   }elseif($step -ceq 'complete'){
    if($State.service){throw 'Runtime must be stopped before completion'}
    $State.completed=$true
    Write-TwoUserJson (Join-Path $Context.evidence 'runtime-complete.json') @{result='verified-runtime-ports';sid=$Context.accountSid;verifyRuntimePorts=$true;steps=$steps;servicesStopped=$true}
   }else{
    if(!$State.service){throw 'No frozen own service generation'}
    $start=[datetime]::UtcNow;$samples=@();$n=0
    do{
     $n++;$health=Invoke-TwoUserRuntimeDriver $Context $App $Ledger 'observe' $State.service.port $State.service.process.pid ($index.ToString()+'-observe-'+$n)
     $own=Get-TwoUserOwnedRuntime $Context $App $Ledger $health.record $RuntimePrefix $State.service.process
     $samples+=@{service=$own;health=$health}
     if($step -ceq 'overlap'){Start-Sleep -Seconds 1}
    }while($step -ceq 'overlap' -and ([datetime]::UtcNow-$start).TotalSeconds -lt 6)
    $reply.samples=$samples;$reply.service=$own;$reply.observedSeconds=([datetime]::UtcNow-$start).TotalSeconds
    if($step -ceq 'stop'){
     # Verify exact own PID/creation/image/SID immediately before manager stop.
     $null=Get-TwoUserOwnedRuntime $Context $App $Ledger $health.record $RuntimePrefix $State.service.process
     $reply.stop=Invoke-TwoUserRuntimeDriver $Context $App $Ledger 'stop' $own.port $own.process.pid ([string]$index+'-stop')
     if(Get-Process -Id $own.process.pid -ErrorAction SilentlyContinue){throw 'Original service PID still exists after stop'}
     if(Get-NetTCPConnection -State Listen -LocalPort $own.port -ErrorAction SilentlyContinue){throw 'Fixture port still listening after stop'}
     $reply.originalGenerationExited=$true;$State.service=$null
    }
   }
   Write-TwoUserJson (Join-Path $Context.evidence ('runtime-reply-'+$index+'.json')) $reply
  }
 }finally{
  if($State.service){
   try{
    $health=Invoke-TwoUserRuntimeDriver $Context $App $Ledger 'observe' $State.service.port $State.service.process.pid 'cleanup-observe'
    $null=Get-TwoUserOwnedRuntime $Context $App $Ledger $health.record $RuntimePrefix $State.service.process
    $null=Invoke-TwoUserRuntimeDriver $Context $App $Ledger 'stop' $State.service.port $State.service.process.pid 'cleanup-stop'
   }catch{# Existing outer cleanup may stop only exact, already tracked generations.
    $State.cooperativeStopFailed=$true
   }
  }
 }
}
function Get-TwoUserDiagnosticStartupState([object[]]$States){
 $failed=@($States|Where-Object {$_.failed})
 @{ready=($States.Count -eq 2 -and @($States|Where-Object {$_.failed -or !$_.ready}).Count -eq 0);allSettled=($States.Count -eq 2 -and @($States|Where-Object {!$_.failed -and !$_.ready}).Count -eq 0);failedSids=@($failed|ForEach-Object {$_.sid})}
}
function Invoke-TwoUserSweepCheck([string]$Name,[scriptblock]$Action){
 if(!$c.diagnosticSweep){return & $Action}
 $record=[ordered]@{name=$Name;status='fail';accepted=$false;diagnosticOnly=$true;firstError=''}
 try{$record.evidence=& $Action;$record.status='pass'}catch{$record.firstError=($_.Exception.Message -split "\r?\n",2)[0];$record.errorType=$_.Exception.GetType().FullName}
 $path=Join-Path $out ('sweep-'+$Name+'.json')
 if(Test-Path -LiteralPath $path){throw 'Preserve earlier per-user diagnostic record'}
 Write-TwoUserJson $path $record
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
 # Read-only access facts for the exact actual ordinary profile. No provisioning
 # or repairs; compare drive-relative prefix and anchored root explicitly.
 $accessFacts=@()
 foreach($path in @(([IO.Path]::GetPathRoot($installed).TrimEnd([char]92)),[IO.Path]::GetPathRoot($installed),$nativeProfile,$installed,$root,(Join-Path $installed 'runtime-bundle'),(Join-Path $installed 'runtime-bundle/runtime-bundle.manifest.json'))){
  try{$file=Get-Item -LiteralPath $path -Force -ErrorAction Stop;$accessFacts+=@{path=$path;exists=$true;resolved=$file.FullName;attributes=$file.Attributes.ToString();directory=$file.PSIsContainer;length=$(if(!$file.PSIsContainer){$file.Length}else{$null})}}
  catch{$accessFacts+=@{path=$path;exists=$false;errorType=$_.Exception.GetType().FullName;error=$_.Exception.Message}}
 }
 $descriptor=Join-Path $installed 'runtime-bundle/runtime-bundle.manifest.json'
 $descriptorHash=(Get-FileHash -LiteralPath $descriptor).Hash.ToLowerInvariant()
 $binary=[IO.File]::ReadAllBytes($exe)
 $pinPresent=[Text.Encoding]::ASCII.GetString($binary).Contains($descriptorHash)
 Write-TwoUserJson (Join-Path $out 'bundle-read-access.json') @{facts=$accessFacts;descriptorSha256=$descriptorHash;descriptorPinPresentInActualExe=$pinPresent;readOnly=$true}
 $ledger=[Collections.Generic.Dictionary[int,object]]::new();$manager=$null;$runtimeState=@{service=$null;completed=$false;cooperativeStopFailed=$false}
 $result=[ordered]@{ephemeralGuard=$c.ephemeralGuard;sid=$sid;profile=$nativeProfile;processHome=$env:USERPROFILE;localAppData=$env:LOCALAPPDATA;desktopRoot=$root;opencodexHome=$nativeHome;installation=$installed;result='fail';runtimeProxyVerification='not-verified';verifyRuntimePorts=($c.verifyRuntimePorts -eq $true);sandbox=$false}
 $start={
  $listener=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0);$listener.Start();$port=$listener.LocalEndpoint.Port;$listener.Stop()
  $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$exe;$info.WorkingDirectory=$installed;$info.UseShellExecute=$false
  # PowerShell/.NET can preserve a removed process environment value as empty.
  # Delete the actual child dictionary keys; empty is not an unset smoke port.
  foreach($key in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_WINDOWS_SMOKE_CDP_PORT','HOME','OPENCODEX_HOME','CODEX_HOME','WEBVIEW2_USER_DATA_FOLDER','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS')){[void]$info.Environment.Remove($key)}
  if($info.Environment.ContainsKey('OPENCODEX_WINDOWS_SMOKE_CDP_PORT') -or $info.Environment.ContainsKey('OPENCODEX_SANDBOX')){throw 'Native manager child still has test overrides'}
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
  Add-TwoUserDescendants $app.fact $ledger
  if(!$listeners.Count -or @($listeners|Where-Object {!$ledger.ContainsKey([int]$_.OwningProcess) -or $_.LocalAddress -notin @('127.0.0.1','::1')}).Count){throw 'NotReady: CDP listener is not owned by this Manager tree'}
  foreach($listenerFact in $listeners){$null=Assert-TwoUserLive $ledger[[int]$listenerFact.OwningProcess]}
  $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$c.node;$info.UseShellExecute=$false;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
  foreach($a in @((Join-Path $code 'two-user-driver.mjs'),"http://127.0.0.1:$($app.port)",$evidence)){$info.ArgumentList.Add($a)}
  if($c.diagnosticSweep){$info.ArgumentList.Add('--diagnostic-sweep')}
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
  if(!$c.diagnosticSweep){
  if($facts.runtime.kind -cne 'managed' -or !(Test-TwoUserSamePath $facts.runtime.path $facts.paths.runtimeEntry.path)){throw 'NotReady: actual runtime source is not the own managed entry'}
  }
  $appLog=Join-Path $root 'logs/app.log'
  if(!(Test-Path -LiteralPath $appLog) -or !(Get-Content -LiteralPath $appLog -Raw).Contains('commit='+$c.commit)){throw 'NotReady: EXE source commit provenance missing'}
  if($c.diagnosticSweep){Write-TwoUserJson (Join-Path $out ('observation-'+$phase+'.json')) @{facts=$facts;window=$window;token=$token;manager=$app.fact;diagnosticOnly=$true;accepted=$false}}
  [pscustomobject]@{facts=$facts;window=$window;directory=$evidence;token=$token}
 }
 try{
  if($c.diagnosticSweep){
   $observed=@{initial=$null}
   $manager=& $start
   # Real SID/token/window is independent of the WebView diagnostic endpoint.
   $windowDeadline=[datetime]::UtcNow.AddSeconds(40);$liveWindow=$null
   do{try{$liveWindow=Get-TwoUserWindow $manager.fact}catch{Start-Sleep -Milliseconds 250}}while(!$liveWindow -and [datetime]::UtcNow -lt $windowDeadline)
   if(!$liveWindow){throw 'NotReady: actual own Manager window missing'}
   $actualToken=Get-WindowsProcessToken -Process $manager.process -ExpectedExecutable $exe
   Assert-WindowsProcessMode -Observation $actualToken -Mode ordinary
   Write-TwoUserJson (Join-Path $out 'diagnostic-startup.json') @{sid=$sid;manager=$manager.fact;window=$liveWindow;token=$actualToken;accepted=$false}
   Invoke-TwoUserSweepCheck 'native-observation' {$observed.initial=& $screen $manager 'initial';$observed.initial.facts}|Out-Null
   $initial=$observed.initial
   # First-screen polling is not a terminal supply observation. Keep this exact
   # generation alive inside the ORIGINAL shared 600-second startup budget;
   # do not impose an earlier secondary cutoff. Never kill a still-verifying worker to
   # install the CLI fixture, or its pending transaction is caused by the test.
   $supplyDeadline=[datetime]::Parse($c.startupDeadlineUtc).ToUniversalTime()
   $supplyTerminal=$null;$supplyPhases=@();$lastPhase=$null
   do{
    $null=Assert-TwoUserLive $manager.fact;Add-TwoUserDescendants $manager.fact $ledger
    $logPath=Join-Path $root 'logs/app.log'
    if(Test-Path -LiteralPath $logPath){$supplyTerminal=Get-Content -LiteralPath $logPath|Select-String 'integrated runtime (provisioned and verified|provisioning failed)'|Select-Object -Last 1}
    $pending=Join-Path $installed '.runtime-bundle-pending.json'
    if(Test-Path -LiteralPath $pending){try{$phase=(Get-Content -LiteralPath $pending -Raw|ConvertFrom-Json).phase;if($phase -cne $lastPhase -and $supplyPhases.Count -lt 64){$supplyPhases+=@{phase=$phase;atUtc=[datetime]::UtcNow.ToString('o')};$lastPhase=$phase}}catch{}}
    if(!$supplyTerminal){Start-Sleep -Milliseconds 250}
   }while(!$supplyTerminal -and [datetime]::UtcNow -lt $supplyDeadline)
   Write-TwoUserJson (Join-Path $out 'supply-worker-terminal.json') @{terminal=$(if($supplyTerminal){$supplyTerminal.Line}else{$null});phases=$supplyPhases;originalStartupDeadlineUtc=$c.startupDeadlineUtc;observationDeadlineUtc=$supplyDeadline.ToString('o');accepted=$false}
   if($supplyTerminal -and $supplyTerminal.Line.Contains('integrated runtime provisioned and verified')){
    Invoke-TwoUserSweepCheck 'supplied-observation' {$observed.initial=& $screen $manager 'supplied';$observed.initial.facts}|Out-Null
    $initial=$observed.initial
   }
   Invoke-TwoUserSweepCheck 'private-bundle-supply' {
    if(!$supplyTerminal){throw 'Actual first-start supply did not reach a terminal result inside the original shared startup budget'}
    $failedSupply=@(Get-Content -LiteralPath (Join-Path $root 'logs/app.log') -ErrorAction SilentlyContinue|Select-String 'integrated runtime provisioning failed'|Select-Object -Last 1)
    if($failedSupply.Count){throw $failedSupply[0].Line}
    if(!$initial){throw 'Actual supply observation unavailable: native CDP failed; app logs retain any actual bundle failure'}
    if($initial.facts.paths.program.consistency -cne 'consistent' -or $initial.facts.paths.runtimeEntry.status -cne 'valid' -or $initial.facts.paths.runtimePrefix.status -cne 'valid' -or $initial.facts.runtime.kind -cne 'managed'){throw 'Actual private first-start runtime supply is not consistent/managed'}
    $initial.facts.paths.program
   }|Out-Null
   Invoke-TwoUserSweepCheck 'runtime-readiness' {
    if(!$initial -or $initial.facts.environment.value.gate -cne 'ready'){throw ('Actual production readiness not verified: '+$initial.facts.environment.error.message)}
    $initial.facts.environment
   }|Out-Null
   Invoke-TwoUserSweepCheck 'path-snapshot' {
    if(!$initial){throw 'Actual path-snapshot IPC unavailable; filesystem pointers do not substitute for a snapshot'}
    if(!(Test-TwoUserSamePath $initial.facts.paths.desktopRoot.path $root) -or !(Test-TwoUserSamePath $initial.facts.paths.opencodexHome.path $nativeHome)){throw 'Actual path snapshot escaped own Profile'}
    $initial.facts.paths
   }|Out-Null
   # CLI test preference may be written only after the real product created its
   # own structure2/pointer/log. No ready/runtime state is injected here.
   $pointer=Join-Path $root 'manager-location.json';$metadata=Join-Path $root 'manager-state/data-root.json'
   $location=Get-Content -LiteralPath $pointer -Raw|ConvertFrom-Json;$meta=Get-Content -LiteralPath $metadata -Raw|ConvertFrom-Json
   if($location.schemaVersion -ne 1 -or !(Test-TwoUserSamePath $location.activeRoot (Join-Path $nativeProfile '.codex-desktop')) -or $meta.structure_version -cne '2'){throw 'Actual own pointer/structure2 prerequisite missing; no preference mutation'}
   $appLog=Join-Path $root 'logs/app.log'
   if(!(Get-Content -LiteralPath $appLog -Raw).Contains('commit='+$c.commit)){throw 'Actual own Manager source provenance differs'}
   if(!$supplyTerminal){throw 'NotReady: still-verifying supply is not interrupted for an IPC fixture'}
   Add-TwoUserDescendants $manager.fact $ledger
   # Close the Manager's owned query Job handles first. Waiting for a child
   # while its owner remains alive can retain a still-supervised generation.
   Stop-TwoUserTracked $manager.fact
   foreach($fact in @($ledger.Values|Sort-Object {[long]$_.creationTicks} -Descending)){Stop-TwoUserTracked $fact};$ledger.Clear();$manager.process.Dispose()
   $prefs=Join-Path $root 'manager-state/preferences.json'
   $document=if(Test-Path -LiteralPath $prefs){Get-Content -LiteralPath $prefs -Raw|ConvertFrom-Json -AsHashtable}else{@{schema_version=2}}
   if(!$document.Contains('appearance')){$document.appearance=@{}}
   $document.appearance.theme='system'
   # Explicit theme avoids first-run theme-import rewriting this backend-only
   # isolated CLI fixture as Windows UI cli=false. No product capability change.
   $document.cli=@{cli_enabled=$true};$document.shell=@{auto_panel=$false;launch_main=$true;launch_with_codex=$false};$document.updates=@{app_update_auto_check=$false}
   $document|ConvertTo-Json -Depth 12|Set-Content -LiteralPath $prefs -Encoding utf8
   Copy-Item -LiteralPath $prefs -Destination (Join-Path $out 'explicit-test-preferences.before-restart.json')
   $prefObservation=Get-Content -LiteralPath $prefs -Raw|ConvertFrom-Json
   if($prefObservation.cli.cli_enabled -ne $true -or $prefObservation.schema_version -ne 2 -or $prefObservation.appearance.theme -cne 'system'){throw 'Explicit isolated CLI/theme preference fixture did not roundtrip'}
   if(Test-Path -LiteralPath (Join-Path $installed 'sweep-state.exe')){
    & (Join-Path $installed 'sweep-state.exe') $root *> (Join-Path $out 'actual-preferences-loader.before-restart.log')
   }
   $manager=& $start
   $windowDeadline=[datetime]::UtcNow.AddSeconds(40);$liveWindow=$null
   do{try{$liveWindow=Get-TwoUserWindow $manager.fact}catch{Start-Sleep -Milliseconds 250}}while(!$liveWindow -and [datetime]::UtcNow -lt $windowDeadline)
   if(!$liveWindow){throw 'NotReady: actual restarted own Manager window missing'}
   $actualToken=Get-WindowsProcessToken -Process $manager.process -ExpectedExecutable $exe
   Assert-WindowsProcessMode -Observation $actualToken -Mode ordinary
   $live=@{window=$liveWindow;token=$actualToken}
  Invoke-TwoUserSweepCheck 'same-user-ipc' {
  $same=Invoke-TwoUserCli $cli @('status','--json');Write-TwoUserJson (Join-Path $out 'same-user-status.json') $same
  $data=Invoke-TwoUserCli $cli @('data-root','show','--json')
  if(!(Test-TwoUserSamePath $data.data.active_data_root $root) -or !(Test-TwoUserSamePath $data.data.opencodex_home $nativeHome)){throw 'Candidate CLI uses another D/H'}
  Write-TwoUserJson (Join-Path $out 'same-user-data-root.json') $data
  $pipe=Get-TwoUserPipeName $sid;$own=[IO.Pipes.NamedPipeClientStream]::new('.',$pipe,[IO.Pipes.PipeDirection]::InOut,[IO.Pipes.PipeOptions]::Asynchronous,[Security.Principal.TokenImpersonationLevel]::Impersonation)
  try{$connectBudget=[Diagnostics.Stopwatch]::StartNew();Wait-TwoUserDiagnosticPipe $manager.fact $pipe 5000;$own.Connect([Math]::Max(1,5000-[int]$connectBudget.ElapsedMilliseconds));[uint32]$serverId=0
   if(![OcxTwoUserWindow]::GetNamedPipeServerProcessId($own.SafePipeHandle.DangerousGetHandle(),[ref]$serverId) -or $serverId -ne $manager.fact.pid){throw 'Actual product pipe server PID differs'}
   $frame=Complete-TwoUserReadonlyPipeFrame $own
   if(!(Test-TwoUserSamePath $frame.data.active_data_root $root)){throw 'Actual owned-pipe frame returned another data root'}
   Write-TwoUserJson (Join-Path $out 'owned-pipe-frame.json') @{serverPid=$serverId;response=$frame;accepted=$false}
  }finally{$own.Dispose()}
  }|Out-Null
  $pipe=Get-TwoUserPipeName $sid
  Add-Type -AssemblyName System.Security.Cryptography.ProtectedData
  $plain=[Security.Cryptography.RandomNumberGenerator]::GetBytes(32)
  try{
   $cipher=[Security.Cryptography.ProtectedData]::Protect($plain,$null,[Security.Cryptography.DataProtectionScope]::CurrentUser)
   $ownPlain=[Security.Cryptography.ProtectedData]::Unprotect($cipher,$null,[Security.Cryptography.DataProtectionScope]::CurrentUser)
   if([Convert]::ToBase64String($plain) -cne [Convert]::ToBase64String($ownPlain)){throw 'Own DPAPI readback failed'}
   [Array]::Clear($ownPlain,0,$ownPlain.Length);[IO.File]::WriteAllBytes((Join-Path $out 'synthetic-dpapi.bin'),$cipher)
  }finally{[Array]::Clear($plain,0,$plain.Length)}

   $result.manager=$manager.fact;$result.window=$liveWindow;$result.applicationToken=$actualToken;$result.pipe=Get-TwoUserPipeName $sid
   $result.accepted=$false;$result.diagnosticOnly=$true
   $result.runtimeReady=(Get-Content -LiteralPath (Join-Path $out 'sweep-runtime-readiness.json') -Raw|ConvertFrom-Json).status -ceq 'pass'
   Write-TwoUserJson (Join-Path $out 'ready.json') $result
   $peerFile=Join-Path $out 'peer.json';$deadline=[datetime]::UtcNow.AddSeconds(720)
   while(!(Test-Path -LiteralPath $peerFile)){
    if(Test-Path -LiteralPath (Join-Path $out 'stop')){throw 'Actual peer startup incomplete'}
    if([datetime]::UtcNow -gt $deadline){throw 'Peer deadline'}
    Add-TwoUserDescendants $manager.fact $ledger;Start-Sleep -Milliseconds 250
   }
  $peer=Get-Content -LiteralPath $peerFile -Raw|ConvertFrom-Json
  if($peer.sid -ceq $sid -or $peer.pipe -cne (Get-TwoUserPipeName $peer.sid)){throw 'Peer identity differs'}
  Invoke-TwoUserSweepCheck 'peer-business-data' {
  $peerDataDenied=$false
  try{$peerFileHandle=[IO.File]::Open($peer.privateFile,'Open','Read','ReadWrite');$peerFileHandle.Dispose()}catch [UnauthorizedAccessException]{$peerDataDenied=$true}
  if(!$peerDataDenied){throw 'Other account actual business data was readable'}
  $result.peerBusinessData='access-denied'
  }|Out-Null
  Invoke-TwoUserSweepCheck 'cross-user-ipc' {
  $peerStream=[IO.Pipes.NamedPipeClientStream]::new('.',$peer.pipe,[IO.Pipes.PipeDirection]::InOut);$denied=$false
  try{$peerStream.Connect(3000)}catch [UnauthorizedAccessException]{$denied=$true}
  catch [IO.IOException]{if(($_.Exception.HResult -band 0xffff) -eq 5){$denied=$true}else{throw 'NotReady: peer pipe denial unproven'}}
  finally{$peerStream.Dispose()}
  if(!$denied){throw 'Other SID pipe accepted or denial unproven'}
  $result.crossPipe='access-denied-real-token';$result.crossPipeServer=$peer.manager.pid
  }|Out-Null
  Invoke-TwoUserSweepCheck 'cross-user-dpapi' {
  $dpapiDenied=$false;$peerBytes=[IO.File]::ReadAllBytes($peer.cipher)
  try{$unexpected=[Security.Cryptography.ProtectedData]::Unprotect($peerBytes,$null,[Security.Cryptography.DataProtectionScope]::CurrentUser);[Array]::Clear($unexpected,0,$unexpected.Length)}catch [Security.Cryptography.CryptographicException]{$dpapiDenied=$true}
  if(!$dpapiDenied){throw 'Other SID synthetic DPAPI accepted'}
  $result.dpapi='own-roundtrip-and-peer-denied-synthetic-subscope-only'
  }|Out-Null
  Invoke-TwoUserSweepCheck 'same-sid-single-instance' {
  $before=@{};foreach($f in @($pointer,$metadata)){$before[$f]=(Get-FileHash -LiteralPath $f).Hash}
  $lockFile=Join-Path $root 'manager-state/app.lock';$lockBefore=Get-Item -LiteralPath $lockFile
  $duplicateInfo=[Diagnostics.ProcessStartInfo]::new();$duplicateInfo.FileName=$exe;$duplicateInfo.WorkingDirectory=$installed;$duplicateInfo.UseShellExecute=$false
  foreach($key in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_WINDOWS_SMOKE_CDP_PORT','HOME','OPENCODEX_HOME','CODEX_HOME','WEBVIEW2_USER_DATA_FOLDER','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS')){[void]$duplicateInfo.Environment.Remove($key)}
  $duplicateProcess=[Diagnostics.Process]::Start($duplicateInfo)
  try{[void]$duplicateProcess.Handle
   if(!$duplicateProcess.WaitForExit(20000)){$dupFact=Get-TwoUserProcessFact $duplicateProcess $sid;$ledger[[int]$dupFact.pid]=$dupFact;throw 'Same-SID second Manager remained alive'}
   if($duplicateProcess.ExitCode -eq 0){throw 'Second Manager did not report rejection'}
   $duplicatePid=$duplicateProcess.Id;$duplicateExit=$duplicateProcess.ExitCode
  }finally{$duplicateProcess.Dispose()}
  $null=Assert-TwoUserLive $manager.fact
  if($c.diagnosticSweep){
   $originalHandle=[IntPtr]::new([long]$live.window.handle)
   if(![OcxTwoUserWindow]::OwnedVisible($originalHandle,$manager.fact.pid)){throw 'Exact original owned visible HWND no longer exists'}
   $after=@{pid=$manager.fact.pid;handle=$live.window.handle;ownedVisible=$true;observedUtc=[datetime]::UtcNow.ToString('o');method='same exact originalHWND user32 ownership/visibility check'}
  }else{$after=Get-TwoUserWindow $manager.fact}
  if($c.diagnosticSweep){Write-TwoUserJson (Join-Path $out 'single-instance-window-readback.json') @{original=$live.window;after=$after;manager=$manager.fact;duplicatePid=$duplicatePid;duplicateExit=$duplicateExit;accepted=$false}}
  if($after.handle -cne $live.window.handle){throw 'Original window changed'}
  foreach($f in @($pointer,$metadata)){if((Get-FileHash -LiteralPath $f).Hash -cne $before[$f]){throw 'Duplicate changed pointer/metadata'}}
  $lockAfter=Get-Item -LiteralPath $lockFile
  if($lockBefore.Length -ne $lockAfter.Length -or $lockBefore.LastWriteTimeUtc -ne $lockAfter.LastWriteTimeUtc){throw 'Duplicate changed lock metadata'}
  $result.duplicate=[ordered]@{rejected=$true;secondPid=$duplicatePid;secondExitCode=$duplicateExit;originalPid=$manager.fact.pid;originalWindowUnchanged=$true;pointerMetadataHashUnchanged=$true;lockSizeAndWriteTimeUnchanged=$true;lockContentHash='unavailable-while-byte-range-lock-held'}
  }|Out-Null

   $result.result='diagnostic-only';Write-TwoUserJson (Join-Path $out 'checked.json') $result
   if($c.verifyRuntimePorts -eq $true -and $result.runtimeReady){
    Invoke-TwoUserRuntimeSession $c $manager $ledger ($initial.facts.paths.runtimePrefix.path -replace '^\\\\\?\\','') $runtimeState
    if(!$runtimeState.completed){throw 'Actual runtime protocol not completed'}
    $result.runtimeProxyVerification='verified-runtime-ports'
   }
   while(!(Test-Path -LiteralPath (Join-Path $out 'stop'))){if([datetime]::UtcNow -gt $deadline){throw 'Controller stop deadline'};Add-TwoUserDescendants $manager.fact $ledger;Start-Sleep -Milliseconds 250}
  }else{
  # Let the real product initialize structure2 before editing isolated CLI prefs.
  $manager=& $start;$initial=& $screen $manager 'initial'
  if($initial.facts.cliEnabled -ne $false){throw 'Default CLI was enabled'}
  if($c.diagnosticSweep){
   Invoke-TwoUserSweepCheck 'private-bundle-supply' {
    if($initial.facts.paths.program.consistency -cne 'consistent' -or $initial.facts.paths.runtimeEntry.status -cne 'valid' -or $initial.facts.paths.runtimePrefix.status -cne 'valid' -or $initial.facts.runtime.kind -cne 'managed'){throw 'Actual private first-start runtime supply is not consistent/managed'}
    $initial.facts.paths.program
   }|Out-Null
   Invoke-TwoUserSweepCheck 'runtime-readiness' {
    if($initial.facts.runtime.kind -cne 'managed' -or $initial.facts.environment.value.gate -cne 'ready'){throw ('Actual production readiness refused: '+$initial.facts.environment.error.message)}
    $initial.facts.environment
   }|Out-Null
   Invoke-TwoUserSweepCheck 'path-snapshot' {if(!(Test-TwoUserSamePath $initial.facts.paths.desktopRoot.path $root) -or !(Test-TwoUserSamePath $initial.facts.paths.opencodexHome.path $nativeHome)){throw 'Actual snapshot D/H escaped current Profile'};$initial.facts.paths}|Out-Null
  }

  Add-TwoUserDescendants $manager.fact $ledger
  foreach($fact in @($ledger.Values|Sort-Object {[long]$_.creationTicks} -Descending)){Stop-TwoUserTracked $fact};$ledger.Clear();$manager.process.Dispose()
  $prefs=Join-Path $root 'manager-state/preferences.json'
  $document=if(Test-Path -LiteralPath $prefs){Get-Content -LiteralPath $prefs -Raw|ConvertFrom-Json -AsHashtable}else{@{schema_version=2}}
  if(!$document.Contains('appearance')){$document.appearance=@{}}
  $document.appearance.theme='system'
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
  Invoke-TwoUserSweepCheck 'same-user-ipc' {
  $same=Invoke-TwoUserCli $cli @('status','--json');Write-TwoUserJson (Join-Path $out 'same-user-status.json') $same
  $data=Invoke-TwoUserCli $cli @('data-root','show','--json')
  if(!(Test-TwoUserSamePath $data.data.active_data_root $root) -or !(Test-TwoUserSamePath $data.data.opencodex_home $nativeHome)){throw 'Candidate CLI uses another D/H'}
  Write-TwoUserJson (Join-Path $out 'same-user-data-root.json') $data
  $pipe=Get-TwoUserPipeName $sid;$own=[IO.Pipes.NamedPipeClientStream]::new('.',$pipe,[IO.Pipes.PipeDirection]::InOut,[IO.Pipes.PipeOptions]::Asynchronous,[Security.Principal.TokenImpersonationLevel]::Impersonation)
  try{$own.Connect(5000);[uint32]$serverId=0
   if(![OcxTwoUserWindow]::GetNamedPipeServerProcessId($own.SafePipeHandle.DangerousGetHandle(),[ref]$serverId) -or $serverId -ne $manager.fact.pid){throw 'Actual product pipe server PID differs'}
   $frame=Complete-TwoUserReadonlyPipeFrame $own
   if(!(Test-TwoUserSamePath $frame.data.active_data_root $root)){throw 'Actual owned-pipe frame returned another data root'}
   Write-TwoUserJson (Join-Path $out 'owned-pipe-frame.json') @{serverPid=$serverId;response=$frame}
  }finally{$own.Dispose()}
  }|Out-Null
  $pipe=Get-TwoUserPipeName $sid
  Add-Type -AssemblyName System.Security.Cryptography.ProtectedData
  $plain=[Security.Cryptography.RandomNumberGenerator]::GetBytes(32)
  try{
   $cipher=[Security.Cryptography.ProtectedData]::Protect($plain,$null,[Security.Cryptography.DataProtectionScope]::CurrentUser)
   $ownPlain=[Security.Cryptography.ProtectedData]::Unprotect($cipher,$null,[Security.Cryptography.DataProtectionScope]::CurrentUser)
   if([Convert]::ToBase64String($plain) -cne [Convert]::ToBase64String($ownPlain)){throw 'Own DPAPI readback failed'}
   [Array]::Clear($ownPlain,0,$ownPlain.Length);[IO.File]::WriteAllBytes((Join-Path $out 'synthetic-dpapi.bin'),$cipher)
  }finally{[Array]::Clear($plain,0,$plain.Length)}
  $result.manager=$manager.fact;$result.window=$live.window;$result.applicationToken=$live.token;$result.pipe=$pipe;$result.defaultCliDisabled=$true;$result.sameUserCli=if($c.diagnosticSweep){(Get-Content -LiteralPath (Join-Path $out 'sweep-same-user-ipc.json') -Raw|ConvertFrom-Json).status -ceq 'pass'}else{$true}
  if($c.diagnosticSweep){$result.accepted=$false;$result.diagnosticOnly=$true;$result.runtimeReady=($live.facts.paths.program.consistency -ceq 'consistent' -and $live.facts.runtime.kind -ceq 'managed' -and $live.facts.environment.value.gate -ceq 'ready')}
  Write-TwoUserJson (Join-Path $out 'ready.json') $result
  $peerFile=Join-Path $out 'peer.json';$deadline=[datetime]::UtcNow.AddSeconds(720)
  while(!(Test-Path -LiteralPath $peerFile)){
   if(Test-Path -LiteralPath (Join-Path $out 'stop')){throw 'Peer startup incomplete'}
   if([datetime]::UtcNow -gt $deadline){throw 'Peer deadline'}
   Add-TwoUserDescendants $manager.fact $ledger;Start-Sleep -Milliseconds 250
  }
  $peer=Get-Content -LiteralPath $peerFile -Raw|ConvertFrom-Json
  if($peer.sid -ceq $sid -or $peer.pipe -cne (Get-TwoUserPipeName $peer.sid)){throw 'Peer identity differs'}
  Invoke-TwoUserSweepCheck 'peer-business-data' {
  $peerDataDenied=$false
  try{$peerFileHandle=[IO.File]::Open($peer.privateFile,'Open','Read','ReadWrite');$peerFileHandle.Dispose()}catch [UnauthorizedAccessException]{$peerDataDenied=$true}
  if(!$peerDataDenied){throw 'Other account actual business data was readable'}
  $result.peerBusinessData='access-denied'
  }|Out-Null
  Invoke-TwoUserSweepCheck 'cross-user-ipc' {
  $peerStream=[IO.Pipes.NamedPipeClientStream]::new('.',$peer.pipe,[IO.Pipes.PipeDirection]::InOut);$denied=$false
  try{$peerStream.Connect(3000)}catch [UnauthorizedAccessException]{$denied=$true}
  catch [IO.IOException]{if(($_.Exception.HResult -band 0xffff) -eq 5){$denied=$true}else{throw 'NotReady: peer pipe denial unproven'}}
  finally{$peerStream.Dispose()}
  if(!$denied){throw 'Other SID pipe accepted or denial unproven'}
  $result.crossPipe='access-denied-real-token';$result.crossPipeServer=$peer.manager.pid
  }|Out-Null
  Invoke-TwoUserSweepCheck 'cross-user-dpapi' {
  $dpapiDenied=$false;$peerBytes=[IO.File]::ReadAllBytes($peer.cipher)
  try{$unexpected=[Security.Cryptography.ProtectedData]::Unprotect($peerBytes,$null,[Security.Cryptography.DataProtectionScope]::CurrentUser);[Array]::Clear($unexpected,0,$unexpected.Length)}catch [Security.Cryptography.CryptographicException]{$dpapiDenied=$true}
  if(!$dpapiDenied){throw 'Other SID synthetic DPAPI accepted'}
  $result.dpapi='own-roundtrip-and-peer-denied-synthetic-subscope-only'
  }|Out-Null
  Invoke-TwoUserSweepCheck 'same-sid-single-instance' {
  $before=@{};foreach($f in @($pointer,$metadata)){$before[$f]=(Get-FileHash -LiteralPath $f).Hash}
  $lockFile=Join-Path $root 'manager-state/app.lock';$lockBefore=Get-Item -LiteralPath $lockFile
  $duplicateInfo=[Diagnostics.ProcessStartInfo]::new();$duplicateInfo.FileName=$exe;$duplicateInfo.WorkingDirectory=$installed;$duplicateInfo.UseShellExecute=$false
  foreach($key in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_WINDOWS_SMOKE_CDP_PORT','HOME','OPENCODEX_HOME','CODEX_HOME','WEBVIEW2_USER_DATA_FOLDER','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS')){[void]$duplicateInfo.Environment.Remove($key)}
  $duplicateProcess=[Diagnostics.Process]::Start($duplicateInfo)
  try{[void]$duplicateProcess.Handle
   if(!$duplicateProcess.WaitForExit(20000)){$dupFact=Get-TwoUserProcessFact $duplicateProcess $sid;$ledger[[int]$dupFact.pid]=$dupFact;throw 'Same-SID second Manager remained alive'}
   if($duplicateProcess.ExitCode -eq 0){throw 'Second Manager did not report rejection'}
   $duplicatePid=$duplicateProcess.Id;$duplicateExit=$duplicateProcess.ExitCode
  }finally{$duplicateProcess.Dispose()}
  $null=Assert-TwoUserLive $manager.fact;$after=Get-TwoUserWindow $manager.fact
  if($c.diagnosticSweep){Write-TwoUserJson (Join-Path $out 'single-instance-window-readback.json') @{original=$live.window;after=$after;manager=$manager.fact;duplicatePid=$duplicatePid;duplicateExit=$duplicateExit;accepted=$false}}
  if($after.handle -cne $live.window.handle){throw 'Original window changed'}
  foreach($f in @($pointer,$metadata)){if((Get-FileHash -LiteralPath $f).Hash -cne $before[$f]){throw 'Duplicate changed pointer/metadata'}}
  $lockAfter=Get-Item -LiteralPath $lockFile
  if($lockBefore.Length -ne $lockAfter.Length -or $lockBefore.LastWriteTimeUtc -ne $lockAfter.LastWriteTimeUtc){throw 'Duplicate changed lock metadata'}
  $result.duplicate=[ordered]@{rejected=$true;secondPid=$duplicatePid;secondExitCode=$duplicateExit;originalPid=$manager.fact.pid;originalWindowUnchanged=$true;pointerMetadataHashUnchanged=$true;lockSizeAndWriteTimeUnchanged=$true;lockContentHash='unavailable-while-byte-range-lock-held'}
  }|Out-Null
  $result.result=if($c.diagnosticSweep){'diagnostic-only'}else{'pass-user-isolation'};Write-TwoUserJson (Join-Path $out 'checked.json') $result
  if($c.verifyRuntimePorts -eq $true -and (!$c.diagnosticSweep -or $result.runtimeReady)){
   Invoke-TwoUserRuntimeSession $c $manager $ledger ($live.facts.paths.runtimePrefix.path -replace '^\\\\\?\\','') $runtimeState
   if(!$runtimeState.completed){throw 'Runtime protocol not completed'}
   $result.runtimeProxyVerification='verified-runtime-ports'
  }
  while(!(Test-Path -LiteralPath (Join-Path $out 'stop'))){if([datetime]::UtcNow -gt $deadline){throw 'Controller stop deadline'};Add-TwoUserDescendants $manager.fact $ledger;Start-Sleep -Milliseconds 250}
  }
 }catch{
  $result.failureKind=$_.Exception.GetType().FullName;$result.failureLine=$_.InvocationInfo.ScriptLineNumber
  if($c.diagnosticSweep){$result.actualFailure=($_.Exception.Message -split "\r?\n",2)[0]}
  $result.failureCode=if($_.Exception.Message.StartsWith('NotReady:')){$_.Exception.Message}else{'Native identity/UI/IPC/cleanup assertion failed; inspect stage and retained evidence'}
  $result.result=if($_.Exception.Message.StartsWith('NotReady:')){'not-ready'}else{'fail'}
  Write-TwoUserJson (Join-Path $out 'failure.json') $result
 }finally{
  if($manager){try{Add-TwoUserDescendants $manager.fact $ledger}catch{}}
  $cleanup=@();foreach($fact in @($ledger.Values|Sort-Object {[long]$_.creationTicks} -Descending)){
   try{Stop-TwoUserTracked $fact;$cleanup+=@{process=$fact;result='stopped-or-exited'}}catch{$cleanup+=@{process=$fact;result='retained-unproven'};$result.result='cleanup-failed'}
  }
  $result.cleanup=$cleanup
  if($c.diagnosticSweep){foreach($stage in @('private-bundle-supply','runtime-readiness','path-snapshot','same-user-ipc','peer-business-data','cross-user-ipc','cross-user-dpapi','same-sid-single-instance')){
   $path=Join-Path $out ('sweep-'+$stage+'.json')
   if(!(Test-Path -LiteralPath $path)){Write-TwoUserJson $path @{name=$stage;status='fail';firstError=('Unmet actual identity/startup prerequisite: '+$result.failureCode);accepted=$false;diagnosticOnly=$true}}
  }}
  $logDirectory=Join-Path $root 'logs'
  if(Test-Path -LiteralPath $logDirectory){
   $retainedLogs=Join-Path $out 'application-logs';New-Item -ItemType Directory -Path $retainedLogs|Out-Null
   foreach($file in @(Get-ChildItem -LiteralPath $logDirectory -File -Filter '*.log'|Select-Object -First 32)){
    if($file.Length -le 8MB -and !($file.Attributes -band [IO.FileAttributes]::ReparsePoint)){Copy-Item -LiteralPath $file.FullName -Destination (Join-Path $retainedLogs $file.Name)}
   }
  }
  if($c.diagnosticSweep -and (Test-Path -LiteralPath (Join-Path $root 'manager-state/preferences.json'))){Copy-Item -LiteralPath (Join-Path $root 'manager-state/preferences.json') -Destination (Join-Path $out 'explicit-test-preferences.after-checks.json')}
  if($c.diagnosticSweep -and (Test-Path -LiteralPath (Join-Path $installed 'sweep-state.exe'))){
   try{& (Join-Path $installed 'sweep-state.exe') $root *> (Join-Path $out 'actual-preferences-loader.after-checks.log')}catch{}
  }
  if($c.diagnosticSweep -and (Test-Path -LiteralPath (Join-Path $installed 'sweep-filesystem.exe'))){
   try{& (Join-Path $installed 'sweep-filesystem.exe') $installed $root *> (Join-Path $out 'filesystem-observations.log')}catch{$result.filesystemObserverError=$_.Exception.GetType().FullName}
  }
  $programDiagnostics=Join-Path $out 'program-diagnostics';New-Item -ItemType Directory -Force $programDiagnostics|Out-Null
  foreach($relative in @('.runtime-bundle-failure.json','.runtime-bundle-pending.json','runtime-bundle/runtime-bundle.manifest.json','runtime/opencodex/.runtime-manifest.json')){
   $path=Join-Path $installed $relative
   if(Test-Path -LiteralPath $path){$file=Get-Item -LiteralPath $path;if(!$file.PSIsContainer -and $file.Length -le 1MB -and !($file.Attributes -band [IO.FileAttributes]::ReparsePoint)){Copy-Item -LiteralPath $path -Destination (Join-Path $programDiagnostics ($relative.Replace('/','_').TrimStart('.')))}}
  }
  try{$result.finalInventory=Get-TwoUserTreeHash (Join-Path $nativeProfile '.codex-desktop');$result.inventory='hashed'}catch{$result.inventory='unavailable';if($result.result -ceq 'pass-user-isolation'){$result.result='evidence-incomplete'}}
  Close-TwoUserProcessHandles
  Write-TwoUserJson (Join-Path $out 'finished.json') $result
  if($manager){$manager.process.Dispose()}
 }
 if($result.result -cne 'pass-user-isolation'){exit 1};exit 0
}
if($Mode -ceq 'Child'){Invoke-TwoUserChild $ConfigPath}

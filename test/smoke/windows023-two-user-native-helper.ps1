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
 try{
  # A retained Process object can observe termination between enumeration and
  # identity query. An exited object needs no kill; a still-live unknown object
  # remains a refusal, including a reused PID or another SID.
  if($p.HasExited){return}
  try{$now=Get-TwoUserProcessFact $p $Fact.sid}catch{if($p.HasExited){return};throw}
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
 ConvertTo-Json -InputObject $Value -Depth 15|Set-Content -LiteralPath $temp -Encoding utf8
 [IO.File]::Move($temp,$Path,$true)
}
function Get-TwoUserRuntimeDriverText {
@'
let runtimeStage='command';let runtimePublic={};
function runtimeFailureCode(error){
 const codes=new Map([['Actual service not healthy within production start budget','runtime_start_not_healthy'],['B claimed running on peer port','peer_port_claimed_running'],['B published an owned service record on A port','peer_port_owned_record'],['No explicit production port-conflict diagnostic','port_conflict_unproven'],['Manager did not confirm stop','manager_stop_not_confirmed'],['Stop did not clear own service record/state','runtime_stop_incomplete'],['Production attestation failed','runtime_attestation_failed'],['Runtime record changed','runtime_record_changed'],['Public manager command failed','public_manager_command_rejected']]);
 return codes.get(error?.message)||'runtime_binding_or_transport_failed';
}
const runtimeSleep=ms=>new Promise(r=>setTimeout(r,ms));
function fixtureConfig(port){assert(Number.isInteger(port)&&port>=1024&&port<=65535);return {port,hostname:'127.0.0.1',providers:{}};}
function publicRecord(record){
 assert(Number.isInteger(record.pid)&&record.pid>0&&Number.isInteger(record.port)&&record.port>=1024&&record.port<=65535,'Runtime public identity invalid');
 return {pid:record.pid,port:record.port};
}
function proofMatches(secret,challenge,pid,port,proof){
 if(typeof secret!=='string'||secret.length!==43||typeof proof!=='string')return false;
 const expected=createHmac('sha256',secret).update('opencodex-local-management-v1\n'+challenge+'\n'+pid+'\n'+port).digest();
 const received=Buffer.from(proof,'base64url');return received.length===expected.length&&timingSafeEqual(received,expected);
}
function runtimeSelfTest(){
 const secret=Buffer.alloc(32,7).toString('base64url'),challenge='fixture';
 const proof=createHmac('sha256',secret).update('opencodex-local-management-v1\nfixture\n42\n54321').digest('base64url');
 assert(proofMatches(secret,challenge,42,54321,proof));assert(!proofMatches(secret,challenge,43,54321,proof));
 assert(!proofMatches(secret,challenge,42,54322,proof));assert(!proofMatches(secret,challenge,42,54321,'bad'));
 assert.deepEqual(publicRecord({pid:42,port:54321,attestationSecret:secret}),{pid:42,port:54321});
 assert(!JSON.stringify(publicRecord({pid:42,port:54321,attestationSecret:secret})).includes(secret));
 assert.deepEqual(fixtureConfig(54321),{port:54321,hostname:'127.0.0.1',providers:{}});assert.throws(()=>fixtureConfig(0));
 assert.notEqual(Buffer.from(JSON.stringify(fixtureConfig(54321)),'utf8')[0],0xef);
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
 const health=async()=>{
  const r=await record();assert(r&&r.port===c.port&&(!c.expectedPid||r.pid===c.expectedPid),'Runtime record changed');
  const challenge=randomBytes(32).toString('base64url');
  // Literal loopback, no proxies/redirects, no admin key or inference request.
  const response=await new Promise((accept,reject)=>{
   const req=loopbackRequest({hostname:'127.0.0.1',port:r.port,path:'/healthz',method:'GET',headers:{'x-opencodex-attestation-challenge':challenge}},res=>{
    let bytes=0;const chunks=[];res.on('data',chunk=>{bytes+=chunk.length;if(bytes>65536){res.destroy();reject(Error('Bounded health required'));}else chunks.push(chunk);});
    res.on('error',reject);res.on('end',()=>accept({status:res.statusCode,proof:res.headers['x-opencodex-attestation-proof'],body:Buffer.concat(chunks)}));
   });req.on('error',reject);req.setTimeout(5000,()=>req.destroy(Error('Loopback health deadline')));req.end();
  });
  assert(response.status===200&&proofMatches(r.attestationSecret,challenge,r.pid,r.port,response.proof),'Production attestation failed');
  const h=JSON.parse(response.body.toString('utf8'));assert(h.service==='opencodex'&&h.pid===r.pid&&h.port===r.port,'Health identity changed');
  const again=await record();assert(again&&again.pid===r.pid&&again.port===r.port&&again.attestationSecret===r.attestationSecret,'Runtime binding changed');
  return {record:publicRecord(r),attestation:'verified-production-challenge',healthService:h.service,version:typeof h.version==='string'?h.version:null};
 };
 const action=async action=>{
  await bound();const r=await runtimeInvoke('process_action',{request:{action,confirm:true}},action==='stop'?15000:35000);
  const value={lifecycleState:r.lifecycleState,result:r.result};runtimePublic.action=value;return value;
 };
 const statusConflict=s=>/conflict|port[_ -]?(in[_ -]?use|occupied)/i.test(String(s.protection))||s.runtime==='external_takeover';
 const diagnosticConflict=async()=>{
  // Read bounded own logs in memory; serialize only a boolean marker, never lines.
  for(const directory of [join(c.home,'logs'),join(c.root,'logs')]){
   let names;try{names=await readdir(directory);}catch(e){if(e.code==='ENOENT')continue;throw e;}
   for(const name of names.filter(n=>/\.log(?:\.1)?$/.test(n)).slice(0,32)){
    const path=join(directory,name),st=await lstat(path);if(!st.isFile()||st.isSymbolicLink()||st.size>8*1024*1024)continue;
    const text=await readFile(path,'utf8');if(/EADDRINUSE|端口.{0,30}(占用|冲突)|port.{0,60}(in use|conflict|occupied)/i.test(text))return true;
   }
  }return false;
 };
 runtimeStage='binding';await bound();
 let result={result:'pass-runtime-step',mode:c.mode,home:c.home,root:c.root,port:c.port};
 if(c.mode==='start'||c.mode==='conflict'){
  runtimeStage='own-config';assert(!(await record()),'Fresh service record required');
  for(const path of [process.env.USERPROFILE,join(process.env.USERPROFILE,'.codex-desktop'),join(process.env.USERPROFILE,'.codex-desktop/opencodex'),c.home]){
   const st=await lstat(path);assert(st.isDirectory()&&!st.isSymbolicLink(),'Own H has a link');
  }
  const configPath=join(c.home,'config.json'),temp=join(c.home,'config.json.two-user-'+randomBytes(12).toString('hex'));
  let old;try{const st=await lstat(configPath);assert(st.isFile()&&!st.isSymbolicLink()&&st.size<512*1024);old=JSON.parse(await readFile(configPath,'utf8'));}catch(e){if(e.code!=='ENOENT')throw e;}
  assert(!old||!old.providers||Object.keys(old.providers).length===0,'Fixture H contains providers');
  const handle=await open(temp,'wx',0o600);
  try{await handle.writeFile(JSON.stringify(fixtureConfig(c.port))+'\n','utf8');await handle.sync();}finally{await handle.close();}
  try{await rename(temp,configPath);}catch(e){await unlink(temp).catch(()=>{});throw e;}
  result.config={path:configPath,providers:0,utf8Bom:false,atomicRename:true};
  runtimeStage='manager-start';result.action=await action('start');
  const until=Date.now()+30000;let last;
  if(c.mode==='conflict'){
   let marker=false;
   do{assert(!(await record()),'B published an owned service record on A port');last=await snapshot();
    assert(last.runtime!=='running','B claimed running on peer port');marker=marker||statusConflict(last)||await diagnosticConflict();await runtimeSleep(250);
   }while(Date.now()<until);
   assert(marker,'No explicit production port-conflict diagnostic');
   result.conflict={rejected:true,ownedRecordAbsent:true,diagnosticMarker:true};result.status=last;
  }else{
   runtimeStage='wait-owned-health';let observed;
   do{try{observed=await health();last=await snapshot();if(last.runtime==='running'&&last.health==='healthy'&&Number(last.pid)===observed.record.pid&&last.port===c.port)break;}catch{}await runtimeSleep(250);}while(Date.now()<until);
   assert(observed&&last?.runtime==='running'&&last.health==='healthy'&&Number(last.pid)===observed.record.pid&&last.port===c.port,'Actual service not healthy within production start budget');
   result={...result,...observed,status:last};
  }
 }else if(c.mode==='observe'){
  runtimeStage='observe';result={...result,...await health(),status:await snapshot()};
  assert(result.status.runtime==='running'&&result.status.health==='healthy'&&Number(result.status.pid)===result.record.pid&&result.status.port===c.port,'Actual running state lost');
 }else{
  runtimeStage='before-stop';result.before=await health();
  runtimeStage='manager-stop';result.action=await action('stop');assert(result.action.result==='stopped','Manager did not confirm stop');
  const until=Date.now()+10000;let last;
  do{last=await snapshot();if(!(await record())&&last.runtime!=='running')break;await runtimeSleep(250);}while(Date.now()<until);
  assert(!(await record())&&last.runtime!=='running','Stop did not clear own service record/state');result.status=last;result.stopped=true;
 }
 runtimeStage='final-binding';await bound();
 await writeFile(join(output,'runtime-step.json'),JSON.stringify(result,null,2)+'\n');
}
'@
}
function Get-TwoUserDriverText {
$text=@'
import assert from 'node:assert/strict';
import {writeFile,readFile,open,rename,lstat,readdir,unlink} from 'node:fs/promises';
import {createHmac,randomBytes,timingSafeEqual} from 'node:crypto';
import {request as loopbackRequest} from 'node:http';
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
 if(commandFile){await runtimeCommand(JSON.parse(await readFile(commandFile,'utf8')));}
 else {
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
 }
}catch(error){
 if(commandFile){await writeFile(join(output,'runtime-step.json'),JSON.stringify({result:'fail',failureCode:runtimeFailureCode(error),stage:runtimeStage,publicFacts:runtimePublic})+'\n');process.exitCode=1;}else{throw error;}
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
function Get-TwoUserFreeRuntimePort([int[]]$Excluded) {
 for($attempt=0;$attempt -lt 20;$attempt++){
  $socket=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0)
  try{$socket.Start();$chosen=[int]$socket.LocalEndpoint.Port}finally{$socket.Stop()}
  if($chosen -ge 1024 -and $chosen -notin $Excluded -and !(Get-NetTCPConnection -State Listen -LocalPort $chosen -ErrorAction SilentlyContinue)){return $chosen}
 }
 throw 'No free ephemeral fixture port; do not evict any listener'
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
  while(!$driver.HasExited){
   Add-TwoUserDescendants $App.fact $Ledger
   Write-TwoUserJson (Join-Path $Context.evidence 'runtime-tracked.json') @($Ledger.Values)
   if([datetime]::UtcNow -gt $deadline){throw 'Runtime driver transport deadline'}
   Start-Sleep -Milliseconds 100
  }
  # Raw errors can contain runtime-record material. Retain controlled stage only.
  $reply=Join-Path $dir 'runtime-step.json'
  if(!(Test-Path -LiteralPath $reply)){throw 'Runtime driver returned no controlled receipt'}
  $value=Get-Content -LiteralPath $reply -Raw|ConvertFrom-Json
  if($driver.ExitCode -ne 0 -or $value.result -cne 'pass-runtime-step'){throw ('Runtime step failed: '+$Label)}
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
  $expectedImages=@((Join-Path $runtimeRoot 'node/node.exe'),(Join-Path $runtimeRoot 'bun/bun.exe'))
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
    }else{Get-TwoUserFreeRuntimePort @([int]$command.peerPort,[int]$App.port)}
    $observed=Invoke-TwoUserRuntimeDriver $Context $App $Ledger $step $port 0 ([string]$index)
    if($step -ceq 'conflict'){
     $listeners=@(Get-NetTCPConnection -State Listen -LocalPort $port -ErrorAction SilentlyContinue)
     if(!$observed.conflict.rejected -or !$listeners.Count -or @($listeners|Where-Object {$_.OwningProcess -ne [int]$command.peerPid}).Count){throw 'Conflict changed or replaced peer listener'}
     $reply.conflict=$observed;$reply.peerListenerUnchanged=$true
    }else{
     $State.service=Get-TwoUserOwnedRuntime $Context $App $Ledger $observed.record $RuntimePrefix $null
     $reply.service=$State.service;$reply.health=$observed
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
 $ledger=[Collections.Generic.Dictionary[int,object]]::new();$manager=$null;$runtimeState=@{service=$null;completed=$false;cooperativeStopFailed=$false}
 $result=[ordered]@{ephemeralGuard=$c.ephemeralGuard;sid=$sid;profile=$nativeProfile;processHome=$env:USERPROFILE;localAppData=$env:LOCALAPPDATA;desktopRoot=$root;opencodexHome=$nativeHome;installation=$installed;result='fail';runtimeProxyVerification='not-verified';verifyRuntimePorts=($c.verifyRuntimePorts -eq $true);sandbox=$false}
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
  if($c.verifyRuntimePorts -eq $true){
   Invoke-TwoUserRuntimeSession $c $manager $ledger ($live.facts.paths.runtimePrefix.path -replace '^\\\\\?\\','') $runtimeState
   if(!$runtimeState.completed){throw 'Runtime protocol not completed'}
   $result.runtimeProxyVerification='verified-runtime-ports'
  }
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

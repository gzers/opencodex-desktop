param([Parameter(Mandatory)][string]$Root)
$ErrorActionPreference='Stop'
$publicRoot=[IO.Path]::GetFullPath($Root)
if($publicRoot -cne 'C:\Users\Public\ocx-r7-private-native-20261118-r1'){throw 'exact prepared privateV2 task root required'}
$privateRoot='C:\Users\marenben\ocx024-r7-private-native-20261118-r1'
$out=Join-Path $publicRoot 'out01'
$utf8=New-Object Text.UTF8Encoding($false)
function WriteOnce([string]$Name,$Value){
 $path=Join-Path $out $Name
 $bytes=$utf8.GetBytes(($Value|ConvertTo-Json -Depth 16))
 $file=[IO.File]::Open($path,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try{$file.Write($bytes,0,$bytes.Length);$file.Flush($true)}finally{$file.Dispose()}
}
if((Test-Path -LiteralPath (Join-Path $out 'intent.json')) -or (Test-Path -LiteralPath (Join-Path $out 'result.json'))){throw 'immutable round already entered; do not retry'}
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
$isAdmin=([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
$session=[Diagnostics.Process]::GetCurrentProcess().SessionId
if($identity.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or $isAdmin -or $session -ne 7 -or $env:USERPROFILE -cne 'C:\Users\marenben'){throw 'exact ordinary marenben session7/Profile required; no elevation or impersonation'}
function PathFacts{
 @('User','Machine')|ForEach-Object{
  $value=[Environment]::GetEnvironmentVariable('Path',$_);$digest=$null
  if($null -ne $value){$hash=[Security.Cryptography.SHA256]::Create();try{$digest=([BitConverter]::ToString($hash.ComputeHash([Text.Encoding]::Unicode.GetBytes($value)))).Replace('-','').ToLowerInvariant()}finally{$hash.Dispose()}}
  @{scope=$_;present=$null -ne $value;sha256=$digest}
 }|ConvertTo-Json -Compress
}
function NewPrivate([string]$Path){
 if($Path -cnotin @($privateRoot,(Join-Path $privateRoot 'I'),(Join-Path $privateRoot 'D'),(Join-Path $privateRoot 'H'))){throw 'exact new own task directory required'}
 if(Test-Path -LiteralPath $Path){throw 'task path already exists; no replace/adopt'}
 $security=New-Object Security.AccessControl.DirectorySecurity
 $security.SetSecurityDescriptorSddlForm(('O:{0}D:P(A;OICI;FA;;;{0})(A;OICI;FA;;;SY)' -f $identity.User.Value))
 [void][IO.Directory]::CreateDirectory($Path,$security)
 if((Get-Item -LiteralPath $Path -Force).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'new task directory reparse refused'}
}
function SnapshotBytes([string]$Path){
 $source=[IO.File]::Open($Path,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
 $buffer=New-Object IO.MemoryStream
 try{if($source.Length -gt 4194304){throw 'task raw log exceeds 4MiB bound'};$source.CopyTo($buffer);if($buffer.Length -gt 4194304){throw 'task raw log snapshot exceeds bound'};return ,$buffer.ToArray()}finally{$source.Dispose();$buffer.Dispose()}
}
$children=New-Object 'Collections.Generic.List[object]'
function SpawnExact([string]$Name,[string]$Arguments,[byte[]]$Payload){
 $info=New-Object Diagnostics.ProcessStartInfo
 $info.FileName=Join-Path $privateRoot 'I/ocxd.exe';$info.Arguments=$Arguments;$info.WorkingDirectory=$privateRoot
 $info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.RedirectStandardInput=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
 $process=New-Object Diagnostics.Process;$process.StartInfo=$info
 $stdout=[IO.File]::Open((Join-Path $out ($Name+'.stdout.log')),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 $stderr=[IO.File]::Open((Join-Path $out ($Name+'.stderr.log')),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 $record=[pscustomobject]@{Name=$Name;Process=$process;Out=$stdout;Err=$stderr;CopyOut=$null;CopyErr=$null;CreationFiletime=$null;ExpectedImage=$info.FileName;Arguments=$Arguments;Closed=$false}
 $children.Add($record)
 if(!$process.Start()){throw 'exact own process creation failed'}
 $record.CreationFiletime=$process.StartTime.ToUniversalTime().ToFileTimeUtc().ToString()
 $record.CopyOut=$process.StandardOutput.BaseStream.CopyToAsync($stdout);$record.CopyErr=$process.StandardError.BaseStream.CopyToAsync($stderr)
 WriteOnce ($Name+'-creation.json') @{pid=$process.Id;creationFiletime=$record.CreationFiletime;exactImage=$info.FileName;arguments=$Arguments;nativeAuthority=$false;ownTaskOnly=$true}
 $stream=$process.StandardInput.BaseStream;$length=[BitConverter]::GetBytes([uint32]$Payload.Length);$stream.Write($length,0,4);$stream.Write($Payload,0,$Payload.Length);$stream.Flush();$process.StandardInput.Close()
 return $record
}
function CloseExited($Record){
 if(!$Record.Process.HasExited){throw 'only exact exited own process may close here'}
 $Record.Process.WaitForExit();[void]$Record.CopyOut.GetAwaiter().GetResult();[void]$Record.CopyErr.GetAwaiter().GetResult();$Record.Out.Dispose();$Record.Err.Dispose();$Record.Closed=$true
}
$before=PathFacts
$result=[ordered]@{scope='real ordinary desktop same-image V2 empty endpoint, no Runtime/Owner/capture or official execution';accepted=$false;publicStartOpen=$false;productionAccepted=$false;normalStop=$false;closed=$false;currentR2Touched=$false;sid=$identity.User.Value;session=$session;elevated=$isAdmin;privateRoot=$privateRoot;taskFilesRetained=$true;children=@();firstError=$null}
WriteOnce 'intent.json' @{time=[DateTimeOffset]::Now.ToString('o');scope=$result.scope;privateRoot=$privateRoot;currentR2Touched=$false;noOfficial=$true;ordinaryDesktop=$true}
try{
 $manifest=Get-Content -LiteralPath (Join-Path $publicRoot 'input-manifest.json') -Raw -Encoding UTF8|ConvertFrom-Json
 if($manifest.scope -cne 'isolated-private-control-v2-empty-fixture' -or $manifest.expectedSid -cne $identity.User.Value -or $manifest.expectedSession -ne $session){throw 'prepared input scope/token/session mismatch'}
 if(@($manifest.files).Count -ne 1 -or $manifest.files[0].name -cne 'ocxd.exe'){throw 'exact single native broker input required'}
 $source=Join-Path $publicRoot 'input/ocxd.exe';$item=Get-Item -LiteralPath $source -Force
 if(($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $item.Length -ne $manifest.files[0].bytes -or (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -cne $manifest.files[0].sha256){throw 'pinned native input bytes differ'}
 NewPrivate $privateRoot;foreach($name in @('I','D','H')){NewPrivate (Join-Path $privateRoot $name)}
 $image=Join-Path $privateRoot 'I/ocxd.exe';[IO.File]::Copy($source,$image,$false)
 if((Get-FileHash -LiteralPath $image -Algorithm SHA256).Hash.ToLowerInvariant() -cne $manifest.files[0].sha256){throw 'native image task copy differs'}
 $bootstrap=@{schema=1;scope=$manifest.scope;expectedSid=$identity.User.Value;expectedSession=$session;profile=$env:USERPROFILE;installation=(Join-Path $privateRoot 'I');data=(Join-Path $privateRoot 'D');home=(Join-Path $privateRoot 'H');brokerSha256=$manifest.files[0].sha256;managerSha256=$manifest.files[0].sha256}
 $payload=$utf8.GetBytes(($bootstrap|ConvertTo-Json -Compress));if($payload.Length -eq 0 -or $payload.Length -gt 65536){throw 'bounded inherited bootstrap required'}
 $hostRecord=SpawnExact 'host' '--internal-runtime-owner024' $payload
 $locator=Join-Path $privateRoot 'D/before-resume-private-control-v2.json';$nativeLog=Join-Path $privateRoot 'D/before-resume-private-control-v2-events.jsonl'
 $wait=[Diagnostics.Stopwatch]::StartNew()
 $initialLog=''
 while(!$initialLog.EndsWith([string][char]10)){
  if($hostRecord.Process.HasExited){CloseExited $hostRecord;throw 'original empty host exited before native endpoint log; preserve first logs'}
  if($wait.Elapsed.TotalSeconds -ge 15){throw 'native endpoint observation timeout; keep exact own host, do not retry'}
  if(Test-Path -LiteralPath $nativeLog){$initialLog=$utf8.GetString((SnapshotBytes $nativeLog))}
  if(!$initialLog.EndsWith([string][char]10)){Start-Sleep -Milliseconds 50}
 }
 $initialLines=@($initialLog.Split([char]10)|Where-Object {$_.Trim()}|ForEach-Object {$_|ConvertFrom-Json})
 if($initialLines.Count -ne 1 -or $initialLines[0].stage -cne 'v2-original-endpoint-created-before-capture' -or $initialLines[0].facts.originalOwnerCreated -ne $false -or $initialLines[0].facts.noOuterJobObservedAtAdmission -ne $true){throw 'actual native empty/no-Job endpoint facts unconfirmed'}
 $broker=$initialLines[0].facts.broker
 if($broker.pid -ne $hostRecord.Process.Id -or $broker.creationFiletime -cne $hostRecord.CreationFiletime -or $broker.sid -cne $identity.User.Value -or $broker.sessionId -ne $session -or $broker.elevated -ne $false -or $broker.elevationType -ne 1){throw 'actual original broker native identity does not match creation'}
 $commands=@(@('diagnostic1','read-worker-diagnostic'),@('history1','read-canonical-history'),@('abort1','begin-canonical-abort'),@('diagnostic2','read-worker-diagnostic'))
 foreach($row in $commands){
  $arguments='--internal-before-resume-control024-v2 '+$row[1];if($row[1] -ceq 'begin-canonical-abort'){$arguments+=' --confirm'}
  $child=SpawnExact $row[0] $arguments $payload
  if(!$child.Process.WaitForExit(15000)){throw 'own native client observation exceeded15s; keep exact child/host, no restart'}
  CloseExited $child
  $raw=$child.Process.ExitCode;$reply=Get-Content -LiteralPath (Join-Path $out ($row[0]+'.stdout.log')) -Raw -Encoding UTF8|ConvertFrom-Json
  if($raw -ne 1 -or $reply.schema -cne 'windows-before-resume-private-control-v2' -or $reply.purpose -cne 'refused' -or $reply.error.detail -cne 'original runtime not yet bound; no Owner authority' -or $reply.normalStop -ne $false -or $reply.closed -ne $false -or $reply.allAccepted -ne $false -or $reply.publicStartOpen -ne $false -or $reply.hostMayExit -ne $false){throw 'empty V2 client must receive exact authenticated refusal, no Owner authority'}
  WriteOnce ($row[0]+'-verified.json') @{raw=$raw;requestId=$reply.requestId;scopeDigest=$reply.scopeDigest;exactEmptyRuntimeRefusal=$true;nativeAuthority=$false;originalBudgetUnchanged=$true}
 }
 $logBytes=SnapshotBytes $nativeLog;$lines=@($utf8.GetString($logBytes).Split([char]10)|Where-Object {$_.Trim()}|ForEach-Object {$_|ConvertFrom-Json})
 $intents=@($lines|Where-Object {$_.stage -ceq 'v2-original-request-intent'});$responses=@($lines|Where-Object {$_.stage -ceq 'v2-original-request-result'})
 if($intents.Count -ne 4 -or $responses.Count -ne 4){throw 'all4 independent native requests must reach original authenticated server once'}
 foreach($entry in $intents){if($entry.facts.bothNoOuterJobObservedAtAdmission -ne $true -or $entry.facts.nativePeerAndLocatorRechecked -ne $true -or $entry.facts.peer.sid -cne $identity.User.Value -or $entry.facts.peer.sessionId -ne $session -or $entry.facts.peer.elevated -ne $false -or $entry.facts.peer.elevationType -ne 1 -or $entry.facts.peer.imageFileId -cne $broker.imageFileId -or $entry.facts.peer.imageVolume -ne $broker.imageVolume){throw 'actual native same-image ordinary peer/server facts differ'}}
 if(@($intents|ForEach-Object {$_.facts.requestId}|Select-Object -Unique).Count -ne 4){throw 'native request replay/identity collision'}
 $dMembers=@(Get-ChildItem -LiteralPath (Join-Path $privateRoot 'D') -Force|Select-Object -ExpandProperty Name|Sort-Object)
 if(($dMembers -join '|') -cne 'before-resume-private-control-v2-events.jsonl|before-resume-private-control-v2.json'){throw 'empty test unexpectedly created Runtime/Owner/canonical data'}
 $result.nativeBothPeersOrdinarySameImageNoJob=$true;$result.fourRequestsRefusedExactly=$true;$result.originalOwnerCreated=$false;$result.nativeBroker=$broker
 # Exact creation handle of this NEW noRuntime fixture only. Separate raw242
 # disposition is never recovery/Stop/Closed. Failure paths retain processes.
 if($hostRecord.Process.HasExited -or $hostRecord.Process.StartTime.ToUniversalTime().ToFileTimeUtc().ToString() -cne $hostRecord.CreationFiletime){throw 'exact own empty host generation changed before separate fixture disposition'}
 Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public static class Ocx024V2FixtureDispose { [DllImport("kernel32.dll",SetLastError=true)] public static extern bool TerminateProcess(IntPtr handle,uint code); }'
 if(![Ocx024V2FixtureDispose]::TerminateProcess($hostRecord.Process.Handle,242)){throw ('exact own empty host disposition failed raw='+[Runtime.InteropServices.Marshal]::GetLastWin32Error())}
 if(!$hostRecord.Process.WaitForExit(5000)){throw 'exact own empty host exit unconfirmed'}
 CloseExited $hostRecord
 if($hostRecord.Process.ExitCode -ne 242){throw 'separate exact own empty fixture raw242 not confirmed'}
 $result.fixtureHostRawExit=242;$result.fixtureDisposed=$true;$result.fixtureForcedNotStopOrClosed=$true;$result.accepted=$true
}catch{$result.firstError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber};WriteOnce 'launcher-error.json' $result.firstError}
finally{
 foreach($child in $children){
  $alive=$true;$raw=$null;try{$alive=!$child.Process.HasExited;if(!$alive){$raw=$child.Process.ExitCode;if(!$child.Closed){CloseExited $child}}}catch{}
  $result.children+=@{name=$child.Name;pid=if($child.CreationFiletime){$child.Process.Id}else{$null};creationFiletime=$child.CreationFiletime;raw=$raw;retainedAlive=$alive;exactTaskOnly=$true}
 }
 try{
  $data=Join-Path $privateRoot 'D';$exports=@()
  if(Test-Path -LiteralPath $data){foreach($name in @('before-resume-private-control-v2.json','before-resume-private-control-v2-events.jsonl')){
   $source=Join-Path $data $name;if(Test-Path -LiteralPath $source){$raw=SnapshotBytes $source;$target=Join-Path $out ('native-'+$name);$dest=[IO.File]::Open($target,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read);try{$dest.Write($raw,0,$raw.Length);$dest.Flush($true)}finally{$dest.Dispose()};$exports+=@{name=$name;bytes=$raw.Length;sha256=(Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant();boundedSnapshot=$true}}
  }}
  $result.exports=$exports
 }catch{$result.exportError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber};$result.accepted=$false}
 $after=PathFacts;$result.dailyPathBefore=ConvertFrom-Json $before;$result.dailyPathAfter=ConvertFrom-Json $after;$result.dailyPathUnchanged=$before -ceq $after
 if(!$result.dailyPathUnchanged){$result.accepted=$false}
 WriteOnce 'result.json' $result
}
if($result.accepted){Write-Output 'V2 native empty endpoint PASS. Own empty fixture raw242 disposed; all task files retained.';exit 0}
Write-Output 'V2 native empty endpoint FAILED. Preserve all task files and reported processes; do not rerun.'
exit 1

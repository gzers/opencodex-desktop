param([Parameter(Mandatory=$true)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'records.ps1')
if(Test-Path -LiteralPath $OutputDirectory){throw 'Fresh behavior test directory required'}
$failed=Invoke-SweepCheck $OutputDirectory '01' 'first' {throw ('actual first error'+[char]10+'second line')}
$passed=Invoke-SweepCheck $OutputDirectory '02' 'later' {@{actual='executed after failure'}}
if($failed.status -cne 'fail' -or $failed.firstError -cne 'actual first error' -or $passed.status -cne 'pass' -or $passed.accepted -ne $false){throw 'Continuation/result contract failed'}
$before=(Get-FileHash (Join-Path $OutputDirectory '01-first.json')).Hash
$rejected=$false
try{Write-SweepRecord $OutputDirectory '01' 'first' pass|Out-Null}catch{$rejected=$true}
if(!$rejected -or (Get-FileHash (Join-Path $OutputDirectory '01-first.json')).Hash -cne $before){throw 'Failure evidence was overwritten'}
. (Join-Path $PSScriptRoot '../windows023-two-user-native-helper.ps1')
$c=@{diagnosticSweep=$true};$out=Join-Path $OutputDirectory 'user1';New-Item -ItemType Directory $out|Out-Null
Invoke-TwoUserSweepCheck 'supply' {throw 'unresolved actual source'}|Out-Null
Invoke-TwoUserSweepCheck 'ipc' {@{ran=$true}}|Out-Null
if((Get-Content (Join-Path $out 'sweep-supply.json') -Raw|ConvertFrom-Json).status -cne 'fail'){throw 'Unready supply was accepted'}
if((Get-Content (Join-Path $out 'sweep-ipc.json') -Raw|ConvertFrom-Json).status -cne 'pass'){throw 'Independent IPC action was suppressed'}
$c.diagnosticSweep=$false;$rejected=$false
try{Invoke-TwoUserSweepCheck 'legacy' {throw 'original assertion'}|Out-Null}catch{$rejected=$true}
if(!$rejected){throw 'Normal acceptance assertion was swallowed'}
Write-Output 'PASS: wrappers continue, refuse overwrite, and preserve normal strict assertions'

$busy=Get-TwoUserDiagnosticStartupState @(@{sid='A';failed=$true;ready=$false},@{sid='B';failed=$false;ready=$false})
if($busy.ready -or $busy.allSettled){throw 'One failed SID prematurely stopped the independently running second SID'}
$settled=Get-TwoUserDiagnosticStartupState @(@{sid='A';failed=$true;ready=$false},@{sid='B';failed=$false;ready=$true})
if($settled.ready -or !$settled.allSettled -or $settled.failedSids.Count -ne 1){throw 'Failed SID became ready or settled independent result was lost'}
$both=Get-TwoUserDiagnosticStartupState @(@{sid='A';failed=$false;ready=$true},@{sid='B';failed=$false;ready=$true})
if(!$both.ready -or !$both.allSettled -or $both.failedSids.Count){throw 'Two real ready coordination receipts were not handled'}
Write-Output 'PASS: real controller state function waits for the independent SID and never converts failure to ready'

class SweepReadonlyFrameFixture {
 [IO.MemoryStream]$Request=[IO.MemoryStream]::new()
 [IO.MemoryStream]$Response
 [string]$Fault=''
 [object]$DecodedRequest
 [void] Write([byte[]]$Bytes,[int]$Offset,[int]$Count){$this.Request.Write($Bytes,$Offset,$Count)}
 [void] Flush(){
  $raw=$this.Request.ToArray();$head=$raw[0..7];if([BitConverter]::IsLittleEndian){[Array]::Reverse($head)}
  if([BitConverter]::ToUInt64([byte[]]$head,0) -ne $raw.Length-8){throw 'Request frame length differs'}
  $this.DecodedRequest=[Text.Encoding]::UTF8.GetString($raw,8,$raw.Length-8)|ConvertFrom-Json
  $id=$this.DecodedRequest.request_id;if($this.Fault -ceq 'wrong-id'){$id='unrelated'}
  $payload=[Text.Encoding]::UTF8.GetBytes((@{request_id=$id;ok=($this.Fault -cne 'failure');data=@{active_data_root='isolated'}}|ConvertTo-Json -Compress))
  $length=if($this.Fault -ceq 'oversize'){[uint64]1048577}else{[uint64]$payload.Length}
  $header=[BitConverter]::GetBytes($length);if([BitConverter]::IsLittleEndian){[Array]::Reverse($header)}
  $this.Response=[IO.MemoryStream]::new();$this.Response.Write($header,0,8)
  if($this.Fault -cne 'truncated'){$this.Response.Write($payload,0,$payload.Length)}
  $this.Response.Position=0
 }
 [Threading.Tasks.Task] ReadExactlyAsync([byte[]]$Bytes,[int]$Offset,[int]$Count,[Threading.CancellationToken]$Token){
  $this.Response.ReadExactly($Bytes,$Offset,$Count)
  return [Threading.Tasks.Task]::CompletedTask
 }
 [void] Dispose(){$this.Request.Dispose();if($this.Response){$this.Response.Dispose()}}
}
foreach($fault in @('','wrong-id','failure','oversize','truncated')){
 $stream=[SweepReadonlyFrameFixture]::new();$stream.Fault=$fault;$rejected=$false
 try{
  try{$reply=Complete-TwoUserReadonlyPipeFrame $stream}catch{$rejected=$true}
  if($fault -ceq ''){
   if($rejected -or $reply.ok -ne $true -or $stream.DecodedRequest.command -cne 'data_root_show' -or $stream.DecodedRequest.confirm -ne $false -or $stream.DecodedRequest.contract_version -ne 1){throw 'Owned PID probe did not complete the actual readonly frame contract'}
  }elseif(!$rejected){throw ('Invalid response accepted: '+$fault)}
 }finally{$stream.Dispose()}
}
$readFixture=[IO.MemoryStream]::new([byte[]]@(1,2,3));$bytes=[byte[]]::new(3)
try{Read-TwoUserBoundedPipeBytes $readFixture $bytes;if(($bytes -join ',') -cne '1,2,3'){throw 'Actual .NET asynchronous exact read differs'}}finally{$readFixture.Dispose()}
Write-Output 'PASS: PID probe completes readonly frame and rejects mismatched, failed, oversized and truncated responses'

Add-Type -TypeDefinition @'
using System;
using System.IO.Pipes;
using System.Threading.Tasks;
public static class SweepDelayedPipe {
 public static NamedPipeClientStream Reader() {
  string name="os-"+Guid.NewGuid().ToString("N").Substring(0,8);
  var server=new NamedPipeServerStream(name,PipeDirection.Out,1,PipeTransmissionMode.Byte,PipeOptions.Asynchronous);
  _=Task.Run(async()=>{using(server){await server.WaitForConnectionAsync();await Task.Delay(150);await server.WriteAsync(new byte[]{7,8,9});await server.FlushAsync();}});
  var client=new NamedPipeClientStream(".",name,PipeDirection.In,PipeOptions.Asynchronous);
  client.Connect(5000);return client;
 }
}
'@
$delayed=[SweepDelayedPipe]::Reader();$delayedBytes=[byte[]]::new(3)
try{Read-TwoUserBoundedPipeBytes $delayed $delayedBytes;if(($delayedBytes -join ',') -cne '7,8,9'){throw 'Actual delayed asynchronous named-pipe read differed'}}finally{$delayed.Dispose()}
Write-Output 'PASS: incomplete actual named-pipe ValueTask awaits completion before GetResult'

$oldHandles=$script:twoUserRetainedHandles
try{
 $signalled=[pscustomobject]@{}
 $signalled|Add-Member ScriptMethod Exited {return $true}
 $script:twoUserRetainedHandles=@{'987654/123'=$signalled}
 # This PID need not exist. The exact retained original handle is authority;
 # consulting or killing a replacement process is neither needed nor allowed.
 Stop-TwoUserTracked @{pid=987654;creationTicks='123';sid='original';path='original.exe'}
 if(Test-TwoUserOriginalExited @{pid=987654;creationTicks='different'}){throw 'Retained handle crossed process generations'}
}finally{$script:twoUserRetainedHandles=$oldHandles}
Write-Output 'PASS: original signalled handle stops cleanup before any reused PID lookup; generation key remains strict'

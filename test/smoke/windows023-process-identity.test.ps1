# Real ephemeral Windows process-handle preflight; no accounts or product state.
param([Parameter(Mandatory=$true)][string]$Node,[Parameter(Mandatory=$true)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'windows023-two-user-native-helper.ps1')
Assert-TwoUserChildGuard
if(Test-Path -LiteralPath $OutputDirectory){throw 'Fresh process identity evidence required'}
New-Item -ItemType Directory -Path $OutputDirectory|Out-Null
$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$hostProcess=[Diagnostics.Process]::GetCurrentProcess();$child=$null;$fact=$null
$report=[ordered]@{accepted=$false;nativeExecuted=$true;status='fail';scope='own retained handles only; not two-user or renderer acceptance'}
try {
 $self=Get-TwoUserProcessFact $hostProcess $sid
 if($self.pid -ne $PID -or $self.sid -cne $sid){throw 'Native current process identity differs'}
 $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$Node;$info.UseShellExecute=$false
 $info.ArgumentList.Add('-e');$info.ArgumentList.Add('setTimeout(()=>{},60000)')
 $child=[Diagnostics.Process]::Start($info);$fact=Get-TwoUserProcessFact $child $sid
 $observed=Get-TwoUserProcessObservation $child
 Assert-TwoUserNativeIdentity $observed $fact.pid $fact.path $fact.creationTicks
 $null=Assert-TwoUserLive $fact
 $refused=$false
 try{Assert-TwoUserNativeIdentity $observed $fact.pid $fact.path ([string]([long]$fact.creationTicks+1))}catch{$refused=$true}
 if(!$refused){throw 'Changed native FILETIME was accepted'}
 $report.self=$self;$report.child=$fact;$report.changedGenerationRejected=$true
 $report.status='pass'
}finally {
 if($child){
  if(!$fact -and !$child.HasExited){$fact=Get-TwoUserProcessFact $child $sid}
  if($fact){Stop-TwoUserTracked $fact}
  if(!$child.WaitForExit(10000)){throw 'Exact own process identity preflight cleanup incomplete'}
  $report.originalChildExited=$true;$child.Dispose()
 }
 Close-TwoUserProcessHandles;$hostProcess.Dispose()
 Write-TwoUserJson (Join-Path $OutputDirectory 'native-process-identity.json') $report
}
if($report.status -cne 'pass' -or !$report.originalChildExited){throw 'Actual retained-handle identity preflight failed'}

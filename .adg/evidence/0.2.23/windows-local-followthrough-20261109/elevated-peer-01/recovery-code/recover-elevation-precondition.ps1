$ErrorActionPreference='Stop'
. 'C:/Users/15119/.codex/worktrees/ocxd-023-windows-r1/code/test/smoke/windows023-two-user-native-helper.ps1'
$b=Join-Path $PSScriptRoot 'elevated-peer-01'
$report=[ordered]@{startedUtc=[datetime]::UtcNow.ToString('o');scope='Only exact bootstrap descendants from failed elevation precondition; no service calls or unrelated termination';entries=@()}
$ledger=[Collections.Generic.Dictionary[int,object]]::new()
try{
 foreach($role in 1..2){
  $old=Get-Content -LiteralPath (Join-Path $b ('provisioned-user'+$role+'.json')) -Raw|ConvertFrom-Json
  $p=Get-Process -Id $old.bootstrap.pid -ErrorAction SilentlyContinue
  if(!$p){$report.entries+=@{role=$role;pid=$old.bootstrap.pid;observation='Original PID absent; no signal'};continue}
  try{
   $now=Get-TwoUserProcessFact $p $old.sid
   if($now.creationTicks -cne $old.bootstrap.creationTicks -or $now.path -ine $old.bootstrap.path){throw 'Bootstrap generation changed; retain'}
   $ledger[[int]$now.pid]=$now
   Add-TwoUserDescendants $now $ledger
  }finally{$p.Dispose()}
 }
 foreach($f in @($ledger.Values|Sort-Object {[long]$_.creationTicks} -Descending)){
  try{Stop-TwoUserTracked $f;$report.entries+=@{pid=$f.pid;sid=$f.sid;image=$f.path;creationTicks=$f.creationTicks;result='exact-owned-exited'}}
  catch{$report.entries+=@{pid=$f.pid;sid=$f.sid;image=$f.path;result='retained-unproven';error=$_.Exception.Message}}
 }
}catch{$report.failure=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
finally{Close-TwoUserProcessHandles;$report.finishedUtc=[datetime]::UtcNow.ToString('o');Write-TwoUserJson (Join-Path $b 'precondition-recovery.json') $report}

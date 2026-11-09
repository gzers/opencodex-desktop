$ErrorActionPreference='Stop'
. 'C:/Users/15119/.codex/worktrees/ocxd-023-windows-r1/code/test/smoke/windows023-two-user-native-helper.ps1'
$b=Join-Path $PSScriptRoot 'elevated-peer-01'
$envFact=Get-Content -LiteralPath (Join-Path $b 'user2/environment.json') -Raw|ConvertFrom-Json
$provision=Get-Content -LiteralPath (Join-Path $b 'provisioned-user2.json') -Raw|ConvertFrom-Json
$report=[ordered]@{startedUtc=[datetime]::UtcNow.ToString('o');scope='Exact original Manager native PID/image/SID/creation and descendants only; subsequent exact retained test Profile';original=$envFact.manager;entries=@();unproven=$false}
$ledger=[Collections.Generic.Dictionary[int,object]]::new()
try{
 $old=$envFact.manager
 if($old.sid -cne $provision.sid -or $envFact.profile -ine $provision.profile){throw 'Source identity mismatch'}
 $p=Get-Process -Id $old.pid -ErrorAction SilentlyContinue
 if($p){try{
  $now=Get-TwoUserProcessFact $p $old.sid
  if($now.creationTicks -cne $old.creationTicks -or $now.path -ine $old.path){throw 'Manager generation changed; retain'}
  $ledger[[int]$now.pid]=$now;Add-TwoUserDescendants $now $ledger
 }finally{$p.Dispose()}}
 foreach($f in @($ledger.Values|Sort-Object {[long]$_.creationTicks} -Descending)){
  try{Stop-TwoUserTracked $f;$report.entries+=@{pid=$f.pid;image=$f.path;sid=$f.sid;result='exact-owned-exited'}}
  catch{$report.unproven=$true;$report.entries+=@{pid=$f.pid;image=$f.path;sid=$f.sid;result='retained-unproven';error=$_.Exception.Message}}
 }
 Close-TwoUserProcessHandles
 $residual=@()
 foreach($row in Get-CimInstance Win32_Process){$owner=Invoke-CimMethod -InputObject $row -MethodName GetOwnerSid -ErrorAction SilentlyContinue;if($owner.Sid -ceq $old.sid){$residual+=@{pid=$row.ProcessId;image=$(if($row.ExecutablePath){$row.ExecutablePath}else{'unknown'});sid=$owner.Sid}}}
 $report.residualProcesses=$residual
 if($residual.Count -or $report.unproven){throw 'Residual identity not cleared; retain account'}
 $user=Get-LocalUser -SID $old.sid
 if($user.Name -cne $provision.name -or $user.Name -notlike 'ocxaccu1*'){throw 'Exact disposable account changed'}
 $until=[datetime]::UtcNow.AddSeconds(30)
 do{$profiles=@(Get-CimInstance Win32_UserProfile|Where-Object SID -ceq $old.sid);if(!@($profiles|Where-Object Loaded).Count){break};Start-Sleep -Milliseconds 500}while([datetime]::UtcNow -lt $until)
 foreach($profile in $profiles){if($profile.Special -or $profile.Loaded -or $profile.LocalPath -ine $provision.profile){throw 'Profile still loaded or changed'};Remove-CimInstance -InputObject $profile}
 if(Test-Path -LiteralPath $provision.profile){throw 'Original profile directory remains'}
 Remove-LocalUser -SID $old.sid
 $report.accountRemoved=!(Get-LocalUser -SID $old.sid -ErrorAction SilentlyContinue)
 $report.profileRemoved=(@(Get-CimInstance Win32_UserProfile|Where-Object SID -ceq $old.sid).Count -eq 0)
 $report.userDirectoryRemoved=!(Test-Path -LiteralPath $provision.profile)
}catch{$report.failure=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
finally{Close-TwoUserProcessHandles;$report.finishedUtc=[datetime]::UtcNow.ToString('o');Write-TwoUserJson (Join-Path $b 'manager-recovery.json') $report}

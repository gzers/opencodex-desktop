param([Parameter(Mandatory)][string]$Controller,[Parameter(Mandatory)][string]$Output)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $Output){throw 'fresh cleanup evidence required'}
$c=Get-Content -LiteralPath $Controller -Raw|ConvertFrom-Json
$r=[ordered]@{passed=$false;at=[datetime]::Now.ToString('o');scope='exact retained administrator test SID; native Profile service only'}
try{
 $id=[Security.Principal.WindowsIdentity]::GetCurrent()
 if(!([Security.Principal.WindowsPrincipal]::new($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'elevated native cleanup controller required'}
 if(!$c.created -or !$c.administratorScenario -or $c.name -notmatch '^ocxw1d[0-9a-f]{9}$' -or $c.sid -ceq $id.User.Value -or [IO.Path]::GetFileName($c.profile) -cne $c.name){throw 'exact newly owned test identity required'}
 $full=[IO.Path]::GetFullPath($c.profile)
 $parent=[IO.Path]::GetDirectoryName($full)
 if($parent -ine (Join-Path $env:SystemDrive 'Users')){throw 'test Profile outside explicit Users root'}
 $state=Get-Content -LiteralPath (Join-Path (Split-Path -Parent $Controller) 'native/msi-final-registration.json') -Raw|ConvertFrom-Json
 if($state.productCode -cne '{67CF3D62-492D-4359-AAFE-8BE6FB17E647}' -or $state.state -ne -1 -or !$state.safeToRemoveTestIdentity){throw 'per-user MSI registration absent readback required'}
 $r.sid=$c.sid;$r.profile=$full;$r.msiState=$state.state
 if(Get-ScheduledTask -TaskName $c.taskName -ErrorAction SilentlyContinue){throw 'owned test task is still registered; retain Profile'}
 $r.taskAbsent=$true
 $profiles=@(Get-CimInstance Win32_UserProfile -Filter ("SID='"+$c.sid+"'"))
 $r.profiles=@($profiles|Select-Object SID,LocalPath,Loaded,Special,Status)
 $owned=@()
 foreach($p in Get-CimInstance Win32_Process -Filter "SessionId=0 AND (Name='pwsh.exe' OR Name='taskhostw.exe' OR Name='msiexec.exe' OR Name='ocxd.exe' OR Name='conhost.exe' OR Name='RuntimeBroker.exe' OR Name='dllhost.exe' OR Name='svchost.exe')"){
  $owner=Invoke-CimMethod -InputObject $p -MethodName GetOwnerSid -ErrorAction SilentlyContinue
  if($owner.ReturnValue -eq 0 -and $owner.Sid -ceq $c.sid){$owned+=@{pid=$p.ProcessId;parent=$p.ParentProcessId;name=$p.Name;created=$p.CreationDate;command=$p.CommandLine}}
 }
 $r.remainingOwnedProcesses=$owned
 if($owned.Count){throw 'test SID still owns processes; retain Profile, no force termination'}
 if($profiles.Count -eq 1){
  if($profiles[0].Special -or [IO.Path]::GetFullPath($profiles[0].LocalPath) -ine $full -or $profiles[0].Loaded){throw 'exact Profile is still loaded or unknown; no force unload or filesystem fallback'}
  $profiles[0]|Remove-CimInstance -ErrorAction Stop
 }elseif($profiles.Count -ne 0){throw 'ambiguous native Profile registration'}
 $r.profileRemoved=!(Test-Path -LiteralPath $full) -and @(Get-CimInstance Win32_UserProfile -Filter ("SID='"+$c.sid+"'")).Count -eq 0
 if(!$r.profileRemoved){throw 'native Profile cleanup incomplete; directory retained'}
 $user=Get-LocalUser -SID $c.sid -ErrorAction SilentlyContinue
 if($user){if($user.Name -cne $c.name){throw 'account identity changed'};Remove-LocalUser -SID $user.SID -ErrorAction Stop}
 $r.accountRemoved=$null -eq (Get-LocalUser -SID $c.sid -ErrorAction SilentlyContinue)
 $r.passed=$r.profileRemoved -and $r.accountRemoved -and $r.taskAbsent
}catch{$r.firstError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
finally{$r|ConvertTo-Json -Depth 8|Set-Content -LiteralPath $Output -Encoding utf8}
if(!$r.passed){exit 1}

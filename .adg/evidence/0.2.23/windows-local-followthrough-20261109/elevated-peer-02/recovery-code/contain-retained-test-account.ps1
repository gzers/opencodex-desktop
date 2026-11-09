$ErrorActionPreference='Stop'
$b=Join-Path $PSScriptRoot 'elevated-peer-02'
$old=Get-Content -LiteralPath (Join-Path $b 'provisioned-user1.json') -Raw|ConvertFrom-Json
$report=[ordered]@{atUtc=[datetime]::UtcNow.ToString('o');sid=$old.sid;name=$old.name;scope='Only the exact retained temporary test account; never daily identities or policy'}
try{
 $user=Get-LocalUser -SID $old.sid
 if($user.Name -cne $old.name -or $user.Name -notlike 'ocxaccu1*' -or $old.profile -ine (Join-Path 'C:/Users' $old.name)){throw 'Exact disposable identity differs'}
 Disable-LocalUser -SID $old.sid
 $admin=Get-LocalGroup -SID 'S-1-5-32-544'
 $member=@(Get-LocalGroupMember -Group $admin|Where-Object {$_.SID.Value -ceq $old.sid})
 if($member.Count){Remove-LocalGroupMember -Group $admin -Member $member}
 $report.accountDisabled=!(Get-LocalUser -SID $old.sid).Enabled
 $report.administratorMembershipRemoved=(!@(Get-LocalGroupMember -Group $admin|Where-Object {$_.SID.Value -ceq $old.sid}).Count)
 $profiles=@(Get-CimInstance Win32_UserProfile|Where-Object SID -ceq $old.sid)
 $report.profileLoaded=[bool]@($profiles|Where-Object Loaded).Count
 if(!$report.profileLoaded){
  foreach($p in $profiles){if($p.Special -or $p.LocalPath -ine $old.profile){throw 'Profile changed'};Remove-CimInstance -InputObject $p}
  if(Test-Path -LiteralPath $old.profile){throw 'Original user directory remains'}
  Remove-LocalUser -SID $old.sid
  $report.accountRemoved=!(Get-LocalUser -SID $old.sid -ErrorAction SilentlyContinue)
  $report.profileRemoved=(@(Get-CimInstance Win32_UserProfile|Where-Object SID -ceq $old.sid).Count -eq 0)
 }else{$report.cleanupBlocked='OS still reports loaded Profile; no forced hive unload or user deletion'}
}catch{$report.failure=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
finally{$report.finishedUtc=[datetime]::UtcNow.ToString('o');$report|ConvertTo-Json -Depth 7|Set-Content -LiteralPath (Join-Path $b 'retained-account-containment.json') -Encoding utf8}

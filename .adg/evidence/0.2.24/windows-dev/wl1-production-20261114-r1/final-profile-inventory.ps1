$ErrorActionPreference='Stop'
$root='C:/Users/Public/ocx-wl1-20261114-r1'
$r=[ordered]@{at=[datetime]::Now.ToString('o');scope='exact five sequential W-L1 test Profiles only; read-only /L backup listing';profiles=@()}
foreach($round in @('out01','out02','out03','out04','out05')){
 $c=Get-Content -LiteralPath (Join-Path $root ($round+'/controller.json')) -Raw|ConvertFrom-Json
 if(!$c.created -or $c.name -notmatch '^ocxw1d[0-9a-f]{9}$' -or [IO.Path]::GetFileName($c.profile) -cne $c.name){throw 'unknown test Profile'}
 $row=@{round=$round;sid=$c.sid;path=$c.profile;accountPresent=$null -ne (Get-LocalUser -SID $c.sid -ErrorAction SilentlyContinue);directoryPresent=Test-Path -LiteralPath $c.profile;profile=@(Get-CimInstance Win32_UserProfile -Filter ("SID='"+$c.sid+"'")|Select-Object SID,LocalPath,Loaded,Special)}
 $dest=Join-Path $root ('final-inventory-placeholder-'+$round)
 if(Test-Path -LiteralPath $dest){throw 'placeholder must be absent'}
 if($row.directoryPresent){
  & robocopy.exe $c.profile $dest /L /E /B /BYTES /R:0 /W:0 /XJ /XJD /XJF /NP /NFL /NDL ('/UNILOG:'+(Join-Path $root ('final-profile-'+$round+'.log')))|Out-Null
  $row.inventoryExit=$LASTEXITCODE;$row.placeholderCreated=Test-Path -LiteralPath $dest
 }
 $r.profiles+=$row
}
$r|ConvertTo-Json -Depth 7|Set-Content -LiteralPath (Join-Path $root 'final-profile-inventory.json') -Encoding utf8

$ErrorActionPreference='Stop'
$root='C:/Users/Public/ocx-wl1-20261114-r1'
$r=[ordered]@{at=[datetime]::Now.ToString('o');scope='read-only exact newly created W-L1 identities/Profiles and generated root; no deletion';profiles=@()}
foreach($round in @('out01','out02')){
 $c=Get-Content -LiteralPath (Join-Path $root ($round+'/controller.json')) -Raw|ConvertFrom-Json
 if(!$c.created -or $c.name -notmatch '^ocxw1d[0-9a-f]{9}$' -or [IO.Path]::GetFileName($c.profile) -cne $c.name){throw 'exact W-L1 identity differs'}
 $state=Get-Content -LiteralPath (Join-Path $root ($round+'/native/wl1-final-owner.json')) -Raw|ConvertFrom-Json
 $p=[ordered]@{round=$round;sid=$c.sid;profile=$c.profile;nativeFinal=$state;accountPresent=$null -ne (Get-LocalUser -SID $c.sid -ErrorAction SilentlyContinue);profileDirectoryPresent=Test-Path -LiteralPath $c.profile;nativeProfile=@(Get-CimInstance Win32_UserProfile -Filter ("SID='"+$c.sid+"'")|Select-Object SID,Loaded,LocalPath,Special);controllerProfileDelete=$c.profileDeleteApi}
 $dest=Join-Path $root ('inventory-only-'+$round)
 if(Test-Path -LiteralPath $dest){throw 'inventory placeholder must be absent'}
 if(Test-Path -LiteralPath $c.profile){
  & robocopy.exe $c.profile $dest /L /E /B /BYTES /R:0 /W:0 /XJ /XJD /XJF /NP /NFL /NDL ('/UNILOG:'+(Join-Path $root ('profile-'+$round+'-inventory.log')))|Out-Null
  $p.inventoryExit=$LASTEXITCODE;$p.placeholderPresent=Test-Path -LiteralPath $dest
 }
 $r.profiles+=$p
}
$r|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $root 'profiles-inventory.json') -Encoding utf8

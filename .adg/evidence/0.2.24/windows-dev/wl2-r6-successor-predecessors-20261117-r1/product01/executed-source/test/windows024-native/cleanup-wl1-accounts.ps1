param([Parameter(Mandatory)][string]$Root,[Parameter(Mandatory)][string[]]$Rounds,[Parameter(Mandatory)][string]$Output)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $Output){throw 'fresh cleanup receipt required'}
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if(!([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'elevated disposable-account cleanup controller required'}
$results=@()
foreach($round in $Rounds){
 $row=[ordered]@{round=$round;accountRemoved=$false;profileRemoved=$false;failureSceneRetained=$true}
 try{
  if($round -notmatch '^out[0-9]{2}$'){throw 'exact round spelling required'}
  $path=Join-Path $Root $round
  $controller=Get-Content -LiteralPath (Join-Path $path 'controller.json') -Raw|ConvertFrom-Json
  $input=Get-Content -LiteralPath (Join-Path $path 'child-input.json') -Raw|ConvertFrom-Json
  $owner=Get-Content -LiteralPath (Join-Path $path 'native/wl1-final-owner.json') -Raw|ConvertFrom-Json
  $launch=Get-Content -LiteralPath (Join-Path $path 'launch-result.json') -Raw|ConvertFrom-Json
  if(!$controller.created -or $controller.sid -cne $owner.sid -or $input.sid -cne $owner.sid -or $owner.sid -ceq $identity.User.Value -or !$owner.safeToRemoveTestIdentity -or $input.testName -cne 'isolated_production_lifecycle_wl1' -or $input.testAdministrator -or $input.msiOnly -or $controller.name -notmatch '^ocxw1d[0-9a-f]{9}$' -or [IO.Path]::GetFileName($controller.profile) -cne $controller.name){throw 'exact terminal W-L1 ordinary identity and Job-cleanup proof required'}
  if($null -eq $launch.exitCode){throw 'controller outcome unknown'}
  $row.sid=$owner.sid;$row.profile=$controller.profile;$row.ownedJobCleanupConfirmed=$true
  $user=Get-LocalUser -SID $owner.sid -ErrorAction SilentlyContinue
  if($user){if($user.Name -cne $controller.name){throw 'account identity changed'};Remove-LocalUser -SID $user.SID -ErrorAction Stop}
  $row.accountRemoved=$null -eq (Get-LocalUser -SID $owner.sid -ErrorAction SilentlyContinue)
  # Preserve failed test directories and their original bytes. This script does
  # not delete Profiles, change ACLs or work around any rejected file deletion.
  $row.profileDirectoryPresent=Test-Path -LiteralPath $controller.profile
  $row.nativeProfile=@(Get-CimInstance Win32_UserProfile -Filter ("SID='"+$owner.sid+"'")|Select-Object SID,Loaded,LocalPath)
 }catch{$row.error=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
 $results+=$row
}
@{scope='only exact disposable account objects after proven test-owned Job cleanup; failed Profiles remain';results=$results}|ConvertTo-Json -Depth 8|Set-Content -LiteralPath $Output -Encoding utf8
if(@($results|Where-Object {!$_.accountRemoved}).Count){exit 1}

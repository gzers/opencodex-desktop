param([Parameter(Mandatory)][string]$Root,[Parameter(Mandatory)][string]$Archive,[Parameter(Mandatory)][string]$Output)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $Output){throw 'immutable cleanup receipt already exists'}
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if(!([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'account/Profile cleanup controller requires elevation'}
$resolved=[IO.Path]::GetFullPath($Root).TrimEnd([char]92,[char]47)
if($resolved -cne 'C:\Users\Public\ocx-wl2-20261114-r1'){throw 'exact new W-L2 root required; no historical cleanup retry'}
$c=Get-Content -LiteralPath (Join-Path $Root 'out01/controller.json') -Raw|ConvertFrom-Json
$childInput=Get-Content -LiteralPath (Join-Path $Root 'out01/child-input.json') -Raw|ConvertFrom-Json
$p=Get-Content -LiteralPath (Join-Path $Root 'out01/native/wl2-independent-prerequisites.json') -Raw|ConvertFrom-Json
$last=Get-Content -LiteralPath (Join-Path $Root 'out01/launch-result.json') -Raw|ConvertFrom-Json
$marker=Get-Content -LiteralPath (Join-Path $Root 'out01/native/wl1-final-owner.json') -Raw|ConvertFrom-Json
$expectedProfile=[IO.Path]::GetFullPath((Join-Path 'C:/Users' $c.name))
if(!$c.created -or $childInput.testName -cne 'isolated_wl2_native_boundaries' -or $childInput.testAdministrator -or !$childInput.componentOnly -or $childInput.msiOnly -or $c.sid -cne $p.sid -or $marker.sid -cne $p.sid -or $c.sid -ceq $identity.User.Value -or $c.name -notmatch '^ocxw1d[0-9a-f]{9}$' -or $c.profile -cne $expectedProfile -or !$marker.safeToRemoveTestIdentity -or $marker.jobCreated -ne $false -or $p.productionExecuted -ne $false -or $p.originalUnchanged -ne $true -or $null -eq $last.exitCode -or !$c.desktopAclRestored -or !$c.inputAclRestored){throw 'exact finished prerequisite-only ordinary identity required'}
$manifest=Get-Content -LiteralPath (Join-Path $Archive 'public-archive-manifest.json') -Raw|ConvertFrom-Json
foreach($f in $manifest.files){
 $source=Join-Path $Root $f.path
 $copy=Join-Path (Join-Path $Archive 'public-raw') $f.path
 if($f.kind -eq 'raw'){
  if((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -cne $f.sha256 -or (Get-FileHash -LiteralPath $copy -Algorithm SHA256).Hash.ToLowerInvariant() -cne $f.sha256){throw 'raw evidence archive verification failed'}
 }elseif($f.kind -eq 'exe-retained-in-workspace'){
  if((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -cne $f.sha256 -or (Get-FileHash -LiteralPath $f.retainedCopy -Algorithm SHA256).Hash.ToLowerInvariant() -cne $f.sha256){throw 'test executable preservation failed'}
 }else{throw 'unknown archive kind'}
}
$result=[ordered]@{sid=$c.sid;profile=$c.profile;scope='only new prerequisite Profile/account; no official Job created; immutable raw failure evidence verified';accountRemoved=$false;profileRemoved=$false;forced=$false;oldRejectedCleanupRetried=$false;at=[datetime]::Now.ToString('o')}
try{
 $profileFacts=@(Get-CimInstance Win32_UserProfile -Filter ("SID='"+$c.sid+"'")|Select-Object SID,Loaded,LocalPath)
 $result.profileBefore=$profileFacts
 Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class Wl2ProfileCleanup {
 [DllImport("userenv.dll",CharSet=CharSet.Unicode,ExactSpelling=true,SetLastError=true)] public static extern bool DeleteProfileW(string sid,string path,string computer);
 public static bool DeleteExactSid(string sid) { return DeleteProfileW(sid,null,null); }
}
'@
 $ok=[Wl2ProfileCleanup]::DeleteExactSid($c.sid)
 $result.nativeDeleteProfileReturned=$ok
 $result.nativeDeleteProfileWin32=if($ok){0}else{[Runtime.InteropServices.Marshal]::GetLastWin32Error()}
 $result.profileRemoved=!(Test-Path -LiteralPath $c.profile)
 # No force hive unload, changed ACL, fallback directory deletion or service
 # restart when native Profile cleanup is unavailable.
 $user=Get-LocalUser -SID $c.sid -ErrorAction SilentlyContinue
 if($user){if($user.Name -cne $c.name){throw 'test account identity changed'};Remove-LocalUser -SID $user.SID -ErrorAction Stop}
 $result.accountRemoved=$null -eq (Get-LocalUser -SID $c.sid -ErrorAction SilentlyContinue)
 $result.profileAfter=@(Get-CimInstance Win32_UserProfile -Filter ("SID='"+$c.sid+"'")|Select-Object SID,Loaded,LocalPath)
}catch{$result.error=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
$result|ConvertTo-Json -Depth 8|Set-Content -LiteralPath $Output -Encoding utf8
if(!$result.accountRemoved -or !$result.profileRemoved){exit 1}

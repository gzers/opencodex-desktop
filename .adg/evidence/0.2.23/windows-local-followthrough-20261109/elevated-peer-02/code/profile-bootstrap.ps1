param([string]$InputFile)
$ErrorActionPreference='Stop'
try{
 $c=Get-Content -LiteralPath $InputFile -Raw | ConvertFrom-Json
 $identity=[Security.Principal.WindowsIdentity]::GetCurrent()
 if($identity.User.Value -cne $c.accountSid){throw 'Unexpected native acceptance account'}
 $profileKey=Join-Path 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList' $identity.User.Value
 $registered=[Environment]::ExpandEnvironmentVariables((Get-ItemProperty -LiteralPath $profileKey -Name ProfileImagePath).ProfileImagePath)
 if(!$registered -or !(Test-Path -LiteralPath $registered -PathType Container) -or (Split-Path -Leaf $registered) -notlike 'ocxacc*'){throw 'Registered temporary Profile missing'}
 # CreateProcessWithLogonW can retain the parent's environment block even with
 # LoadUserProfile. Resolve the OS-registered identity before expanding the new
 # account's Known Folder definitions; never create under runneradmin.
 $env:USERPROFILE=$registered
 $env:HOMEDRIVE=[IO.Path]::GetPathRoot($registered).TrimEnd('\')
 $env:HOMEPATH=$registered.Substring($env:HOMEDRIVE.Length)
 $env:APPDATA=Join-Path $registered 'AppData/Roaming'
 $env:LOCALAPPDATA=Join-Path $registered 'AppData/Local'

 Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public static class OcxFirstLogonFolders {
 [DllImport("shell32.dll", ExactSpelling=true)] static extern int SHGetKnownFolderPath(ref Guid id, uint flags, IntPtr token, out IntPtr path);
 public static string Read(string value, IntPtr token) {
  Guid id=new Guid(value);IntPtr path=IntPtr.Zero;
  try { Marshal.ThrowExceptionForHR(SHGetKnownFolderPath(ref id,0x4000,token,out path)); return Marshal.PtrToStringUni(path); }
  finally { if(path!=IntPtr.Zero) Marshal.FreeCoTaskMem(path); }
 }
}
"@
 $profile=$registered
 $roaming=[OcxFirstLogonFolders]::Read('3eb685db-65f9-4cf6-a03a-e3ef65729f3d',$identity.Token)
 $local=[OcxFirstLogonFolders]::Read('f1b32785-6fba-4fcf-9d55-7b8e7f157091',$identity.Token)
 $imageLocalCorrected=$env:OCX_CI_LOCAL_PROFILE_INITIALIZED -ceq '1'
 if(!$local.StartsWith($registered+'\',[StringComparison]::OrdinalIgnoreCase)){
  if($imageLocalCorrected){throw 'Temporary Profile Known Folder initialization did not persist'}
  # The disposable image can clone Default User's absolute LocalAppData value.
  # Initialize only this just-created SID's standard user folder and start a
  # fresh same-account process so Shell path caches cannot hide the readback.
  $ownLocal=[IO.Path]::GetFullPath((Join-Path (Join-Path $registered 'AppData') 'Local'))
  New-Item -ItemType Directory -Force $ownLocal | Out-Null
  $userHive=[Microsoft.Win32.Registry]::Users
  foreach($leaf in @('User Shell Folders','Shell Folders')){
   $subkey=$identity.User.Value+'\Software\Microsoft\Windows\CurrentVersion\Explorer\'+$leaf
   $key=$userHive.OpenSubKey($subkey,$true)
   if(!$key){throw 'Loaded temporary account folder registry missing'}
   try{
    if($leaf -ceq 'User Shell Folders'){$key.SetValue('Local AppData','%USERPROFILE%\AppData\Local',[Microsoft.Win32.RegistryValueKind]::ExpandString)}
    else{$key.SetValue('Local AppData',$ownLocal,[Microsoft.Win32.RegistryValueKind]::String)}
    $key.Flush()
   }finally{$key.Dispose()}
  }
  $env:OCX_CI_LOCAL_PROFILE_INITIALIZED='1'
  & (Join-Path $PSHOME 'pwsh.exe') -NoLogo -NoProfile -NonInteractive -File $PSCommandPath -InputFile $InputFile
  exit $LASTEXITCODE
 }
 [ordered]@{registered=$registered;profile=$profile;roaming=$roaming;local=$local;processProfile=$env:USERPROFILE} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path (Split-Path -Parent $InputFile) 'ordinary-known-folders.json') -Encoding utf8
 if(!$profile -or !$roaming.StartsWith($profile+'\',[StringComparison]::OrdinalIgnoreCase) -or !$local.StartsWith($profile+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Known Folders escaped the temporary account Profile'}
 New-Item -ItemType Directory -Force $roaming,$local | Out-Null
 $env:USERPROFILE=$profile;$env:APPDATA=$roaming;$env:LOCALAPPDATA=$local
 $env:TEMP=Join-Path $local 'Temp';$env:TMP=$env:TEMP
 New-Item -ItemType Directory -Force $env:TEMP | Out-Null
 [ordered]@{result='pass';method='Native created temporary Profile; fresh-process Known Folder readback; only new SID user-folder registry initialized if image default was outside';imageLocalAppDataCorrected=$imageLocalCorrected;sid=$identity.User.Value;profile=$profile;roaming=$roaming;local=$local} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path (Split-Path -Parent $InputFile) 'ordinary-profile.json') -Encoding utf8
 if($c.requestElevation){
  @{requestedUtc=[datetime]::UtcNow.ToString('o');method='Windows Shell RunAs consent';sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value;profile=$env:USERPROFILE}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $c.evidence 'elevation-request.json') -Encoding utf8
  $elevated=Start-Process -FilePath (Join-Path $PSHOME 'pwsh.exe') -Verb RunAs -WindowStyle Hidden -WorkingDirectory $c.evidence -ArgumentList @('-NoLogo','-NoProfile','-NonInteractive','-File',('"'+$c.singleUserScript+'"'),'-Mode','Child','-InputFile',('"'+$InputFile+'"')) -Wait -PassThru
  try{$code=$elevated.ExitCode;@{finishedUtc=[datetime]::UtcNow.ToString('o');exitCode=$code;pid=$elevated.Id}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $c.evidence 'elevation-exit.json') -Encoding utf8}finally{$elevated.Dispose()}
  exit $code
}else{& $c.singleUserScript -Mode Child -InputFile $InputFile}
 exit $LASTEXITCODE
}catch{[Console]::Error.WriteLine('Profile bootstrap failed: '+$_.Exception.GetType().Name);exit 1}

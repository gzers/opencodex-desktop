param(
 [Parameter(Mandatory=$true)][string]$Executable,
 [Parameter(Mandatory=$true)][string]$Node,
 [Parameter(Mandatory=$true)][string]$OutputDirectory,
 [switch]$ProfileOnly
)
$ErrorActionPreference='Stop'
if(!$IsWindows -or $env:GITHUB_ACTIONS -ne 'true'){throw 'Restricted to an ephemeral Windows Actions runner'}
$taskExe=(Resolve-Path -LiteralPath $Executable).Path
$taskNode=(Resolve-Path -LiteralPath $Node).Path
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Force $taskOutput|Out-Null
$taskName='ocxacc'+[guid]::NewGuid().ToString('N').Substring(0,8)
$taskSecret=[Security.Cryptography.RandomNumberGenerator]::GetBytes(32)
$taskSecure=ConvertTo-SecureString ([Convert]::ToBase64String($taskSecret)+'aA1!') -AsPlainText -Force
$taskUser=$null
try {
 $taskUser=New-LocalUser -Name $taskName -Password $taskSecure -AccountNeverExpires -PasswordNeverExpires -Description 'Disposable OpenCodeX CI acceptance account'
 $taskUsers=Get-LocalGroup -SID 'S-1-5-32-545'
 Add-LocalGroupMember -Group $taskUsers -Member $taskUser
 if(!('OcxNativeProfileProvision' -as [type])){
  Add-Type -TypeDefinition @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class OcxNativeProfileProvision {
 [DllImport("userenv.dll", CharSet=CharSet.Unicode, ExactSpelling=true)]
 public static extern int CreateProfile(string sid,string name,StringBuilder path,uint size);
}
"@
 }
 $taskProfilePath=[Text.StringBuilder]::new(260)
 $taskProfileResult=[OcxNativeProfileProvision]::CreateProfile($taskUser.SID.Value,$taskName,$taskProfilePath,260)
 if($taskProfileResult -ne 0){[Runtime.InteropServices.Marshal]::ThrowExceptionForHR($taskProfileResult)}
 if(!(Test-Path -LiteralPath $taskProfilePath.ToString() -PathType Container)){throw 'Windows Profile service did not provision the temporary account'}

 & icacls.exe $taskOutput /grant ($taskName+':(OI)(CI)M') | Out-Null
 if($LASTEXITCODE -ne 0){throw 'Cannot provision isolated acceptance output directory'}
 $taskChild=Join-Path $taskOutput 'ordinary-child.ps1'
 $taskConfig=Join-Path $taskOutput 'ordinary-input.json'
 [ordered]@{executable=$taskExe;node=$taskNode;output=(Join-Path $taskOutput 'startup');script=(Join-Path $PSScriptRoot 'windows-startup.ps1');commit=(& git rev-parse HEAD).Trim();accountSid=$taskUser.SID.Value;profileOnly=[bool]$ProfileOnly} | ConvertTo-Json | Set-Content -LiteralPath $taskConfig -Encoding utf8
 @'
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
 [DllImport("shell32.dll", CharSet=CharSet.Unicode, ExactSpelling=true)] static extern int SHSetKnownFolderPath(ref Guid id, uint flags, IntPtr token, string path);
 public static void InitializeLocal(string path) {
  Guid id=new Guid("f1b32785-6fba-4fcf-9d55-7b8e7f157091");
  Marshal.ThrowExceptionForHR(SHSetKnownFolderPath(ref id,0,IntPtr.Zero,path));
 }
 public static string Read(string value) {
  Guid id=new Guid(value);IntPtr path=IntPtr.Zero;
  try { Marshal.ThrowExceptionForHR(SHGetKnownFolderPath(ref id,0x4000,IntPtr.Zero,out path)); return Marshal.PtrToStringUni(path); }
  finally { if(path!=IntPtr.Zero) Marshal.FreeCoTaskMem(path); }
 }
}
"@
 $profile=$registered
 $roaming=[OcxFirstLogonFolders]::Read('3eb685db-65f9-4cf6-a03a-e3ef65729f3d')
 $local=[OcxFirstLogonFolders]::Read('f1b32785-6fba-4fcf-9d55-7b8e7f157091')
 $imageLocalCorrected=$false
 if(!$local.StartsWith($registered+'\',[StringComparison]::OrdinalIgnoreCase)){
  # Some hosted images seed Default User's LocalAppData with runneradmin's
  # absolute directory. Initialize only the freshly-created current test SID's
  # standard Local folder; never change the runner/daily account or system ACL.
  $ownLocal=Join-Path $registered 'AppData/Local'
  New-Item -ItemType Directory -Force $ownLocal | Out-Null
  [OcxFirstLogonFolders]::InitializeLocal($ownLocal)
  $local=[OcxFirstLogonFolders]::Read('f1b32785-6fba-4fcf-9d55-7b8e7f157091')
  $imageLocalCorrected=$true
 }

 [ordered]@{registered=$registered;profile=$profile;roaming=$roaming;local=$local;processProfile=$env:USERPROFILE} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path (Split-Path -Parent $InputFile) 'ordinary-known-folders.json') -Encoding utf8
 if(!$profile -or !$roaming.StartsWith($profile+'\',[StringComparison]::OrdinalIgnoreCase) -or !$local.StartsWith($profile+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Known Folders escaped the temporary account Profile'}
 New-Item -ItemType Directory -Force $roaming,$local | Out-Null
 $env:USERPROFILE=$profile;$env:APPDATA=$roaming;$env:LOCALAPPDATA=$local
 $env:TEMP=Join-Path $local 'Temp';$env:TMP=$env:TEMP
 New-Item -ItemType Directory -Force $env:TEMP | Out-Null
 [ordered]@{result='pass';method='Native created temporary Profile; Known Folder readback; image-default LocalAppData initialized only for the new test SID if inherited outside';imageLocalAppDataCorrected=$imageLocalCorrected;sid=$identity.User.Value;profile=$profile;roaming=$roaming;local=$local} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path (Split-Path -Parent $InputFile) 'ordinary-profile.json') -Encoding utf8
 if($c.profileOnly){exit 0}
 & $c.script -Executable $c.executable -Node $c.node -OutputDirectory $c.output -Mode ordinary -ExpectedCommit $c.commit
 if($LASTEXITCODE -ne 0){exit $LASTEXITCODE}
 exit 0
}catch{[Console]::Error.WriteLine($_.Exception.Message);[Console]::Error.WriteLine($_.ScriptStackTrace);exit 1}
'@ | Set-Content -LiteralPath $taskChild -Encoding utf8
 $taskCredential=[pscredential]::new(($env:COMPUTERNAME+'\'+$taskName),$taskSecure)
 $taskProcess=Start-Process -FilePath (Join-Path $PSHOME 'pwsh.exe') -Credential $taskCredential -LoadUserProfile -WorkingDirectory $taskOutput -ArgumentList @('-NoLogo','-NoProfile','-NonInteractive','-File',('"'+$taskChild+'"'),'-InputFile',('"'+$taskConfig+'"')) -RedirectStandardOutput (Join-Path $taskOutput 'ordinary-stdout.log') -RedirectStandardError (Join-Path $taskOutput 'ordinary-stderr.log') -PassThru
 if(!$taskProcess.WaitForExit(300000)){$taskProcess.Kill($true);throw 'Ordinary native startup deadline expired'}
 if($taskProcess.ExitCode -ne 0){
  foreach($name in @('ordinary-stderr.log','ordinary-known-folders.json','startup/startup-stderr.log')){
   $path=Join-Path $taskOutput $name
   if(Test-Path -LiteralPath $path){Get-Content -LiteralPath $path -Tail 20 | Write-Output}
  }
  throw 'Ordinary native startup failed; see retained output'
 }
 if($ProfileOnly){Get-Content -LiteralPath (Join-Path $taskOutput 'ordinary-profile.json') -Raw | Write-Output;return}
 $taskReport=Get-Content -LiteralPath (Join-Path $taskOutput 'startup/startup.json') -Raw|ConvertFrom-Json
 if($taskReport.result -cne 'pass' -or $taskReport.applicationToken.elevated -ne $false -or !$taskReport.applicationModeMatched){throw 'Ordinary application token evidence missing'}
 [ordered]@{result='pass';scope='temporary nonadministrator local account with genuine loaded Profile; isolated sandbox, final EXE and WebView2 business checks';accountSid=$taskUser.SID.Value;applicationElevated=$taskReport.applicationToken.elevated;applicationModeMatched=$taskReport.applicationModeMatched;dailyUserChanged=$false} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskOutput 'ordinary-summary.json') -Encoding utf8
} finally {
 if($taskUser){Remove-LocalUser -SID $taskUser.SID}
 [Array]::Clear($taskSecret,0,$taskSecret.Length)
}

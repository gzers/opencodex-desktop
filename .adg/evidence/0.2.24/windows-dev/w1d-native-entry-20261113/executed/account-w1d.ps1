param([Parameter(Mandatory)][string]$Config,[switch]$Child)
$ErrorActionPreference='Stop'
$c=Get-Content -LiteralPath $Config -Raw|ConvertFrom-Json
if($Child){
 $report=[ordered]@{passed=$false;scope='one fresh ordinary account; W1-D only'}
 try{
  $id=[Security.Principal.WindowsIdentity]::GetCurrent()
  if($id.User.Value -cne $c.sid){throw 'exact isolated account SID required'}
  if(([Security.Principal.WindowsPrincipal]::new($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'ordinary test token required'}
  $registered=(Get-ItemProperty -LiteralPath ('HKLM:/SOFTWARE/Microsoft/Windows NT/CurrentVersion/ProfileList/'+$id.User.Value) -Name ProfileImagePath).ProfileImagePath
  if($registered -cne $c.profile -or [IO.Path]::GetFileName($registered) -notlike 'ocxw1d*'){throw 'new OS profile differs'}
  $env:USERPROFILE=$registered;$env:HOME=$registered
  $env:HOMEDRIVE=[IO.Path]::GetPathRoot($registered).TrimEnd([char]92)
  $env:HOMEPATH=$registered.Substring($env:HOMEDRIVE.Length)
  $env:APPDATA=Join-Path $registered 'AppData/Roaming'
  $env:LOCALAPPDATA=Join-Path $registered 'AppData/Local'
  $env:TEMP=Join-Path $env:LOCALAPPDATA 'Temp';$env:TMP=$env:TEMP
  [void][IO.Directory]::CreateDirectory($env:TEMP)
  foreach($name in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_HOME','CODEX_HOME','CLAUDE_CONFIG_DIR','SHELL')){[Environment]::SetEnvironmentVariable($name,$null,'Process')}
  $machine=[Environment]::GetEnvironmentVariable('Path','Machine')
  $own=[Environment]::GetEnvironmentVariable('Path','User')
  $env:PATH=$machine+';'+$own
  $env:OCXD024_COMPONENT_ONLY=if($c.componentOnly){"1"}else{$null}
  $env:OCXD024_ACCOUNT_SID=$id.User.Value
  $env:OCXD024_PWSH=(Join-Path $PSHOME 'pwsh.exe')
  $env:OCXD024_NATIVE_INPUT=$c.input
  $env:OCXD024_NATIVE_EVIDENCE=$c.evidence
  $report.sid=$id.User.Value;$report.profile=$registered;$report.session=[Diagnostics.Process]::GetCurrentProcess().SessionId
  $testName=if($c.testName){$c.testName}else{'isolated_account_terminal_w1d'}
  if($testName -cnotin @('isolated_account_terminal_w1d','isolated_schema3_invariants_w1d')){throw 'exact W1-D ignored test required'}
  & $c.test --exact ('modules::runtime::terminal_cli::terminal_cli_windows::native_tests::'+$testName) --ignored --test-threads=1 --nocapture 1> (Join-Path $c.evidence 'test.stdout.log') 2> (Join-Path $c.evidence 'test.stderr.log')
  $report.exitCode=$LASTEXITCODE
  $report.passed=$LASTEXITCODE -eq 0
 }catch{$report.firstError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber};$_|Out-String|Set-Content -LiteralPath (Join-Path $c.evidence 'child-error.log') -Encoding utf8}
 finally{
  if($registered -and [IO.Path]::GetFileName($registered) -like 'ocxw1d*'){
   try{
    $files=@();$total=0L
    foreach($relative in @('.codex-desktop/desktop','w1d native/data/desktop','w1d native/custom H 雪','w1d native/I 空格 & !/runtime/bin')){
     $stateRoot=Join-Path $registered $relative
     if(Test-Path -LiteralPath $stateRoot){
      foreach($file in Get-ChildItem -LiteralPath $stateRoot -File -Recurse -Force){
       if($file.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'reparse state archive refused'}
       if($file.Length -gt 16777216 -or ($total+$file.Length) -gt 33554432 -or $files.Count -gt 512){throw 'bounded private state archive limit'}
       $bytes=[IO.File]::ReadAllBytes($file.FullName);$total+=$bytes.Length
       $files+=@{relative=[IO.Path]::GetRelativePath($registered,$file.FullName);bytes=$bytes.Length;sha256=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant();base64=[Convert]::ToBase64String($bytes)}
      }
     }
    }
    ConvertTo-Json -InputObject @($files) -Depth 8|Set-Content -LiteralPath (Join-Path $c.evidence 'profile-state.private.json') -Encoding utf8
    $report.privateStateArchived=$true
   }catch{$report.archiveError=$_.Exception.Message}
  }
  $report|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $c.evidence 'child-result.json') -Encoding utf8
 }
 if(!$report.passed){exit 1};exit 0
}
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if(!([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'controller requires elevation solely for one disposable account/Profile'}
if(!(Test-Path -LiteralPath $c.input -PathType Container) -or (Test-Path -LiteralPath (Join-Path $c.output 'controller.json'))){throw 'fresh explicit controller output and existing input required'}
$report=[ordered]@{passed=$false;scope='one isolated account; no W2/two-user acceptance';created=$false;accountRemoved=$false;profileRemoved=$false;at=[datetime]::UtcNow.ToString('o')}
$user=$null;$profile=$null;$process=$null;$desktop=$null;$secret=$null
function Save-Controller {$report|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $c.output 'controller.json') -Encoding utf8}
try{
 $secret=ConvertTo-SecureString ([Convert]::ToBase64String([Security.Cryptography.RandomNumberGenerator]::GetBytes(36))+'aA1!') -AsPlainText -Force
 $name='ocxw1d'+[guid]::NewGuid().ToString('N').Substring(0,9)
 $user=New-LocalUser -Name $name -Password $secret -AccountNeverExpires -PasswordNeverExpires -Description 'Disposable OpenCodex024 W1-D native test'
 Add-LocalGroupMember -Group (Get-LocalGroup -SID 'S-1-5-32-545') -Member $user
 $report.created=$true;$report.name=$name;$report.sid=$user.SID.Value;Save-Controller
 Add-Type -TypeDefinition @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class W1cProfile {
 [DllImport("userenv.dll",CharSet=CharSet.Unicode,ExactSpelling=true)] public static extern int CreateProfile(string sid,string name,StringBuilder path,uint size);
 [DllImport("userenv.dll",CharSet=CharSet.Unicode,SetLastError=true)] public static extern bool DeleteProfileW(string sid,string path,string computer);
}
'@
 $path=[Text.StringBuilder]::new(260)
 $hr=[W1cProfile]::CreateProfile($user.SID.Value,$name,$path,260)
 if($hr -ne 0){[Runtime.InteropServices.Marshal]::ThrowExceptionForHR($hr)}
 $profile=$path.ToString();$report.profile=$profile
 if([IO.Path]::GetFileName($profile) -cne $name){throw 'created profile spelling differs'}
 # Reuse the existing read/restore desktop ACL class, not its old CI scenario.
 $tokens=$null;$errors=$null
 $ast=[Management.Automation.Language.Parser]::ParseFile($c.desktopSource,[ref]$tokens,[ref]$errors)
 if($errors.Count){throw 'desktop helper source parse error'}
 $nodes=@($ast.FindAll({param($n)$n -is [Management.Automation.Language.StringConstantExpressionAst] -and $n.Value.Contains('public sealed class OcxTemporaryDesktopAccess')},$true))
 if($nodes.Count -ne 1){throw 'exact desktop ACL helper source required'}
 Add-Type -TypeDefinition $nodes[0].Value
 $desktop=[OcxTemporaryDesktopAccess]::new($user.SID.Value)
 $report.desktopAccessTemporarilyGranted=$true
 & icacls.exe $c.input /grant ($name+':(OI)(CI)RX') 1> (Join-Path $c.output 'input-acl.log')
 if($LASTEXITCODE -ne 0){throw 'test input ACL failed'}
 $evidence=Join-Path $c.output 'native'
 [void][IO.Directory]::CreateDirectory($evidence)
 & icacls.exe $evidence /grant ($name+':(OI)(CI)M') 1> (Join-Path $c.output 'evidence-acl.log')
 if($LASTEXITCODE -ne 0){throw 'test evidence ACL failed'}
 $childConfig=Join-Path $c.output 'child-input.json'
 @{sid=$user.SID.Value;profile=$profile;input=$c.input;evidence=$evidence;test=$c.test;componentOnly=[bool]$c.componentOnly;testName=$c.testName}|ConvertTo-Json|Set-Content -LiteralPath $childConfig -Encoding utf8
 $credential=[pscredential]::new(($env:COMPUTERNAME+'\'+$name),$secret)
 $process=Start-Process -FilePath (Join-Path $PSHOME 'pwsh.exe') -Credential $credential -LoadUserProfile -WorkingDirectory $c.input -WindowStyle Hidden -ArgumentList @('-NoLogo','-NoProfile','-NonInteractive','-File',('"'+$PSCommandPath+'"'),'-Config',('"'+$childConfig+'"'),'-Child') -RedirectStandardOutput (Join-Path $c.output 'bootstrap.stdout.log') -RedirectStandardError (Join-Path $c.output 'bootstrap.stderr.log') -PassThru
 $null=$process.Handle;$report.childPid=$process.Id;$report.childCreation=$process.StartTime.ToString('o');Save-Controller
 $deadline=[datetime]::UtcNow.AddMinutes(20)
 while(!$process.WaitForExit(1000)){if([datetime]::UtcNow -gt $deadline){$process.Kill($true);[void]$process.WaitForExit(5000);throw 'owned isolated test tree deadline; retained evidence'}}
 $report.childExit=$process.ExitCode
 $report.passed=$process.ExitCode -eq 0
}catch{$report.firstError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber};$_|Out-String|Set-Content -LiteralPath (Join-Path $c.output 'controller-error.log') -Encoding utf8}
finally{
 if($process){$process.Dispose()}
 if($desktop){try{$desktop.Dispose();$report.desktopAclRestored=$true}catch{$report.desktopRestoreError=$_.Exception.Message}}
 if($user){
  try{
   if($profile -and [IO.Path]::GetFileName($profile) -ceq $user.Name){
    # Native profile deletion targets only the exact newly created SID; evidence
    # remains in the external output. Unknown/loaded profile failures stay visible.
    $deleted=[W1cProfile]::DeleteProfileW($user.SID.Value,$null,$null)
    $report.profileDeleteApi=$deleted;$report.profileDeleteWin32=[Runtime.InteropServices.Marshal]::GetLastWin32Error()
    $report.profileRemoved=!(Test-Path -LiteralPath $profile)
    if(!$report.profileRemoved){
     $nativeProfile=@(Get-CimInstance Win32_UserProfile -Filter ("SID='"+$user.SID.Value+"'"))
     if($nativeProfile.Count -eq 1 -and !$nativeProfile[0].Loaded -and !$nativeProfile[0].Special -and $nativeProfile[0].LocalPath -ceq $profile){
      $nativeProfile[0]|Remove-CimInstance -ErrorAction Stop
      $report.profileRemoved=!(Test-Path -LiteralPath $profile)
      $report.profileServiceFallback='exact new SID via Win32_UserProfile; no recursive path deletion'
     }
    }
   }
   Remove-LocalUser -SID $user.SID -ErrorAction Stop
   $report.accountRemoved=$null -eq (Get-LocalUser -SID $user.SID -ErrorAction SilentlyContinue)
  }catch{$report.cleanupError=$_.Exception.Message}
 }
 if($secret){$secret.Dispose()}
 Save-Controller
}
if(!$report.passed){exit 1}

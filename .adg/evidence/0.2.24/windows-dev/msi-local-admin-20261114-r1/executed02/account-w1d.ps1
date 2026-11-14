param([Parameter(Mandatory)][string]$Config,[switch]$Child)
$ErrorActionPreference='Stop'
$c=Get-Content -LiteralPath $Config -Raw|ConvertFrom-Json
if($Child){
 $report=[ordered]@{passed=$false;scope=if($c.testAdministrator){'one disposable administrator; MSI only; not ordinary-user acceptance'}else{'one fresh ordinary account; W1-D only'}}
 try{
  $id=[Security.Principal.WindowsIdentity]::GetCurrent()
  if($id.User.Value -cne $c.sid){throw 'exact isolated account SID required'}
  $elevated=([Security.Principal.WindowsPrincipal]::new($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
  if($elevated -ne [bool]$c.testAdministrator){throw 'exact expected test token required'}
  if($c.testAdministrator -and (!$c.msiOnly -or !$c.authorizedLocalAdministratorMsi -or $c.testName -cne 'isolated_msi_uninstall_administrator' -or $c.sid -ceq $c.controllerSid -or $c.expectedGuestComputer -ine $env:COMPUTERNAME)){throw 'explicit disposable administrator MSI scenario required'}
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
  $env:OCXD024_MSI_ONLY_GUEST=if($c.msiOnly -and !$c.testAdministrator){'1'}else{$null}
  $env:OCXD024_MSI_LOCAL_ADMIN=if($c.testAdministrator){'1'}else{$null}
  $env:OCXD024_MSI_GUEST_NAME=if($c.msiOnly){$c.expectedGuestComputer}else{$null}
  $env:OCXD024_MSI_RESUME_EVIDENCE=if($c.resumeEvidence){$c.resumeEvidence}else{$null}
  if($c.msiOnly){
   $policyRows=@()
   foreach($hive in @('HKLM:','HKCU:')){
    $path=$hive+'/SOFTWARE/Policies/Microsoft/Windows/Installer'
    $value=if(Test-Path -LiteralPath $path){Get-ItemProperty -LiteralPath $path|Select-Object DisableMSI,AlwaysInstallElevated,DisableUserInstalls}else{$null}
    $policyRows+=@{path=$path;exists=Test-Path -LiteralPath $path;values=$value;tokenSid=$id.User.Value;absenceDoesNotProvePermission=$true}
   }
   $policyRows|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $c.evidence 'test-token-policy.json') -Encoding utf8
   $adminScenario=[bool]$c.testAdministrator
   if(@($policyRows|Where-Object {$_.values -and (($null -ne $_.values.DisableMSI -and $_.values.DisableMSI -ne 0 -and !($adminScenario -and $_.values.DisableMSI -eq 1)) -or $_.values.AlwaysInstallElevated -eq 1 -or $_.values.DisableUserInstalls -eq 1)}).Count){throw 'test-token policy incompatible; no MSI run'}
  }
  $report.sid=$id.User.Value;$report.profile=$registered;$report.session=[Diagnostics.Process]::GetCurrentProcess().SessionId;$report.elevated=$elevated
  $testName=if($c.testName){$c.testName}else{'isolated_account_terminal_w1d'}
  if($testName -cnotin @('isolated_account_terminal_w1d','isolated_schema3_invariants_w1d','isolated_msi_uninstall_only','isolated_msi_uninstall_administrator')){throw 'exact Windows ignored test required'}
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
if($c.msiOnly){
 if($c.testAdministrator){
  if(!$c.authorizedLocalAdministratorMsi -or $c.expectedGuestComputer -ine $env:COMPUTERNAME -or $c.componentOnly -or $c.testName -cne 'isolated_msi_uninstall_administrator'){throw 'explicit local administrator MSI authorization required before account creation'}
 }elseif($env:COMPUTERNAME -ieq 'EZIO-MINI' -or $c.expectedGuestComputer -ine $env:COMPUTERNAME -or !$c.preparedPerUserMsiGuest -or $c.componentOnly -or $c.testName -cne 'isolated_msi_uninstall_only'){throw 'explicit separate prepared MSI guest required before account creation'}
}
if($c.testAdministrator -and !$c.msiOnly){throw 'administrator scope is MSI-only'}
$report=[ordered]@{passed=$false;scope='one isolated account; no W2/two-user acceptance';created=$false;accountRemoved=$false;profileRemoved=$false;at=[datetime]::UtcNow.ToString('o')}
$user=$null;$profile=$null;$process=$null;$desktop=$null;$secret=$null;$taskName=$null;$taskStarted=$false;$inputAcl=$null
function Save-Controller {$report|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $c.output 'controller.json') -Encoding utf8}
try{
 $secret=ConvertTo-SecureString ([Convert]::ToBase64String([Security.Cryptography.RandomNumberGenerator]::GetBytes(36))+'aA1!') -AsPlainText -Force
 if($c.resumeController){
  if(!$c.testAdministrator -or !$c.resumeEvidence){throw 'resume is only for the original disposable administrator MSI identity'}
  $prior=Get-Content -LiteralPath $c.resumeController -Raw|ConvertFrom-Json
  $state=Get-Content -LiteralPath (Join-Path $c.resumeEvidence 'msi-final-registration.json') -Raw|ConvertFrom-Json
  if(!$prior.created -or !$prior.administratorScenario -or $prior.accountRemoved -or $prior.name -notmatch '^ocxw1d[0-9a-f]{9}$' -or $state.productCode -cne '{67CF3D62-492D-4359-AAFE-8BE6FB17E647}' -or $state.state -ne 5){throw 'exact prior retained MSI owner required'}
  $user=Get-LocalUser -SID $prior.sid -ErrorAction Stop
  if($user.Name -cne $prior.name -or $user.SID.Value -ceq $identity.User.Value){throw 'prior test identity differs'}
  $name=$user.Name
  Set-LocalUser -SID $user.SID -Password $secret -ErrorAction Stop
  $profile=$prior.profile;$report.reusedOriginalTestSid=$true
 }else{
  $name='ocxw1d'+[guid]::NewGuid().ToString('N').Substring(0,9)
  $user=New-LocalUser -Name $name -Password $secret -AccountNeverExpires -PasswordNeverExpires -Description 'Disposable OpenCodex024 W1-D native test'
  Add-LocalGroupMember -Group (Get-LocalGroup -SID 'S-1-5-32-545') -Member $user
  if($c.testAdministrator){Add-LocalGroupMember -Group (Get-LocalGroup -SID 'S-1-5-32-544') -Member $user}
 }
 if($c.testAdministrator){$report.administratorScenario=$true;$report.ordinaryUserAcceptance=$false}
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
 if(!$c.resumeController){
  $path=[Text.StringBuilder]::new(260)
  $hr=[W1cProfile]::CreateProfile($user.SID.Value,$name,$path,260)
  if($hr -ne 0){[Runtime.InteropServices.Marshal]::ThrowExceptionForHR($hr)}
  $profile=$path.ToString()
 }
 $report.profile=$profile
 if([IO.Path]::GetFileName($profile) -cne $name){throw 'created profile spelling differs'}
 if(!$c.testAdministrator){
 # Reuse the existing read/restore desktop ACL class, not its old CI scenario.
 $tokens=$null;$errors=$null
 $ast=[Management.Automation.Language.Parser]::ParseFile($c.desktopSource,[ref]$tokens,[ref]$errors)
 if($errors.Count){throw 'desktop helper source parse error'}
 $nodes=@($ast.FindAll({param($n)$n -is [Management.Automation.Language.StringConstantExpressionAst] -and $n.Value.Contains('public sealed class OcxTemporaryDesktopAccess')},$true))
 if($nodes.Count -ne 1){throw 'exact desktop ACL helper source required'}
 Add-Type -TypeDefinition $nodes[0].Value
 $desktop=[OcxTemporaryDesktopAccess]::new($user.SID.Value)
 $report.desktopAccessTemporarilyGranted=$true
 }else{$report.desktopAclRestored=$true;$report.desktopAccess='not modified; dedicated scheduled task session'}
 $inputAcl=Get-Acl -LiteralPath $c.input
 & icacls.exe $c.input /grant ($name+':(OI)(CI)RX') 1> (Join-Path $c.output 'input-acl.log')
 if($LASTEXITCODE -ne 0){throw 'test input ACL failed'}
 $evidence=Join-Path $c.output 'native'
 [void][IO.Directory]::CreateDirectory($evidence)
 & icacls.exe $evidence /grant ($name+':(OI)(CI)M') 1> (Join-Path $c.output 'evidence-acl.log')
 if($LASTEXITCODE -ne 0){throw 'test evidence ACL failed'}
 $childConfig=Join-Path $c.output 'child-input.json'
 @{sid=$user.SID.Value;profile=$profile;input=$c.input;evidence=$evidence;test=$c.test;componentOnly=[bool]$c.componentOnly;testName=$c.testName;msiOnly=[bool]$c.msiOnly;expectedGuestComputer=$c.expectedGuestComputer;testAdministrator=[bool]$c.testAdministrator;authorizedLocalAdministratorMsi=[bool]$c.authorizedLocalAdministratorMsi;controllerSid=$identity.User.Value;resumeEvidence=$c.resumeEvidence}|ConvertTo-Json|Set-Content -LiteralPath $childConfig -Encoding utf8
 $credential=[pscredential]::new(($env:COMPUTERNAME+'\'+$name),$secret)
 if($c.testAdministrator){
  $taskName='ocx024-msi-'+[guid]::NewGuid().ToString('N')
  $report.taskName=$taskName;Save-Controller
  $action=New-ScheduledTaskAction -Execute (Join-Path $PSHOME 'pwsh.exe') -WorkingDirectory $c.input -Argument ('-NoLogo -NoProfile -NonInteractive -File "'+$PSCommandPath+'" -Config "'+$childConfig+'" -Child')
  $settings=New-ScheduledTaskSettingsSet -ExecutionTimeLimit (New-TimeSpan -Minutes 20) -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
  $plain=$credential.GetNetworkCredential().Password
  try{$null=Register-ScheduledTask -TaskName $taskName -Action $action -Settings $settings -User $credential.UserName -Password $plain -RunLevel Highest -ErrorAction Stop}finally{$plain=$null}
  Start-ScheduledTask -TaskName $taskName -ErrorAction Stop;$taskStarted=$true
  $deadline=[datetime]::UtcNow.AddMinutes(20)
  do{
   Start-Sleep -Milliseconds 1000
   $task=Get-ScheduledTask -TaskName $taskName -ErrorAction Stop
   $info=Get-ScheduledTaskInfo -TaskName $taskName -ErrorAction Stop
   if([datetime]::UtcNow -gt $deadline){Stop-ScheduledTask -TaskName $taskName;throw 'owned isolated test tree deadline; retained evidence'}
  }while($task.State -eq 'Running' -or $task.State -eq 'Queued' -or $info.LastRunTime.Year -lt 2026)
  $report.childExit=$info.LastTaskResult;$report.passed=$info.LastTaskResult -eq 0
 }else{
 $process=Start-Process -FilePath (Join-Path $PSHOME 'pwsh.exe') -Credential $credential -LoadUserProfile -WorkingDirectory $c.input -WindowStyle Hidden -ArgumentList @('-NoLogo','-NoProfile','-NonInteractive','-File',('"'+$PSCommandPath+'"'),'-Config',('"'+$childConfig+'"'),'-Child') -RedirectStandardOutput (Join-Path $c.output 'bootstrap.stdout.log') -RedirectStandardError (Join-Path $c.output 'bootstrap.stderr.log') -PassThru
 $null=$process.Handle;$report.childPid=$process.Id;$report.childCreation=$process.StartTime.ToString('o');Save-Controller
 $deadline=[datetime]::UtcNow.AddMinutes(20)
 while(!$process.WaitForExit(1000)){if([datetime]::UtcNow -gt $deadline){$process.Kill($true);[void]$process.WaitForExit(5000);throw 'owned isolated test tree deadline; retained evidence'}}
 $report.childExit=$process.ExitCode
 $report.passed=$process.ExitCode -eq 0
 }
}catch{$report.firstError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber};$_|Out-String|Set-Content -LiteralPath (Join-Path $c.output 'controller-error.log') -Encoding utf8}
finally{
 if($taskName){try{Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction Stop;$report.taskRemoved=$null -eq (Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue)}catch{$report.taskCleanupError=$_.Exception.Message;$report.passed=$false}}
 if($process){$process.Dispose()}
 if($inputAcl){try{Set-Acl -LiteralPath $c.input -AclObject $inputAcl;$report.inputAclRestored=(Get-Acl -LiteralPath $c.input).Sddl -ceq $inputAcl.Sddl}catch{$report.inputAclError=$_.Exception.Message;$report.passed=$false}}
 if($desktop){try{$desktop.Dispose();$report.desktopAclRestored=$true}catch{$report.desktopRestoreError=$_.Exception.Message}}
 if($user){
  try{
   $allowIdentityRemoval=$true
   if($c.msiOnly){
    $statePath=Join-Path (Join-Path $c.output 'native') 'msi-final-registration.json'
    if(!$process -and !$taskStarted -and !$c.resumeController){$report.msiNeverStarted='child process/task was not started; identity cleanup allowed'}elseif(!(Test-Path -LiteralPath $statePath)){$allowIdentityRemoval=$false;$report.cleanupError='MSI registration outcome unknown; account/Profile retained'}else{
     $last=Get-Content -LiteralPath $statePath -Raw|ConvertFrom-Json
     $allowIdentityRemoval=$last.productCode -ceq '{67CF3D62-492D-4359-AAFE-8BE6FB17E647}' -and $last.state -eq -1 -and $last.safeToRemoveTestIdentity
     if(!$allowIdentityRemoval){$report.cleanupError='MSI product is still registered or unknown; account/Profile retained for native recovery'}
    }
   }
   if($allowIdentityRemoval){
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
   }
  }catch{$report.cleanupError=$_.Exception.Message}
 }
 if($secret){$secret.Dispose()}
 if($c.msiOnly -and (!$report.accountRemoved -or !$report.profileRemoved -or !$report.desktopAclRestored -or $report.cleanupError)){$report.passed=$false}
 Save-Controller
}
if(!$report.passed){exit 1}

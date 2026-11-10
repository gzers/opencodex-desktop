# Invoke once as administrator; starts one NEW ordinary account and retains its Profile handle.
param([Parameter(Mandatory)][string]$Directory,[Parameter(Mandatory)][string]$InputRoot,[Parameter(Mandatory)][string]$ArchiveRoot,[switch]$RestorationRevision2,[switch]$RestorationRevision4,[switch]$RestorationRevision5)
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
Add-Type -Path (Join-Path $PSScriptRoot 'Native.cs')
$parent=[Security.Principal.WindowsIdentity]::GetCurrent()
if(!([Security.Principal.WindowsPrincipal]::new($parent)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'administrator provisioner required; runtime host remains ordinary'}
$dir=[IO.Path]::GetFullPath($Directory)
if((Split-Path -Parent $dir) -ine 'C:\ProgramData' -or (Split-Path -Leaf $dir) -cnotmatch '^ocxd299-proof-[a-f0-9]{10}$' -or (Test-Path -LiteralPath $dir)){throw 'fresh explicitly scoped ProgramData directory required'}
$null=New-Item -ItemType Directory -Path $dir
$report=[ordered]@{schema='official299-windows-controller/1';accepted=$false;stage='input';firstError=$null;cleanup=@();ordinary=$null;profileUnloaded=$false;profileDeleted=$false;accountDeleted=$false;noScheduledTaskCreated=$true;parent=[Official299Proof.Native]::ObserveSelf()}
function Save { [IO.File]::WriteAllText((Join-Path $dir 'controller.json'),($report|ConvertTo-Json -Depth 35)) }
function Hash([string]$p){(Get-FileHash -LiteralPath $p -Algorithm SHA256).Hash.ToLowerInvariant()}
$account=$null;$token=[IntPtr]::Zero;$profileInfo=[Official299Proof.Native+PROFILE]::new();$pi=[Official299Proof.Native+PI]::new();$envBlock=[IntPtr]::Zero;$password=$null;$profile=$null
try {
 if((Get-Item -LiteralPath (Join-Path $ArchiveRoot 'official.tgz')).Length -ne 19923693 -or (Hash (Join-Path $ArchiveRoot 'official.tgz')) -cne 'b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c'){throw 'fixed official archive identity failed'}
 $acl=[Security.AccessControl.DirectorySecurity]::new();$acl.SetOwner($parent.User);$acl.SetAccessRuleProtection($true,$false)
 foreach($s in @($parent.User.Value,'S-1-5-18','S-1-5-32-544')){$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($s),'FullControl','ContainerInherit,ObjectInherit','None','Allow'))};Set-Acl -LiteralPath $dir -AclObject $acl
 $report.stage='provision-account';$name='ocx299'+[guid]::NewGuid().ToString('N').Substring(0,10)
 $password=[Convert]::ToBase64String([Security.Cryptography.RandomNumberGenerator]::GetBytes(32))+'aA1!'
 $account=New-LocalUser -Name $name -Password (ConvertTo-SecureString $password -AsPlainText -Force) -AccountNeverExpires -Description 'Disposable official299 targeted proof'
 $sid=$account.SID.Value;$report.account=@{name=$name;sid=$sid};Save
 Add-LocalGroupMember -Group (Get-LocalGroup -SID 'S-1-5-32-545') -Member $account
 if(@(Get-LocalGroupMember -Group (Get-LocalGroup -SID 'S-1-5-32-544')|Where-Object {$_.SID.Value -ceq $sid}).Count){throw 'new account unexpectedly administrator'}
 $buf=[Text.StringBuilder]::new(260);$hr=[Official299Proof.Native]::CreateProfile($sid,$name,$buf,260);$report.createProfileHResult=$hr
 if($hr -ne 0){[Runtime.InteropServices.Marshal]::ThrowExceptionForHR($hr)}
 $profile=$buf.ToString();$report.profile=$profile;Save
 if((Split-Path -Leaf $profile) -cne $name){throw 'OS Profile path differs'}
 $code=Join-Path $dir 'code';$output=Join-Path $dir 'ordinary';$tools=Join-Path $dir 'powershell';$inputs=Join-Path $dir 'input';$archive=Join-Path $dir 'archive'
 $null=New-Item -ItemType Directory -Path $code,$output,$inputs,$archive
 if(@(@($RestorationRevision2,$RestorationRevision4,$RestorationRevision5)|Where-Object {$_}).Count -gt 1){throw 'select one revision'}
 $hostLeaf=if($RestorationRevision5){'host-r5.ps1'}elseif($RestorationRevision4){'host-r4.ps1'}elseif($RestorationRevision2){'host-r2.ps1'}else{'host.ps1'}
 $copyLeaves=@('Native.cs',$hostLeaf,'policy.ps1');if($RestorationRevision2 -or $RestorationRevision4 -or $RestorationRevision5){$copyLeaves+=@('Recovery.cs','lock-worker.ts')};if($RestorationRevision4 -or $RestorationRevision5){$copyLeaves+=@('FiveRecovery.cs','producer-r4.ts','coordinator-r4.ts')};if($RestorationRevision5){$copyLeaves+=@('cache-candidate.ts','cache-candidate-cli.ts')}
 foreach($f in $copyLeaves){Copy-Item -LiteralPath (Join-Path $PSScriptRoot $f) -Destination $code}
 Copy-Item -LiteralPath $PSHOME -Destination $tools -Recurse
 Copy-Item -LiteralPath (Join-Path $InputRoot 'node_modules') -Destination $inputs -Recurse
 Copy-Item -LiteralPath (Join-Path $InputRoot 'package-lock.json') -Destination $inputs
 Copy-Item -LiteralPath (Join-Path $ArchiveRoot 'package') -Destination $archive -Recurse
 $bun=Join-Path $inputs 'node_modules/@oven/bun-windows-x64/bin/bun.exe'
 $input=@{sid=$sid;profile=$profile;output=$output;inputRoot=$inputs;archiveRoot=$archive;bunSha256=(Hash $bun);bunVersion=(& $bun --version).Trim();archiveSha256=(Hash (Join-Path $ArchiveRoot 'official.tgz'));lockSha256=(Hash (Join-Path $inputs 'package-lock.json'))}
 $report.input=$input;$report.hostHashes=@{native=(Hash (Join-Path $code 'Native.cs'));host=(Hash (Join-Path $code $hostLeaf));hostLeaf=$hostLeaf;provisioner=(Hash $PSCommandPath);powershell=(Hash (Join-Path $tools 'pwsh.exe'))}
 if($RestorationRevision2 -or $RestorationRevision4 -or $RestorationRevision5){$report.hostHashes.recovery=Hash (Join-Path $code 'Recovery.cs');$report.hostHashes.lockWorker=Hash (Join-Path $code 'lock-worker.ts')}
 if($RestorationRevision4 -or $RestorationRevision5){$report.hostHashes.five=Hash (Join-Path $code 'FiveRecovery.cs');$report.hostHashes.producer=Hash (Join-Path $code 'producer-r4.ts');$report.hostHashes.coordinator=Hash (Join-Path $code 'coordinator-r4.ts')}
 if($RestorationRevision5){$report.hostHashes.cacheCandidate=Hash (Join-Path $code 'cache-candidate.ts');$report.hostHashes.cacheCli=Hash (Join-Path $code 'cache-candidate-cli.ts')}
 $config=Join-Path $dir 'input.json';[IO.File]::WriteAllText($config,($input|ConvertTo-Json -Depth 8))
 # Grants affect only this freshly created fixture tree; no existing Profile/source ACL is repaired.
 $acl=Get-Acl $dir;$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($account.SID,'ReadAndExecute','ContainerInherit,ObjectInherit','None','Allow'));Set-Acl -LiteralPath $dir -AclObject $acl
 $acl=Get-Acl $output;$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($account.SID,'Modify','ContainerInherit,ObjectInherit','None','Allow'));Set-Acl -LiteralPath $output -AclObject $acl
 $report.stage='load-profile';[Official299Proof.Native]::EnableProfilePrivileges()
 [Official299Proof.Native]::Check([Official299Proof.Native]::LogonUserW($name,$env:COMPUTERNAME,$password,2,0,[ref]$token))
 $profileInfo.size=[Runtime.InteropServices.Marshal]::SizeOf($profileInfo);$profileInfo.flags=1;$profileInfo.name=$name
 [Official299Proof.Native]::Check([Official299Proof.Native]::LoadUserProfileW($token,[ref]$profileInfo));$report.originalProfileHandleRetained=$true;Save
 $ev=@{SystemRoot=$env:SystemRoot;WINDIR=$env:WINDIR;COMSPEC=(Join-Path $env:SystemRoot 'System32/cmd.exe');PATH=($tools+';'+$env:SystemRoot+'\System32');PATHEXT='.COM;.EXE;.BAT;.CMD';USERPROFILE=$profile;HOME=$profile;USERNAME=$name;USERDOMAIN=$env:COMPUTERNAME;APPDATA=(Join-Path $profile 'AppData/Roaming');LOCALAPPDATA=(Join-Path $profile 'AppData/Local');TEMP=(Join-Path $profile 'AppData/Local/Temp');TMP=(Join-Path $profile 'AppData/Local/Temp')}
 $envBlock=[Official299Proof.Native]::EnvironmentBlock($ev);$si=[Official299Proof.Native+SI]::new();$si.cb=[Runtime.InteropServices.Marshal]::SizeOf($si)
 $command='"'+(Join-Path $tools 'pwsh.exe')+'" -NoLogo -NoProfile -NonInteractive -File "'+(Join-Path $code $hostLeaf)+'" -InputFile "'+$config+'"'
 $report.stage='ordinary-host'
 [Official299Proof.Native]::Check([Official299Proof.Native]::CreateProcessWithLogonW($name,$env:COMPUTERNAME,$password,0,(Join-Path $tools 'pwsh.exe'),[Text.StringBuilder]::new($command),0x08000400,$envBlock,$dir,[ref]$si,[ref]$pi))
 $report.ordinaryOriginalIdentity=[Official299Proof.Native]::Observe($pi.process);$password=$null;Save
 if($report.ordinaryOriginalIdentity.sid -cne $sid -or $report.ordinaryOriginalIdentity.elevated){throw 'ordinary host token mismatch'}
 $watch=[Diagnostics.Stopwatch]::StartNew()
 while([Official299Proof.Native]::WaitForSingleObject($pi.process,0) -eq 258){if($watch.ElapsedMilliseconds -gt 180000){throw 'ordinary transport envelope expired; no lifecycle retry'};Start-Sleep -Milliseconds 200}
 [uint32]$codeExit=0;[Official299Proof.Native]::Check([Official299Proof.Native]::GetExitCodeProcess($pi.process,[ref]$codeExit));$report.ordinaryOriginalExit=$codeExit
 if(!(Test-Path -LiteralPath (Join-Path $output 'result.json'))){throw 'ordinary host result missing; original exit retained'}
 $child=Get-Content -LiteralPath (Join-Path $output 'result.json') -Raw|ConvertFrom-Json;$report.ordinary=$child
 $report.accepted=($codeExit -eq 0 -and $child.accepted);$report.stage='ordinary-finished'
} catch {$report.firstError=@{stage=$report.stage;type=$_.Exception.GetType().FullName;message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber;hresult=$_.Exception.HResult}}
finally {
 $password=$null
 if($envBlock -ne [IntPtr]::Zero){[Runtime.InteropServices.Marshal]::FreeHGlobal($envBlock)}
 $childExited=($pi.process -eq [IntPtr]::Zero -or [Official299Proof.Native]::WaitForSingleObject($pi.process,0) -eq 0)
 if($pi.thread -ne [IntPtr]::Zero){$null=[Official299Proof.Native]::CloseHandle($pi.thread)}
 if($pi.process -ne [IntPtr]::Zero -and $childExited){$null=[Official299Proof.Native]::CloseHandle($pi.process)}
 if($childExited -and $profileInfo.handle -ne [IntPtr]::Zero){
  $ok=[Official299Proof.Native]::UnloadUserProfile($token,$profileInfo.handle);$err=[Runtime.InteropServices.Marshal]::GetLastWin32Error();$report.profileUnloaded=$ok;$report.cleanup+=@{kind='original-hProfile-unload';success=$ok;win32=$(if($ok){0}else{$err})}
 }
 if($token -ne [IntPtr]::Zero){$null=[Official299Proof.Native]::CloseHandle($token)}
 if($account){
  try{
   $current=Get-LocalUser -SID $account.SID
   if($current.Name -cne $name){throw 'exact new account changed'}
   Disable-LocalUser -SID $account.SID
   $prof=@(Get-CimInstance Win32_UserProfile -Filter ("SID='"+$account.SID.Value+"'"))
   $report.cleanup+=@{kind='profile-readback';values=@($prof|Select-Object SID,LocalPath,Loaded,Special)}
   $lastJob=if($null -ne $report.ordinary){@($report.ordinary.cleanup|Where-Object {$_.PSObject.Properties.Name -contains 'jobActive'})|Select-Object -Last 1}else{$null}
   $jobClear=($null -eq $report.ordinary -and $pi.process -eq [IntPtr]::Zero) -or ($null -ne $report.ordinary -and $null -ne $lastJob -and $lastJob.jobActive -eq 0 -and $lastJob.jobMembers.Count -eq 0)
   if($childExited -and $jobClear -and $prof.Count -le 1 -and ($prof.Count -eq 0 -or (!$prof[0].Loaded -and !$prof[0].Special -and $prof[0].LocalPath -ieq $profile))){
    if($prof.Count){$prof[0]|Remove-CimInstance}
    $report.profileDeleted=(@(Get-CimInstance Win32_UserProfile -Filter ("SID='"+$account.SID.Value+"'")).Count -eq 0 -and !(Test-Path -LiteralPath $profile))
    if($report.profileDeleted){Remove-LocalUser -SID $account.SID;$report.accountDeleted=(@(Get-LocalUser|Where-Object {$_.SID.Value -ceq $account.SID.Value}).Count -eq 0)}
   }else{$report.cleanup+=@{kind='retained-profile-account';reason='loaded-or-incomplete-owned-process-proof';childExited=$childExited;jobClear=$jobClear}}
  }catch{$report.cleanup+=@{kind='cleanup-first-error';type=$_.Exception.GetType().FullName;message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
 }
 Save
}
if(!$report.accepted -or !$report.profileDeleted -or !$report.accountDeleted){exit 1}

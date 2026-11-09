param(
 [ValidateSet('Controller','Child')][string]$Mode='Controller',
 [string]$InputFile,
 [string]$Directory,
 [string]$BundleDirectory,
 [string]$Node,
 [ValidateSet('msi','nsis')][string]$Kind='nsis'
)
$ErrorActionPreference='Stop'
$installerMode=$Mode
. (Join-Path $PSScriptRoot 'windows023-two-user-native-helper.ps1')
. (Join-Path $PSScriptRoot 'windows-process-token.ps1')
if(!$IsWindows){throw 'Native Windows required'}
if($installerMode -eq 'Child'){
 $c=Get-Content -LiteralPath $InputFile -Raw|ConvertFrom-Json
 $id=[Security.Principal.WindowsIdentity]::GetCurrent();$sid=$id.User.Value
 if($sid -cne $c.sid -or $env:USERPROFILE -ine $c.profile -or (Split-Path -Leaf $c.profile) -notlike 'ocxaccu1*'){throw 'Exact new installation-owner Profile required'}
 $token=Get-WindowsHarnessToken;Assert-WindowsProcessMode $token ordinary
 $manifest=Get-Content -LiteralPath (Join-Path $c.bundle 'installer-build.json') -Raw|ConvertFrom-Json
 $result=[ordered]@{accepted=$false;source=$manifest.source;kind=$c.kind;sid=$sid;profile=$c.profile;token=$token;cells=@();passed=$false;scope='Fresh local owner; default/custom install, exact image, offline supply, same-version reinstall retention, uninstall user-data retention; no service Start/sync'}
 $ledger=[Collections.Generic.Dictionary[int,object]]::new()
 $currentInstall=$null;$package=$null;$installed=$false
 function Invoke-LocalInstaller([string]$file,[string]$arguments,[string]$label){
  $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$file;$info.Arguments=$arguments;$info.UseShellExecute=$false;$info.CreateNoWindow=$true
  $p=[Diagnostics.Process]::Start($info);[void]$p.Handle
  try{if(!$p.WaitForExit(180000)){throw 'Installer transport deadline; no retry'};$exit=$p.ExitCode;Write-TwoUserJson (Join-Path $c.out ($label+'-exit.json')) @{pid=$p.Id;originalHandleExited=$p.HasExited;exitCode=$exit;kind=$c.kind;label=$label;atUtc=[datetime]::UtcNow.ToString('o')};return $exit}finally{$p.Dispose()}
 }
 function Check-InstalledImage([string]$root,[string]$label){
  $exe=Join-Path $root 'opencodex-desktop.exe'
  $raw=& $c.node (Join-Path $PSScriptRoot 'windows-bundle-identity.mjs') (Join-Path $c.bundle 'opencodex-desktop.exe') $exe $c.kind
  if($LASTEXITCODE -ne 0){throw 'Installed image does not match exact bundler transformation'}
  Write-TwoUserJson (Join-Path $c.out ($label+'-image.json')) ($raw|ConvertFrom-Json)
  if((Get-Item -LiteralPath $exe).VersionInfo.ProductVersion -cne '0.2.23'){throw 'Installed version differs'}
  $d=Get-Content -LiteralPath (Join-Path $root 'runtime-bundle/runtime-bundle.manifest.json') -Raw|ConvertFrom-Json
  $tgz=Join-Path $root 'runtime-bundle/maintained-runtime.tgz'
  if($d.runtimeVersion -cne '2.97.0-desktop023.lifecycle.3' -or (Get-Item -LiteralPath $tgz).Length -ne 15413397 -or (Get-FileHash -LiteralPath $tgz).Hash -ine 'ec14baaee354c6729d1d2545ab9165d207b14a3a4239f354662163087eeb0739'){throw 'Installed runtime bundle differs'}
  if((Get-FileHash -LiteralPath (Join-Path $root ('runtime-bundle/'+$d.archiveName))).Hash -ine $d.archiveSha256){throw 'Installed complete offline archive differs'}
 }
 function Observe-InstalledStartup([string]$root,[string]$label){
  $out=Join-Path $c.out $label;New-Item -ItemType Directory -Path $out|Out-Null
  $exe=Join-Path $root 'opencodex-desktop.exe'
  $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$exe;$info.WorkingDirectory=$root;$info.UseShellExecute=$false;$info.CreateNoWindow=$true
  foreach($key in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_WINDOWS_SMOKE_CDP_PORT','HOME','OPENCODEX_HOME','CODEX_HOME','CLAUDE_CONFIG_DIR','WEBVIEW2_USER_DATA_FOLDER','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS')){[void]$info.Environment.Remove($key)}
  $log=Join-Path $c.profile '.codex-desktop/desktop/logs/app.log'
  $logStart=if(Test-Path -LiteralPath $log){@(Get-Content -LiteralPath $log).Count}else{0}
  $manager=[Diagnostics.Process]::Start($info);$fact=Get-TwoUserProcessFact $manager $sid;$ledger[[int]$fact.pid]=$fact
  try{
   $appToken=Get-WindowsProcessToken $manager $exe;Assert-WindowsProcessMode $appToken ordinary
   $deadline=[datetime]::UtcNow.AddSeconds(600);$line=$null;$window=$null
   do{
    Add-TwoUserDescendants $fact $ledger
    try{$window=Get-TwoUserWindow $fact}catch{}
    if(Test-Path -LiteralPath $log){$line=Get-Content -LiteralPath $log|Select-Object -Skip $logStart|Select-String 'integrated runtime (provisioned and verified|provisioning failed)'|Select-Object -Last 1}
    if($line -and $window){break};if($manager.HasExited){throw 'Installed application exited before first-screen/supply'}
    Start-Sleep -Milliseconds 300
   }while([datetime]::UtcNow -lt $deadline)
   if(!$window -or !$line -or !$line.Line.Contains('integrated runtime provisioned and verified')){throw 'Installed native first-screen/offline supply not proved'}
   if(!(Get-Content -LiteralPath $log -Raw).Contains('commit='+$manifest.source)){throw 'Installed build source differs'}
   Copy-Item -LiteralPath $log -Destination (Join-Path $out 'app.log')
   Write-TwoUserJson (Join-Path $out 'startup.json') @{sid=$sid;profile=$c.profile;session=[Diagnostics.Process]::GetCurrentProcess().SessionId;manager=$fact;token=$appToken;window=$window;supply=$line.Line;sandbox=$false;serviceStartExecuted=$false;actualUtc=[datetime]::UtcNow.ToString('o')}
  }finally{
   try{Add-TwoUserDescendants $fact $ledger}catch{if(!$manager.HasExited){throw}}
   foreach($f in @($ledger.Values|Sort-Object {[long]$_.creationTicks} -Descending)){Stop-TwoUserTracked $f}
   $manager.Dispose();Close-TwoUserProcessHandles;$ledger.Clear()
  }
 }
 try{
  foreach($f in $manifest.files){$p=Join-Path $c.bundle $f.name;$null=Assert-TwoUserUnder $c.bundle $p;if((Get-Item -LiteralPath $p).Length -ne $f.bytes -or (Get-FileHash -LiteralPath $p).Hash -ine $f.sha256){throw 'Input installer manifest differs'}}
  $pattern=if($c.kind -eq 'nsis'){'*-setup.exe'}else{'*.msi'}
  $packages=@(Get-ChildItem -LiteralPath $c.bundle -Filter $pattern -File);if($packages.Count -ne 1){throw 'Exact one installer package required'};$package=$packages[0].FullName
  $default=Join-Path $env:LOCALAPPDATA 'Programs/OpenCodeX Desktop'
  foreach($mode in @('default','custom')){
   $currentInstall=if($mode -eq 'default'){$default}else{Join-Path $c.profile 'Installer fixture Ω/自定义 程序'}
   $null=Assert-TwoUserUnder $c.profile $currentInstall;Assert-TwoUserNoReparseAncestors $currentInstall
   if(Test-Path -LiteralPath (Join-Path $currentInstall 'opencodex-desktop.exe')){throw 'Installation fixture not fresh'}
   $arguments=if($c.kind -eq 'nsis'){'/S'+$(if($mode -eq 'custom'){' /D='+$currentInstall}else{''})}else{'/i "'+$package+'" /qn /norestart /L*v "'+(Join-Path $c.out ($mode+'-install.log'))+'"'+$(if($mode -eq 'custom'){' INSTALLDIR="'+$currentInstall+'"'}else{''})}
   $exit=Invoke-LocalInstaller $(if($c.kind -eq 'nsis'){$package}else{'msiexec.exe'}) $arguments ($mode+'-install')
   if($exit -notin @(0,3010)){throw ('Native installer refused: '+$exit+'; no policy change/retry')}
   $installed=$true;Check-InstalledImage $currentInstall ($mode+'-installed')
   Observe-InstalledStartup $currentInstall ($mode+'-startup')
   $sentinel=Join-Path $c.profile ('.codex-desktop/desktop/exports/local-installer-'+$mode+'.sentinel')
   New-Item -ItemType Directory -Path (Split-Path -Parent $sentinel) -Force|Out-Null
   if(Test-Path -LiteralPath $sentinel){throw 'Fresh sentinel required'}
   [IO.File]::WriteAllText($sentinel,'new isolated owner data retention')
   $sentinelHash=(Get-FileHash -LiteralPath $sentinel).Hash
   $runtimeBefore=Get-TwoUserTreeHash (Join-Path $currentInstall 'runtime')
   $reinstallArgs=if($c.kind -eq 'nsis'){'/S /D='+$currentInstall}else{'/i "'+$package+'" /qn /norestart REINSTALL=ALL REINSTALLMODE=vomus /L*v "'+(Join-Path $c.out ($mode+'-reinstall.log'))+'"'}
   $reinstall=Invoke-LocalInstaller $(if($c.kind -eq 'nsis'){$package}else{'msiexec.exe'}) $reinstallArgs ($mode+'-reinstall')
   if($reinstall -notin @(0,3010)){throw ('Same-version reinstall failed: '+$reinstall)}
   Check-InstalledImage $currentInstall ($mode+'-reinstalled')
   $runtimeAfter=Get-TwoUserTreeHash (Join-Path $currentInstall 'runtime')
   if($runtimeAfter.sha256 -cne $runtimeBefore.sha256 -or (Get-FileHash -LiteralPath $sentinel).Hash -cne $sentinelHash){throw 'Reinstall changed provisioned runtime or user data'}
   $uninstallArgs=if($c.kind -eq 'nsis'){'/S'}else{'/x "'+$package+'" /qn /norestart /L*v "'+(Join-Path $c.out ($mode+'-uninstall.log'))+'"'}
   $uninstall=Invoke-LocalInstaller $(if($c.kind -eq 'nsis'){Join-Path $currentInstall 'uninstall.exe'}else{'msiexec.exe'}) $uninstallArgs ($mode+'-uninstall')
   if($uninstall -notin @(0,3010)){throw ('Uninstall failed: '+$uninstall)}
   $until=[datetime]::UtcNow.AddSeconds(30)
   while((Test-Path -LiteralPath (Join-Path $currentInstall 'opencodex-desktop.exe')) -and [datetime]::UtcNow -lt $until){Start-Sleep -Milliseconds 200}
   if((Test-Path -LiteralPath (Join-Path $currentInstall 'opencodex-desktop.exe')) -or (Get-FileHash -LiteralPath $sentinel).Hash -cne $sentinelHash){throw 'Uninstall image-removal/user-data-retention not proved'}
   $installed=$false
   $result.cells+=@{mode=$mode;installRoot=$currentInstall;installExit=$exit;reinstallExit=$reinstall;uninstallExit=$uninstall;runtimePreservedOnReinstall=$true;runtimeTreeSha256=$runtimeBefore.sha256;userDataRetained=$true;imageRemoved=$true;sentinel=$sentinel;sentinelSha256=$sentinelHash}
   Write-TwoUserJson (Join-Path $c.out ('cell-'+$mode+'.json')) $result.cells[-1]
  }
  $result.passed=$true
 }catch{$result.firstFailure=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber;atUtc=[datetime]::UtcNow.ToString('o')}}
 finally{
  if($installed -and $package){try{
   $uninstall=if($c.kind -eq 'nsis'){Invoke-LocalInstaller (Join-Path $currentInstall 'uninstall.exe') '/S' 'failure-uninstall'}else{Invoke-LocalInstaller 'msiexec.exe' ('/x "'+$package+'" /qn /norestart /L*v "'+(Join-Path $c.out 'failure-uninstall.log')+'"') 'failure-uninstall'}
   $result.failureCleanupUninstallExit=$uninstall
  }catch{$result.failureCleanupError=$_.Exception.Message}}
  Write-TwoUserJson (Join-Path $c.out 'installer-finished.json') $result
 }
 exit 0
}

$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if(!([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Elevated local controller required'}
$taskRoot=Assert-TwoUserLocalPath $Directory;$bundle=Assert-TwoUserLocalPath $BundleDirectory
Assert-TwoUserNoReparseAncestors $taskRoot;Assert-TwoUserNoReparseAncestors $bundle
if(Test-Path -LiteralPath (Join-Path $taskRoot 'controller.json')){throw 'Fresh installer batch required'}
$manifest=Get-Content -LiteralPath (Join-Path $bundle 'installer-build.json') -Raw|ConvertFrom-Json
$bundleAcl=Get-Acl -LiteralPath $bundle
$summary=[ordered]@{accepted=$false;kind=$Kind;source=$manifest.source;run=$manifest.run;startedUtc=[datetime]::UtcNow.ToString('o');controllerToken=(Get-WindowsHarnessToken);scope='Only one new ordinary installer owner; no machine policy or daily install changes';accountRemoved=$false;profileRemoved=$false;userDirectoryRemoved=$false}
Write-TwoUserJson (Join-Path $taskRoot 'controller.json') $summary
$reuse=Get-TwoUserReuseParts (Join-Path $PSScriptRoot 'windows-ordinary-startup.ps1')
if(!('OcxNativeProfileProvision' -as [type])){Add-Type -TypeDefinition $reuse.ProfileType}
if(!('OcxTemporaryDesktopAccess' -as [type])){Add-Type -TypeDefinition $reuse.DesktopType}
$code=Join-Path $taskRoot 'code';New-Item -ItemType Directory -Path $code|Out-Null
foreach($n in @('windows023-local-installer.ps1','windows023-two-user-native-helper.ps1','windows-process-token.ps1','windows-bundle-identity.mjs')){Copy-Item -LiteralPath (Join-Path $PSScriptRoot $n) -Destination $code}
$reuse.Bootstrap.Replace('& $c.nativeHelper -Mode Child -ConfigPath $InputFile','& $c.installerScript -Mode Child -InputFile $InputFile')|Set-Content -LiteralPath (Join-Path $code 'profile-bootstrap.ps1') -Encoding utf8
$user=$null;$profile=$null;$child=$null;$fact=$null;$access=$null;$credential=$null;$secure=$null;$unproven=$false
$ledger=[Collections.Generic.Dictionary[int,object]]::new();$cleanup=@()
$entropy=[Security.Cryptography.RandomNumberGenerator]::GetBytes(32)
try{
 $secure=ConvertTo-SecureString ([Convert]::ToBase64String($entropy)+'aA1!') -AsPlainText -Force
 $name='ocxaccu1'+[guid]::NewGuid().ToString('N').Substring(0,9)
 $user=New-LocalUser -Name $name -Password $secure -AccountNeverExpires -PasswordNeverExpires -Description 'Disposable local installer test'
 Add-LocalGroupMember -Group (Get-LocalGroup -SID 'S-1-5-32-545') -Member $user
 $p=[Text.StringBuilder]::new(260);$hr=[OcxNativeProfileProvision]::CreateProfile($user.SID.Value,$name,$p,260)
 if($hr -ne 0){[Runtime.InteropServices.Marshal]::ThrowExceptionForHR($hr)};$profile=$p.ToString()
 $summary.sid=$user.SID.Value;$summary.accountName=$name;$summary.profile=$profile
 $access=[OcxTemporaryDesktopAccess]::new($user.SID.Value)
 $out=Join-Path $taskRoot 'owner';New-Item -ItemType Directory -Path $out|Out-Null
 & icacls.exe $taskRoot /grant ('*'+$user.SID.Value+':(OI)(CI)RX')|Out-Null;if($LASTEXITCODE -ne 0){throw 'Batch RX failed'}
 & icacls.exe $out /grant ('*'+$user.SID.Value+':(OI)(CI)M')|Out-Null;if($LASTEXITCODE -ne 0){throw 'Owner evidence write failed'}
 & icacls.exe $bundle /grant ('*'+$user.SID.Value+':(OI)(CI)RX')|Out-Null;if($LASTEXITCODE -ne 0){throw 'Exact new package RX failed'}
 $config=Join-Path $out 'input.json'
 Write-TwoUserJson $config @{sid=$user.SID.Value;accountSid=$user.SID.Value;profile=$profile;profileOnly=$false;installerScript=(Join-Path $code 'windows023-local-installer.ps1');kind=$Kind;bundle=$bundle;node=$Node;out=$out}
 Write-TwoUserJson (Join-Path $taskRoot 'provisioned.json') $summary
 $credential=[pscredential]::new(($env:COMPUTERNAME+[char]92+$name),$secure)
 $child=Start-Process -FilePath (Join-Path $PSHOME 'pwsh.exe') -Credential $credential -LoadUserProfile -WorkingDirectory $out -ArgumentList @('-NoLogo','-NoProfile','-NonInteractive','-File',('"'+(Join-Path $code 'profile-bootstrap.ps1')+'"'),'-InputFile',('"'+$config+'"')) -RedirectStandardOutput (Join-Path $out 'bootstrap-stdout.log') -RedirectStandardError (Join-Path $out 'bootstrap-stderr.log') -PassThru
 $fact=Get-TwoUserProcessFact $child $user.SID.Value;$ledger[[int]$fact.pid]=$fact
 $deadline=[datetime]::UtcNow.AddMinutes(25)
 while(!$child.HasExited){Add-TwoUserDescendants $fact $ledger;if([datetime]::UtcNow -gt $deadline){throw 'Installer batch transport deadline; no retry'};Start-Sleep -Milliseconds 500}
 $summary.bootstrapExitCode=$child.ExitCode
}catch{$summary.firstFailure=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber;atUtc=[datetime]::UtcNow.ToString('o')}}
finally{
 [Array]::Clear($entropy,0,$entropy.Length)
 if($fact){try{Add-TwoUserDescendants $fact $ledger}catch{$unproven=$true;$cleanup+=@{phase='descendants';error=$_.Exception.Message}}}
 foreach($f in @($ledger.Values|Sort-Object {[long]$_.creationTicks} -Descending)){try{Stop-TwoUserTracked $f;$cleanup+=@{pid=$f.pid;image=$f.path;sid=$f.sid;result='original-exited-or-exact-owned-cleanup'}}catch{$unproven=$true;$cleanup+=@{pid=$f.pid;image=$f.path;sid=$f.sid;result='retained-unproven';error=$_.Exception.Message}}}
 if($child){if(!$child.HasExited){$unproven=$true};$child.Dispose()};Close-TwoUserProcessHandles
 if($access){try{$access.Dispose();$summary.desktopAclRestored=$true}catch{$summary.desktopAclRestored=$false}}
 try{Set-Acl -LiteralPath $bundle -AclObject $bundleAcl;$summary.bundleRootAclRestored=$true}catch{$summary.bundleRootAclRestored=$false}
 $residual=@();if($user){foreach($row in Get-CimInstance Win32_Process){$owner=Invoke-CimMethod -InputObject $row -MethodName GetOwnerSid -ErrorAction SilentlyContinue;if($owner.Sid -ceq $user.SID.Value){$residual+=@{pid=$row.ProcessId;image=$(if($row.ExecutablePath){$row.ExecutablePath}else{'unknown'});sid=$owner.Sid}}}}
 $summary.residualProcesses=$residual;if($residual.Count){$unproven=$true}
 if($user -and !$unproven){try{
  $current=Get-LocalUser -SID $user.SID;if($current.Name -cne $name){throw 'Installer owner changed'}
  $until=[datetime]::UtcNow.AddSeconds(30)
  do{$profiles=@(Get-CimInstance Win32_UserProfile|Where-Object SID -ceq $user.SID.Value);if(!@($profiles|Where-Object Loaded).Count){break};Start-Sleep -Milliseconds 500}while([datetime]::UtcNow -lt $until)
  foreach($p in $profiles){if($p.Special -or $p.Loaded -or $p.LocalPath -ine $profile){throw 'Profile loaded or changed'};Remove-CimInstance -InputObject $p}
  if($profile -and (Test-Path -LiteralPath $profile)){throw 'Owner directory remains'}
  Remove-LocalUser -SID $user.SID
  $summary.accountRemoved=!(Get-LocalUser -SID $user.SID -ErrorAction SilentlyContinue);$summary.profileRemoved=(@(Get-CimInstance Win32_UserProfile|Where-Object SID -ceq $user.SID.Value).Count -eq 0);$summary.userDirectoryRemoved=!(Test-Path -LiteralPath $profile)
 }catch{$summary.cleanupFailure=$_.Exception.Message}}
 if($credential){$credential.Password.Dispose()}elseif($secure){$secure.Dispose()}
 $summary.identityUnproven=$unproven;$summary.cleanup=$cleanup;$summary.finishedUtc=[datetime]::UtcNow.ToString('o')
 Write-TwoUserJson (Join-Path $taskRoot 'controller-finished.json') $summary
}

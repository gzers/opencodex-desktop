param(
 [ValidateSet('Controller','Child')][string]$Mode='Controller',
 [string]$InputFile,
 [string]$Directory,
 [string]$ProgramDirectory,
 [string]$Node,
 [ValidatePattern('^[a-f0-9]{40}$')][string]$ExpectedSource='8445d65dca0af76c3275f8617a76c1a726fb8873',
 [ValidatePattern('^[0-9]+$')][string]$ExpectedBuildRun='42602603906'
)
$ErrorActionPreference='Stop'
$user1Mode=$Mode
. (Join-Path $PSScriptRoot 'windows023-two-user-native-helper.ps1')
. (Join-Path $PSScriptRoot 'windows-process-token.ps1')
$expected=$ExpectedSource
if(!$IsWindows){throw 'Native Windows required'}
if($user1Mode -eq 'Child'){
 $c=Get-Content -LiteralPath $InputFile -Raw|ConvertFrom-Json
 if($c.commit -cnotmatch '^[a-f0-9]{40}$'){throw 'Explicit full product source required'}
 $expected=$c.commit
 $identity=[Security.Principal.WindowsIdentity]::GetCurrent();$sid=$identity.User.Value
 if($sid -cne $c.accountSid -or $env:USERPROFILE -ine $c.profile -or (Split-Path -Leaf $c.profile) -notlike 'ocxaccu1*'){throw 'Exact newly provisioned user1 required'}
 if(([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'user1 must be a genuine nonadministrator'}
 $out=$c.evidence;$profile=$c.profile;$installed=Join-Path $env:LOCALAPPDATA 'Programs/OpenCodeX Desktop'
 $root=Join-Path $profile '.codex-desktop/desktop';$nativeHome=Join-Path $profile '.codex-desktop/opencodex/home'
 $ledger=[Collections.Generic.Dictionary[int,object]]::new();$objects=@{};$manager=$null;$driver=$null
 $result=[ordered]@{accepted=$false;scope='one actual user1 only';source=$expected;sid=$sid;profile=$profile;sessionId=[Diagnostics.Process]::GetCurrentProcess().SessionId;harnessToken=(Get-WindowsHarnessToken);startConfirmed=$false;normalStopExecuted=$false;cleanup='pending'}
 function Admit-User1Process([Diagnostics.Process]$p){
  $fact=Get-TwoUserProcessFact $p $sid;$ledger[[int]$fact.pid]=$fact
  $key=[string]$fact.pid+'/'+$fact.creationTicks
  if(!$objects.ContainsKey($key)){$objects[$key]=@{process=$p;fact=$fact}}else{$p.Dispose()}
  $fact
 }
 function Observe-User1Tree {
  if(!$managerFact){return}
  $null=Assert-TwoUserRootForDescendantObservation $managerFact
  $rows=@(Get-CimInstance Win32_Process);$parents=[Collections.Generic.HashSet[int]]::new()
  foreach($f in $ledger.Values){[void]$parents.Add([int]$f.pid)}
  do{$changed=$false;foreach($row in $rows){
   if(!$parents.Contains([int]$row.ParentProcessId) -or $parents.Contains([int]$row.ProcessId) -or $row.CreationDate.ToUniversalTime().Ticks -lt ([long]$managerFact.creationTicks-9)){continue}
   $p=Get-Process -Id $row.ProcessId -ErrorAction SilentlyContinue;if(!$p){continue}
   try{$fact=Admit-User1Process $p;[void]$parents.Add([int]$fact.pid);$changed=$true}
   catch{if($p.HasExited){$p.Dispose();continue};throw}
  }}while($changed)
  $record=Join-Path $nativeHome 'runtime-port.json'
  if(Test-Path -LiteralPath $record){
   $r=Get-Content -LiteralPath $record -Raw|ConvertFrom-Json
   if($r.pid -gt 0 -and $r.port -gt 0){
    if(!$ledger.ContainsKey([int]$r.pid)){throw 'Runtime PID not observed in original Manager launch family'}
    $f=$ledger[[int]$r.pid];$live=Assert-TwoUserLive $f
    $listeners=@(Get-NetTCPConnection -State Listen -LocalPort $r.port -ErrorAction SilentlyContinue)
    Write-TwoUserJson (Join-Path $out 'runtime-record-native.json') @{pid=$r.pid;port=$r.port;native=$live;recordAcl=(Get-Acl -LiteralPath $record).Sddl;listeners=@($listeners|Select-Object LocalAddress,LocalPort,OwningProcess);observedUtc=[datetime]::UtcNow.ToString('o');secretOmitted=$true}
   }
  }
 }
 try{
  Assert-WindowsProcessMode $result.harnessToken ordinary
  if(Test-Path -LiteralPath $installed){throw 'Fresh user1 private installation required'}
  New-Item -ItemType Directory -Path $installed|Out-Null
  $before=Get-TwoUserTreeHash $c.candidateDirectory
  foreach($entry in Get-ChildItem -LiteralPath $c.candidateDirectory -Force){Copy-Item -LiteralPath $entry.FullName -Destination $installed -Recurse}
  $copy=Get-TwoUserTreeHash $installed
  if($copy.sha256 -cne $before.sha256){throw 'Sealed private copy differs'}
  Write-TwoUserJson (Join-Path $out 'installation-copy.json') @{sha256=$copy.sha256;sealedOriginalSha256=$before.sha256;owner=(Get-Acl $installed).GetOwner([Security.Principal.SecurityIdentifier]).Value}
  $exe=Join-Path $installed 'opencodex-desktop.exe'
  $listener=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0);$listener.Start();$port=$listener.LocalEndpoint.Port;$listener.Stop()
  $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$exe;$info.WorkingDirectory=$installed;$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
  foreach($key in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_WINDOWS_SMOKE_CDP_PORT','HOME','OPENCODEX_HOME','CODEX_HOME','CLAUDE_CONFIG_DIR','WEBVIEW2_USER_DATA_FOLDER','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS')){[void]$info.Environment.Remove($key)}
  $info.Environment['WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS']='--remote-debugging-address=127.0.0.1 --remote-debugging-port='+$port
  $manager=[Diagnostics.Process]::Start($info);$managerOut=$manager.StandardOutput.ReadToEndAsync();$managerErr=$manager.StandardError.ReadToEndAsync()
  $managerFact=Admit-User1Process $manager;$result.manager=$managerFact
  $result.applicationToken=Get-WindowsProcessToken $manager $exe;Assert-WindowsProcessMode $result.applicationToken ordinary
  $window=$null;$windowDeadline=[datetime]::UtcNow.AddSeconds(40)
  do{try{$window=Get-TwoUserWindow $managerFact}catch{Start-Sleep -Milliseconds 250}}while(!$window -and [datetime]::UtcNow -lt $windowDeadline)
  if(!$window){throw 'Actual user1 native window unavailable'}
  Write-TwoUserJson (Join-Path $out 'environment.json') ($result+[ordered]@{os=[Environment]::OSVersion.VersionString;machine=$env:COMPUTERNAME;userInteractive=[Environment]::UserInteractive;userProfile=$env:USERPROFILE;localAppData=$env:LOCALAPPDATA;sandbox=$false;actualUtc=[datetime]::UtcNow.ToString('o');desktop=$window;powershell=$PSVersionTable.PSVersion.ToString()})
  $supplyDeadline=[datetime]::UtcNow.AddSeconds(600);$terminal=$null
  do{
   Observe-User1Tree
   $log=Join-Path $root 'logs/app.log'
   if(Test-Path $log){$terminal=Get-Content $log|Select-String 'integrated runtime (provisioned and verified|provisioning failed)'|Select-Object -Last 1}
   if(!$terminal){Start-Sleep -Milliseconds 300}
  }while(!$terminal -and [datetime]::UtcNow -lt $supplyDeadline)
  Write-TwoUserJson (Join-Path $out 'supply-terminal.json') @{line=$terminal.Line;accepted=$false}
  if(!$terminal -or !$terminal.Line.Contains('integrated runtime provisioned and verified')){throw 'Real first-start offline supply not confirmed'}
  if(!(Get-Content $log -Raw).Contains('commit='+$expected)){throw 'Program build source differs'}
  $driverInput=Join-Path $out 'driver-input.json';$release=Join-Path $out 'driver-release'
  Write-TwoUserJson $driverInput @{commit=$expected;root=$root;home=$nativeHome;driverRelease=$release;coordination=$c.coordination}
  $di=[Diagnostics.ProcessStartInfo]::new();$di.FileName=$c.node;$di.UseShellExecute=$false;$di.CreateNoWindow=$true;$di.RedirectStandardOutput=$true;$di.RedirectStandardError=$true
  foreach($arg in @((Join-Path $c.code 'user1-driver.mjs'),('http://127.0.0.1:'+$port),$out,$driverInput)){$di.ArgumentList.Add($arg)}
  $driver=[Diagnostics.Process]::Start($di);$driverOut=$driver.StandardOutput.ReadToEndAsync();$driverErr=$driver.StandardError.ReadToEndAsync();$driverFact=Admit-User1Process $driver
  # Peer coordination is outside each original Start/Stop/status action budget.
  $deadline=[datetime]::UtcNow.AddSeconds($(if($c.coordination){900}else{120}))
  while(!$driver.HasExited){
   Observe-User1Tree
   if(Test-Path -LiteralPath (Join-Path $out 'user1-result.json')){[IO.File]::WriteAllText($release,'parent observed result')}
   if([datetime]::UtcNow -gt $deadline){throw 'Transport envelope elapsed; no lifecycle retry'}
   Start-Sleep -Milliseconds 200
  }
  Write-TwoUserJson (Join-Path $out 'driver-exit.json') @{identity=$driverFact;exitCode=$driver.ExitCode;originalHandleExited=$driver.HasExited}
  [IO.File]::WriteAllText((Join-Path $out 'driver-stderr.log'),$driverErr.GetAwaiter().GetResult())
  if(!(Test-Path (Join-Path $out 'user1-result.json'))){throw 'Actual driver result missing; evidence incomplete'}
  $actual=Get-Content (Join-Path $out 'user1-result.json') -Raw|ConvertFrom-Json
  $result.startConfirmed=$actual.startConfirmed;$result.normalStopExecuted=$actual.normalStopExecuted;$result.normalStopConfirmed=$actual.normalStopConfirmed
  $result.result=if($driver.ExitCode -eq 0){'diagnostic-start-stop-confirmed'}else{'diagnostic-failed'}
 }catch{$result.result='diagnostic-failed';$result.error=$_.Exception.Message;$result.failureLine=$_.InvocationInfo.ScriptLineNumber}
 finally{
  # Capture real original-handle exit codes BEFORE cleanup; unknown remains null.
  $exits=@(foreach($entry in $objects.Values){$p=$entry.process;$hasExited=$p.HasExited;@{identity=$entry.fact;originalHandleExitedBeforeCleanup=$hasExited;exitCodeBeforeCleanup=$(if($hasExited){$p.ExitCode}else{$null})}})
  Write-TwoUserJson (Join-Path $out 'original-exits-before-cleanup.json') $exits
  try{if($managerFact){Observe-User1Tree}}catch{$result.cleanupObservationFailure=$_.Exception.Message}
  $cleanup=@();foreach($fact in @($ledger.Values|Sort-Object {[long]$_.creationTicks} -Descending)){
   try{Stop-TwoUserTracked $fact;$cleanup+=@{identity=$fact;result='original-exited-or-exact-owned-cleanup'}}catch{$cleanup+=@{identity=$fact;result='retained-unproven';error=$_.Exception.Message}}
  }
  foreach($base in @(@{path=(Join-Path $root 'logs');label='desktop-logs'},@{path=(Join-Path $nativeHome 'logs');label='runtime-logs'})){
   if(Test-Path -LiteralPath $base.path){$dest=Join-Path $out $base.label;New-Item -ItemType Directory -Path $dest -Force|Out-Null;foreach($f in Get-ChildItem -LiteralPath $base.path -File){if($f.Length -le 6MB -and $f.Extension -in @('.log','.txt')){Copy-Item -LiteralPath $f.FullName -Destination $dest}}}
  }
  if($manager -and $manager.HasExited){[IO.File]::WriteAllText((Join-Path $out 'manager-stderr.log'),$managerErr.GetAwaiter().GetResult());[IO.File]::WriteAllText((Join-Path $out 'manager-stdout.log'),$managerOut.GetAwaiter().GetResult())}
  Write-TwoUserJson (Join-Path $out 'runtime-tracked.json') @($ledger.Values)
  foreach($entry in $objects.Values){$entry.process.Dispose()};Close-TwoUserProcessHandles
  $result.cleanup=$cleanup;Write-TwoUserJson (Join-Path $out 'child-finished.json') $result
 }
 exit 0
}

# Local machine controller, explicitly authorized by the single-user handoff.
# The original CI-only two-user controller and peer-dependent Child are never run.
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if(!([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Elevated local controller required to create/delete the one disposable user1'}
$taskRoot=Assert-TwoUserLocalPath $Directory;$program=Assert-TwoUserLocalPath $ProgramDirectory
if(Test-Path -LiteralPath (Join-Path $taskRoot 'controller.json')){throw 'Fresh batch required; no retries or evidence replacement'}
$manifest=Get-Content -LiteralPath (Join-Path $program 'core-build.json') -Raw|ConvertFrom-Json
if($manifest.source -cne $expected -or $manifest.run -cne $ExpectedBuildRun){throw 'Specified source-bound artifact required'}
foreach($file in $manifest.files){$path=Join-Path $program $file.path;$null=Assert-TwoUserUnder $program $path;Assert-TwoUserNoReparseAncestors $path;if((Get-Item $path).Length -ne $file.bytes -or (Get-FileHash $path).Hash -ine $file.sha256){throw 'Specified program member differs'}}
$runtime=Join-Path $program 'runtime-bundle/maintained-runtime.tgz'
if((Get-Item $runtime).Length -ne 15413397 -or (Get-FileHash $runtime).Hash -ine 'ec14baaee354c6729d1d2545ab9165d207b14a3a4239f354662163087eeb0739'){throw 'Specified lifecycle.3 runtime differs'}
$original=Get-TwoUserTreeHash $program
$summary=[ordered]@{accepted=$false;source=$expected;scope='user1-only; no peer, core-sweep or installer';startedUtc=[datetime]::UtcNow.ToString('o');controllerToken=(Get-WindowsHarnessToken);programTreeSha256=$original.sha256;accountsCreated=0}
Write-TwoUserJson (Join-Path $taskRoot 'controller.json') $summary
$reuse=Get-TwoUserReuseParts (Join-Path $PSScriptRoot 'windows-ordinary-startup.ps1')
if(!('OcxNativeProfileProvision' -as [type])){Add-Type -TypeDefinition $reuse.ProfileType}
if(!('OcxTemporaryDesktopAccess' -as [type])){Add-Type -TypeDefinition $reuse.DesktopType}
$code=Join-Path $taskRoot 'code';New-Item -ItemType Directory -Path $code|Out-Null
foreach($name in @('windows023-user1-start-stop.ps1','windows023-two-user-native-helper.ps1','windows-process-token.ps1','windows-cdp.mjs')){Copy-Item -LiteralPath (Join-Path $PSScriptRoot $name) -Destination $code}
$bootstrap=$reuse.Bootstrap.Replace('& $c.nativeHelper -Mode Child -ConfigPath $InputFile','& $c.singleUserScript -Mode Child -InputFile $InputFile')
$bootstrap|Set-Content -LiteralPath (Join-Path $code 'profile-bootstrap.ps1') -Encoding utf8
$control=Get-TwoUserRuntimeDriverText
# Only omit the old failed-start supplemental diagnostic sequence. This request
# permits normal Stop only after success; every original start/stop predicate stays.
$from=$control.IndexOf('    await diagnosticReadiness();',[StringComparison]::Ordinal)
$to=$control.IndexOf('    const confirmedLate=', $from,[StringComparison]::Ordinal)
if($from -lt 0 -or $to -le $from){throw 'Original failure-diagnostic boundary drifted'}
$control=$control.Remove($from,$to-$from)
$driver=(Get-Content -LiteralPath (Join-Path $PSScriptRoot 'windows023-user1-start-stop.mjs') -Raw).Replace('/*USER1_RUNTIME_CONTROL*/',$control)
[IO.File]::WriteAllText((Join-Path $code 'user1-driver.mjs'),$driver)
& $Node --check (Join-Path $code 'user1-driver.mjs');if($LASTEXITCODE -ne 0){throw 'Actual generated driver syntax failed'}
$user=$null;$profile=$null;$child=$null;$fact=$null;$access=$null;$credential=$null;$secure=$null
$ledger=[Collections.Generic.Dictionary[int,object]]::new();$cleanup=@();$unproven=$false
$name='ocxaccu1'+[guid]::NewGuid().ToString('N').Substring(0,9)
$entropy=[Security.Cryptography.RandomNumberGenerator]::GetBytes(32)
try{
 $secure=ConvertTo-SecureString ([Convert]::ToBase64String($entropy)+'aA1!') -AsPlainText -Force
 $user=New-LocalUser -Name $name -Password $secure -AccountNeverExpires -PasswordNeverExpires -Description 'Disposable exact user1 start-stop diagnosis'
 $summary.accountsCreated=1;$summary.accountSid=$user.SID.Value;$summary.accountName=$name
 Add-LocalGroupMember -Group (Get-LocalGroup -SID 'S-1-5-32-545') -Member $user
 $p=[Text.StringBuilder]::new(260);$hr=[OcxNativeProfileProvision]::CreateProfile($user.SID.Value,$name,$p,260)
 if($hr -ne 0){[Runtime.InteropServices.Marshal]::ThrowExceptionForHR($hr)};$profile=$p.ToString();$summary.profile=$profile
 Write-TwoUserJson (Join-Path $taskRoot 'provisioned.json') $summary
 $access=[OcxTemporaryDesktopAccess]::new($user.SID.Value)
 $out=Join-Path $taskRoot 'user1';New-Item -ItemType Directory -Path $out|Out-Null
 & icacls.exe $taskRoot /grant ('*'+$user.SID.Value+':(OI)(CI)RX')|Out-Null;if($LASTEXITCODE -ne 0){throw 'Batch read ACL failed'}
 & icacls.exe $out /grant ('*'+$user.SID.Value+':(OI)(CI)M')|Out-Null;if($LASTEXITCODE -ne 0){throw 'Own evidence write ACL failed'}
 $input=Join-Path $out 'input.json'
 Write-TwoUserJson $input @{accountSid=$user.SID.Value;profile=$profile;commit=$expected;profileOnly=$false;singleUserScript=(Join-Path $code 'windows023-user1-start-stop.ps1');candidateDirectory=$program;node=$Node;evidence=$out;code=$code}
 $credential=[pscredential]::new(($env:COMPUTERNAME+[char]92+$name),$secure)
 $child=Start-Process -FilePath (Join-Path $PSHOME 'pwsh.exe') -Credential $credential -LoadUserProfile -WorkingDirectory $out -ArgumentList @('-NoLogo','-NoProfile','-NonInteractive','-File',('"'+(Join-Path $code 'profile-bootstrap.ps1')+'"'),'-InputFile',('"'+$input+'"')) -RedirectStandardOutput (Join-Path $out 'bootstrap-stdout.log') -RedirectStandardError (Join-Path $out 'bootstrap-stderr.log') -PassThru
 $fact=Get-TwoUserProcessFact $child $user.SID.Value;$ledger[[int]$fact.pid]=$fact
 $deadline=[datetime]::UtcNow.AddSeconds(800)
 while(!$child.HasExited){Add-TwoUserDescendants $fact $ledger;if([datetime]::UtcNow -gt $deadline){throw 'User1 envelope elapsed; retain evidence, do not retry'};Start-Sleep -Milliseconds 500}
 $summary.bootstrapExitCode=$child.ExitCode
}catch{$summary.failure=$_.Exception.Message;$summary.failureLine=$_.InvocationInfo.ScriptLineNumber}
finally{
 [Array]::Clear($entropy,0,$entropy.Length)
 if($fact){try{Add-TwoUserDescendants $fact $ledger}catch{$unproven=$true;$cleanup+=@{phase='descendants';error=$_.Exception.Message}}}
 if($user){
  $tracked=Join-Path $taskRoot 'user1/runtime-tracked.json'
  if(Test-Path $tracked){foreach($f in @(Get-Content $tracked -Raw|ConvertFrom-Json)){if($f.sid -cne $user.SID.Value){$unproven=$true;continue};$ledger[[int]$f.pid]=$f}}
 }
 foreach($f in @($ledger.Values|Sort-Object {[long]$_.creationTicks} -Descending)){try{Stop-TwoUserTracked $f;$cleanup+=@{pid=$f.pid;image=$f.path;sid=$f.sid;result='exact-owned-exited'}}catch{$unproven=$true;$cleanup+=@{pid=$f.pid;image=$f.path;sid=$f.sid;result='retained';error=$_.Exception.Message}}}
 if($child){if(!$child.HasExited){$unproven=$true};$child.Dispose()};Close-TwoUserProcessHandles
 if($access){try{$access.Dispose();$summary.desktopAclRestored=$true}catch{$summary.desktopAclRestored=$false}}
 $residual=@();if($user){foreach($row in Get-CimInstance Win32_Process){$owner=Invoke-CimMethod -InputObject $row -MethodName GetOwnerSid -ErrorAction SilentlyContinue;if($owner.Sid -ceq $user.SID.Value){$residual+=@{pid=$row.ProcessId;image=$(if($row.ExecutablePath){$row.ExecutablePath}else{'unknown'});sid=$owner.Sid;session=$row.SessionId}}}}
 $summary.residualProcesses=$residual
 if($residual.Count){$unproven=$true}
 if($user -and !$unproven){try{
  $current=Get-LocalUser -SID $user.SID;if($current.Name -cne $name){throw 'Disposable account changed'}
  $until=[datetime]::UtcNow.AddSeconds(30)
  do{$profiles=@(Get-CimInstance Win32_UserProfile|Where-Object SID -ceq $user.SID.Value);if(!@($profiles|Where-Object Loaded).Count){break};Start-Sleep -Milliseconds 500}while([datetime]::UtcNow -lt $until)
  foreach($p in $profiles){if($p.Special -or $p.Loaded -or $p.LocalPath -ine $profile){throw 'Profile still loaded or identity changed'};Remove-CimInstance -InputObject $p}
  if($profile -and (Test-Path -LiteralPath $profile)){throw 'Original user directory remains'}
  Remove-LocalUser -SID $user.SID
  $summary.accountRemoved=!(Get-LocalUser -SID $user.SID -ErrorAction SilentlyContinue);$summary.profileRemoved=(@(Get-CimInstance Win32_UserProfile|Where-Object SID -ceq $user.SID.Value).Count -eq 0);$summary.userDirectoryRemoved=!(Test-Path -LiteralPath $profile)
 }catch{$summary.cleanupFailure=$_.Exception.Message}}
 $summary.cleanup=$cleanup;$summary.identityUnproven=$unproven
 if($credential){$credential.Password.Dispose()}elseif($secure){$secure.Dispose()}
 $summary.programUnchanged=(Get-TwoUserTreeHash $program).sha256 -ceq $original.sha256
 $summary.finishedUtc=[datetime]::UtcNow.ToString('o')
 Write-TwoUserJson (Join-Path $taskRoot 'controller-finished.json') $summary
}

param(
 [Parameter(Mandatory=$true)][switch]$LocalWorkstation,
 [Parameter(Mandatory=$true)][string]$Directory,
 [Parameter(Mandatory=$true)][string]$ProgramDirectory,
 [Parameter(Mandatory=$true)][string]$Node,
 [Parameter(Mandatory=$true)][ValidatePattern('^[a-f0-9]{40}$')][string]$ExpectedSource,
 [Parameter(Mandatory=$true)][ValidatePattern('^[0-9]+$')][string]$ExpectedBuildRun
)
$ErrorActionPreference='Stop'
if(!$IsWindows -or !$LocalWorkstation){throw 'Explicit native local workstation scope required'}
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if(!([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Elevated local controller required'}
. (Join-Path $PSScriptRoot 'windows023-two-user-native-helper.ps1')
. (Join-Path $PSScriptRoot 'windows-process-token.ps1')
$taskRoot=Assert-TwoUserLocalPath $Directory
$program=Assert-TwoUserLocalPath $ProgramDirectory
Assert-TwoUserNoReparseAncestors $taskRoot
Assert-TwoUserNoReparseAncestors $program
if(Test-Path -LiteralPath (Join-Path $taskRoot 'controller.json')){throw 'Fresh batch required; no replay'}
$manifest=Get-Content -LiteralPath (Join-Path $program 'core-build.json') -Raw|ConvertFrom-Json
if($manifest.source -cne $ExpectedSource -or $manifest.run -cne $ExpectedBuildRun){throw 'Sealed program source/run differs'}
foreach($file in $manifest.files){
 $p=Join-Path $program $file.path;$null=Assert-TwoUserUnder $program $p;Assert-TwoUserNoReparseAncestors $p
 if((Get-Item -LiteralPath $p).Length -ne $file.bytes -or (Get-FileHash -LiteralPath $p).Hash -ine $file.sha256){throw 'Sealed member differs'}
}
$nodeVersion=(& $Node --version).Trim()
if($LASTEXITCODE -ne 0 -or $nodeVersion -cne 'v26.8.1'){throw 'Pinned diagnostic Node required'}
$original=Get-TwoUserTreeHash $program
$accounts=[Collections.Generic.List[object]]::new()
$access=[Collections.Generic.List[object]]::new()
$ledger=[Collections.Generic.Dictionary[int,object]]::new()
$summary=[ordered]@{accepted=$false;source=$ExpectedSource;buildRun=$ExpectedBuildRun;scope='Two new ordinary SIDs on local workstation; concurrent health, stopB/peerA preserved, stopA, exact cleanup';startedUtc=[datetime]::UtcNow.ToString('o');controllerToken=(Get-WindowsHarnessToken);programTreeSha256=$original.sha256;budgetsMilliseconds=@{start=30000;stop=10000;status=5000};protocolPassed=$false;accountsCreated=0}
Write-TwoUserJson (Join-Path $taskRoot 'controller.json') $summary
$reuse=Get-TwoUserReuseParts (Join-Path $PSScriptRoot 'windows-ordinary-startup.ps1')
if(!('OcxNativeProfileProvision' -as [type])){Add-Type -TypeDefinition $reuse.ProfileType}
if(!('OcxTemporaryDesktopAccess' -as [type])){Add-Type -TypeDefinition $reuse.DesktopType}
$code=Join-Path $taskRoot 'code';New-Item -ItemType Directory -Path $code|Out-Null
foreach($name in @('windows023-local-two-user.ps1','windows023-user1-start-stop.ps1','windows023-user1-start-stop.mjs','windows023-two-user-native-helper.ps1','windows-process-token.ps1','windows-cdp.mjs')){Copy-Item -LiteralPath (Join-Path $PSScriptRoot $name) -Destination $code}
$bootstrap=$reuse.Bootstrap.Replace('& $c.nativeHelper -Mode Child -ConfigPath $InputFile','& $c.singleUserScript -Mode Child -InputFile $InputFile')
$bootstrap|Set-Content -LiteralPath (Join-Path $code 'profile-bootstrap.ps1') -Encoding utf8
$control=Get-TwoUserRuntimeDriverText
$from=$control.IndexOf('    await diagnosticReadiness();',[StringComparison]::Ordinal)
$to=$control.IndexOf('    const confirmedLate=',$from,[StringComparison]::Ordinal)
if($from -lt 0 -or $to -le $from){throw 'Failure diagnostic boundary changed'}
$control=$control.Remove($from,$to-$from)
$driver=(Get-Content -LiteralPath (Join-Path $PSScriptRoot 'windows023-user1-start-stop.mjs') -Raw).Replace('/*USER1_RUNTIME_CONTROL*/',$control)
[IO.File]::WriteAllText((Join-Path $code 'user1-driver.mjs'),$driver)
& $Node --check (Join-Path $code 'user1-driver.mjs');if($LASTEXITCODE -ne 0){throw 'Generated driver syntax failed'}
function Wait-LocalPeerFile($account,[string]$name,[int]$seconds=180){
 $until=[datetime]::UtcNow.AddSeconds($seconds)
 while(!(Test-Path -LiteralPath (Join-Path $account.evidence $name))){
  foreach($a in $accounts){if($a.process.HasExited -or (Test-Path -LiteralPath (Join-Path $a.evidence 'user1-result.json'))){
   $resultFile=Join-Path $a.evidence 'user1-result.json'
   if(!(Test-Path -LiteralPath $resultFile) -or !((Get-Content -LiteralPath $resultFile -Raw|ConvertFrom-Json).normalStopConfirmed)){throw 'Peer exited or failed; preserve first failure, no retry'}
  }}
  if([datetime]::UtcNow -gt $until){throw ('Peer transport envelope elapsed: '+$name)}
  Start-Sleep -Milliseconds 200
 }
}
function Signal-LocalPeer($a,[string]$name){[IO.File]::WriteAllText((Join-Path $a.evidence $name),'controller exact phase')}
try{
 foreach($role in 1..2){
  $entropy=[Security.Cryptography.RandomNumberGenerator]::GetBytes(32);$secure=$null;$a=$null
  try{
   $name='ocxaccu1'+[guid]::NewGuid().ToString('N').Substring(0,9)
   $secure=ConvertTo-SecureString ([Convert]::ToBase64String($entropy)+'aA1!') -AsPlainText -Force
   $user=New-LocalUser -Name $name -Password $secure -AccountNeverExpires -PasswordNeverExpires -Description 'Disposable local Windows peer lifecycle verification'
   $a=[pscustomobject]@{role=$role;name=$name;sid=$user.SID.Value;profile=$null;evidence=(Join-Path $taskRoot ('user'+$role));process=$null;fact=$null;credential=$null;unproven=$false;cleanup=@()}
   $accounts.Add($a);$summary.accountsCreated=$accounts.Count
   Add-LocalGroupMember -Group (Get-LocalGroup -SID 'S-1-5-32-545') -Member $user
   $path=[Text.StringBuilder]::new(260);$hr=[OcxNativeProfileProvision]::CreateProfile($a.sid,$name,$path,260)
   if($hr -ne 0){[Runtime.InteropServices.Marshal]::ThrowExceptionForHR($hr)}
   $a.profile=$path.ToString();if((Split-Path -Leaf $a.profile) -cne $name){throw 'Exact Profile identity differs'}
   $access.Add([OcxTemporaryDesktopAccess]::new($a.sid))
   New-Item -ItemType Directory -Path $a.evidence|Out-Null
   & icacls.exe $taskRoot /grant ('*'+$a.sid+':(OI)(CI)RX')|Out-Null;if($LASTEXITCODE -ne 0){throw 'Evidence read ACL failed'}
   & icacls.exe $a.evidence /grant ('*'+$a.sid+':(OI)(CI)M')|Out-Null;if($LASTEXITCODE -ne 0){throw 'Own evidence write ACL failed'}
   $configPath=Join-Path $a.evidence 'input.json'
   Write-TwoUserJson $configPath @{accountSid=$a.sid;profile=$a.profile;commit=$ExpectedSource;profileOnly=$false;singleUserScript=(Join-Path $code 'windows023-user1-start-stop.ps1');candidateDirectory=$program;node=$Node;evidence=$a.evidence;code=$code;coordination=@{role=$role}}
   $a.credential=[pscredential]::new(($env:COMPUTERNAME+[char]92+$name),$secure)
   $a.process=Start-Process -FilePath (Join-Path $PSHOME 'pwsh.exe') -Credential $a.credential -LoadUserProfile -WorkingDirectory $a.evidence -ArgumentList @('-NoLogo','-NoProfile','-NonInteractive','-File',('"'+(Join-Path $code 'profile-bootstrap.ps1')+'"'),'-InputFile',('"'+$configPath+'"')) -RedirectStandardOutput (Join-Path $a.evidence 'bootstrap-stdout.log') -RedirectStandardError (Join-Path $a.evidence 'bootstrap-stderr.log') -PassThru
   $a.fact=Get-TwoUserProcessFact $a.process $a.sid;$ledger[[int]$a.fact.pid]=$a.fact
   Write-TwoUserJson (Join-Path $taskRoot ('provisioned-user'+$role+'.json')) @{role=$role;name=$name;sid=$a.sid;profile=$a.profile;bootstrap=$a.fact}
  }finally{[Array]::Clear($entropy,0,$entropy.Length);if($secure -and (!$a -or !$a.credential)){$secure.Dispose()}}
 }
 if($accounts[0].sid -ceq $accounts[1].sid){throw 'Distinct real SIDs required'}
 foreach($a in $accounts){Wait-LocalPeerFile $a 'started.json' 720}
 $starts=@($accounts|ForEach-Object {Get-Content -LiteralPath (Join-Path $_.evidence 'started.json') -Raw|ConvertFrom-Json})
 if($starts[0].pid -eq $starts[1].pid -or $starts[0].port -eq $starts[1].port){throw 'Distinct actual service PIDs and ports required'}
 foreach($a in $accounts){Signal-LocalPeer $a 'observe-overlap'}
 foreach($a in $accounts){Wait-LocalPeerFile $a 'overlap-confirmed.json'}
 $summary.overlap=@($starts|Select-Object pid,port,source);$summary.distinctSids=@($accounts|Select-Object role,sid)
 Write-TwoUserJson (Join-Path $taskRoot 'overlap-confirmed.json') @{services=$summary.overlap;sids=$summary.distinctSids;observedUtc=[datetime]::UtcNow.ToString('o')}
 Signal-LocalPeer $accounts[1] 'stop-now'
 Wait-LocalPeerFile $accounts[1] 'user1-result.json'
 $b=Get-Content -LiteralPath (Join-Path $accounts[1].evidence 'user1-result.json') -Raw|ConvertFrom-Json
 if(!$b.normalStopConfirmed){throw 'B normal Stop failed; no repeat'}
 Signal-LocalPeer $accounts[0] 'observe-after-peer-stop'
 Wait-LocalPeerFile $accounts[0] 'survivor-confirmed.json'
 $summary.peerPreservedAfterNormalStop=$true
 Signal-LocalPeer $accounts[0] 'stop-now'
 Wait-LocalPeerFile $accounts[0] 'user1-result.json'
 $aResult=Get-Content -LiteralPath (Join-Path $accounts[0].evidence 'user1-result.json') -Raw|ConvertFrom-Json
 if(!$aResult.normalStopConfirmed){throw 'A normal Stop failed; no repeat'}
 foreach($a in $accounts){if(!$a.process.WaitForExit(90000)){throw 'Child evidence envelope elapsed'}}
 $summary.protocolPassed=$true
}catch{$summary.firstFailure=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber;atUtc=[datetime]::UtcNow.ToString('o')}}
finally{
 foreach($a in $accounts){try{Signal-LocalPeer $a 'abort'}catch{}}
 foreach($a in $accounts){
  if($a.process){try{
   if($a.fact){Add-TwoUserDescendants $a.fact $ledger}
   if(!$a.process.HasExited){[void]$a.process.WaitForExit(20000)}
  }catch{$a.unproven=$true;$a.cleanup+=@{phase='bootstrap-descendants';error=$_.Exception.Message}}}
  $tracked=Join-Path $a.evidence 'runtime-tracked.json'
  if(Test-Path -LiteralPath $tracked){foreach($f in @(Get-Content -LiteralPath $tracked -Raw|ConvertFrom-Json)){
   if($f.sid -cne $a.sid){$a.unproven=$true;continue}
   $ledger[[int]$f.pid]=$f
  }}
 }
 foreach($f in @($ledger.Values|Sort-Object {[long]$_.creationTicks} -Descending)){
  $owner=@($accounts|Where-Object sid -ceq $f.sid)[0]
  try{Stop-TwoUserTracked $f;$owner.cleanup+=@{pid=$f.pid;image=$f.path;sid=$f.sid;result='original-exited-or-exact-owned-cleanup'}}
  catch{$owner.unproven=$true;$owner.cleanup+=@{pid=$f.pid;image=$f.path;sid=$f.sid;result='retained-unproven';error=$_.Exception.Message}}
 }
 foreach($a in $accounts){if($a.process){if(!$a.process.HasExited){$a.unproven=$true}else{$a.cleanup+=@{phase='bootstrap-exit';exitCode=$a.process.ExitCode}};$a.process.Dispose();$a.process=$null}}
 Close-TwoUserProcessHandles
 $summary.desktopAclRestored=$true
 for($i=$access.Count-1;$i -ge 0;$i--){try{$access[$i].Dispose()}catch{$summary.desktopAclRestored=$false}}
 $residual=@()
 foreach($row in Get-CimInstance Win32_Process){
  $owner=Invoke-CimMethod -InputObject $row -MethodName GetOwnerSid -ErrorAction SilentlyContinue
  foreach($a in $accounts){if($owner.Sid -ceq $a.sid){$a.unproven=$true;$residual+=@{pid=$row.ProcessId;image=$(if($row.ExecutablePath){$row.ExecutablePath}else{'unknown'});sid=$owner.Sid;session=$row.SessionId}}}
 }
 $summary.residualProcesses=$residual
 foreach($a in $accounts){
  $clean=[ordered]@{sid=$a.sid;name=$a.name;profile=$a.profile;identityUnproven=$a.unproven;processCleanup=$a.cleanup;accountRemoved=$false;profileRemoved=$false;userDirectoryRemoved=$false}
  if(!$a.unproven){try{
   $current=Get-LocalUser -SID $a.sid;if($current.Name -cne $a.name){throw 'Disposable identity changed'}
   $until=[datetime]::UtcNow.AddSeconds(30)
   do{$profiles=@(Get-CimInstance Win32_UserProfile|Where-Object SID -ceq $a.sid);if(!@($profiles|Where-Object Loaded).Count){break};Start-Sleep -Milliseconds 500}while([datetime]::UtcNow -lt $until)
   foreach($p in $profiles){if($p.Special -or $p.Loaded -or $p.LocalPath -ine $a.profile){throw 'Profile loaded or identity changed'};Remove-CimInstance -InputObject $p}
   if($a.profile -and (Test-Path -LiteralPath $a.profile)){throw 'Original user directory remains'}
   Remove-LocalUser -SID $a.sid
   $clean.accountRemoved=!(Get-LocalUser -SID $a.sid -ErrorAction SilentlyContinue)
   $clean.profileRemoved=(@(Get-CimInstance Win32_UserProfile|Where-Object SID -ceq $a.sid).Count -eq 0)
   $clean.userDirectoryRemoved=(!$a.profile -or !(Test-Path -LiteralPath $a.profile))
  }catch{$clean.failure=$_.Exception.Message}}
  if($a.credential){$a.credential.Password.Dispose()}
  Write-TwoUserJson (Join-Path $a.evidence 'account-cleanup.json') $clean
 }
 $summary.accountCleanup=@($accounts|ForEach-Object{Get-Content -LiteralPath (Join-Path $_.evidence 'account-cleanup.json') -Raw|ConvertFrom-Json})
 $summary.programUnchanged=(Get-TwoUserTreeHash $program).sha256 -ceq $original.sha256
 $summary.boundedTwoSidPassed=$summary.protocolPassed -and $summary.desktopAclRestored -and $summary.programUnchanged -and !$residual.Count -and !@($summary.accountCleanup|Where-Object {!$_.accountRemoved -or !$_.profileRemoved -or !$_.userDirectoryRemoved -or $_.identityUnproven}).Count
 $summary.finishedUtc=[datetime]::UtcNow.ToString('o')
 Write-TwoUserJson (Join-Path $taskRoot 'controller-finished.json') $summary
}

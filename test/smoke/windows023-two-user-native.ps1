param(
 [Parameter(Mandatory=$true)][string]$Executable,
 [Parameter(Mandatory=$true)][string]$Node,
 [Parameter(Mandatory=$true)][string]$OutputDirectory,
 [Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-f]{40}$')][string]$ExpectedCommit,
 [switch]$VerifyRuntimePorts,
 [switch]$DiagnosticSweep
)
$ErrorActionPreference='Stop'
# Guard before ANY directory/account/ACL/process mutation.
if(!$IsWindows -or $env:GITHUB_ACTIONS -cne 'true' -or [string]::IsNullOrWhiteSpace($env:RUNNER_TEMP)){throw 'Restricted to an ephemeral Windows Actions runner'}
$taskEphemeralGuard=[ordered]@{passed=$true;windows=$true;githubActions=$env:GITHUB_ACTIONS;runnerTemp=[IO.Path]::GetFullPath($env:RUNNER_TEMP);checkedBeforeProvisioning=$true}
. (Join-Path $PSScriptRoot 'windows023-two-user-native-helper.ps1')
. (Join-Path $PSScriptRoot 'windows-process-token.ps1')
$taskRepo=(Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../..')).ProviderPath
$taskCommit=(& git -C $taskRepo rev-parse HEAD).Trim()
if($LASTEXITCODE -ne 0 -or $taskCommit -cne $ExpectedCommit){throw 'Exact sealed source commit required'}
$taskExe=(Resolve-Path -LiteralPath $Executable).ProviderPath
$taskNode=(Resolve-Path -LiteralPath $Node).ProviderPath
$taskCandidate=Split-Path -Parent $taskExe
$taskOutput=Assert-TwoUserLocalPath $OutputDirectory
$null=Assert-TwoUserUnder $env:RUNNER_TEMP $taskOutput
if(Test-Path -LiteralPath $taskOutput){throw 'Use a new evidence directory; never overwrite evidence'}
if($taskOutput.StartsWith($taskCandidate+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Output must not be inside sealed candidate'}
Assert-TwoUserNoReparseAncestors $taskOutput
Assert-TwoUserNoReparseAncestors $taskCandidate
New-Item -ItemType Directory -Path $taskOutput|Out-Null
$taskAclBefore=Get-Acl -LiteralPath $taskOutput
$taskAccounts=[Collections.Generic.List[object]]::new()
$taskAccess=[Collections.Generic.List[object]]::new()
$taskLedger=[Collections.Generic.Dictionary[int,object]]::new()
$taskSummary=[ordered]@{schemaVersion=1;ephemeralGuard=$taskEphemeralGuard;result='not-ready';expectedCommit=$ExpectedCommit;nativeExecuted=$false;verifyRuntimePorts=$VerifyRuntimePorts.IsPresent;twoSidNativePassed=$false;userIsolation='not-verified';fullVersionGatePassed=$false;runtimeProxyPortIsolation='not-verified';cleanup='pending';evidenceDirectory=$taskOutput;unverified=@('Complete runtime proxy ports: no start/stop or conflict experiment performed','Synthetic DPAPI proves OS CurrentUser sub-scope, not all application credential flows','ocxd has no peer-pipe override; cross-pipe denial uses actual other-SID .NET client','Instance lock contents cannot be hashed while Windows byte-range lock is held; size/time plus schema/pointer hashes checked')}
$taskCandidateBefore=$null
$taskDesktopRestored=$true
try {
 $identity=[Security.Principal.WindowsIdentity]::GetCurrent()
 if(!([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){
  $taskSummary.result='blocked';$taskSummary.reason='noSecondUserPermission';throw 'Local account/Profile provision permission unavailable'
 }
 foreach($tool in @($taskExe,$taskNode,(Join-Path $taskCandidate 'ocxd.exe'))){
  if(!(Test-Path -LiteralPath $tool -PathType Leaf)){throw 'NotReady: real Desktop/Node/ocxd candidate missing'}
  $stream=[IO.File]::OpenRead($tool)
  try{$reader=[IO.BinaryReader]::new($stream);if($reader.ReadUInt16() -ne 0x5a4d){throw 'NotReady: native Windows executable required'}}finally{$stream.Dispose()}
 }
 $nodePin=Read-TwoUserRuntimePolicy (Join-Path $taskRepo '.github/runtime-platforms-023.json')
 $nodeVersion=(& $taskNode --version).Trim()
 if($LASTEXITCODE -ne 0 -or $nodeVersion -cne ('v'+$nodePin.node.version)){throw 'NotReady: real test-driver Node does not match source-pinned version'}
 $taskSummary.nodeVersion=$nodeVersion
 # Read the ACTUAL candidate descriptor/archive only; never stage/download here.
 $bundle=Join-Path $taskCandidate 'runtime-bundle/runtime-bundle.manifest.json'
 if(!(Test-Path -LiteralPath $bundle -PathType Leaf)){throw 'NotReady: complete staged runtime-bundle resources missing'}
 $descriptor=Get-Content -LiteralPath $bundle -Raw|ConvertFrom-Json
 if($descriptor.archiveName -notmatch '^[a-zA-Z0-9_.-]+$' -or $descriptor.archiveSha256 -notmatch '^[0-9a-f]{64}$'){throw 'NotReady: runtime descriptor/archive identity invalid'}
 $archive=Join-Path (Split-Path -Parent $bundle) $descriptor.archiveName
 if(!(Test-Path -LiteralPath $archive -PathType Leaf) -or (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash -ine $descriptor.archiveSha256){throw 'NotReady: complete runtime archive hash differs'}
 $taskCandidateBefore=Get-TwoUserTreeHash $taskCandidate
 Write-TwoUserJson (Join-Path $taskOutput 'sealed-candidate-before.json') $taskCandidateBefore
 $taskSummary.executableSha256=(Get-FileHash -LiteralPath $taskExe).Hash.ToLowerInvariant()
 $taskSummary.nodeSha256=(Get-FileHash -LiteralPath $taskNode).Hash.ToLowerInvariant()
 $reuse=Get-TwoUserReuseParts (Join-Path $PSScriptRoot 'windows-ordinary-startup.ps1')
 if(!('OcxNativeProfileProvision' -as [type])){Add-Type -TypeDefinition $reuse.ProfileType}
 if(!('OcxTemporaryDesktopAccess' -as [type])){Add-Type -TypeDefinition $reuse.DesktopType}
 $code=Join-Path $taskOutput 'code';New-Item -ItemType Directory -Path $code|Out-Null
 foreach($name in @('windows023-two-user-native-helper.ps1','windows-cdp.mjs','windows-process-token.ps1')){Copy-Item -LiteralPath (Join-Path $PSScriptRoot $name) -Destination (Join-Path $code $name)}
 $reuse.Bootstrap|Set-Content -LiteralPath (Join-Path $code 'profile-bootstrap.ps1') -Encoding utf8
 Get-TwoUserDriverText|Set-Content -LiteralPath (Join-Path $code 'two-user-driver.mjs') -Encoding utf8
 $taskSummary.reusedSourceSha256=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'windows-ordinary-startup.ps1')).Hash.ToLowerInvariant()
 $taskSummary.scriptHashes=@(foreach($name in @('windows023-two-user-native.ps1','windows023-two-user-native-helper.ps1','windows-process-token.ps1','windows-cdp.mjs')){@{name=$name;sha256=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $name)).Hash.ToLowerInvariant()}})
 $sweepStartupDeadline=if($DiagnosticSweep){[datetime]::UtcNow.AddSeconds(600)}else{$null}
 foreach($index in 1..2){
  $name='ocxacc'+[guid]::NewGuid().ToString('N').Substring(0,10)
  $entropy=[Security.Cryptography.RandomNumberGenerator]::GetBytes(32);$secure=$null;$account=$null
  try{
   $secure=ConvertTo-SecureString ([Convert]::ToBase64String($entropy)+'aA1!') -AsPlainText -Force
   try{$user=New-LocalUser -Name $name -Password $secure -AccountNeverExpires -PasswordNeverExpires -Description 'Disposable two-SID Desktop CI account'}
   catch{$taskSummary.result='blocked';$taskSummary.reason='noSecondUserPermission';throw 'Cannot provision second ordinary CI user'}
   # Track immediately, before any later provision step can fail.
   $account=[pscustomobject]@{name=$name;sid=$user.SID.Value;profile=$null;evidence=(Join-Path $taskOutput ('user'+$index));process=$null;fact=$null;credential=$null;runtimeSequence=0;runtimeCleanupUnproven=$false}
   $taskAccounts.Add($account)
   if($account.sid -ceq $identity.User.Value){throw 'Refusing runner/daily SID'}
   Add-LocalGroupMember -Group (Get-LocalGroup -SID 'S-1-5-32-545') -Member $user
   $nativeProfilePath=[Text.StringBuilder]::new(260)
   $hr=[OcxNativeProfileProvision]::CreateProfile($account.sid,$name,$nativeProfilePath,260)
   $taskSummary.createProfileHResult=[int]$hr
   if($hr -ne 0){[Runtime.InteropServices.Marshal]::ThrowExceptionForHR($hr)}
   $account.profile=Assert-TwoUserLocalPath $nativeProfilePath.ToString()
   if((Split-Path -Leaf $account.profile) -cne $name){throw 'OS registered temporary Profile differs'}
   $taskAccess.Add([OcxTemporaryDesktopAccess]::new($account.sid))
   New-Item -ItemType Directory -Path $account.evidence|Out-Null
   # Parent/read-only code is shared; only each user's evidence folder is writable.
   & icacls.exe $taskOutput /grant ('*'+$account.sid+':(OI)(CI)RX')|Out-Null
   if($LASTEXITCODE -ne 0){throw 'Evidence read ACL failed'}
   & icacls.exe $account.evidence /grant ('*'+$account.sid+':(OI)(CI)M')|Out-Null
   if($LASTEXITCODE -ne 0){throw 'Evidence write ACL failed'}
   $config=Join-Path $account.evidence 'input.json'
   Write-TwoUserJson $config ([ordered]@{ephemeralGuard=$taskEphemeralGuard;accountSid=$account.sid;profile=$account.profile;commit=$ExpectedCommit;nativeHelper=(Join-Path $code 'windows023-two-user-native-helper.ps1');candidateDirectory=$taskCandidate;executableName=(Split-Path -Leaf $taskExe);node=$taskNode;evidence=$account.evidence;code=$code;profileOnly=$false;verifyRuntimePorts=$VerifyRuntimePorts.IsPresent;runtimeRole=$index;diagnosticSweep=$DiagnosticSweep.IsPresent;startupDeadlineUtc=$(if($sweepStartupDeadline){$sweepStartupDeadline.ToString('o')}else{$null})})
   $account.credential=[pscredential]::new(($env:COMPUTERNAME+'\'+$name),$secure)
   $account.process=Start-Process -FilePath (Join-Path $PSHOME 'pwsh.exe') -Credential $account.credential -LoadUserProfile -WorkingDirectory $account.evidence -ArgumentList @('-NoLogo','-NoProfile','-NonInteractive','-File',('"'+(Join-Path $code 'profile-bootstrap.ps1')+'"'),'-InputFile',('"'+$config+'"')) -RedirectStandardOutput (Join-Path $account.evidence 'child-stdout.log') -RedirectStandardError (Join-Path $account.evidence 'child-stderr.log') -PassThru
   $account.fact=Get-TwoUserProcessFact $account.process $account.sid;$taskLedger[[int]$account.fact.pid]=$account.fact
   $taskSummary.nativeExecuted=$true
  }finally{[Array]::Clear($entropy,0,$entropy.Length);if($secure -and (!$account -or !$account.credential)){$secure.Dispose()}}
 }
 if($taskAccounts[0].sid -ceq $taskAccounts[1].sid){throw 'Two distinct real SIDs required'}
 $deadline=if($DiagnosticSweep){$sweepStartupDeadline}else{[datetime]::UtcNow.AddSeconds(600)}
 do{
  $startupStates=@()
  foreach($account in $taskAccounts){
   $failed=$account.process.HasExited -or (Test-Path -LiteralPath (Join-Path $account.evidence 'failure.json'))
   if($failed){
    if(!$DiagnosticSweep){throw 'NotReady: ordinary child exited or reported failure before both Managers ready'}
    $startupStates+=@{sid=$account.sid;failed=$true;ready=$false};continue
   }
   Add-TwoUserDescendants $account.fact $taskLedger
   $hasReady=Test-Path -LiteralPath (Join-Path $account.evidence 'ready.json')
   $startupStates+=@{sid=$account.sid;failed=$false;ready=$hasReady}
  }
  $startupState=Get-TwoUserDiagnosticStartupState $startupStates
  $ready=$startupState.ready
  if($DiagnosticSweep -and $startupState.allSettled -and $startupState.failedSids.Count){throw 'NotReady: at least one actual SID failed; other SID startup was allowed to finish independently'}
  if([datetime]::UtcNow -gt $deadline){throw 'NotReady: two-profile native startup deadline'}
  if(!$ready){Start-Sleep -Milliseconds 500}
 }while(!$ready)
 $facts=@($taskAccounts|ForEach-Object {Get-Content -LiteralPath (Join-Path $_.evidence 'ready.json') -Raw|ConvertFrom-Json})
 foreach($i in 0..1){
  if($facts[$i].sid -cne $taskAccounts[$i].sid){throw 'Child SID receipt differs'}
  $null=Assert-TwoUserLive $facts[$i].manager;$null=Get-TwoUserWindow $facts[$i].manager
  if($facts[$i].profile -ine $taskAccounts[$i].profile){throw 'Profile receipt differs'}
  $peer=$facts[1-$i]
  Write-TwoUserJson (Join-Path $taskAccounts[$i].evidence 'peer.json') @{sid=$peer.sid;pipe=$peer.pipe;manager=$peer.manager;cipher=(Join-Path $taskAccounts[1-$i].evidence 'synthetic-dpapi.bin');privateFile=(Join-Path $peer.desktopRoot 'manager-state/preferences.json')}
 }
 if($facts[0].desktopRoot -ieq $facts[1].desktopRoot -or $facts[0].opencodexHome -ieq $facts[1].opencodexHome -or $facts[0].pipe -ceq $facts[1].pipe){throw 'Real user data/pipe identities collide'}
 $deadline=[datetime]::UtcNow.AddSeconds(90)
 do{
  $checked=$true
  foreach($account in $taskAccounts){
   if($account.process.HasExited -or (Test-Path -LiteralPath (Join-Path $account.evidence 'failure.json'))){throw 'Native cross-user/duplicate check failed'}
   Add-TwoUserDescendants $account.fact $taskLedger
   $checked=$checked -and (Test-Path -LiteralPath (Join-Path $account.evidence 'checked.json'))
  }
  if([datetime]::UtcNow -gt $deadline){throw 'Native isolation check deadline'}
  if(!$checked){Start-Sleep -Milliseconds 250}
 }while(!$checked)
 # Both original Manager generations/windows must overlap now, after peer tests.
 $overlapStart=[datetime]::UtcNow
 foreach($n in 1..6){foreach($f in $facts){$null=Assert-TwoUserLive $f.manager;$null=Get-TwoUserWindow $f.manager};Start-Sleep -Seconds 1}
 $taskSummary.overlap=[ordered]@{fromUtc=$overlapStart.ToString('o');toUtc=[datetime]::UtcNow.ToString('o');minimumSeconds=6;observedSeconds=([datetime]::UtcNow-$overlapStart).TotalSeconds;managers=@($facts|ForEach-Object {$_.manager})}
 $taskSummary.users=@($taskAccounts|ForEach-Object {Get-Content -LiteralPath (Join-Path $_.evidence 'checked.json') -Raw|ConvertFrom-Json})
 foreach($userFacts in $taskSummary.users){
  if(!$DiagnosticSweep -and ($userFacts.result -cne 'pass-user-isolation' -or $userFacts.applicationToken.elevated -ne $false -or $userFacts.peerBusinessData -cne 'access-denied' -or $userFacts.crossPipe -cne 'access-denied-real-token' -or !$userFacts.duplicate.rejected)){throw 'Actual native checked facts incomplete'}
 }
 if($VerifyRuntimePorts.IsPresent -and (!$DiagnosticSweep -or @($facts|Where-Object {!$_.runtimeReady}).Count -eq 0)){
  foreach($userFacts in $taskSummary.users){if($userFacts.verifyRuntimePorts -ne $true){throw 'Both account children must opt into runtime-port verification'}}
  $taskSummary.runtimeProtocol=Invoke-TwoUserRuntimeProtocol $taskAccounts $taskLedger
  Write-TwoUserJson (Join-Path $taskOutput 'runtime-port-protocol.json') $taskSummary.runtimeProtocol
  $taskSummary.runtimeProxyPortIsolation='verified-runtime-ports'
  $taskSummary.unverified=@($taskSummary.unverified|Where-Object {$_ -notlike 'Complete runtime proxy ports:*'})
 }
 if($DiagnosticSweep){$taskSummary.accepted=$false;$taskSummary.diagnosticOnly=$true;$taskSummary.result='diagnostic-only';$taskSummary.runtimePortPrerequisitesMet=(@($facts|Where-Object {!$_.runtimeReady}).Count -eq 0)}else{$taskSummary.userIsolation='verified-two-native-SIDs';$taskSummary.result='pass-user-isolation'}
}catch{
 if($taskSummary.result -notin @('blocked')){$taskSummary.result=if($_.Exception.Message.StartsWith('NotReady:')){'not-ready'}else{'fail'}}
 # Type + controlled stage only; never serialize credentials or native argv.
 $taskSummary.failureCode=if($_.Exception.Message.StartsWith('NotReady:')){$_.Exception.Message}else{'Native preflight/identity/isolation assertion failed; inspect retained stage/evidence'}
 $taskSummary.failureKind=$_.Exception.GetType().FullName
 $taskSummary.failureStage=$_.InvocationInfo.ScriptLineNumber
}finally{
 $cleanup=[Collections.Generic.List[object]]::new()
 foreach($account in $taskAccounts){
  try{Set-Content -LiteralPath (Join-Path $account.evidence 'stop') -Value 'stop' -Encoding ascii}catch{}
 }
 foreach($account in $taskAccounts){
  if($account.process){
   try{if(!$account.process.HasExited){Add-TwoUserDescendants $account.fact $taskLedger};[void]$account.process.WaitForExit(20000)}catch{}
   $finished=Join-Path $account.evidence 'finished.json'
   if($taskSummary.result -ceq 'pass-user-isolation'){
    if(!(Test-Path -LiteralPath $finished)){$taskSummary.result='evidence-incomplete'}
    else{$last=Get-Content -LiteralPath $finished -Raw|ConvertFrom-Json;if($last.result -cne 'pass-user-isolation' -or $last.inventory -cne 'hashed' -or ($VerifyRuntimePorts.IsPresent -and $last.runtimeProxyVerification -cne 'verified-runtime-ports')){$taskSummary.result='evidence-incomplete'}}
   }
  }
 }
 # Public launch ledger lets exact cleanup survive an intermediate parent exit.
 foreach($account in $taskAccounts){
  $tracked=Join-Path $account.evidence 'runtime-tracked.json'
  if(Test-Path -LiteralPath $tracked){
   try{foreach($fact in @(Get-Content -LiteralPath $tracked -Raw|ConvertFrom-Json)){
    if($fact.sid -cne $account.sid -or [long]$fact.creationTicks -lt [long]$account.fact.creationTicks){throw 'Runtime cleanup ledger escaped new account/generation'}
    if(Test-TwoUserOriginalExited $fact){continue}
    $observedProcess=Get-Process -Id $fact.pid -ErrorAction SilentlyContinue
    if($observedProcess){
     try{
      if(!$observedProcess.HasExited){
       try{$observedFact=Get-TwoUserProcessFact $observedProcess $fact.sid}catch{if(!$observedProcess.HasExited){throw};$observedFact=$null}
       if($observedFact){
        if($observedFact.creationTicks -cne $fact.creationTicks -or $observedFact.path -ine $fact.path){throw 'Refusing changed runtime ledger PID/SID/image/creation'}
        $taskLedger[[int]$fact.pid]=$fact
       }
      }
     }finally{$observedProcess.Dispose()}
    }
   }}catch{$account.runtimeCleanupUnproven=$true;$cleanup.Add(@{sid=$account.sid;result='runtime-ledger-refused';reason=($_.Exception.Message -split "\r?\n",2)[0]});$taskSummary.result='cleanup-failed'}
  }
 }
 foreach($fact in @($taskLedger.Values|Sort-Object {[long]$_.creationTicks} -Descending)){
  try{Stop-TwoUserTracked $fact;$cleanup.Add(@{pid=$fact.pid;sid=$fact.sid;creationTicks=$fact.creationTicks;image=$fact.path;result='stopped-or-exited'})}
  catch{$cleanup.Add(@{pid=$fact.pid;sid=$fact.sid;result='retained-unproven'});$taskSummary.result='cleanup-failed'}
 }
 Close-TwoUserProcessHandles
 # Restore nested desktop grants in reverse order to exactly restore original ACL.
 for($i=$taskAccess.Count-1;$i -ge 0;$i--){try{$taskAccess[$i].Dispose()}catch{$taskDesktopRestored=$false;$taskSummary.result='cleanup-failed'}}
 foreach($account in $taskAccounts){
  try{
   if($account.runtimeCleanupUnproven){throw 'Runtime identity unproven; retain new account/Profile'}
   if(@($cleanup|Where-Object {$_.sid -ceq $account.sid -and $_.result -ceq 'retained-unproven'}).Count){throw 'Live ownership not proven; retain corresponding Profile/account'}
   # Release the exact terminated bootstrap handle BEFORE Profile unload.
   # A retained Windows process object can keep its logon token referenced.
   # Do not release or delete anything while that original handle is still live.
   if($account.process){
    if(!$account.process.HasExited){throw 'Original bootstrap handle is still live; retain Profile/account'}
    $account.process.Dispose();$account.process=$null
   }
   $current=Get-LocalUser -SID $account.sid -ErrorAction Stop
   if($current.Name -cne $account.name -or $current.Name -notlike 'ocxacc*'){throw 'Cleanup account identity changed'}
   $profiles=@(Get-CimInstance Win32_UserProfile|Where-Object {$_.SID -ceq $account.sid})
   foreach($p in $profiles){
    if($p.LocalPath -ine $account.profile -or (Split-Path -Leaf $p.LocalPath) -cne $account.name -or $p.Special){throw 'Refusing untracked Profile deletion'}
    $unloadDeadline=[datetime]::UtcNow.AddSeconds(30)
    while($p.Loaded -and [datetime]::UtcNow -lt $unloadDeadline){Start-Sleep -Milliseconds 500;$p=Get-CimInstance Win32_UserProfile|Where-Object {$_.SID -ceq $account.sid}}
    if($p.Loaded){throw 'Temporary Profile still loaded; retain and report cleanup failure'}
    Remove-CimInstance -InputObject $p -ErrorAction Stop
    if(Test-Path -LiteralPath $account.profile){throw 'Profile removal did not remove own private copy and data'}
   }
   Remove-LocalUser -SID $account.sid -ErrorAction Stop
   $cleanup.Add(@{sid=$account.sid;profile=$account.profile;result='account-and-profile-removed'})
  }catch{$cleanup.Add(@{sid=$account.sid;result='account-or-profile-retained';reason=($_.Exception.Message -split "\r?\n",2)[0]});$taskSummary.result='cleanup-failed'}
  if($account.credential){$account.credential.Password.Dispose()};if($account.process){$account.process.Dispose()}
 }
 try{Set-Acl -LiteralPath $taskOutput -AclObject $taskAclBefore;$taskSummary.evidenceAclRestored=$true}catch{$taskSummary.result='cleanup-failed'}
 if($taskCandidateBefore){
  try{$after=Get-TwoUserTreeHash $taskCandidate;Write-TwoUserJson (Join-Path $taskOutput 'sealed-candidate-after.json') $after
   $taskSummary.sealedCandidateUnchanged=$after.sha256 -ceq $taskCandidateBefore.sha256
   if(!$taskSummary.sealedCandidateUnchanged){$taskSummary.result='fail'}
  }catch{$taskSummary.result='fail'}
 }
 $taskSummary.cleanup=$cleanup.ToArray();$taskSummary.desktopAclRestored=$taskDesktopRestored
 $taskSummary.twoSidNativePassed=$taskSummary.result -ceq 'pass-user-isolation'
 $taskSummary.fullVersionGatePassed=Test-TwoUserRuntimeFullGate $VerifyRuntimePorts.IsPresent $taskSummary.twoSidNativePassed $taskSummary.runtimeProxyPortIsolation ($taskSummary.runtimeProtocol.servicesStopped -eq $true -and $taskSummary.sealedCandidateUnchanged -eq $true -and $taskSummary.desktopAclRestored -eq $true -and $taskSummary.evidenceAclRestored -eq $true)
 if($VerifyRuntimePorts.IsPresent -and !$taskSummary.fullVersionGatePassed -and $taskSummary.result -ceq 'pass-user-isolation'){$taskSummary.result='runtime-proof-incomplete'}
 $taskSummary.finishedUtc=[datetime]::UtcNow.ToString('o')
 Write-TwoUserJson (Join-Path $taskOutput 'two-user-native.json') $taskSummary
}
if(!$DiagnosticSweep -and $taskSummary.result -cne 'pass-user-isolation'){throw 'Two-user native evidence is not passing; inspect two-user-native.json and retained user receipts'}
$taskSummary|ConvertTo-Json -Depth 15

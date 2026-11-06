# Parent compilation + one genuine ordinary-account Rust-binary suite.
param(
 [Parameter(Mandatory=$true)][string]$Cargo,
 [Parameter(Mandatory=$true)][string]$ManifestPath,
 [Parameter(Mandatory=$true)][string]$Node,
 [Parameter(Mandatory=$true)][string]$OutputDirectory,
 [Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-f]{40}$')][string]$ExpectedCommit,
 [string[]]$CargoArguments=@('--locked','--workspace','--features','integration-test','--no-fail-fast'),
 [string[]]$TestArguments=@(),
 [ValidateRange(1,86400)][int]$SuiteTimeoutSeconds=2400
)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'windows023-rust-ordinary-helper.ps1')
Assert-RustOrdinaryEphemeral # Before any build, output, account, Profile or ACL write.
. (Join-Path $PSScriptRoot 'windows023-two-user-native-helper.ps1')
. (Join-Path $PSScriptRoot 'windows-process-token.ps1')
if(!(Get-Command Start-Process).Parameters.ContainsKey('Environment')){throw 'PowerShell 7.4+ required for secret-free RunAs environment'}
$taskEphemeralGuard=$true
$taskRepo=(Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../..')).ProviderPath
$taskCommit=(& git -C $taskRepo rev-parse HEAD).Trim()
if($LASTEXITCODE -ne 0 -or $taskCommit -cne $ExpectedCommit){throw 'Exact source commit required'}
$taskManifest=(Resolve-Path -LiteralPath $ManifestPath).ProviderPath
$null=Assert-TwoUserUnder $taskRepo $taskManifest
$taskCargo=(Resolve-Path -LiteralPath $Cargo).ProviderPath;$taskNode=(Resolve-Path -LiteralPath $Node).ProviderPath
$taskOutput=Assert-TwoUserLocalPath $OutputDirectory;$null=Assert-TwoUserUnder $env:RUNNER_TEMP $taskOutput
if(Test-Path -LiteralPath $taskOutput){throw 'New private evidence directory required; preserve earlier failures'}
foreach($path in @($taskRepo,$taskManifest,$taskCargo,$taskNode,$taskOutput)){Assert-TwoUserNoReparseAncestors $path}
$taskBuildArgs=Get-RustOrdinaryBuildArguments $CargoArguments $taskManifest
Assert-RustOrdinaryTestArguments $TestArguments
$taskIdentity=[Security.Principal.WindowsIdentity]::GetCurrent()
if(!([Security.Principal.WindowsPrincipal]::new($taskIdentity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'noOrdinaryUserPermission: parent must compile and provision as administrator'}
New-Item -ItemType Directory -Path $taskOutput|Out-Null
$taskChanges=[Collections.Generic.List[object]]::new();$taskAccess=$null;$taskUser=$null;$taskProfile=$null;$taskProcess=$null;$taskFact=$null;$taskSecure=$null
$taskLedger=[Collections.Generic.Dictionary[int,object]]::new();$taskRetain=$false;$taskExit=1
$taskReport=[ordered]@{schemaVersion=1;ephemeralGuard=$taskEphemeralGuard;result='fail';scope='binaries';fullWorkspacePassed=$false;expectedCommit=$ExpectedCommit;nativeSuiteExecuted=$false;allBinariesAttempted=$false;docTestsExecuted=$false;adminSpecializedGroups='must-run-independently-in-parent-always-path';cargoArguments=$CargoArguments;testArguments=$TestArguments;suiteTimeoutSeconds=$SuiteTimeoutSeconds;sourceOrArtifactAclRestored=$false;desktopAclRestored=$false;accountAndProfileRemoved=$false;exitCode=1}
try{
 $taskReport.stage='source-policy'
 $nodeVersion=(& $taskNode --version).Trim()
 $nodePin=Read-TwoUserRuntimePolicy (Join-Path $taskRepo '.github/runtime-platforms-023.json')
 if($LASTEXITCODE -ne 0 -or $nodeVersion -cne ('v'+$nodePin.node.version)){throw 'Real source-pinned Node required for unchanged native fixtures'}
 $taskReport.node=@{path=$taskNode;sha256=(Get-FileHash -LiteralPath $taskNode -Algorithm SHA256).Hash.ToLowerInvariant();version=$nodeVersion}
 $taskReport.stage='compile-test-binaries'
 # Compilation stays in the parent administrator. Exact JSON artifacts, not globs.
 & $taskCargo metadata --manifest-path $taskManifest --locked --no-deps --format-version 1 > (Join-Path $taskOutput 'cargo-metadata.json') 2> (Join-Path $taskOutput 'cargo-metadata.stderr.log')
 if($LASTEXITCODE -ne 0){throw 'Cargo metadata failed'}
 & $taskCargo @taskBuildArgs > (Join-Path $taskOutput 'cargo-no-run.jsonl') 2> (Join-Path $taskOutput 'cargo-no-run.stderr.log')
 if($LASTEXITCODE -ne 0){$taskExit=$LASTEXITCODE;throw 'Parent Cargo no-run compilation failed'}
 $metadata=Get-Content -LiteralPath (Join-Path $taskOutput 'cargo-metadata.json') -Raw|ConvertFrom-Json
 $taskHarnessTargets=@(Get-RustOrdinaryHarnessAdmission $metadata)
 $taskArtifacts=@(Get-RustOrdinaryArtifacts @(Get-Content -LiteralPath (Join-Path $taskOutput 'cargo-no-run.jsonl')) $metadata)
 Assert-RustOrdinaryArtifactCoverage $taskHarnessTargets $taskArtifacts
 $taskReport.harnessContract='metadata-covered-default-libtest-only; custom/ambiguous harness refused'
 foreach($artifact in $taskArtifacts){
  $null=Assert-TwoUserUnder $taskRepo $artifact.workingDirectory
  Assert-TwoUserNoReparseAncestors $artifact.executable
  $artifact.sha256=(Get-FileHash -LiteralPath $artifact.executable -Algorithm SHA256).Hash.ToLowerInvariant()
 }
 $taskDlls=[Collections.Generic.Dictionary[string,object]]::new([StringComparer]::OrdinalIgnoreCase)
 foreach($artifact in $taskArtifacts){foreach($dll in Get-ChildItem -LiteralPath (Split-Path -Parent $artifact.executable) -File -Filter '*.dll'){
  Assert-TwoUserNoReparseAncestors $dll.FullName
  $taskDlls[$dll.FullName]=@{executable=$dll.FullName;sha256=(Get-FileHash -LiteralPath $dll.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}
 }}
 $taskReport.runtimeDlls=@($taskDlls.Values)
 $taskReport.artifacts=$taskArtifacts
 Write-TwoUserJson (Join-Path $taskOutput 'frozen-artifacts.json') @{expectedCommit=$ExpectedCommit;artifacts=$taskArtifacts;runtimeDlls=@($taskDlls.Values);testArguments=$TestArguments;node=$taskReport.node}
 $taskReport.stage='provision-ordinary-account'
 $parts=Get-RustOrdinaryBootstrap (Join-Path $PSScriptRoot 'windows-ordinary-startup.ps1')
 if(!('OcxNativeProfileProvision' -as [type])){Add-Type -TypeDefinition $parts.ProfileType}
 if(!('OcxTemporaryDesktopAccess' -as [type])){Add-Type -TypeDefinition $parts.DesktopType}
 $code=Join-Path $taskOutput 'code';$evidence=Join-Path $taskOutput 'ordinary'
 New-Item -ItemType Directory -Path $code,$evidence|Out-Null
 foreach($name in @('windows023-rust-ordinary-helper.ps1','windows023-two-user-native-helper.ps1','windows-process-token.ps1')){Copy-Item -LiteralPath (Join-Path $PSScriptRoot $name) -Destination (Join-Path $code $name)}
 $parts.Bootstrap|Set-Content -LiteralPath (Join-Path $code 'profile-bootstrap.ps1') -Encoding utf8
 $taskName='ocxacc'+[guid]::NewGuid().ToString('N').Substring(0,10)
 $entropy=[Security.Cryptography.RandomNumberGenerator]::GetBytes(32)
 try{$taskSecure=ConvertTo-SecureString ([Convert]::ToBase64String($entropy)+'aA1!') -AsPlainText -Force}finally{[Array]::Clear($entropy,0,$entropy.Length)}
 $taskUser=New-LocalUser -Name $taskName -Password $taskSecure -AccountNeverExpires -Description 'Disposable ordinary Rust suite CI account'
 if($taskUser.SID.Value -ceq $taskIdentity.User.Value){throw 'Refuse existing parent SID'}
 Add-LocalGroupMember -Group (Get-LocalGroup -SID 'S-1-5-32-545') -Member $taskUser
 # Reuse the MAX_PATH-sized CreateProfile contract already exercised by the
 # ordinary-startup native harness. Retain the exact HRESULT before throwing.
 $profileBuffer=[Text.StringBuilder]::new(260);$hr=[OcxNativeProfileProvision]::CreateProfile($taskUser.SID.Value,$taskName,$profileBuffer,260)
 $taskReport.createProfileHResult=[int]$hr
 if($hr -ne 0){[Runtime.InteropServices.Marshal]::ThrowExceptionForHR($hr)}
 $taskProfile=Assert-TwoUserLocalPath $profileBuffer.ToString()
 if((Split-Path -Leaf $taskProfile) -cne $taskName){throw 'OS registered new Profile differs'}
 $taskReport.accountSid=$taskUser.SID.Value;$taskReport.profile=$taskProfile
 $taskAccess=[OcxTemporaryDesktopAccess]::new($taskUser.SID.Value)
 # Read grants only for tracked source files, exact emitted binaries/DLL dependencies,
 # and new harness artifacts. Never grant runner Profile, .git or recursive target writes.
 $read=[Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
 $tracked=@(& git -C $taskRepo -c core.quotepath=false ls-files)
 if($LASTEXITCODE -ne 0){throw 'Tracked source enumeration failed'}
 $sourceInputs=[Collections.Generic.List[object]]::new()
 foreach($rel in $tracked){
  if(($rel.Replace([char]92,[char]47)) -match '(^|/)\.git(/|$)'){throw 'Git internals cannot be ordinary test inputs'}
  $path=Join-Path $taskRepo $rel
  if(Test-Path -LiteralPath $path -PathType Leaf){$null=Assert-TwoUserUnder $taskRepo $path;[void]$read.Add($path);$sourceInputs.Add(@{path=$path;sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()})}
 }
 Write-TwoUserJson (Join-Path $taskOutput 'source-input-hashes.json') @{expectedCommit=$ExpectedCommit;files=$sourceInputs.ToArray()}
 foreach($artifact in $taskArtifacts){
  [void]$read.Add($artifact.executable)
 }
 foreach($dll in $taskDlls.Values){[void]$read.Add($dll.executable)}
 foreach($file in Get-ChildItem -LiteralPath $code -File){[void]$read.Add($file.FullName)}
 $directories=[Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
 foreach($path in $read){
  $parent=Split-Path -Parent $path
  # Only known artifact/source directories; never alter an ancestor user Profile.
  while($parent -and ($parent -ieq $taskRepo -or $parent.StartsWith($taskRepo+'\',[StringComparison]::OrdinalIgnoreCase) -or $parent -ieq $code)){[void]$directories.Add($parent);$parent=Split-Path -Parent $parent}
 }
 $taskReport.stage='grant-exact-read-access'
 foreach($path in @($directories)+@($read)){Add-RustOrdinaryReadAccess $path $taskUser.SID.Value $taskChanges}
 foreach($path in @($taskOutput,$evidence)){
  $acl=Get-Acl -LiteralPath $path;$taskChanges.Add([pscustomobject]@{path=$path;accessSddl=$acl.GetSecurityDescriptorSddlForm([Security.AccessControl.AccessControlSections]::Access)})
  $rights=if($path -ceq $evidence){'Modify'}else{'ReadAndExecute'}
  $inherit=if($path -ceq $evidence){'ContainerInherit, ObjectInherit'}else{'None'}
  $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($taskUser.SID.Value),$rights,$inherit,'None','Allow'));Set-Acl -LiteralPath $path -AclObject $acl
 }
 $config=Join-Path $evidence 'input.json'
 Write-TwoUserJson $config @{ephemeralGuard=$true;accountSid=$taskUser.SID.Value;profile=$taskProfile;profileOnly=$false;nativeHelper=(Join-Path $code 'windows023-rust-ordinary-helper.ps1');code=$code;evidence=$evidence;commit=$ExpectedCommit;artifacts=$taskArtifacts;runtimeDlls=@($taskDlls.Values);testArguments=$TestArguments;suiteTimeoutSeconds=$SuiteTimeoutSeconds;node=$taskNode;nodeSha256=$taskReport.node.sha256}
 $credential=[pscredential]::new(($env:COMPUTERNAME+'\'+$taskName),$taskSecure)
 $taskChildEnvironment=@{GITHUB_ACTIONS='true';RUNNER_ENVIRONMENT='github-hosted';ImageOS=$env:ImageOS;RUNNER_TEMP=$taskOutput;TEMP=(Join-Path $taskProfile 'AppData/Local/Temp');TMP=(Join-Path $taskProfile 'AppData/Local/Temp')}
 $taskReport.stage='run-ordinary-suite'
 $taskProcess=Start-Process -FilePath (Join-Path $PSHOME 'pwsh.exe') -Credential $credential -LoadUserProfile -UseNewEnvironment -Environment $taskChildEnvironment -WorkingDirectory $evidence -ArgumentList @('-NoLogo','-NoProfile','-NonInteractive','-File',('"'+(Join-Path $code 'profile-bootstrap.ps1')+'"'),'-InputFile',('"'+$config+'"')) -RedirectStandardOutput (Join-Path $evidence 'bootstrap.stdout.log') -RedirectStandardError (Join-Path $evidence 'bootstrap.stderr.log') -PassThru
 $taskFact=Get-TwoUserProcessFact $taskProcess $taskUser.SID.Value;$taskLedger[[int]$taskFact.pid]=$taskFact
 $taskReport.nativeSuiteExecuted=$true;$until=[datetime]::UtcNow.AddSeconds($SuiteTimeoutSeconds+120)
 while(!$taskProcess.HasExited){try{Add-TwoUserDescendants $taskFact $taskLedger}catch{if(!$taskProcess.HasExited){throw}};if([datetime]::UtcNow -gt $until){throw 'Ordinary suite controller envelope exhausted'};Start-Sleep -Milliseconds 100}
 $taskExit=$taskProcess.ExitCode
 $child=Get-Content -LiteralPath (Join-Path $evidence 'ordinary-suite.json') -Raw|ConvertFrom-Json
 if($child.accountSid -cne $taskUser.SID.Value -or $child.expectedCommit -cne $ExpectedCommit -or $child.exitCode -ne $taskExit -or !(Test-TwoUserSamePath $child.profile $taskProfile)){throw 'Child suite receipt does not match actual process exit/identity'}
 $taskReport.child=$child;$taskReport.allBinariesAttempted=$child.allBinariesAttempted
 $taskReport.result=$child.result
}catch{
 $taskReport.failureCode='ordinary_suite_build_provision_or_execution_failed';$taskReport.failureType=$_.Exception.GetType().FullName;$taskReport.failureLine=$_.InvocationInfo.ScriptLineNumber
 $taskReport.failureHResult=[int]$_.Exception.HResult
 if($_.Exception.InnerException){$taskReport.innerFailureHResult=[int]$_.Exception.InnerException.HResult}
 if($taskExit -eq 0){$taskExit=1}
}finally{
 $cleanup=[Collections.Generic.List[object]]::new()
 if($taskProcess -and !$taskProcess.HasExited){try{Add-TwoUserDescendants $taskFact $taskLedger}catch{$taskRetain=$true}}
 if($taskUser){
  $trackedPath=Join-Path $taskOutput 'ordinary/owned-processes.json'
  if(Test-Path -LiteralPath $trackedPath){try{foreach($fact in @(Get-Content -LiteralPath $trackedPath -Raw|ConvertFrom-Json)){
   if($fact.sid -cne $taskUser.SID.Value -or [long]$fact.creationTicks -lt [long]$taskFact.creationTicks){throw 'Child cleanup ledger escaped created SID/generation'}
   if(Get-Process -Id $fact.pid -ErrorAction SilentlyContinue){$null=Assert-TwoUserLive $fact;$taskLedger[[int]$fact.pid]=$fact}
  }}catch{$taskRetain=$true}}
 }
 foreach($fact in @($taskLedger.Values|Sort-Object {[long]$_.creationTicks} -Descending)){
  try{Stop-TwoUserTracked $fact;$cleanup.Add(@{process=$fact;result='stopped-or-exited'})}catch{$taskRetain=$true;$cleanup.Add(@{process=$fact;result='retained-unproven'})}
 }
 # A leaked/untracked process of this NEW SID is not permission to kill it.
 if($taskUser){
  $untracked=@()
  foreach($row in @(Get-CimInstance Win32_Process)){
   try{$owner=Invoke-CimMethod -InputObject $row -MethodName GetOwnerSid -ErrorAction Stop
    if($owner.ReturnValue -eq 0 -and $owner.Sid -ceq $taskUser.SID.Value){$untracked+=@{pid=$row.ProcessId;sid=$owner.Sid;image=$row.ExecutablePath;creationUtc=$row.CreationDate.ToUniversalTime().ToString('o')}}
   }catch{}
  }
  $taskReport.remainingNewSidProcesses=$untracked
  if($untracked.Count){$taskRetain=$true}
 }
 if($taskAccess){try{$taskAccess.Dispose();$taskReport.desktopAclRestored=$true}catch{$taskRetain=$true}}
 $restored=$true;$aclErrors=@();for($i=$taskChanges.Count-1;$i -ge 0;$i--){try{Restore-RustOrdinaryReadAccess $taskChanges[$i]}catch{$restored=$false;$aclErrors+=@{path=$taskChanges[$i].path;type=$_.Exception.GetType().FullName;hResult=[int]$_.Exception.HResult}}}
 $taskReport.aclRestoreErrors=$aclErrors
 $taskReport.sourceOrArtifactAclRestored=$restored
 if($taskUser -and !$taskRetain){try{
  $current=Get-LocalUser -SID $taskUser.SID.Value
  if($current.Name -cne $taskName -or $current.Name -notlike 'ocxacc*'){throw 'Created account identity changed'}
  foreach($profile in @(Get-CimInstance Win32_UserProfile|Where-Object {$_.SID -ceq $taskUser.SID.Value})){
   if($profile.Special -or $profile.LocalPath -ine $taskProfile){throw 'Refuse untracked Profile deletion'}
   $until=[datetime]::UtcNow.AddSeconds(30)
   while($profile.Loaded -and [datetime]::UtcNow -lt $until){Start-Sleep -Milliseconds 250;$profile=Get-CimInstance Win32_UserProfile|Where-Object {$_.SID -ceq $taskUser.SID.Value}}
   if($profile.Loaded){throw 'Created Profile still loaded'}
   Remove-CimInstance -InputObject $profile
   if(Test-Path -LiteralPath $taskProfile){throw 'Created Profile removal did not finish'}
  }
  Remove-LocalUser -SID $taskUser.SID.Value;$taskReport.accountAndProfileRemoved=$true
 }catch{$taskRetain=$true}}
 if($taskUser -and ($taskRetain -or !$restored)){$taskReport.result='cleanup-failed';if($taskExit -eq 0){$taskExit=1}}
 foreach($artifact in @(@($taskReport.artifacts)+@($taskReport.runtimeDlls)|Where-Object {$_})){try{if((Get-FileHash -LiteralPath $artifact.executable -Algorithm SHA256).Hash -ine $artifact.sha256){throw 'Artifact bytes changed'}}catch{$taskReport.result='artifact-changed';if($taskExit -eq 0){$taskExit=1}}}
 $sourceUnchanged=$true
 if($sourceInputs){foreach($input in $sourceInputs){try{if((Get-FileHash -LiteralPath $input.path -Algorithm SHA256).Hash -ine $input.sha256){$sourceUnchanged=$false}}catch{$sourceUnchanged=$false}}}
 $taskReport.sourceBytesUnchanged=$sourceUnchanged
 if(!$sourceUnchanged){$taskReport.result='source-changed';if($taskExit -eq 0){$taskExit=1}}
 $taskReport.cleanup=$cleanup.ToArray();$taskReport.exitCode=$taskExit
 if($taskSecure){$taskSecure.Dispose()};if($taskProcess){$taskProcess.Dispose()}
 Write-TwoUserJson (Join-Path $taskOutput 'rust-ordinary.json') $taskReport
 # Bounded public diagnostics are available in the job log before artifact upload.
 # Never serialize the credential, raw exception message, environment or test data.
 [ordered]@{result=$taskReport.result;stage=$taskReport.stage;failureCode=$taskReport.failureCode;failureType=$taskReport.failureType;failureLine=$taskReport.failureLine;failureHResult=$taskReport.failureHResult;innerFailureHResult=$taskReport.innerFailureHResult;createProfileHResult=$taskReport.createProfileHResult;nativeSuiteExecuted=$taskReport.nativeSuiteExecuted;allBinariesAttempted=$taskReport.allBinariesAttempted;exitCode=$taskExit}|ConvertTo-Json -Compress|Write-Output
}
exit $taskExit

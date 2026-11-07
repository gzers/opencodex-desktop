# Offline AST / pure fixtures. No native API, Rust executable or account is run.
param([Parameter(Mandatory=$true)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $taskOutput){throw 'Fresh static evidence directory required'}
New-Item -ItemType Directory -Path $taskOutput|Out-Null
$taskMain=Join-Path $PSScriptRoot 'windows023-rust-ordinary.ps1'
$taskHelper=Join-Path $PSScriptRoot 'windows023-rust-ordinary-helper.ps1'
. (Join-Path $PSScriptRoot 'windows023-two-user-native-helper.ps1')
. $taskHelper
$checks=[Collections.Generic.List[object]]::new()
function Check([bool]$Value,[string]$Name){if(!$Value){throw ('Static fixture failed: '+$Name)};$checks.Add(@{name=$Name;result='pass'})}
function Refuses([scriptblock]$Action,[string]$Name){$refused=$false;try{& $Action|Out-Null}catch{$refused=$true};Check $refused $Name}
foreach($file in @($taskMain,$taskHelper,$PSCommandPath)){
 $tokens=$null;$errors=$null;$ast=[Management.Automation.Language.Parser]::ParseFile($file,[ref]$tokens,[ref]$errors)
 Check ($errors.Count -eq 0) ('parser-'+[IO.Path]::GetFileName($file))
 $reserved=@($ast.FindAll({param($n) $n -is [Management.Automation.Language.AssignmentStatementAst] -and $n.Left -is [Management.Automation.Language.VariableExpressionAst] -and $n.Left.VariablePath.UserPath -in @('HOME','home','PID','Host','PSHOME','PSScriptRoot','PSCommandPath','CODEX_HOME')},$true))
 Check ($reserved.Count -eq 0) ('no-reserved-variable-assignment-'+[IO.Path]::GetFileName($file))
}
$main=Get-Content -LiteralPath $taskMain -Raw;$helper=Get-Content -LiteralPath $taskHelper -Raw
Check ($main.IndexOf('Assert-RustOrdinaryEphemeral # Before') -lt $main.IndexOf('New-Item') -and $main.IndexOf('Assert-RustOrdinaryEphemeral # Before') -lt $main.IndexOf('& $taskCargo metadata')) 'ephemeral-guard-before-compilation-and-all-native-mutations'
Check ($helper.Contains("RUNNER_ENVIRONMENT -cne 'github-hosted'") -and $helper.Contains('$env:ImageOS')) 'fresh-hosted-runner-required-not-just-GHA-flag'
Check ($main.Contains('New-LocalUser') -and $main.Contains('CreateProfile') -and $main.Contains('-LoadUserProfile') -and $main.Contains("'S-1-5-32-545'")) 'one-new-ordinary-account-real-profile-loaded'
$parts=Get-RustOrdinaryBootstrap (Join-Path $PSScriptRoot 'windows-ordinary-startup.ps1')
Check ($parts.Bootstrap.Contains('OcxFirstLogonFolders') -and $parts.Bootstrap.Contains('ProfileList') -and !$parts.Bootstrap.Contains('& $c.script')) 'existing-profile-bootstrap-reused-no-product-startup'
Check ($parts.Bootstrap.IndexOf('[Environment]::GetEnvironmentVariables') -lt $parts.Bootstrap.IndexOf('$c=Get-Content') -and !$parts.Bootstrap.Contains("'GITHUB_TOKEN'")) 'parent-secret-environment-scrubbed-before-loading-public-config'
Check ($parts.Bootstrap.Contains("'OCX_CI_LOCAL_PROFILE_INITIALIZED'") -and $parts.Bootstrap.Contains("'RUNNER_ENVIRONMENT'")) 'known-folder-reexec-keeps-only-explicit-guard-state'
$bootstrap=Join-Path $taskOutput 'bootstrap-fixture.ps1';$parts.Bootstrap|Set-Content -LiteralPath $bootstrap -Encoding utf8
$tokens=$null;$errors=$null;$null=[Management.Automation.Language.Parser]::ParseFile($bootstrap,[ref]$tokens,[ref]$errors)
Check ($errors.Count -eq 0) 'generated-profile-bootstrap-parser'
Initialize-RustOrdinaryLaunch
Check ([bool]('OcxRustOrdinaryLaunch' -as [type])) 'native-launch-managed-compilation-only-no-PInvoke'
# Quote exact Windows argv in managed code without starting any process.
Check ([OcxRustOrdinaryLaunch]::Quote('') -ceq '""') 'windows-argv-empty-preserved'
Check ([OcxRustOrdinaryLaunch]::Quote('a b') -ceq '"a b"') 'windows-argv-spaces-preserved'
Check ([OcxRustOrdinaryLaunch]::Quote('a"b') -ceq '"a\"b"') 'windows-argv-quotes-preserved'
Check ([OcxRustOrdinaryLaunch]::Quote('C:\last\') -ceq '"C:\last\\"') 'windows-argv-trailing-slash-preserved'
Refuses {[OcxRustOrdinaryLaunch]::Quote(('bad'+[char]0+'argv'))} 'windows-argv-NUL-refused'
$argsOriginal=@('--locked','--workspace','--features','integration-test','--no-fail-fast')
$built=@(Get-RustOrdinaryBuildArguments $argsOriginal 'C:\code\Cargo.toml')
Check (($built -join '|') -ceq 'test|--manifest-path|C:\code\Cargo.toml|--locked|--workspace|--features|integration-test|--no-fail-fast|--no-run|--message-format=json') 'original-Cargo-build-scope-features-lock-preserved-with-no-run-JSON'
foreach($bad in @('--exclude','--lib','--test','some-test-filter','--doc','--no-run','--message-format=json','--config')){Refuses {Get-RustOrdinaryBuildArguments (@('--workspace','--no-fail-fast')+$bad) 'C:\code\Cargo.toml'} ('cargo-scope-narrowing-or-override-refused-'+$bad)}
Refuses {Get-RustOrdinaryBuildArguments @('--workspace') 'C:\code\Cargo.toml'} 'no-fail-fast-must-remain-explicit'
Assert-RustOrdinaryTestArguments @('--test-threads=4','--nocapture')
Check $true 'original-worker-count-and-nocapture-admitted'
foreach($bad in @('--skip','--ignored','--exact','filter','--test-threads=0')){Refuses {Assert-RustOrdinaryTestArguments @('--test-threads=4',$bad)} ('no-test-filter-or-skip-'+$bad)}
Refuses {Assert-RustOrdinaryTestArguments @('--test-threads=4','--test-threads=8')} 'duplicate-worker-override-refused'
Check ((Get-RustOrdinarySuiteExit @(0,0,0) 3) -eq 0) 'complete-all-zero-exits-pass'
Check ((Get-RustOrdinarySuiteExit @(101,0,7) 3) -eq 101) 'first-original-test-failure-exit-preserved'
Check ((Get-RustOrdinarySuiteExit @(0,7,101) 3) -eq 7) 'later-original-test-failure-exit-preserved'
Refuses {Get-RustOrdinarySuiteExit @() 0} 'empty-suite-never-green'
Refuses {Get-RustOrdinarySuiteExit @(0) 2} 'partial-suite-never-green'
Refuses {Get-RustOrdinarySuiteExit @(0,$null) 2} 'missing-native-exit-never-cast-to-success'
Refuses {Get-RustOrdinarySuiteExit @('0') 1} 'textual-exit-is-not-native-integer-authority'
$metadata=@{packages=@(@{id='workspace#lib';manifest_path='C:\code\apps\desktop\tauri\Cargo.toml'},@{id='workspace#other';manifest_path='C:\code\tools\Cargo.toml'})}
$messages=@(
 @{reason='compiler-artifact';package_id='workspace#lib';target=@{name='opencodex_desktop_lib';kind=@('lib')};profile=@{test=$true};executable='C:\code\target\debug\deps\lib-exact.exe'},
 @{reason='compiler-artifact';package_id='workspace#other';target=@{name='tool';kind=@('bin')};profile=@{test=$true};executable='C:\code\target\debug\deps\tool-exact.exe'},
 @{reason='compiler-artifact';package_id='workspace#lib';target=@{name='not-test';kind=@('bin')};profile=@{test=$false};executable='C:\code\target\debug\not-a-test.exe'},
 @{reason='compiler-artifact';package_id='workspace#lib';target=@{name='no-exe';kind=@('lib')};profile=@{test=$true};executable=$null},
 @{reason='build-finished';success=$true}
)
$lines=@($messages|ForEach-Object {$_|ConvertTo-Json -Depth 6 -Compress})
$artifacts=@(Get-RustOrdinaryArtifacts $lines $metadata)
Check ($artifacts.Count -eq 2 -and $artifacts[0].executable -ceq $messages[0].executable -and $artifacts[1].executable -ceq $messages[1].executable) 'exact-all-Cargo-test-artifacts-multiple-packages-no-latest-glob'
Check ((@(Get-RustOrdinaryArtifacts ($lines+@($lines[0])) $metadata)).Count -eq 2) 'repeated-identical-artifact-executed-once'
Refuses {Get-RustOrdinaryArtifacts @('{broken') $metadata} 'malformed-build-JSON-fails-closed'
Refuses {Get-RustOrdinaryArtifacts @('{"reason":"build-finished","success":true}') $metadata} 'zero-test-artifacts-never-pass'
$foreign=$messages[0].Clone();$foreign.package_id='unlisted'
Refuses {Get-RustOrdinaryArtifacts @($foreign|ConvertTo-Json -Depth 5 -Compress) $metadata} 'foreign-package-test-executable-refused'
Check ($main.Contains('Get-FileHash -LiteralPath $artifact.executable') -and $main.Contains('frozen-artifacts.json') -and $helper.Contains('Test executable/DLL changed after parent no-run compilation')) 'exact-executable-DLL-hashes-and-source-commit-frozen'
Check ($helper.Contains('CreateProcessW') -and $helper.Contains('0x08000004') -and $helper.IndexOf('Write-TwoUserJson $LedgerPath') -lt $helper.IndexOf('$pending.Resume()')) 'test-process-native-identity-persisted-before-resume'
Check ($helper.Contains('$native=$pending.Observe()') -and $helper.Contains('OpenProcessToken(native,8') -and $helper.Contains('GetTokenInformation(token,1,buffer') -and $helper.Contains('Suspended native process TokenUser/image/creation differs')) 'actual-ordinary-token-image-SID-creation-required'
Check ($helper.Contains('new SafeProcessHandle(process,true)') -and $helper.Contains('new SafeWaitHandle(primaryThread,true)') -and $helper.Contains('new OcxRustOrdinaryPending(info.process,info.thread)')) 'CreateProcess-original-process-and-thread-handles-retained-with-safe-ownership'
Check ($helper.Contains('if(!resumed&&!native.IsClosed&&!native.IsInvalid)') -and $helper.Contains('TerminateProcess(native,1)') -and !$helper.Contains('$p.Kill()')) 'failed-suspended-launch-cleaned-only-through-unique-created-handle'
Check ($helper.Contains('sidProvided=$false') -and $helper.Contains('imageProvided=$false') -and $helper.Contains('creationProvided=$false') -and $helper.Contains('Supplementary CIM contradicts retained created-process identity')) 'CIM-missing-is-explicit-not-agreement-and-contradiction-is-fatal'
$nativeFixture=[OcxRustOrdinaryIdentity]::new()
$nativeFixture.ProcessId=42;$nativeFixture.Sid='S-1-5-21-1-2-3-1001';$nativeFixture.Executable='C:\fixture\test.exe'
$nativeFixture.CreationTimeUtc=[datetime]::new(638000000000000009,[DateTimeKind]::Utc)
Check ((@([OcxRustOrdinaryLaunch]::CimContradictions($nativeFixture,$null,$null,$null,$null))).Count -eq 0) 'supplementary-CIM-all-absent-is-no-contradiction-not-a-proof'
Check ((@([OcxRustOrdinaryLaunch]::CimContradictions($nativeFixture,42,$nativeFixture.Sid,'c:\fixture\TEST.EXE',638000000000000000))).Count -eq 0) 'CIM-microsecond-truncation-nine-100ns-units-accepted'
Check ((@([OcxRustOrdinaryLaunch]::CimContradictions($nativeFixture,42,$nativeFixture.Sid,$nativeFixture.Executable,$nativeFixture.CreationTimeUtc.Ticks))).Count -eq 0) 'CIM-exact-values-accepted-as-supplementary'
foreach($entry in @(
 @{name='pid';pid=43;sid=$null;image=$null;ticks=$null},
 @{name='sid';pid=$null;sid='S-1-5-21-1-2-3-1002';image=$null;ticks=$null},
 @{name='image';pid=$null;sid=$null;image='C:\fixture\other.exe';ticks=$null},
 @{name='creation';pid=$null;sid=$null;image=$null;ticks=637999999999999999},
 @{name='creation';pid=$null;sid=$null;image=$null;ticks=638000000000000010}
 )){
 $different=@([OcxRustOrdinaryLaunch]::CimContradictions($nativeFixture,$entry.pid,$entry.sid,$entry.image,$entry.ticks))
 Check ($different.Count -eq 1 -and $different[0] -ceq $entry.name) ('CIM-supplied-contradiction-refused-'+$entry.name+'-'+$entry.ticks)
}
Check ($helper.Contains("+'.launch.json')") -and $helper.Contains("identityAuthority='retained-CreateProcess-handle'")) 'bounded-kernel-CIM-launch-diagnostic-persisted-before-error-propagation'
Check ($helper.Contains('Actual token Profile TEMP/KnownFolder differs') -and $helper.Contains("Join-Path $"+'c.profile '+"'AppData/Local'") -and $helper.Contains('Fresh std-style fixture owner is not the ordinary TokenUser')) 'own-registered-Profile-and-LocalAppData-TEMP-with-owner-readback'
Check ($helper.Contains('$identity.Owner.Value') -and $helper.Contains('no owner repair is permitted')) 'TokenOwner-is-evidence-never-default-owner-mutation'
Check ($helper.Contains('Deliberately continue every Cargo-emitted test binary') -and $helper.Contains('$exitCode=[int]$result.exitCode')) 'failed-binary-exit-kept-and-rest-of-suite-continues'
Check ($main.Contains('must-run-independently-in-parent-always-path') -and $main.Contains('docTestsExecuted=$false')) 'admin-specialized-groups-and-doc-scope-not-claimed-by-binary-suite'
Check ($helper.Contains('InheritanceFlags]::None') -and !($main -match '(?i)icacls[^\r\n]* /T(?:\s|$)') -and !$helper.Contains('SetOwner')) 'read-ACL-exact-artifacts-source-only-no-recursive-repair-or-owner-change'
Check ($main.Contains('$taskChanges.Count-1') -and $main.Contains('$taskAccess.Dispose()') -and $main.Contains('sourceOrArtifactAclRestored')) 'exact-source-ACL-and-desktop-grants-restored'
Check ($main.Contains('Get-LocalUser -SID') -and $main.Contains('$profile.LocalPath -ine $taskProfile') -and $main.Contains('remainingNewSidProcesses') -and $main.Contains('if($untracked.Count){$taskRetain=$true}')) 'unknown-new-SID-process-retains-account-not-killed'
Check ($helper.Contains('Stop-TwoUserTracked $fact') -and !$helper.Contains('.Kill($true)') -and !$main.Contains('.Kill($true)')) 'cleanup-reuses-only-exact-PID-creation-image-SID-helper'
foreach($forbidden in @('Stop-Process -Name','SetTokenInformation','TakeOwn','/setowner','Set-Service','New-NetFirewallRule','Set-DnsClient','Invoke-WebRequest')){Check (!$main.Contains($forbidden) -and !$helper.Contains($forbidden)) ('no-'+$forbidden)}
Check ($helper.Contains('accessSddl=$acl.GetSecurityDescriptorSddlForm') -and $helper.Contains('Exact original read-ACL restoration readback differs') -and $main.Contains('Restore-RustOrdinaryReadAccess $taskChanges[$i]')) 'original-DACL-snapshot-is-immutable-string-and-restored-with-readback'
# Actual restoration function with a pure ACL fixture: no OS ACL calls on Mac.
$oldGetAcl=Get-Item Function:Get-Acl -ErrorAction SilentlyContinue
$oldSetAcl=Get-Item Function:Set-Acl -ErrorAction SilentlyContinue
$aclState=@{sddl='temporary-grant';sets=0;readbackOverride=$null;setFails=$false}
$fakeAcl=[pscustomobject]@{}
$fakeAcl|Add-Member ScriptMethod SetSecurityDescriptorSddlForm {param($sddl,$section) $aclState.sddl=$sddl}
$fakeAcl|Add-Member ScriptMethod GetSecurityDescriptorSddlForm {param($section) if($aclState.readbackOverride){$aclState.readbackOverride}else{$aclState.sddl}}
try{
 function Get-Acl {param($LiteralPath) $fakeAcl}
 function Set-Acl {param($LiteralPath,$AclObject) $aclState.sets++;if($aclState.setFails){throw 'Fixture write-access failure'}}
 $change=@{path='fixture-only';accessSddl='original-access'}
 Restore-RustOrdinaryReadAccess $change
 Check ($aclState.sddl -ceq 'original-access' -and $aclState.sets -eq 1) 'exact-DACL-restoration-pure-fixture'
 Check ($change.aclRestoreDiagnostic.stage -ceq 'verified' -and $change.aclRestoreDiagnostic.readbackDiffers -eq $false) 'ACL-diagnostic-equal-readback-remains-verified'
 $aclState.readbackOverride='different-ordered-ACE-or-control-flags'
 $change=@{path='fixture-only';accessSddl='original-access'}
 $caught=$null;try{Restore-RustOrdinaryReadAccess $change}catch{$caught=$_}
 Check ($null -ne $caught -and $change.aclRestoreDiagnostic.stage -ceq 'compare' -and $change.aclRestoreDiagnostic.readbackDiffers -eq $true) 'ACL-drift-still-fails-no-semantic-bypass'
 $diagnostic=$caught.Exception.Data['aclRestoreDiagnostic']
 Check ($diagnostic.originalAccessSddl -ceq 'original-access' -and $diagnostic.readbackAccessSddl -ceq $aclState.readbackOverride -and $diagnostic.path -ceq 'fixture-only') 'ACL-error-retains-exact-original-and-readback-DACL-only'
 $aclState.readbackOverride=$null;$aclState.setFails=$true
 $change=@{path='fixture-only';accessSddl='original-access'}
 $caught=$null;try{Restore-RustOrdinaryReadAccess $change}catch{$caught=$_}
 Check ($null -ne $caught -and $change.aclRestoreDiagnostic.stage -ceq 'write-access' -and $null -eq $change.aclRestoreDiagnostic.readbackAccessSddl) 'ACL-write-failure-distinguished-from-readback-drift'
 # Actual preflight4 DACL; fixture only, never apply it to any real object.
 $aclState.setFails=$false
 $actualOriginal='D:(A;;FA;;;BA)(A;OICIID;FA;;;BA)(A;OICIID;FA;;;SY)(A;OICIIOID;GA;;;CO)(A;OICIID;0x1200a9;;;BU)(A;CIID;LC;;;BU)(A;CIID;DC;;;BU)'
 $actualReadback='D:AI'+$actualOriginal.Substring(2)
 $aclState.readbackOverride=$actualReadback
 $change=@{path='preflight4-DACL-fixture-only';accessSddl=$actualOriginal}
 Restore-RustOrdinaryReadAccess $change
 Check ($change.aclRestoreDiagnostic.restorationKind -ceq 'normalized-auto-inherited-only' -and $change.aclRestoreDiagnostic.exact -eq $false -and $change.aclRestoreDiagnostic.readbackDiffers -eq $true) 'actual-preflight4-only-AI-addition-reported-normalized-never-exact'
 Check ($change.aclRestoreDiagnostic.orderedAceTextUnchanged -eq $true -and $change.aclRestoreDiagnostic.protectionUnchanged -eq $true -and [int][Security.AccessControl.ControlFlags]::DiscretionaryAclAutoInherited -eq 0x0400) 'AI-is-specific-control-0400-and-all-ordered-ACE-text-retained'
 Check ($change.aclRestoreDiagnostic.nativeDescriptorVerified -eq [bool]$IsWindows) 'native-RawSecurityDescriptor-verification-only-on-Windows-not-claimed-on-Mac'
 $counterexamples=@(
  @{name='protection-added';original=$actualOriginal;readback=$actualReadback.Replace('D:AI','D:PAI')},
  @{name='protection-removed';original=$actualOriginal.Replace('D:','D:P');readback=$actualReadback},
  @{name='auto-inherit-request-added';original=$actualOriginal;readback=$actualReadback.Replace('D:AI','D:ARAI')},
  @{name='AI-removed';original=$actualReadback;readback=$actualOriginal},
  @{name='OI-removed';original=$actualOriginal;readback=$actualReadback.Replace('OICIID','CIID')},
  @{name='CI-removed';original=$actualOriginal;readback=$actualReadback.Replace('OICIID','OIID')},
  @{name='IO-removed';original=$actualOriginal;readback=$actualReadback.Replace('OICIIOID','OICIID')},
  @{name='ID-removed';original=$actualOriginal;readback=$actualReadback.Replace('OICIID','OICI')},
  @{name='access-mask-changed';original=$actualOriginal;readback=$actualReadback.Replace('0x1200a9','FA')},
  @{name='SID-changed';original=$actualOriginal;readback=$actualReadback.Replace(';;;BU',';;;BA')},
  @{name='ACE-order-changed';original=$actualOriginal;readback=$actualReadback.Replace('(A;CIID;LC;;;BU)(A;CIID;DC;;;BU)','(A;CIID;DC;;;BU)(A;CIID;LC;;;BU)')},
  @{name='extra-ACE';original=$actualOriginal;readback=($actualReadback+'(A;;FA;;;WD)')}
 )
 foreach($case in $counterexamples){
  $aclState.readbackOverride=$case.readback
  $change=@{path='counterexample-only';accessSddl=$case.original}
  Refuses {Restore-RustOrdinaryReadAccess $change} ('ACL-AI-exception-refuses-'+$case.name)
  Check ($change.aclRestoreDiagnostic.restorationKind -ceq 'unverified' -and $change.aclRestoreDiagnostic.exact -eq $false) ('ACL-counterexample-never-verified-'+$case.name)
 }
}finally{
 if($oldGetAcl){Set-Item Function:Get-Acl $oldGetAcl.ScriptBlock}else{Remove-Item Function:Get-Acl}
 if($oldSetAcl){Set-Item Function:Set-Acl $oldSetAcl.ScriptBlock}else{Remove-Item Function:Set-Acl}
}
$collision=$messages[0].Clone();$collision.target=@{name='different-test';kind=@('test')}
Refuses {Get-RustOrdinaryArtifacts @($lines[0],($collision|ConvertTo-Json -Depth 5 -Compress)) $metadata} 'different-artifact-same-executable-never-silently-dropped'
# Exercise the reused exact-cleanup helper with fake process facts only.
$originalFact=(Get-Item Function:Get-TwoUserProcessFact).ScriptBlock
$priorGetProcess=Get-Item Function:Get-Process -ErrorAction SilentlyContinue
$cleanupState=@{kills=0;mode='creation'}
$fakeProcess=[pscustomobject]@{}
$fakeProcess|Add-Member ScriptMethod Kill {$cleanupState.kills++}
$fakeProcess|Add-Member ScriptMethod WaitForExit {param($ms) $true}
$fakeProcess|Add-Member ScriptMethod Dispose {}
try{
 function Get-Process {param($Id,$ErrorAction) $fakeProcess}
 function Get-TwoUserProcessFact {param($Process,$ExpectedSid)
  if($cleanupState.mode -ceq 'sid'){throw 'Fixture different SID'}
  @{pid=42;sid=$ExpectedSid;path=$(if($cleanupState.mode -ceq 'image'){'changed.exe'}else{'test.exe'});creationTicks=$(if($cleanupState.mode -ceq 'creation'){'2'}else{'1'})}
 }
 $frozen=@{pid=42;sid='S-1-5-21-1-2-3-1001';path='test.exe';creationTicks='1'}
 foreach($mode in @('creation','image','sid')){
  $cleanupState.mode=$mode;Refuses {Stop-TwoUserTracked $frozen} ('ordinary-cleanup-refuses-changed-'+$mode)
  Check ($cleanupState.kills -eq 0) ('ordinary-cleanup-zero-kills-after-'+$mode)
 }
 $cleanupState.mode='matching';Stop-TwoUserTracked $frozen
 Check ($cleanupState.kills -eq 1) 'ordinary-cleanup-stops-only-matching-frozen-fixture'
}finally{
 Set-Item Function:Get-TwoUserProcessFact $originalFact
 if($priorGetProcess){Set-Item Function:Get-Process $priorGetProcess.ScriptBlock}else{Remove-Item Function:Get-Process}
}
Assert-RustOrdinaryTestArguments @()
Check ($main.Contains('[string[]]$TestArguments=@()')) 'default-workers-unchanged-no-test-arguments'
Check ($main.Contains('-UseNewEnvironment -Environment $taskChildEnvironment') -and !$main.Contains('RUST_TEST_THREADS')) 'RunAs-new-environment-before-credentials-no-worker-override'
$hmeta=@{workspace_members=@('workspace#lib');packages=@(@{id='workspace#lib';manifest_path='fixture.toml';targets=@(@{name='lib';kind=@('lib');test=$true},@{name='build';kind=@('custom-build');test=$false})})}
$h=@(Get-RustOrdinaryHarnessAdmission $hmeta {param($p) '[package] name="fixture"'})
Check ($h.Count -eq 1) 'metadata-default-libtest-harness-admission'
Refuses {Get-RustOrdinaryHarnessAdmission $hmeta {param($p) '[lib] harness = false'}} 'custom-harness-false-explicitly-refused'
Assert-RustOrdinaryArtifactCoverage $h @(@{packageId='workspace#lib';target='lib';kinds=@('lib')})
Check $true 'all-metadata-test-targets-match-emitted-artifacts'
Refuses {Assert-RustOrdinaryArtifactCoverage $h @()} 'missing-test-artifact-never-green'
Check ($main.Contains("scope='binaries';fullWorkspacePassed="+'$false') -and $main.Contains('docTestsExecuted=$false')) 'binaries-only-no-doctest-or-fullworkspace-claim'
$oldActions=$env:GITHUB_ACTIONS
try{$env:GITHUB_ACTIONS='false';$refused=Join-Path $taskOutput 'no-native-output'
 Refuses {& $taskMain -Cargo 'none' -ManifestPath 'none' -Node 'none' -OutputDirectory $refused -ExpectedCommit ('a'*40)} 'actual-parent-guard-refuses-before-native-output'
 Check (!(Test-Path -LiteralPath $refused)) 'refused-parent-created-no-output'
 Refuses {Assert-RustOrdinaryEphemeral} 'actual-child-guard-refuses-outside-ephemeral-runner'
}finally{$env:GITHUB_ACTIONS=$oldActions}
$result=[ordered]@{schemaVersion=1;result='pass-static-only';checks=$checks.ToArray();checksPassed=$checks.Count;nativeExecuted=$false;accountsCreated=$false;rustBinariesExecuted=$false;ordinarySuitePassed=$false;files=@()}
foreach($path in @($taskMain,$taskHelper,$PSCommandPath)){$result.files+=@{path=$path;sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()}}
$result|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $taskOutput 'static-tests.json') -Encoding utf8
$result|ConvertTo-Json -Depth 10

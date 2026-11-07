# Offline parser/fixture tests ONLY. This never certifies Windows native facts.
param([Parameter(Mandatory=$true)][string]$OutputDirectory,[string]$Node='node')
$ErrorActionPreference='Stop'
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $taskOutput){throw 'Fresh offline test output required'}
New-Item -ItemType Directory -Path $taskOutput|Out-Null
$taskHelper=Join-Path $PSScriptRoot 'windows023-two-user-native-helper.ps1'
$taskMain=Join-Path $PSScriptRoot 'windows023-two-user-native.ps1'
. $taskHelper
$taskChecks=[Collections.Generic.List[object]]::new()
function Assert-Test([bool]$Condition,[string]$Name){if(!$Condition){throw ('Offline contract failed: '+$Name)};$taskChecks.Add(@{name=$Name;result='pass'})}
function Assert-Throws([scriptblock]$Action,[string]$Name){$caught=$false;try{& $Action|Out-Null}catch{$caught=$true};Assert-Test $caught $Name}
$emptyPath=Join-Path $taskOutput 'empty-ledger.json';Write-TwoUserJson $emptyPath @()
Assert-Test ((Get-Content -LiteralPath $emptyPath -Raw).Trim() -ceq '[]') 'empty-ledger-is-a-real-JSON-array-not-missing-file'
$singlePath=Join-Path $taskOutput 'one-ledger.json';Write-TwoUserJson $singlePath @(@{pid=42})
Assert-Test ((Get-Content -LiteralPath $singlePath -Raw).Trim().StartsWith('[')) 'single-ledger-retains-array-container'
foreach($file in @($taskMain,$taskHelper,$PSCommandPath)){
 $tokens=$null;$errors=$null
 $null=[Management.Automation.Language.Parser]::ParseFile($file,[ref]$tokens,[ref]$errors)
 Assert-Test ($errors.Count -eq 0) ('parse-'+[IO.Path]::GetFileName($file))
 $ast=[Management.Automation.Language.Parser]::ParseFile($file,[ref]$tokens,[ref]$errors)
 $reserved=@($ast.FindAll({param($n) $n -is [Management.Automation.Language.AssignmentStatementAst] -and $n.Left -is [Management.Automation.Language.VariableExpressionAst] -and $n.Left.VariablePath.UserPath -in @('HOME','Host','PID','PSHOME','PSScriptRoot','PSCommandPath','CODEX_HOME')},$true))
 Assert-Test ($reserved.Count -eq 0) ('no-reserved-variable-assignment-'+[IO.Path]::GetFileName($file))
}
$parts=Get-TwoUserReuseParts (Join-Path $PSScriptRoot 'windows-ordinary-startup.ps1')
Assert-Test ($parts.ProfileType.Contains('CreateProfile') -and $parts.DesktopType.Contains('IDisposable')) 'reuse-original-native-types'
Assert-Test ($parts.Bootstrap.Contains('OcxFirstLogonFolders') -and $parts.Bootstrap.Contains('ProfileList') -and $parts.Bootstrap.Contains('$c.nativeHelper -Mode Child')) 'reuse-real-SID-profile-bootstrap'
Assert-Test (!$parts.Bootstrap.Contains('& $c.script') -and !$parts.Bootstrap.Contains('windows-startup.ps1')) 'no-sandbox-startup-tail'
# Managed compilation only: do NOT call constructors or any P/Invoke method.
if(!('OcxNativeProfileProvision' -as [type])){Add-Type -TypeDefinition $parts.ProfileType}
if(!('OcxTemporaryDesktopAccess' -as [type])){Add-Type -TypeDefinition $parts.DesktopType}
Initialize-TwoUserWindowProbe
Assert-Test ([bool]('OcxNativeProfileProvision' -as [type]) -and [bool]('OcxTemporaryDesktopAccess' -as [type]) -and [bool]('OcxTwoUserWindow' -as [type])) 'native-interop-managed-compile-only'
$bootstrap=Join-Path $taskOutput 'bootstrap-fixture.ps1';$parts.Bootstrap|Set-Content -LiteralPath $bootstrap -Encoding utf8
$tokens=$null;$errors=$null;$null=[Management.Automation.Language.Parser]::ParseFile($bootstrap,[ref]$tokens,[ref]$errors)
Assert-Test ($errors.Count -eq 0) 'generated-bootstrap-parses'
$fixture=Join-Path $taskOutput 'changed-ordinary.ps1'
(Get-Content -LiteralPath (Join-Path $PSScriptRoot 'windows-ordinary-startup.ps1') -Raw).Replace(' if($c.profileOnly){exit 0}',' if($c.changedBoundary){exit 0}')|Set-Content -LiteralPath $fixture
Assert-Throws {Get-TwoUserReuseParts $fixture} 'upstream-reuse-drift-fails-closed'
$main=Get-Content -LiteralPath $taskMain -Raw;$helper=Get-Content -LiteralPath $taskHelper -Raw
 $policyPath=Join-Path $PSScriptRoot '../../.github/runtime-platforms-023.json'
 $policyBytes=Get-FileHash -LiteralPath $policyPath -Algorithm SHA256
 $policy=Read-TwoUserRuntimePolicy $policyPath
 Assert-Test ($policy.node.version -ceq '26.8.1' -and $policy.packageLock.packages.Contains('') -and $policy.packageLock.packages[''].name) 'actual-runtime-policy-preserves-empty-npm-root-key'
 Assert-Test ((Get-FileHash -LiteralPath $policyPath -Algorithm SHA256).Hash -ceq $policyBytes.Hash) 'actual-runtime-policy-reader-is-read-only'
 Assert-Test ($main.Contains('Read-TwoUserRuntimePolicy (Join-Path $taskRepo') -and (Get-Content (Join-Path $PSScriptRoot 'windows023-rust-ordinary.ps1') -Raw).Contains('Read-TwoUserRuntimePolicy (Join-Path $taskRepo')) 'both-native-controllers-consume-same-dictionary-policy-reader'
Assert-Test ($main.IndexOf('Restricted to an ephemeral Windows Actions runner') -lt $main.IndexOf('New-Item')) 'CI-guard-before-mutation'
Assert-Test ($main.IndexOf('$taskEphemeralGuard=') -lt $main.IndexOf('New-LocalUser') -and $main.Contains('ephemeralGuard=$taskEphemeralGuard') -and $helper.Contains('Controller ephemeralGuard missing')) 'ephemeralGuard-before-any-account-and-child-recheck'
Assert-Test ($helper.Contains('launch_main=$true') -and !$helper.Contains('launch_main=$false') -and $helper.Contains('auto_panel=$false')) 'isolated-CLI-restart-preserves-visible-main-window'
Assert-Test ($main.Contains('Assert-TwoUserNoReparseAncestors $taskOutput') -and $main.Contains('Assert-TwoUserNoReparseAncestors $taskCandidate')) 'controller-native-boundary-reparse-checks'
Assert-Test ($main.Contains('New-LocalUser') -and $main.Contains('-LoadUserProfile') -and $main.Contains('OcxNativeProfileProvision') -and $main.Contains('noSecondUserPermission')) 'real-users-loaded-profiles-and-explicit-blocked'
Assert-Test ($main.Contains('$taskAccess.Count-1') -and $main.Contains('Set-Acl -LiteralPath $taskOutput -AclObject $taskAclBefore') -and $main.Contains('Remove-CimInstance -InputObject $p') -and $main.Contains('Remove-LocalUser -SID $account.sid')) 'tracked-profile-account-and-ACL-cleanup'
Assert-Test ($main.Contains('Assert-TwoUserUnder $env:RUNNER_TEMP') -and $main.Contains('sealed-candidate-after.json')) 'private-runner-output-and-sealed-candidate-readback'
Assert-Test ($helper.Contains('GetOwnerSid') -and $helper.Contains('creationTicks') -and $helper.Contains('Refusing changed PID/SID/image/creation cleanup')) 'cleanup-requires-four-process-identities'
Assert-Test (!$helper.Contains('.Kill($true)') -and !$main.Contains('.Kill($true)') -and !($helper -match 'Stop-Process.+-Name') -and !($main -match 'Stop-Process.+-Name')) 'no-name-port-or-unverified-tree-kill'
foreach($forbidden in @('Set-DnsClient','New-NetFirewallRule','Set-NetFirewall','Stop-Service','Start-Service','Set-Service','Invoke-WebRequest','Start-BitsTransfer','Register-ScheduledTask')){
 Assert-Test (!$main.Contains($forbidden) -and !$helper.Contains($forbidden)) ('no-'+$forbidden)
}
Assert-Test (!$helper.Contains("Environment['OPENCODEX_SANDBOX']") -and !$helper.Contains("Environment['OPENCODEX_SANDBOX_ROOT']")) 'never-enable-sandbox-profile'
Assert-Test ($helper.Contains('same-user-status.json') -and $helper.Contains('same-user-data-root.json') -and $helper.Contains('GetNamedPipeServerProcessId')) 'actual-candidate-CLI-and-owned-pipe-server'
Assert-Test ($helper.Contains('peerBusinessData') -and $helper.Contains('access-denied-real-token') -and $helper.Contains('CurrentUser') -and $helper.Contains('synthetic-subscope-only')) 'cross-user-rejection-subscope-labels'
Assert-Test ($main.Contains('source-pinned version') -and $main.Contains('$taskNode --version')) 'source-pinned-real-node-preflight'
Assert-Test ($main.Contains('twoSidNativePassed=$false') -and $main.Contains('$taskSummary.twoSidNativePassed=$taskSummary.result -ceq')) 'native-pass-only-after-terminal-cleanup'
Assert-Test ($main.Contains('runtimeProxyPortIsolation=''not-verified''') -and $main.Contains('fullVersionGatePassed=$false')) 'no-false-complete-runtime-gate'
Assert-Test ($helper.Contains('lockContentHash=''unavailable-while-byte-range-lock-held''')) 'unavailable-lock-bytes-not-invented'
Assert-Test ($main.Contains('minimumSeconds=6') -and $main.Contains('Get-TwoUserWindow $f.manager')) 'overlap-is-live-generations-and-visible-owned-windows'
# Pure naming fixture: synthetic strings here are never used as client tokens.
$one=Get-TwoUserPipeName 'S-1-5-21-1-2-3-1001';$two=Get-TwoUserPipeName 'S-1-5-21-1-2-3-1002'
Assert-Test ($one -cne $two -and $one -match '^OpenCodeXDesktop-[0-9a-f]{24}$') 'distinct-SID-pipe-name-fixture-only'
Assert-Throws {Get-TwoUserPipeName 'same-rdp-session'} 'reject-non-SID-naming-input'
Assert-Test (Test-TwoUserSamePath 'C:\Users\test' '\\?\C:\Users\test') 'backend-extended-path-comparison-only'
Assert-Test (!(Test-TwoUserSamePath 'C:\Users\test' 'C:\Users\peer')) 'path-comparison-does-not-confuse-users'
foreach($valid in @('D:\a\repo','C:/Program Files/node/node.exe','D:\a/repo\Cargo.toml','C:\Users\用户\fixture')){
 Assert-Test (Test-TwoUserLocalSpelling $valid) ('local-path-spelling-accepted-'+$valid)
}
foreach($invalid in @('','C:relative','relative','C:\','C:/','\\server\share','\\?\C:\fixture','\\.\pipe\fixture','C:\fixture\..\peer','C:/fixture/./peer','C:\fixture\file:stream',"C:\fixture`npeer")){
 Assert-Test (!(Test-TwoUserLocalSpelling $invalid)) ('local-path-spelling-refused-'+$invalid)
}
Assert-Test ($helper.Contains('Get-NetTCPConnection -State Listen -LocalPort $app.port') -and $helper.Contains('$ledger.ContainsKey([int]$_.OwningProcess)')) 'CDP-listener-is-owned-not-just-a-port'
# Exercise actual guards on any host, never the Windows account code.
$oldActions=[Environment]::GetEnvironmentVariable('GITHUB_ACTIONS','Process')
try{
 $env:GITHUB_ACTIONS='false'
 $refused=Join-Path $taskOutput 'must-not-exist'
 Assert-Throws {& $taskMain -Executable 'not-an-exe' -Node 'not-a-node' -OutputDirectory $refused -ExpectedCommit ('a'*40)} 'actual-controller-refuses-outside-Windows-CI'
 Assert-Throws {& $taskMain -Executable 'not-an-exe' -Node 'not-a-node' -OutputDirectory $refused -ExpectedCommit ('a'*40) -VerifyRuntimePorts} 'actual-runtime-flag-controller-refuses-outside-Windows-CI'
 Assert-Test (!(Test-Path -LiteralPath $refused)) 'guard-created-no-native-output'
 Assert-Throws {Assert-TwoUserChildGuard} 'actual-child-refuses-outside-Windows-CI'
}finally{[Environment]::SetEnvironmentVariable('GITHUB_ACTIONS',$oldActions,'Process')}
# Parse the emitted driver with a real Node; do not run it against any browser.
$driver=Join-Path $taskOutput 'driver-fixture.mjs';Get-TwoUserDriverText|Set-Content -LiteralPath $driver -Encoding utf8
& $Node --check $driver
Assert-Test ($LASTEXITCODE -eq 0) 'generated-real-node-driver-syntax-only'
$tree=Join-Path $taskOutput 'tree-fixture';New-Item -ItemType Directory -Path $tree|Out-Null
Set-Content -LiteralPath (Join-Path $tree 'one.txt') -Value 'first'
$before=Get-TwoUserTreeHash $tree
Set-Content -LiteralPath (Join-Path $tree 'one.txt') -Value 'changed'
$after=Get-TwoUserTreeHash $tree
Assert-Test ($before.sha256 -cne $after.sha256 -and $after.entries.Count -eq 1) 'actual-hash-fixture-detects-change'
# Actual filesystem fixture: directory links are rejected before traversal.
$link=Join-Path $taskOutput 'tree-link'
New-Item -ItemType $(if($IsWindows){'Junction'}else{'SymbolicLink'}) -Path $link -Target $tree|Out-Null
Assert-Throws {Get-TwoUserTreeHash $link} 'tree-root-link-not-followed'
Remove-Item -LiteralPath $link
$link=Join-Path $tree 'child-link'
New-Item -ItemType $(if($IsWindows){'Junction'}else{'SymbolicLink'}) -Path $link -Target $taskOutput|Out-Null
Assert-Throws {Get-TwoUserTreeHash $tree} 'tree-child-link-not-followed'
Remove-Item -LiteralPath $link
# Controlled pure cleanup fixture: no Windows calls or actual process started.
$originalGetProcess=(Get-Item Function:Get-TwoUserProcessFact).ScriptBlock
$oldGetProcess=Get-Item Function:Get-Process -ErrorAction SilentlyContinue
$fixtureState=@{kills=0;disposes=0;mode='creation'}
$fixtureProcess=[pscustomobject]@{}
$fixtureProcess|Add-Member ScriptMethod Kill {$fixtureState.kills++}
$fixtureProcess|Add-Member ScriptMethod WaitForExit {param($ms) $true}
$fixtureProcess|Add-Member ScriptMethod Dispose {$fixtureState.disposes++}
try{
 function Get-Process {param($Id,$ErrorAction) $fixtureProcess}
 function Get-TwoUserProcessFact {param($Process,$ExpectedSid)
  if($fixtureState.mode -ceq 'sid'){throw 'Fixture SID mismatch'}
  [pscustomobject]@{pid=424242;sid=$ExpectedSid;path=$(if($fixtureState.mode -ceq 'image'){'different-image'}else{'fixture-image'});creationTicks=$(if($fixtureState.mode -ceq 'creation'){'222'}else{'111'})}
 }
 $expected=@{pid=424242;sid='S-1-5-21-1-2-3-1001';path='fixture-image';creationTicks='111'}
 foreach($which in @('creation','image','sid')){
  $fixtureState.mode=$which
  Assert-Throws {Stop-TwoUserTracked $expected} ('cleanup-rejects-changed-'+$which+'-without-kill')
  Assert-Test ($fixtureState.kills -eq 0) ('zero-kills-after-'+$which+'-refusal')
 }
 $fixtureState.mode='matching';Stop-TwoUserTracked $expected
 Assert-Test ($fixtureState.kills -eq 1 -and $fixtureState.disposes -eq 4) 'cleanup-only-matching-fixture-handle-and-disposes-all'
}finally{
 Set-Item Function:Get-TwoUserProcessFact -Value $originalGetProcess
 if($oldGetProcess){Set-Item Function:Get-Process -Value $oldGetProcess.ScriptBlock}else{Remove-Item Function:Get-Process}
}
$taskHelperSource=Get-Content -LiteralPath $taskHelper -Raw
Assert-Test ($taskHelperSource.Contains(".codex-desktop/opencodex/home")) 'actual-v2-home-leaf-is-home'
Assert-Test ($taskHelperSource.Contains("Windows UI CLI capability boundary changed")) 'windows-ui-cli-remains-disabled-with-isolated-pipe-fixture'

# Runtime-port increment: parser/contracts + pure fixtures only, never start a proxy.
$tokens=$null;$errors=$null
$mainAst=[Management.Automation.Language.Parser]::ParseFile($taskMain,[ref]$tokens,[ref]$errors)
$flag=@($mainAst.ParamBlock.Parameters|Where-Object {$_.Name.VariablePath.UserPath -ceq 'VerifyRuntimePorts'})
Assert-Test ($flag.Count -eq 1 -and $flag[0].StaticType -eq [Management.Automation.SwitchParameter] -and !$flag[0].DefaultValue) 'runtime-port-switch-is-ASCII-explicit-default-off'
Assert-Test ($main.Contains('verifyRuntimePorts=$VerifyRuntimePorts.IsPresent;runtimeRole=$index') -and $helper.Contains('Runtime ports require explicit controller and child flag') -and $main.Contains('Both account children must opt into')) 'runtime-port-flag-bound-to-both-account-children'
Assert-Test ((Get-TwoUserRuntimeSteps 1) -join ',' -ceq 'start,after-conflict,overlap,after-peer-stop,stop,complete') 'actual-A-runtime-controlled-step-order'
Assert-Test ((Get-TwoUserRuntimeSteps 2) -join ',' -ceq 'conflict,start,overlap,stop,complete') 'actual-B-runtime-controlled-step-order'
Assert-Throws {Get-TwoUserRuntimeSteps 0} 'unknown-runtime-role-refused'
foreach($requested in @($false,$true)){foreach($isolation in @($false,$true)){foreach($proof in @('not-verified','verified-runtime-ports')){foreach($terminal in @($false,$true)){
 $actual=Test-TwoUserRuntimeFullGate $requested $isolation $proof $terminal
 Assert-Test ($actual -eq ($requested -and $isolation -and $proof -ceq 'verified-runtime-ports' -and $terminal)) ('runtime-full-gate-'+$requested+'-'+$isolation+'-'+$proof+'-'+$terminal)
}}}}
Assert-Test ($helper.Contains("same(c.home,join(process.env.USERPROFILE,'.codex-desktop/opencodex/home'))") -and $helper.Contains("same(c.root,join(process.env.USERPROFILE,'.codex-desktop/desktop'))") -and $helper.Contains("open(temp,'wx',0o600)") -and $helper.Contains('await handle.sync()') -and $helper.Contains('await rename(temp,configPath)')) 'runtime-config-written-atomically-by-own-user-Node-under-real-H'
Assert-Test ($helper.Contains("runtimeInvoke('process_action',{request:{action,confirm:true}}") -and $helper.Contains('Date.now()+30000') -and $helper.Contains('Date.now()+10000')) 'runtime-public-manager-actions-and-production-start-stop-budgets'
Assert-Test ($helper.Contains('opencodex-local-management-v1') -and $helper.Contains('timingSafeEqual') -and $helper.Contains("loopbackRequest({hostname:'127.0.0.1'") -and $helper.Contains("response.status===200")) 'runtime-production-attestation-literal-loopback-only'
Assert-Test ($helper.Contains('B published an owned service record on A port') -and $helper.Contains('B claimed running on peer port') -and $helper.Contains('No explicit production port-conflict diagnostic') -and $helper.Contains('Peer action changed original service generation')) 'runtime-conflict-requires-refusal-and-original-A-generation'
Assert-Test ($helper.Contains('Original service generation changed') -and $helper.Contains('Get-WindowsProcessToken -Process $p -ExpectedExecutable $fact.path') -and $helper.Contains('Runtime PID was not observed in this fixture launch tree') -and $helper.Contains('Verify exact own PID/creation/image/SID immediately before manager stop')) 'runtime-stop-and-fallback-require-exact-owned-launch-generation'
Assert-Test ($main.Contains('runtimeCleanupUnproven') -and $main.Contains('Runtime identity unproven; retain new account/Profile') -and $helper.Contains('Existing outer cleanup may stop only exact')) 'runtime-unproven-cleanup-retains-profile-and-no-port-kill'
Assert-Test ($helper.Contains('publicRecord(r)') -and !$helper.Contains('JSON.stringify(r)') -and $helper.Contains('Raw errors can contain runtime-record material') -and $helper.Contains("diagnosticMarker:true")) 'runtime-evidence-projects-public-facts-and-controlled-errors-only'
Assert-Test ($helper.Contains("after-peer-stop") -and $helper.Contains('observedSeconds -lt 6') -and $helper.Contains('Original service PID still exists after stop') -and $main.Contains('Test-TwoUserRuntimeFullGate')) 'runtime-gate-requires-overlap-B-stop-A-survival-and-terminal-cleanup'
Assert-Test ($helper.Contains("'node/node.exe'") -and $helper.Contains("'bun/bun.exe'") -and $helper.Contains('$runtimeRoot=Split-Path -Parent $RuntimePrefix')) 'runtime-engine-identity-uses-actual-sibling-bundle-layout'
Assert-Test ($helper.Contains('publicFacts:runtimePublic') -and $helper.Contains('runtime_start_not_healthy') -and $helper.Contains('public_manager_command_rejected') -and !$helper.Contains('stack:error.stack')) 'runtime-real-failures-retain-public-status-and-controlled-codes'
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'windows-cdp.mjs') -Destination (Join-Path $taskOutput 'windows-cdp.mjs')
& $Node $driver --runtime-self-test
Assert-Test ($LASTEXITCODE -eq 0) 'actual-generated-runtime-secret-projection-proof-PID-port-binding-and-empty-config-pure-fixtures'

$result=[ordered]@{schemaVersion=1;result='pass-static-only';checks=$taskChecks.ToArray();checksPassed=$taskChecks.Count;nativeExecuted=$false;accountsCreated=$false;managerStarted=$false;twoSidNativePassed=$false;fullVersionGatePassed=$false;runtimeProxyPortIsolation='not-verified';host=$PSVersionTable.OS;powershell=$PSVersionTable.PSVersion.ToString();files=@()}
foreach($f in @($taskMain,$taskHelper,$PSCommandPath)){$result.files+=@{path=$f;sha256=(Get-FileHash -LiteralPath $f -Algorithm SHA256).Hash.ToLowerInvariant()}}
$result|ConvertTo-Json -Depth 9|Set-Content -LiteralPath (Join-Path $taskOutput 'static-tests.json') -Encoding utf8
$result|ConvertTo-Json -Depth 9


if(!$helper.Contains('[void]$info.Environment.Remove($key)') -or !$helper.Contains("Native manager child still has test overrides")){throw 'Real native child must remove override keys, not inherit empty smoke values'}

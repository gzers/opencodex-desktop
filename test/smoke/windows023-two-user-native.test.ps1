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
Assert-Test ($helper.Contains('Get-NetTCPConnection -State Listen -LocalPort $app.port') -and $helper.Contains('$ledger.ContainsKey([int]$_.OwningProcess)')) 'CDP-listener-is-owned-not-just-a-port'
# Exercise actual guards on any host, never the Windows account code.
$oldActions=[Environment]::GetEnvironmentVariable('GITHUB_ACTIONS','Process')
try{
 $env:GITHUB_ACTIONS='false'
 $refused=Join-Path $taskOutput 'must-not-exist'
 Assert-Throws {& $taskMain -Executable 'not-an-exe' -Node 'not-a-node' -OutputDirectory $refused -ExpectedCommit ('a'*40)} 'actual-controller-refuses-outside-Windows-CI'
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
$result=[ordered]@{schemaVersion=1;result='pass-static-only';checks=$taskChecks.ToArray();checksPassed=$taskChecks.Count;nativeExecuted=$false;accountsCreated=$false;managerStarted=$false;twoSidNativePassed=$false;runtimeProxyPortIsolation='not-verified';host=$PSVersionTable.OS;powershell=$PSVersionTable.PSVersion.ToString();files=@()}
foreach($f in @($taskMain,$taskHelper,$PSCommandPath)){$result.files+=@{path=$f;sha256=(Get-FileHash -LiteralPath $f -Algorithm SHA256).Hash.ToLowerInvariant()}}
$result|ConvertTo-Json -Depth 9|Set-Content -LiteralPath (Join-Path $taskOutput 'static-tests.json') -Encoding utf8
$result|ConvertTo-Json -Depth 9

# Pure AST/function fixtures. Never dot-source the guarded execution body.
param([Parameter(Mandatory=$true)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $OutputDirectory){throw 'Fresh fixture output required'}
$null=New-Item -ItemType Directory -Path $OutputDirectory
$tokens=$null;$errors=$null
$ast=[Management.Automation.Language.Parser]::ParseFile((Join-Path $PSScriptRoot 'windows023-installer-retention.ps1'),[ref]$tokens,[ref]$errors)
if($errors.Count){throw ($errors|Out-String)}
$names=@('Test-RetentionSamePath','Assert-RetentionPlainPath','Get-RetentionTree','Assert-RetentionTree','Test-RetentionLegacyRead','Assert-RetentionReadback','Assert-RetentionNoMigration','Write-RetentionJson','Invoke-RetentionInstaller','Install-RetentionPackage','Get-RetentionExpectedRoot','Get-RetentionContextBlock','Assert-RetentionCandidateMsiBlock')
foreach($name in $names){
    $node=@($ast.FindAll({param($n) $n -is [Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -ceq $name},$true))
    if($node.Count -ne 1){throw 'Pure fixture function missing/ambiguous'}
    . ([scriptblock]::Create($node[0].Extent.Text))
}
function Assert-Fixture($Condition,[string]$Message){if(!$Condition){throw $Message}}
function Assert-Refused([scriptblock]$Action){$refused=$false;try{& $Action}catch{$refused=$true};Assert-Fixture $refused 'Expected guard rejection'}
$checks=@()
Assert-Fixture (Test-RetentionSamePath '\\?\C:\Users\fixture\desktop' 'C:\Users\fixture\desktop') 'Canonical Windows local-drive path alias rejected'
$checks+='Rust canonical Windows paths compare with the same registered local path'

$tree=Join-Path $OutputDirectory 'synthetic U with spaces'
$null=New-Item -ItemType Directory -Path (Join-Path $tree 'empty')
[IO.File]::WriteAllBytes((Join-Path $tree 'seed.bin'),[byte[]]@(0,1,2,255))
$snapshot=Get-RetentionTree $tree
Assert-RetentionTree $tree $snapshot
[IO.File]::WriteAllBytes((Join-Path $tree 'seed.bin'),[byte[]]@(0,1,3,255))
Assert-Refused {Assert-RetentionTree $tree $snapshot}
[IO.File]::WriteAllBytes((Join-Path $tree 'seed.bin'),[byte[]]@(0,1,2,255))
Remove-Item -LiteralPath (Join-Path $tree 'empty')
Assert-Refused {Assert-RetentionTree $tree $snapshot}
$checks+='full U tree detects same-size byte changes and empty directory loss'
$external=Join-Path $OutputDirectory 'external';$null=New-Item -ItemType Directory -Path $external
[IO.File]::WriteAllText((Join-Path $external 'untouched'),'external sentinel')
$null=New-Item -ItemType SymbolicLink -Path (Join-Path $tree 'link') -Target $external
Assert-Refused {Get-RetentionTree $tree}
Assert-Refused {Assert-RetentionPlainPath (Join-Path $tree 'link/missing')}
Assert-Fixture ([IO.File]::ReadAllText((Join-Path $external 'untouched')) -ceq 'external sentinel') 'Fixture link target changed'
$checks+='links and linked ancestors rejected without traversing target'
$taskDesktop=Join-Path $OutputDirectory 'U/desktop';$taskHome=Join-Path $OutputDirectory 'U/opencodex/home'
$taskAdmission=@{candidate=@{head=('a'*40)}}
$install=@{root=(Join-Path $OutputDirectory 'actual MSI root');exe=(Join-Path $OutputDirectory 'actual MSI root/opencodex-desktop.exe')}
$facts=@{about=@{version='0.2.23'};paths=@{managerExecutable=@{path=$install.exe};installationRoot=@{path=$install.root};desktopRoot=@{path=$taskDesktop};opencodexHome=@{path=$taskHome}};
    config=@{currentDataRoot=$taskDesktop;currentOpencodexHome=$taskHome};preferences=@{launch_with_codex=$false;cold_sync=$false;app_update_auto_check=$false}}
$fresh='startup version=0.2.23 commit='+('a'*40)
$null=Assert-RetentionReadback $facts $install $fresh
Assert-Refused {Assert-RetentionReadback $facts $install ('startup version=0.2.23 commit='+('b'*40))}
Assert-Refused {Assert-RetentionReadback $facts $install ''}
$facts.about.version='0.2.22';Assert-Refused {Assert-RetentionReadback $facts $install $fresh};$facts.about.version='0.2.23'
$facts.config.currentOpencodexHome=Join-Path $OutputDirectory 'legacy H';Assert-Refused {Assert-RetentionReadback $facts $install $fresh};$facts.config.currentOpencodexHome=$taskHome
$facts.preferences.cold_sync=$true;Assert-Refused {Assert-RetentionReadback $facts $install $fresh};$facts.preferences.cold_sync=$false
$checks+='actual version, fresh embedded SHA, H binding and no-sync preference required'
$legacy=Join-Path $OutputDirectory 'legacy'
$roots=@(@{path=$legacy})
$event=@{path=(Join-Path $legacy 'child');process=(Join-Path $OutputDirectory 'opencodex-desktop.exe');mask='0x1'}
Assert-Fixture (Test-RetentionLegacyRead $event $roots) 'Legacy ReadData was not caught'
$event.mask='0x80';Assert-Fixture (!(Test-RetentionLegacyRead $event $roots)) 'Metadata read misclassified as scan'
$event.mask='0x1';$event.path=Join-Path ($legacy+'-sibling') 'child'
Assert-Fixture (!(Test-RetentionLegacyRead $event $roots)) 'Sibling prefix was misclassified'
$event.path=Join-Path $legacy 'child';$event.process=Join-Path $OutputDirectory 'pwsh.exe'
Assert-Fixture (!(Test-RetentionLegacyRead $event $roots)) 'Harness positive control misclassified as product'
$event.process=Join-Path $install.root 'runtime/node.exe'
Assert-Fixture (Test-RetentionLegacyRead $event $roots @($install.root)) 'Installed runtime child scan was not caught'
$checks+='legacy audit catches product/runtime read/list and separates positive controls and sibling paths'
$taskOutput=Join-Path $OutputDirectory 'installer-mocks';$null=New-Item -ItemType Directory -Path $taskOutput
$taskResult=@{steps=@()};$mockStarts=0;$mockDisposed=$false;$mockExit=1603;$mockTimedOut=$false
function Start-Process {
    param($FilePath,$ArgumentList,[switch]$PassThru)
    $script:mockStarts++
    $fake=[pscustomobject]@{Id=12345;ExitCode=$mockExit}
    $fake|Add-Member ScriptMethod WaitForExit {param($Milliseconds) !$script:mockTimedOut}
    $fake|Add-Member ScriptMethod Dispose {$script:mockDisposed=$true}
    $fake
}
Assert-Refused {Invoke-RetentionInstaller 'synthetic-never-executed.exe' '/S' 'synthetic-failure'}
Assert-Fixture ($mockStarts -eq 1 -and $mockDisposed -and $taskResult.steps[0].exitCode -eq 1603) 'Original installer failure lost or retried'
$mockTimedOut=$true;$mockDisposed=$false
Assert-Refused {Invoke-RetentionInstaller 'synthetic-never-executed.exe' '/S' 'synthetic-timeout'}
Assert-Fixture ($mockStarts -eq 2 -and $mockDisposed -and $taskResult.steps[1].timedOut) 'Installer timeout lost or retried'
$checks+='mocked installer original failure/timeout retained with no retry'
function Invoke-RetentionInstaller {param($File,$Arguments,$Phase) $script:mockArguments=$Arguments;$script:mockFile=$File}
$taskAdmission.kind='nsis';Install-RetentionPackage 'synthetic-candidate-setup.exe' 'mock-default-nsis'
Assert-Fixture ($mockArguments -ceq '/S' -and $mockFile -ceq 'synthetic-candidate-setup.exe') 'NSIS default path overridden'
$taskAdmission.kind='msi';Install-RetentionPackage 'synthetic-candidate.msi' 'mock-default-msi'
Assert-Fixture ($mockArguments -notmatch '(?i)INSTALLDIR|TARGETDIR|/D=' -and $mockArguments.Contains('/qn /norestart')) 'MSI default path overridden'
$checks+='both installer commands preserve real defaults without directory override'
$taskUser=Join-Path $OutputDirectory 'migration-U';$null=New-Item -ItemType Directory -Path $taskUser
$oldSource=Join-Path $OutputDirectory 'old-synthetic-data';[IO.File]::WriteAllText($oldSource,'unique synthetic legacy bytes')
$taskSeeds=@(@{path=$oldSource;sha256=(Get-FileHash -LiteralPath $oldSource).Hash.ToLowerInvariant()})
Assert-RetentionNoMigration 'synthetic-no-migration'
$copy=Join-Path $taskUser 'copied-old-seed';Copy-Item -LiteralPath $oldSource -Destination $copy
Assert-Refused {Assert-RetentionNoMigration 'synthetic-forbidden-migration'}
$checks+='legacy payload copied into U/H is rejected'
$fixtureLocal=Join-Path $OutputDirectory 'profile/AppData/Local';$fixtureMachine=Join-Path $OutputDirectory 'Program Files'
foreach($kind in @('nsis','msi')){
    Assert-Fixture (Test-RetentionSamePath (Get-RetentionExpectedRoot $kind '0.2.23' $fixtureLocal $fixtureMachine) (Join-Path $fixtureLocal 'Programs/OpenCodeX Desktop')) '023 default differs from user Programs contract'
}
Assert-Fixture (Test-RetentionSamePath (Get-RetentionExpectedRoot 'nsis' '0.2.22' $fixtureLocal $fixtureMachine) (Join-Path $fixtureLocal 'OpenCodeX Desktop')) '022 NSIS actual default changed'
Assert-Fixture (Test-RetentionSamePath (Get-RetentionExpectedRoot 'msi' '0.2.22' $fixtureLocal $fixtureMachine) (Join-Path $fixtureMachine 'OpenCodeX Desktop')) '022 MSI actual default changed'
Assert-Refused {Get-RetentionExpectedRoot 'msi' '0.1.9' $fixtureLocal $fixtureMachine}
$registration=@{hive='LocalMachine';key='{C5476FDD-CB86-46AF-A97F-96DD56A32CA0}'}
$block=Get-RetentionContextBlock 'msi' 'adjacent' $registration
Assert-Fixture ($block.automaticReplacementSupported -eq $false -and $block.candidateRejectionVerified -eq $false) 'Harness description falsely certifies actual product rejection'
Assert-Fixture ($null -ne (Get-RetentionContextBlock 'msi' 'explicit-conversion' $registration)) 'Conversion must prove candidate rejection first'
Assert-Fixture ($null -eq (Get-RetentionContextBlock 'msi' 'fresh023' $null)) 'Fresh 023 installation was blocked'
$checks+='023 uses current-user Programs; 022 keeps actual defaults; MSI context classification never certifies product rejection'
# Mock ONLY the installer process. Execute the real rejection-verifier function
# against temporary bytes to prove its command and evidence requirements.
$taskOutput=Join-Path $OutputDirectory 'block-fixture';$null=New-Item -ItemType Directory -Path $taskOutput
$taskLocal=$fixtureLocal;$taskAdmission=@{candidatePackage='synthetic-candidate.msi'}
$baselineRoot=Join-Path $OutputDirectory '022-program';$null=New-Item -ItemType Directory -Path $baselineRoot
[IO.File]::WriteAllText((Join-Path $baselineRoot 'opencodex-desktop.exe'),'never executable synthetic baseline')
$baseline=@{root=$baselineRoot;sha256=(Get-FileHash -LiteralPath (Join-Path $baselineRoot 'opencodex-desktop.exe')).Hash.ToLowerInvariant()}
$script:blockMockMessage='OCX023_MACHINE022_BLOCK: synthetic product response';$script:blockWrites=$false;$script:blockInvocations=0
function Invoke-RetentionInstaller {
    param($File,$Arguments,$Phase,$ExpectedExitCodes)
    Assert-Fixture ($ExpectedExitCodes.Count -eq 1 -and $ExpectedExitCodes[0] -eq 1603) 'Product block must require exact MSI failure code'
    Assert-Fixture ($Arguments.Contains('/i "synthetic-candidate.msi"') -and $Arguments -notmatch '(?i)INSTALLDIR|TARGETDIR|/D=') 'Block verifier did not execute exact candidate defaults'
    $script:blockInvocations++
    [IO.File]::WriteAllText((Join-Path $taskOutput 'candidate-machine-context-block.log'),$script:blockMockMessage)
    if($script:blockWrites){[IO.File]::WriteAllText((Join-Path $baseline.root 'changed'),'candidate mutation')}
}
function Assert-RetentionSeeds {param($Phase)}
function Find-RetentionInstall {param($Version) $baseline}
Assert-RetentionCandidateMsiBlock $baseline
Assert-Fixture ($blockInvocations -eq 1) 'Candidate block process was not invoked'
$script:blockMockMessage='generic MSI failure';Assert-Refused {Assert-RetentionCandidateMsiBlock $baseline}
$script:blockMockMessage='OCX023_MACHINE022_BLOCK:';$script:blockWrites=$true;Assert-Refused {Assert-RetentionCandidateMsiBlock $baseline}
$checks+='real block-verifier invokes candidate defaults, rejects generic MSI failures and detects old program mutations'
@{result='pass-pure-fixtures';checks=$checks;installerExecuted=$false;windowsAcceptancePassed=$false;source='AST of windows023-installer-retention.ps1'}|
    ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $OutputDirectory 'fixture-receipt.json') -Encoding utf8
$checks

# Task-local execution method, not a product implementation or CI substitute.
param([Parameter(Mandatory)][string]$RunDirectory,[switch]$Execute)
$ErrorActionPreference='Stop'
$taskBoundary='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop\test\runtime'
$taskRun=[IO.Path]::GetFullPath($RunDirectory)
if(!$taskRun.StartsWith($taskBoundary+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned installer runtime required'}
if(!(Test-Path -LiteralPath $taskRun -PathType Container)){throw 'Existing prepared runtime required'}
function Assert-OrdinaryPath([string]$Path){
    $taskFull=[IO.Path]::GetFullPath($Path)
    if($taskFull -cne $Path){throw 'Canonical absolute path required'}
    $taskCursor=$taskFull
    while($taskCursor){
        if(Test-Path -LiteralPath $taskCursor){
            if((Get-Item -LiteralPath $taskCursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Path alias refused'}
        }
        $taskCursor=[IO.Path]::GetDirectoryName($taskCursor)
    }
}
Assert-OrdinaryPath $taskRun
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskPackages=@{
    baseline=@{path=(Join-Path $taskCode 'test/software/OpenCodeX.Desktop_0.1.9_x64_en-US.msi')
        sha256='b33dd0c890511467e72a453daaceb4b60b6f5af64dcae5dba54f69ed46bbad6a';version='0.1.9'
        productCode='{7C063778-E6A6-4D0E-9770-4C4349101866}'
        installedExeSha256='626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608'}
    candidate=@{path=(Join-Path $taskCode 'test/software/candidate-823a8a-38841117656/installers/apps/desktop/tauri/target/release/bundle/msi/OpenCodeX Desktop_0.1.10_x64_en-US.msi')
        sha256='e64491715bc5e510d06eb8a18f723b6700be800e4555a44c4e2207059bf0318c';version='0.1.10'
        productCode='{FF3E0F1E-C4FD-49F5-8AE0-CA6F410DA8AF}'
        installedExeSha256='2614489ff3a7c62110fb74fd288432b37c3abeaa5cdaa3119551d66c97640e53'}
}
$taskPriorCode='{7069DA97-F262-4585-98CA-AE416E6833A5}'
$taskPriorLocation='D:\Software\Develop\OpenCodeX Desktop'
$taskPriorExe=Join-Path $taskPriorLocation 'opencodex-desktop.exe'
$taskInstall=Join-Path $taskRun 'install with spaces'
$taskCustom=Join-Path $taskRun 'custom-root'
$taskInstalled=Join-Path $taskInstall 'opencodex-desktop.exe'
foreach($taskPath in @($taskInstall,$taskCustom,$taskPriorLocation)){Assert-OrdinaryPath $taskPath}
function Assert-NoNative {
    if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Require native0; never stop a product here'}
}
Assert-NoNative
foreach($taskPackage in $taskPackages.Values){
    Assert-OrdinaryPath $taskPackage.path
    if((Get-FileHash -LiteralPath $taskPackage.path).Hash.ToLowerInvariant() -cne $taskPackage.sha256){throw 'Pinned package changed'}
}
$taskCandidateEvidence='D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261016-WIN-823-CANDIDATE\verification.json'
if((Get-FileHash -LiteralPath $taskCandidateEvidence).Hash.ToLowerInvariant() -cne 'b35c9820862f79a85e2d8b9696fd97efc713ff25826ba58d98fbde33932edeef'){throw '823 candidate admission differs'}
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
$taskHarness=Get-WindowsHarnessToken
$taskPlan=[ordered]@{
    source='823a8a0615153068594daef530c2e80f1d95fc29';packages=$taskPackages
    priorLocalProductCode=$taskPriorCode;priorInstallLocation=$taskPriorLocation
    priorLocalExeSha256='43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67'
    priorCachedPackage='C:\WINDOWS\Installer\6dfb1063.msi'
    priorCachedPackageSha256='1a5a894863904970e835ce7e425ccc74aa9ae0f0810c88ed1cd0b23080a8aa81'
    installDirectory=$taskInstall;customRootFixture=$taskCustom
    steps=@('uninstall-existing-local-019','install-official-019','upgrade-to-823','default-uninstall-823','reinstall-823','final-default-uninstall-823')
    argumentsCommon='/qn /norestart AUTOLAUNCHAPP=""';maximumSecondsPerStep=120
    expectedInstallSuccessExitCode=0;automaticRetry=$false;productLaunchAllowed=$false
    evidenceScope='host MSI bytes/upgrade/default-uninstall retention with seeded fixtures; no application root/first-screen acceptance'
    seededCustomRootIsNotAnApplicationBinding=$true;explicitPurgeTested=$false
    administratorAndNewHumanWindowRequiredForExecute=$true;releaseAccepted=$false
}
$taskPlanPath=Join-Path $taskRun 'execution-plan.json'
if(!$Execute){
    if(Test-Path -LiteralPath $taskPlanPath){throw 'Preserve earlier execution plan'}
    $taskPlan | ConvertTo-Json -Depth 9 | Set-Content -LiteralPath $taskPlanPath -Encoding utf8
    [ordered]@{status='prepared-only';actualHarnessToken=$taskHarness;administratorRequired=$true
        productStarted=$false;installerExecuted=$false;taskConfigurationDirectoriesChanged=$false
        existingLocal019IsOfficialBaseline=$false;fullInstallerGatePassed=$false} | ConvertTo-Json -Depth 6
    return
}
# Reject before the marker or any installer action when elevation/window is absent.
Assert-WindowsProcessMode -Observation $taskHarness -Mode administrator
$taskSavedJson=Get-Content -LiteralPath $taskPlanPath -Raw -Encoding utf8
$taskCurrentJson=$taskPlan | ConvertTo-Json -Depth 9 -Compress
# Object member order is irrelevant; values, array order and all fields must match.
if(![System.Text.Json.Nodes.JsonNode]::DeepEquals(
    [System.Text.Json.Nodes.JsonNode]::Parse($taskSavedJson),
    [System.Text.Json.Nodes.JsonNode]::Parse($taskCurrentJson))){throw 'Prepared installer plan drifted'}
function Assert-HumanWindow {
    $taskHuman=Get-Content -LiteralPath (Join-Path $taskRun 'human-window.json') -Raw -Encoding utf8 | ConvertFrom-Json
    $taskNow=[datetimeoffset]::UtcNow
    if($taskHuman.explicitHumanStart -ne $true -or $taskHuman.scope -cne 'one-host-msi-retention-execution' -or
       $taskHuman.automaticContinuation -ne $false -or !$taskHuman.userMessage -or
       ([datetimeoffset]$taskHuman.authorizedAtUtc) -gt $taskNow -or
       ([datetimeoffset]$taskHuman.expiresAtUtc) -le $taskNow -or
       (([datetimeoffset]$taskHuman.expiresAtUtc)-([datetimeoffset]$taskHuman.authorizedAtUtc)).TotalMinutes -gt 30){
        throw 'New current explicit human window required'
    }
}
Assert-HumanWindow
. 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE\methods\process-user-sid.ps1'
$taskSelf=[Diagnostics.Process]::GetCurrentProcess()
try {$taskSid=Get-AcceptanceProcessSidEvidence -Process $taskSelf -ExpectedExecutable $taskSelf.MainModule.FileName}
finally {$taskSelf.Dispose()}
if($taskSid.sidSha256 -cne '373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310'){throw 'Administrator token user differs'}
if(Test-Path -LiteralPath (Join-Path $taskRun 'execution-attempt.json')){throw 'Execution already attempted; inspect it instead of repeating'}
if((Test-Path -LiteralPath $taskInstall) -or (Test-Path -LiteralPath $taskCustom)){throw 'Execution fixture must be fresh'}
$taskPriorKey='HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\'+$taskPriorCode
$taskPrior=Get-ItemProperty -LiteralPath $taskPriorKey
if($taskPrior.DisplayName -cne 'OpenCodeX Desktop' -or $taskPrior.DisplayVersion -cne '0.1.9' -or
   $taskPrior.InstallLocation.TrimEnd('\') -cne $taskPriorLocation -or
   (Get-FileHash -LiteralPath $taskPriorExe).Hash.ToLowerInvariant() -cne $taskPlan.priorLocalExeSha256 -or
   (Get-FileHash -LiteralPath $taskPlan.priorCachedPackage).Hash.ToLowerInvariant() -cne $taskPlan.priorCachedPackageSha256){
    throw 'Prior local installation differs; preserve it'
}
foreach($taskPackage in $taskPackages.Values){
    if(Test-Path -LiteralPath ('HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\'+$taskPackage.productCode)){
        throw 'Another admitted package is already installed; inspect before mutation'
    }
}
$taskResult=[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');result='fail'
    planSha256=(Get-FileHash -LiteralPath $taskPlanPath).Hash.ToLowerInvariant()
    actualAdministratorToken=$taskSid;steps=@();productLaunched=$false;explicitDataPurgeTested=$false
    customRootIsSeededSiblingNotApplicationBinding=$true;fullInstallerGatePassed=$false;releaseAccepted=$false}
$taskResult | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskRun 'execution-attempt.json') -Encoding utf8
function Invoke-Step([string]$Name,[string]$Arguments){
    Assert-HumanWindow
    Assert-NoNative
    $taskLog=Join-Path $taskRun ($Name+'.original.log')
    $taskChild=Start-Process -FilePath "$env:SystemRoot\System32\msiexec.exe" -ArgumentList ($Arguments+' /qn /norestart AUTOLAUNCHAPP="" /L*v "'+$taskLog+'"') -WindowStyle Hidden -PassThru
    try {
        $taskTimer=[Diagnostics.Stopwatch]::StartNew()
        while(!$taskChild.WaitForExit(500)){
            Assert-NoNative
            if($taskTimer.Elapsed.TotalSeconds -ge 120){throw 'Installer observation limit reached; do not kill or retry'}
        }
        $taskExit=$taskChild.ExitCode
        $taskResult.steps+=@{name=$Name;exitCode=$taskExit;atUtc=[datetimeoffset]::UtcNow.ToString('o')}
        if($taskExit -ne 0){throw "Installer step returned $taskExit; preserve state and do not retry"}
    }finally {$taskChild.Dispose()}
    Assert-NoNative
}
function Install-Package($Package,[string]$Step){
    Invoke-Step $Step ('/i "'+$Package.path+'" INSTALLDIR="'+$taskInstall+'"')
    if((Get-Item -LiteralPath $taskInstalled).VersionInfo.ProductVersion -cne $Package.version -or
       (Get-FileHash -LiteralPath $taskInstalled).Hash.ToLowerInvariant() -cne $Package.installedExeSha256){throw 'Actual installed image differs'}
    $taskResult.steps+=@{name=$Step+'-image';version=$Package.version;sha256=$Package.installedExeSha256}
}
function Uninstall-Candidate([string]$Step){
    Invoke-Step $Step ('/x '+$taskPackages.candidate.productCode)
    if(Test-Path -LiteralPath $taskInstalled){throw 'Uninstall retained the main executable'}
}
function Assert-Retained([string]$Phase){
    foreach($taskFile in $script:taskFixtures){
        if((Get-FileHash -LiteralPath $taskFile.path).Hash.ToLowerInvariant() -cne $taskFile.sha256){throw 'Retention fixture changed'}
    }
    $taskResult.steps+=@{name=$Phase;allSeededFixturesUnchanged=$true;files=$script:taskFixtures}
}
try {
    Invoke-Step 'uninstall-existing-local-019' ('/x '+$taskPriorCode)
    if(Test-Path -LiteralPath $taskPriorExe){throw 'Prior executable still exists after uninstall'}
    Install-Package $taskPackages.baseline 'install-official-019'
    $script:taskFixtures=@()
    foreach($taskRelative in @('install with spaces/data/manager-state/preferences.json','install with spaces/data/backups/retained/backup-manifest.json','install with spaces/data/backups/retained/preferences.json','custom-root/manager-state/preferences.json','custom-root/backups/retained-payload.json','outside-root.sentinel')){
        $taskPath=[IO.Path]::GetFullPath((Join-Path $taskRun $taskRelative))
        if(!$taskPath.StartsWith($taskRun+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Fixture outside this owned run'}
        Assert-OrdinaryPath $taskPath
        $null=New-Item -ItemType Directory -Force -Path (Split-Path -Parent $taskPath)
        if(Test-Path -LiteralPath $taskPath){throw 'Never overwrite a pre-existing fixture'}
        [IO.File]::WriteAllText($taskPath,('{"hostInstallerFixture":"'+$taskRelative+'","retain":true}'),[Text.UTF8Encoding]::new($false))
        $script:taskFixtures+=@{path=$taskPath;sha256=(Get-FileHash -LiteralPath $taskPath).Hash.ToLowerInvariant();bytes=(Get-Item -LiteralPath $taskPath).Length}
    }
    Assert-Retained 'before-upgrade'
    Install-Package $taskPackages.candidate 'upgrade-to-823'
    Assert-Retained 'after-upgrade'
    Uninstall-Candidate 'default-uninstall-823'
    Assert-Retained 'after-default-uninstall'
    Install-Package $taskPackages.candidate 'reinstall-823'
    Assert-Retained 'after-reinstall'
    Uninstall-Candidate 'final-default-uninstall-823'
    Assert-Retained 'after-final-default-uninstall'
    $taskResult.result='pass'
}catch{
    $taskResult.errorType=$_.Exception.GetType().FullName
    $taskResult.error=$_.Exception.Message
    throw
}finally {
    $taskResult.finishedAtUtc=[datetimeoffset]::UtcNow.ToString('o')
    $taskResult.installerStateMustBeReadBeforeAnyNextAction=$true
    $taskResult | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $taskRun 'execution-result.json') -Encoding utf8
}

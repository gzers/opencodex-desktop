param(
    [Parameter(Mandatory=$true)][string]$AdmissionPath,
    [switch]$Execute
)
$ErrorActionPreference = 'Stop'
if (!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7 -or
    $env:GITHUB_ACTIONS -cne 'true' -or $env:GITHUB_REPOSITORY -cne 'gzers/opencodex-desktop' -or
    $env:GITHUB_EVENT_NAME -cne 'workflow_dispatch' -or $env:GITHUB_RUN_ATTEMPT -cne '1') {
    throw 'Installer retention is restricted to one fresh Windows Actions dispatch'
}
$taskRepo=(Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../..')).Path
$taskAdmission=Get-Content -LiteralPath $AdmissionPath -Raw -Encoding utf8 | ConvertFrom-Json
$taskSource=(& git -C $taskRepo rev-parse HEAD).Trim()
if ($taskSource -cne $taskAdmission.harnessSource -or
    $taskAdmission.candidateSource -cne '43142bdbcc7a0b9f1325103febae3f44dce407ab' -or
    $taskAdmission.kind -cnotin @('msi','nsis')) { throw 'Installer admission identity changed' }
$taskOutput=[IO.Path]::GetFullPath($taskAdmission.outputDirectory)
if (!$taskOutput.Equals((Join-Path $taskRepo 'test/out/windows-installer-retention'),[StringComparison]::OrdinalIgnoreCase)) {
    throw 'Evidence must remain in the explicit task output'
}
foreach($taskName in @('baseline','candidate')) {
    $taskPackage=(Resolve-Path -LiteralPath $taskAdmission.($taskName+'Package')).Path
    if (!(Split-Path -Parent $taskPackage).Equals((Join-Path $taskRepo 'test/software/installer-retention'),[StringComparison]::OrdinalIgnoreCase)) {
        throw 'Only admitted task-owned installers may run'
    }
    if ((Get-Item -LiteralPath $taskPackage).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Installer link refused' }
    if ((Get-FileHash -LiteralPath $taskPackage).Hash.ToLowerInvariant() -cne $taskAdmission.($taskName+'Sha256')) {
        throw 'Installer digest changed'
    }
}
if (@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count) {
    throw 'Native process exists; do not alter a running product'
}
if (!$Execute) {
    [ordered]@{status='prepared-only';harnessSource=$taskSource;candidateSource=$taskAdmission.candidateSource
        kind=$taskAdmission.kind;installerExecuted=$false;applicationLaunched=$false} | ConvertTo-Json
    return
}
$taskAttempt=Join-Path $taskOutput 'execution-attempt.json'
if(Test-Path -LiteralPath $taskAttempt){throw 'Preserve the existing attempt; never retry an installer for a passing result'}
$taskBoundary=Join-Path ([IO.Path]::GetFullPath($env:RUNNER_TEMP)) ('OpenCodex-Retention-'+[guid]::NewGuid().ToString('N'))
if(Test-Path -LiteralPath $taskBoundary){throw 'Dedicated fixture boundary already exists'}
New-Item -ItemType Directory -Path $taskBoundary | Out-Null
$taskInstall=Join-Path $taskBoundary 'install with spaces'
$taskCustom=Join-Path $taskBoundary 'custom-root'
$taskSentinel=Join-Path $taskBoundary 'outside-root.sentinel'
New-Item -ItemType Directory -Path $taskCustom | Out-Null
[IO.File]::WriteAllText($taskSentinel,'unrelated sibling must remain',[Text.UTF8Encoding]::new($false))
$taskInstalled=Join-Path $taskInstall 'opencodex-desktop.exe'
$taskResult=[ordered]@{
    scope='Real ephemeral Windows installer upgrade/uninstall with seeded fixtures; no app/root-resolution/ordinary-token/UI acceptance'
    harnessSource=$taskSource;candidateSource=$taskAdmission.candidateSource;kind=$taskAdmission.kind
    fixtureBoundary=$taskBoundary;installDirectory=$taskInstall;result='fail';steps=@()
    baselinePackageSha256=$taskAdmission.baselineSha256;candidatePackageSha256=$taskAdmission.candidateSha256
    defaultDataFixtureSeeded=$true;customRootIsSeededSiblingNotApplicationBinding=$true
    productLaunched=$false;explicitDataPurgeTested=$false;hostMachineAcceptanceClaimed=$false
    fullVersionGatePassed=$false;releaseRequested=$false
}
$taskResult | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskAttempt -Encoding utf8
$taskMayBeInstalled=$false
function Invoke-OwnedInstaller([string]$Package,[string]$Arguments,[string]$Step) {
    $taskProcess=Start-Process -FilePath $Package -ArgumentList $Arguments -WindowStyle Hidden -PassThru
    if(!$taskProcess.WaitForExit(120000)){throw "Installer step timed out: $Step; do not retry"}
    $taskCode=$taskProcess.ExitCode
    $taskResult.steps += [ordered]@{name=$Step;atUtc=[datetimeoffset]::UtcNow.ToString('o');exitCode=$taskCode}
    $taskProcess.Dispose()
    if($taskCode -ne 0){throw "Installer step $Step exit $taskCode; reboot-required is not accepted"}
}
function Install-Owned([string]$Package,[string]$Step) {
    $script:taskMayBeInstalled=$true
    if($taskAdmission.kind -ceq 'msi') {
        $taskLog=Join-Path $taskOutput ($Step+'.log')
        Invoke-OwnedInstaller "$env:SystemRoot\System32\msiexec.exe" ('/i "'+$Package+'" /qn /norestart INSTALLDIR="'+$taskInstall+'" /L*v "'+$taskLog+'"') $Step
    } else { Invoke-OwnedInstaller $Package ('/S /D='+$taskInstall) $Step }
}
function Uninstall-Owned([string]$Step) {
    if($taskAdmission.kind -ceq 'msi') {
        Invoke-OwnedInstaller "$env:SystemRoot\System32\msiexec.exe" ('/x "'+$taskAdmission.candidatePackage+'" /qn /norestart /L*v "'+(Join-Path $taskOutput ($Step+'.log'))+'"') $Step
    } else {
        $taskUninstaller=Join-Path $taskInstall 'uninstall.exe'
        if(!(Test-Path -LiteralPath $taskUninstaller)){throw 'Exact installed uninstaller missing'}
        Invoke-OwnedInstaller $taskUninstaller '/S' $Step
    }
    $taskDeadline=[datetimeoffset]::UtcNow.AddSeconds(30)
    while((Test-Path -LiteralPath $taskInstalled) -and [datetimeoffset]::UtcNow -lt $taskDeadline){Start-Sleep -Milliseconds 200}
    if(Test-Path -LiteralPath $taskInstalled){throw 'Uninstall left the candidate executable'}
    $script:taskMayBeInstalled=$false
}
function Read-OwnedImage([string]$ExpectedVersion,[string]$ExpectedHash) {
    $taskItem=Get-Item -LiteralPath $taskInstalled
    if($taskItem.VersionInfo.ProductVersion -cne $ExpectedVersion){throw 'Installed image version differs'}
    $taskHash=(Get-FileHash -LiteralPath $taskInstalled).Hash.ToLowerInvariant()
    if($ExpectedHash -and $taskHash -cne $ExpectedHash){throw 'Installed image differs from admitted payload'}
    [ordered]@{version=$taskItem.VersionInfo.ProductVersion;bytes=$taskItem.Length;sha256=$taskHash}
}
function Assert-Fixtures([string]$Phase) {
    $taskObserved=@()
    foreach($taskRow in $script:taskExpectedFixtures) {
        $taskPath=Join-Path $taskBoundary $taskRow.relative
        if(!(Test-Path -LiteralPath $taskPath -PathType Leaf)){throw "Fixture missing at $Phase"}
        $taskHash=(Get-FileHash -LiteralPath $taskPath).Hash.ToLowerInvariant()
        if($taskHash -cne $taskRow.sha256 -or (Get-Item -LiteralPath $taskPath).Length -ne $taskRow.bytes){throw "Fixture changed at $Phase"}
        $taskObserved+=$taskRow
    }
    $taskResult.steps += [ordered]@{name=$Phase;atUtc=[datetimeoffset]::UtcNow.ToString('o');files=$taskObserved;allBytesUnchanged=$true}
}
try {
    Install-Owned $taskAdmission.baselinePackage 'install-official-019'
    $taskOldHash=$(if($taskAdmission.kind -ceq 'msi'){$taskAdmission.baselineMsiExeSha256}else{''})
    $taskResult.baselineImage=Read-OwnedImage '0.1.9' $taskOldHash
    $taskPaths=@(
        'install with spaces/data/manager-state/preferences.json',
        'install with spaces/data/backups/retained/backup-manifest.json',
        'install with spaces/data/backups/retained/preferences.json',
        'custom-root/manager-state/preferences.json',
        'custom-root/backups/retained-payload.json')
    foreach($taskRelative in $taskPaths) {
        $taskPath=Join-Path $taskBoundary $taskRelative
        New-Item -ItemType Directory -Path (Split-Path -Parent $taskPath) -Force | Out-Null
        [IO.File]::WriteAllText($taskPath,('{"fixture":"'+$taskRelative+'","mustRetain":true}'),[Text.UTF8Encoding]::new($false))
    }
    $script:taskExpectedFixtures=@(@($taskPaths)+@('outside-root.sentinel')|ForEach-Object {
        $taskPath=Join-Path $taskBoundary $_
        [ordered]@{relative=$_;bytes=(Get-Item -LiteralPath $taskPath).Length;sha256=(Get-FileHash -LiteralPath $taskPath).Hash.ToLowerInvariant()}
    })
    Assert-Fixtures 'before-upgrade'
    Install-Owned $taskAdmission.candidatePackage 'upgrade-to-431'
    $taskResult.candidateImage=Read-OwnedImage '0.1.10' $taskAdmission.candidateInstalledExeSha256
    Assert-Fixtures 'after-upgrade'
    Uninstall-Owned 'default-uninstall'
    Assert-Fixtures 'after-default-uninstall'
    Install-Owned $taskAdmission.candidatePackage 'reinstall-431'
    $taskResult.reinstalledImage=Read-OwnedImage '0.1.10' $taskAdmission.candidateInstalledExeSha256
    Assert-Fixtures 'after-reinstall'
    Uninstall-Owned 'final-default-uninstall'
    Assert-Fixtures 'after-final-uninstall'
    if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Unexpected product process'}
    $taskResult.result='pass'
} catch {
    $taskResult.errorType=$_.Exception.GetType().Name
    $taskResult.error=$_.Exception.Message
    throw
} finally {
    # No broad file removal or second uninstall attempt after a failed step.
    # A failed machine state is retained on this disposable runner for evidence.
    $taskResult.installerStateMayRemain=$taskMayBeInstalled
    $taskResult.finishedAtUtc=[datetimeoffset]::UtcNow.ToString('o')
    $taskResult | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $taskOutput 'retention-result.json') -Encoding utf8
}
$taskResult | ConvertTo-Json -Depth 12

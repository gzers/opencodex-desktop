param([switch]$Execute)
$ErrorActionPreference='Stop'
$taskRun=[IO.Path]::GetFullPath($PSScriptRoot)
$taskExpected='C:\Users\15119\.codex\worktrees\windows-mesh-initial-size\opencodex-desktop\test\runtime\host-nsis-20261017T1220'
if($taskRun -cne $taskExpected){throw 'Exact owned run required'}
$taskCursor=Get-Item -LiteralPath $taskRun
while($taskCursor){if($taskCursor.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Path alias refused'};$taskCursor=$taskCursor.Parent}
$taskRepo=[IO.Path]::GetFullPath((Join-Path $taskRun '../../..'))
$taskInstall=Join-Path $taskRun 'install with spaces'
$taskCustom=Join-Path $taskRun 'custom-root-seeded-only'
$taskExe=Join-Path $taskInstall 'opencodex-desktop.exe'
$taskUninstaller=Join-Path $taskInstall 'uninstall.exe'
$taskCandidate=Join-Path $taskRepo 'test/software/candidate-ce670c-38996189227/installers/apps/desktop/tauri/target/release/bundle/nsis/OpenCodeX Desktop_0.1.10_x64-setup.exe'
$taskBaseline=Join-Path $taskRun 'baseline019-setup.exe'
$taskResultPath=Join-Path $taskRun 'execution-result.json'
$taskUninstallKey='HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\OpenCodeX Desktop'
$taskController=[Diagnostics.Process]::GetCurrentProcess()
. (Join-Path $taskRepo 'test/smoke/windows-process-token.ps1')
$taskControllerToken=Get-WindowsProcessToken -Process $taskController -ExpectedExecutable $taskController.MainModule.FileName
Assert-WindowsProcessMode -Observation $taskControllerToken -Mode ordinary
$taskController.Dispose()
function Assert-NativeZero {
    if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Product process appeared; stop without kill/retry'}
}
function Read-Protection {
    $taskPointer=(Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop' -Name StartupDataRootV1).StartupDataRootV1
    [ordered]@{
        startupPointerSha256=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($taskPointer))).ToLowerInvariant()
        originalPreferencesSha256=(Get-FileHash -LiteralPath (Join-Path $env:APPDATA 'com.gzers.opencodex.desktop/manager-state/preferences.json')).Hash.ToLowerInvariant()
        boundFixturePreferencesSha256=(Get-FileHash -LiteralPath 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\foreground-02-current-20261015\install-a\data\manager-state\preferences.json').Hash.ToLowerInvariant()
    }
}
function Invoke-Step([string]$File,[string]$Arguments,[string]$Name) {
    Assert-NativeZero
    $taskRow=[ordered]@{name=$Name;startedAtUtc=[datetimeoffset]::UtcNow.ToString('o');executable=$File;arguments=$Arguments;result='running'}
    $script:taskResult.steps+=@($taskRow)
    $script:taskResult | ConvertTo-Json -Depth 9 | Set-Content -LiteralPath $taskResultPath -Encoding utf8
    $taskProcess=Start-Process -FilePath $File -ArgumentList $Arguments -WindowStyle Hidden -PassThru
    try{
        [void]$taskProcess.SafeHandle
        $taskRow.processId=$taskProcess.Id
        $taskRow.actualToken=Get-WindowsProcessToken -Process $taskProcess -ExpectedExecutable $File
        Assert-WindowsProcessMode -Observation $taskRow.actualToken -Mode ordinary
        $taskTimer=[Diagnostics.Stopwatch]::StartNew()
        while(!$taskProcess.WaitForExit(500)){Assert-NativeZero;if($taskTimer.Elapsed.TotalSeconds -gt 120){throw 'Bounded installer timeout; no retry or termination'}}
        $taskRow.exitCode=$taskProcess.ExitCode
        if($taskRow.exitCode -ne 0){throw "Installer step failed: $Name"}
        $taskRow.finishedAtUtc=[datetimeoffset]::UtcNow.ToString('o');$taskRow.result='pass'
        Assert-NativeZero
        Write-Output ($Name+': exit0')
    }finally{$taskProcess.Dispose();$script:taskResult | ConvertTo-Json -Depth 9 | Set-Content -LiteralPath $taskResultPath -Encoding utf8}
}
function Check-Image([string]$Version,[string]$Hash) {
    $taskImage=Get-Item -LiteralPath $taskExe
    $taskObservedHash=(Get-FileHash -LiteralPath $taskExe).Hash.ToLowerInvariant()
    if($taskImage.VersionInfo.ProductVersion -cne $Version -or ($Hash -and $taskObservedHash -cne $Hash)){throw 'Installed payload differs'}
    [ordered]@{version=$taskImage.VersionInfo.ProductVersion;sha256=$taskObservedHash;bytes=$taskImage.Length}
}
function Check-Seeds([string]$Phase) {
    $taskRows=@(foreach($taskSeed in $script:taskSeeds){
        $taskPath=Join-Path $taskRun $taskSeed.relative
        if((Get-FileHash -LiteralPath $taskPath).Hash.ToLowerInvariant() -cne $taskSeed.sha256){throw "Retained fixture differs: $Phase"}
        $taskSeed
    })
    $script:taskResult.steps+=@([ordered]@{name=$Phase;atUtc=[datetimeoffset]::UtcNow.ToString('o');allFixtureBytesUnchanged=$true;files=$taskRows})
}
function Uninstall-Owned([string]$Name) {
    # This exact root is below the checked runtime, and was created by this attempt.
    if(!$taskInstall.StartsWith($taskRun+'\',[StringComparison]::OrdinalIgnoreCase) -or
       (Get-Item -LiteralPath $taskInstall).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Uninstall target differs'}
    Invoke-Step $taskUninstaller '/S' $Name
    $taskTimer=[Diagnostics.Stopwatch]::StartNew()
    while((Test-Path -LiteralPath $taskExe) -or (Test-Path -LiteralPath $taskUninstallKey)){
        Assert-NativeZero
        if($taskTimer.Elapsed.TotalSeconds -gt 30){throw 'Uninstall completion not observed; no retry'}
        Start-Sleep -Milliseconds 200
    }
}
Assert-NativeZero
foreach($taskRegistryRoot in @('HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall','HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall','HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall')){
    if(@(Get-ChildItem -LiteralPath $taskRegistryRoot -ErrorAction SilentlyContinue | Get-ItemProperty | Where-Object {$_.DisplayName -like '*OpenCodeX*'}).Count){throw 'Existing registered installation; stop'}
}
if((Get-FileHash -LiteralPath $taskBaseline).Hash.ToLowerInvariant() -cne 'e2538debc1f8acf5b5ed931f3e68d9b128263742c230fbc151513f2833a38006' -or
   (Get-FileHash -LiteralPath $taskCandidate).Hash.ToLowerInvariant() -cne 'ee10b53f741ccdc5959c9360136e64d9c87fd3f73d0528446cdf7bb4a8c38ba3'){throw 'Package digest differs'}
if(!$Execute){Write-Output 'Exact NSIS packages and native0 verified; no execution';return}
if((Test-Path -LiteralPath $taskResultPath) -or (Test-Path -LiteralPath $taskInstall) -or (Test-Path -LiteralPath $taskCustom)){throw 'One declared attempt only'}
$script:taskResult=[ordered]@{source='ce670ce76bb1c29f70989ee54838d5b31e97059c';candidateRunId=38996189227;kind='nsis';scope='host silent installer bytes/019 upgrade/default-uninstall/reinstall with seeded data; no app binding/UI acceptance';controllerToken=$taskControllerToken;beforeProtection=(Read-Protection);installDirectory=$taskInstall;customSiblingIsNotAnApplicationBinding=$true;productLaunched=$false;explicitPurgeTested=$false;uiInputInjected=$false;automaticRetry=$false;result='running';steps=@();fullInstallerGatePassed=$false;releaseAccepted=$false}
$script:taskResult | ConvertTo-Json -Depth 9 | Set-Content -LiteralPath $taskResultPath -Encoding utf8
try{
    Invoke-Step $taskBaseline ('/S /NS /D='+$taskInstall) 'install-official-019'
    $script:taskResult.baselineImage=Check-Image '0.1.9' ''
    $taskRelativePaths=@('install with spaces/data/manager-state/preferences.json','install with spaces/data/backups/retained/backup-manifest.json','install with spaces/data/backups/retained/preferences.json','custom-root-seeded-only/manager-state/preferences.json','custom-root-seeded-only/backups/retained-payload.json','outside-root.sentinel')
    foreach($taskRelative in $taskRelativePaths){
        $taskPath=Join-Path $taskRun $taskRelative
        New-Item -ItemType Directory -Path (Split-Path -Parent $taskPath) -Force | Out-Null
        [IO.File]::WriteAllText($taskPath,('{"nsisRetentionFixture":"'+$taskRelative+'"}'),[Text.UTF8Encoding]::new($false))
    }
    $script:taskSeeds=@(foreach($taskRelative in $taskRelativePaths){[ordered]@{relative=$taskRelative;sha256=(Get-FileHash -LiteralPath (Join-Path $taskRun $taskRelative)).Hash.ToLowerInvariant();bytes=(Get-Item -LiteralPath (Join-Path $taskRun $taskRelative)).Length}})
    Check-Seeds 'before-upgrade'
    Invoke-Step $taskCandidate ('/S /NS /D='+$taskInstall) 'upgrade-to-ce'
    $script:taskResult.candidateImage=Check-Image '0.1.10' '059bc70d158e66863cab0aa0b828c4107806724e3b13e2b2cd337fc33ccf77a5'
    Check-Seeds 'after-upgrade'
    Uninstall-Owned 'default-uninstall-ce'
    Check-Seeds 'after-default-uninstall'
    Invoke-Step $taskCandidate ('/S /NS /D='+$taskInstall) 'reinstall-ce'
    $script:taskResult.reinstalledImage=Check-Image '0.1.10' '059bc70d158e66863cab0aa0b828c4107806724e3b13e2b2cd337fc33ccf77a5'
    Check-Seeds 'after-reinstall'
    Uninstall-Owned 'final-default-uninstall-ce'
    Check-Seeds 'after-final-default-uninstall'
    $script:taskResult.afterProtection=Read-Protection
    if(($script:taskResult.beforeProtection | ConvertTo-Json -Compress) -cne ($script:taskResult.afterProtection | ConvertTo-Json -Compress)){throw 'Original configuration changed'}
    $script:taskResult.result='pass'
}catch{$script:taskResult.errorType=$_.Exception.GetType().Name;$script:taskResult.error=$_.Exception.Message;$script:taskResult.result='fail';throw}
finally{
    $script:taskResult.finishedAtUtc=[datetimeoffset]::UtcNow.ToString('o')
    $script:taskResult.finalExecutablePresent=Test-Path -LiteralPath $taskExe
    $script:taskResult.finalUninstallEntryPresent=Test-Path -LiteralPath $taskUninstallKey
    $script:taskResult.nativeCount=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count
    $script:taskResult | ConvertTo-Json -Depth 9 | Set-Content -LiteralPath $taskResultPath -Encoding utf8
}

$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskCandidate=Get-Content -LiteralPath (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-8BE-CANDIDATE/candidate-descriptor.json') -Raw | ConvertFrom-Json
if($taskCandidate.source -cne '8be87e2d54a895d4fe02e31300100a1ffed90121' -or
    !$taskCandidate.sameShaCiAndArtifactVerified -or $taskCandidate.installerExecuted){throw 'Precise new candidate not admitted'}
$taskRuntime=Join-Path $taskCode 'test/runtime/idle-pair-8be-settled-20261016'
$taskRoot=Join-Path $taskCode 'test/runtime/foreground-02-current-20261015/install-a/data'
$taskBaseline=[IO.Path]::GetFullPath((Join-Path $env:APPDATA 'com.gzers.opencodex.desktop'))
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native product exists'}
if((Test-Path -LiteralPath $taskRuntime) -or (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'contract.json'))){throw 'New pair preparation already exists'}
$taskPointer=(Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop').StartupDataRootV1 | ConvertFrom-Json
if($taskPointer.anchor -notlike '*foreground-02-current-20261015\install-a\data'){throw 'Unexpected current anchor'}
$taskBinding=Get-Content -LiteralPath (Join-Path $taskRoot 'manager-state/data-root.json') -Raw | ConvertFrom-Json
if($taskBinding.structure_version -cne '1' -or $taskBinding.active_data_root){throw 'Pair fixture binding is no longer a standalone owned root'}
foreach($taskInventoryRoot in @($taskRoot,$taskBaseline)){
    $taskItem=Get-Item -LiteralPath $taskInventoryRoot -Force
    if(!$taskItem.PSIsContainer -or $taskItem.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Fixture root alias refused'}
    if(@(Get-ChildItem -LiteralPath $taskInventoryRoot -Recurse -Force | Where-Object {$_.Attributes -band [IO.FileAttributes]::ReparsePoint}).Count){throw 'Fixture inventory alias refused'}
    $taskRuntimeState=Get-Content -LiteralPath (Join-Path $taskInventoryRoot 'manager-state/runtime.json') -Raw | ConvertFrom-Json
    if($taskRuntimeState.source -cne 'unresolved' -or $taskRuntimeState.path){throw 'Runtime was configured since the previous owned fixture; stop before copying'}
}
New-Item -ItemType Directory -Path $taskRuntime | Out-Null
$taskPrefsPath=Join-Path $taskRoot 'manager-state/preferences.json'
$taskPrefs=Get-Content -LiteralPath $taskPrefsPath -Raw | ConvertFrom-Json
if($taskPrefs.schema_version -ne 2){throw 'Expected known current fixture preferences'}
Copy-Item -LiteralPath $taskPrefsPath -Destination (Join-Path $PSScriptRoot 'before-preferences.original.json')
$taskPrefs.appearance.theme='light';$taskPrefs.appearance.visual_effects='high'
$taskPrefs.appearance.glow_render='mesh';$taskPrefs.appearance.interface_scale=100
$taskPrefs.shell.auto_panel=$false;$taskPrefs.shell.launch_with_codex=$false
$taskPrefs.shell.keep_proxy_on_close=$false;$taskPrefs.shell.launch_main=$true
$taskPrefs.updates.app_update_auto_check=$false;$taskPrefs.maintenance.startup_cleanup=$false
$taskPrefs | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $taskPrefsPath -Encoding utf8
$taskDestination=[IO.Path]::GetFullPath((Join-Path $taskRuntime 'baseline-previous'))
if($taskBaseline -cne ([IO.Path]::GetFullPath((Join-Path $env:APPDATA 'com.gzers.opencodex.desktop'))) -or
    !$taskDestination.StartsWith($taskRuntime+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Unexpected fixture move endpoints'}
Move-Item -LiteralPath $taskBaseline -Destination $taskDestination
Copy-Item -LiteralPath $taskRoot -Destination $taskBaseline -Recurse
if((Get-FileHash -LiteralPath $taskPrefsPath).Hash -cne
    (Get-FileHash -LiteralPath (Join-Path $taskBaseline 'manager-state/preferences.json')).Hash){throw 'Paired preferences differ after cloning'}
[ordered]@{
    schemaVersion=1;scope='same-host fresh-process foreground idle subset after color-only buffer optimization'
    candidateSource=$taskCandidate.source;candidateExeSha256=$taskCandidate.sha256
    baselineSource='de6862d40389d527527b9db8ef52e186a58b625f'
    baselineExeSha256='626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608'
    order=@('baseline019','candidate0110');settleSecondsPerVersion=60;measurementSecondsPerVersion=600
    deadlineRule='last completed counter observation minus first completed counter observation >= 600 seconds'
    theme='light';visualEffects='high';glowRender='mesh';interfaceScale=100
    bothVersionsNewProcess=$true;identicalOwnedManagerFixtureClone=$true
    singleNativeInstance=$true;foregroundLossStopsAttempt=$true
    cpuUnit='one logical processor percent from retained observed process-family handles'
    rssUnit='summed live WorkingSet64, shared pages not deduplicated'
    budgetsUnchanged=@{cpuIncreasePercentagePoints=0.5;rssIncreaseMiB=10}
    shortLivedChildrenFullyCovered=$false;fullPerformanceGatePassed=$false
} | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'contract.json') -Encoding utf8
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');nativeCount=0
    sourceRoot=$taskRoot;baselineRoot=$taskBaseline;previousBaselineRetainedAt=$taskDestination
    fixturePreferencesSha256=(Get-FileHash -LiteralPath $taskPrefsPath).Hash.ToLowerInvariant()
    actualCandidateExecutable=$taskCandidate.executable;candidateSha256=$taskCandidate.sha256
    userDailySidecarsTouched=$false;knownFolderOrHomeEnvironmentRedirected=$false
    productStarted=$false;fullVersionGatePassed=$false
} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'preparation.original.json') -Encoding utf8

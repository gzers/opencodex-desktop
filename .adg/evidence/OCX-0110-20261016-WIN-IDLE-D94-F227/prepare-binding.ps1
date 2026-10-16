# Read-only preparation. Does not create a product instance or change fixtures.
$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskDiagnostic='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop'
if(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'manifest.json')){throw 'Frozen preparation'}
if(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'contract.json')){throw 'Preparation already recorded'}
$taskCandidate=Get-Content -LiteralPath (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261016-WIN-D94-CANDIDATE/candidate-descriptor.json') -Raw | ConvertFrom-Json
$taskPrevious=Get-Content -LiteralPath (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261016-WIN-IDLE-D94-019/contract.json') -Raw | ConvertFrom-Json
$taskSource=(git -c core.longpaths=true -C $taskDiagnostic rev-parse HEAD).Trim()
if($taskSource -cne 'f2271f361b9da562b62c71afcdbeb15e61d183e2'){throw 'Collector source differs'}
if($taskCandidate.source -cne $taskPrevious.candidateSource -or
   $taskCandidate.sha256 -cne $taskPrevious.candidateExeSha256 -or
   !$taskCandidate.sameShaCiAndArtifactVerified){throw 'Candidate identity differs'}
$taskBaseline=Join-Path $taskCode 'test/software/acceptance-kit-02a0d4-20261015/bin/baseline019/opencodex-desktop.exe'
if((Get-FileHash -LiteralPath $taskCandidate.executable).Hash.ToLowerInvariant() -cne $taskCandidate.sha256 -or
   (Get-FileHash -LiteralPath $taskBaseline).Hash.ToLowerInvariant() -cne $taskPrevious.baselineExeSha256){throw 'Executable bytes differ'}
$taskMethods=[ordered]@{}
foreach($taskPath in @('test/smoke/windows-idle-family.ps1','test/smoke/windows-idle-observe.ps1','test/smoke/windows-process-token.ps1')){
    $taskMethods[$taskPath]=(Get-FileHash -LiteralPath (Join-Path $taskDiagnostic $taskPath)).Hash.ToLowerInvariant()
}
$taskRoot=Join-Path $taskCode 'test/runtime/foreground-02-current-20261015/install-a/data'
$taskBaselineRoot=Join-Path $env:APPDATA 'com.gzers.opencodex.desktop'
$taskPointer=(Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop').StartupDataRootV1 | ConvertFrom-Json
if($taskPointer.anchor -cnotin @($taskRoot,('\\?\'+$taskRoot))){throw 'Anchor changed'}
$taskBindings=foreach($taskFixture in @($taskRoot,$taskBaselineRoot)){
    $taskItem=Get-Item -LiteralPath $taskFixture
    if($taskItem.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Fixture alias refused'}
    $taskRuntime=Get-Content -LiteralPath (Join-Path $taskFixture 'manager-state/runtime.json') -Raw | ConvertFrom-Json
    if($taskRuntime.source -cne 'unresolved' -or $taskRuntime.path){throw 'Fixture runtime changed'}
    $taskPrefs=Get-Content -LiteralPath (Join-Path $taskFixture 'manager-state/preferences.json') -Raw | ConvertFrom-Json
    if($taskPrefs.appearance.theme -cne 'light' -or $taskPrefs.appearance.visual_effects -cne 'high' -or
       $taskPrefs.appearance.glow_render -cne 'mesh' -or $taskPrefs.appearance.interface_scale -ne 100 -or
       $taskPrefs.shell.auto_panel -or $taskPrefs.shell.launch_with_codex -or
       $taskPrefs.shell.keep_proxy_on_close -or !$taskPrefs.shell.launch_main -or
       $taskPrefs.updates.app_update_auto_check -or $taskPrefs.maintenance.startup_cleanup){
        throw 'Fixture scenario changed'
    }
    [ordered]@{
        root=$taskFixture
        preferencesSha256=(Get-FileHash -LiteralPath (Join-Path $taskFixture 'manager-state/preferences.json')).Hash.ToLowerInvariant()
        runtimeSha256=(Get-FileHash -LiteralPath (Join-Path $taskFixture 'manager-state/runtime.json')).Hash.ToLowerInvariant()
    }
}
$taskContract=[ordered]@{
    schemaVersion=1;preparationOnly=$true;foregroundWindowGranted=$false
    collectorSource=$taskSource;collectorCiRunId=38834762492
    candidateSource=$taskCandidate.source;candidateExecutable=$taskCandidate.executable
    candidateExeSha256=$taskCandidate.sha256
    baselineSource=$taskPrevious.baselineSource;baselineExecutable=$taskBaseline
    baselineExeSha256=$taskPrevious.baselineExeSha256
    order=@('baseline019','candidate0110');settleSecondsPerVersion=60
    minimumCompletedCounterSpanSeconds=600
    zeroReadOrFrameFailuresRequired=$true
    clientWidth=2360;clientHeight=1520
    theme='light';visualEffects='high';glowRender='mesh';interfaceScale=100
    budgetsUnchanged=$taskPrevious.budgetsUnchanged
    methodWorkingFileSha256=$taskMethods
    sidMethodPath=(Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/methods/process-user-sid.ps1')
    sidMethodSha256='0072ea451bedc8b75d7e9eea6660286dec374e546d5cc0d296e93cf77794b072'
    fixtureRoot=$taskRoot;baselineFixtureRoot=$taskBaselineRoot
    priorSamplesUsedInNewComparison=$false
    productBuildRequested=$false;etwReplayRequested=$false
    fullPerformanceGatePassed=$false;historicalFailuresCleared=$false
}
$taskContract | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'contract.json') -Encoding utf8
$taskShared=[ordered]@{}
foreach($taskName in @('AGENTS.md','README.md','README.en.md','LICENSE','.gitignore')){
    $taskHashes=@(
        (Get-FileHash -LiteralPath (Join-Path (Get-Location) $taskName)).Hash
        (Get-FileHash -LiteralPath (Join-Path $taskCode $taskName)).Hash
        (Get-FileHash -LiteralPath (Join-Path $taskDiagnostic $taskName)).Hash
    )
    $taskShared[$taskName]=@($taskHashes | Select-Object -Unique).Count -eq 1
}
[ordered]@{
    observedAtUtc=[datetimeoffset]::UtcNow.ToString('o')
    nativeCount=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count
    fixtureReadback=$taskBindings;fixtureModified=$false;hkcuModified=$false
    dailyExeSha256=(Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLowerInvariant()
    taskSha256=(Get-FileHash -LiteralPath '.adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json').Hash.ToLowerInvariant()
    sharedFilesIdentical=$taskShared
    productStarted=$false;foregroundActivated=$false;longMeasurementExecuted=$false
    authorizationReceiptCreated=$false;originalSidecarsTouched=$false
} | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'preparation-readback.json') -Encoding utf8

# Read-only preparation. This script grants no foreground or recording authority.
$ErrorActionPreference='Stop'
$taskDocs='D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop'
$taskRepo='C:\Users\15119\.codex\worktrees\windows-working-set\opencodex-desktop'
$taskContract=Get-Content -LiteralPath (Join-Path $taskDocs '.adg/evidence/OCX-0110-20261016-WIN-B4-FRONT-PREP/contract.json') -Raw | ConvertFrom-Json
$taskRestoration=Get-Content -LiteralPath (Join-Path $taskDocs '.adg/evidence/OCX-0110-20261016-WIN-B4-RESTORATION/restoration-readback.original.json') -Raw | ConvertFrom-Json
$taskOutput=Join-Path $PSScriptRoot 'diagnostic-preflight.original.json'
if(Test-Path -LiteralPath $taskOutput){throw 'Preserve the first preparation readback'}
if((Get-FileHash -LiteralPath $taskContract.directoryIdentityMethod).Hash.ToLowerInvariant() -cne $taskContract.directoryIdentityMethodSha256){throw 'Frozen directory method differs'}
. $taskContract.directoryIdentityMethod
$taskNativeBefore=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'")
if($taskNativeBefore.Count){throw 'Read-only preparation expects native0'}
$taskProfile=Get-RecoveryDirectoryIdentity -Path $taskContract.baselineFixtureRoot
$taskFixture=Get-RecoveryDirectoryIdentity -Path $taskContract.fixtureRoot
$taskPrefs=(Get-FileHash -LiteralPath (Join-Path $taskContract.baselineFixtureRoot 'manager-state/preferences.json')).Hash.ToLowerInvariant()
$taskPointer=[string](Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop').StartupDataRootV1
$taskPointerSha=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($taskPointer))).ToLowerInvariant()
$taskAnchor=($taskPointer | ConvertFrom-Json).anchor
if($taskProfile.fileId -cne $taskRestoration.originalDirectory.fileId -or
   $taskProfile.creationFileTime -cne $taskRestoration.originalDirectory.creationFileTime -or
   $taskPrefs -cne $taskRestoration.restoredPreferencesSha256 -or
   $taskPointerSha -cne $taskRestoration.startupPointerSha256){throw 'Restored ordinary profile or startup pointer changed'}
if($taskAnchor -cnotin @($taskContract.fixtureRoot,('\\?\'+$taskContract.fixtureRoot))){throw 'Expected isolated fixture anchor required'}
$taskRuntime=Get-Content -LiteralPath (Join-Path $taskContract.fixtureRoot 'manager-state/runtime.json') -Raw | ConvertFrom-Json
if($taskRuntime.source -cne 'unresolved' -or $taskRuntime.path){throw 'Fixture runtime must remain unresolved'}
$taskExeHash=(Get-FileHash -LiteralPath $taskContract.candidateExecutable).Hash.ToLowerInvariant()
if($taskExeHash -cne $taskContract.candidateExeSha256){throw 'Exact b4 executable changed'}
$taskDecoder='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\bd2-etw-preparation-20261014'
$taskDecoders=[ordered]@{}
foreach($taskPair in @(
    @('Microsoft.Diagnostics.Tracing.TraceEvent.dll','401e948dac99d0f9756ef96d5a8b8e4736e19e9f82ce8cdd744495951e3a94bb'),
    @('Microsoft.Diagnostics.FastSerialization.dll','83e89baf0bbecca8cfa194cff13365f716849e22efc374b669206f26998a019f')
)){
    $taskHash=(Get-FileHash -LiteralPath (Join-Path $taskDecoder $taskPair[0])).Hash.ToLowerInvariant()
    if($taskHash -cne $taskPair[1]){throw 'Pinned decoder differs'}
    $taskDecoders[$taskPair[0]]=$taskHash
}
$taskHead=(git -C $taskRepo rev-parse HEAD).Trim()
if($LASTEXITCODE -ne 0 -or $taskHead -cne '391e70979ffbb32a548c0d05bdeeb62160d942d3'){throw 'Exact recording source changed'}
$taskTrackedChanges=@(git -C $taskRepo status --porcelain --untracked-files=no)
if($LASTEXITCODE -ne 0 -or $taskTrackedChanges.Count){throw 'Diagnostic implementation has tracked changes'}
$taskAdmin=([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
$taskNativeAfter=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'")
if($taskNativeAfter.Count){throw 'Native set changed during preparation'}
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');passed=$true;preparationOnly=$true
    nativeCountBefore=0;nativeCountAfter=0;toolSource=$taskHead;trackedToolChanges=0
    productSource=$taskContract.candidateSource;productExecutable=$taskContract.candidateExecutable
    productExeSha256=$taskExeHash;originalProfile=$taskProfile;isolatedFixture=$taskFixture
    originalPreferencesSha256=$taskPrefs;startupPointerSha256=$taskPointerSha
    isolatedPreferencesSha256=(Get-FileHash -LiteralPath (Join-Path $taskContract.fixtureRoot 'manager-state/preferences.json')).Hash.ToLowerInvariant()
    fixtureRuntimeUnresolved=$true;decoderDirectory=$taskDecoder;decoderHashes=$taskDecoders
    actualControllerElevated=$taskAdmin;freshHumanForegroundSignalPresent=$false
    productStarted=$false;uiInputInjected=$false;tracingStarted=$false;selfElevationPerformed=$false
    originalProfileMoved=$false;configurationWritten=$false;recordingExecutionAdmitted=$false
    fullProductGatePassed=$false;rssFailureCleared=$false
} | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $taskOutput -Encoding utf8
Get-Content -LiteralPath $taskOutput

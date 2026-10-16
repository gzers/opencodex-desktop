# Deferred until the next explicit human foreground window; not executed during preparation.
param([Parameter(Mandatory)][string]$RunDirectory)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'run-binding.ps1')
$taskRun=Assert-PreparedIdleRun -RunDirectory $RunDirectory -RequireCurrentHumanWindow
$taskContract=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'contract.json') -Raw | ConvertFrom-Json
$taskCi=Get-Content -LiteralPath (Join-Path $taskRun 'ci-status.json') -Raw | ConvertFrom-Json
if($taskCi.runId -ne $taskContract.collectorCiRunId -or $taskCi.source -cne $taskContract.collectorSource -or
   $taskCi.status -cne 'completed' -or $taskCi.conclusion -cne 'success'){throw 'Exact completed collector CI required'}
if(Test-Path -LiteralPath (Join-Path $taskRun 'fixture-ready.json')){throw 'Fixture already prepared; do not repeat'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native product exists'}
$taskRoot=[IO.Path]::GetFullPath($taskContract.fixtureRoot)
$taskBaseline=[IO.Path]::GetFullPath($taskContract.baselineFixtureRoot)
$taskSaved=[IO.Path]::GetFullPath((Join-Path $taskRun 'baseline-previous'))
$taskExpectedRoot='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\foreground-02-current-20261015\install-a\data'
$taskExpectedBaseline=[IO.Path]::GetFullPath((Join-Path $env:APPDATA 'com.gzers.opencodex.desktop'))
if($taskRoot -cne $taskExpectedRoot -or $taskBaseline -cne $taskExpectedBaseline -or
   !$taskSaved.StartsWith($taskRun+'\',[StringComparison]::OrdinalIgnoreCase) -or
   (Test-Path -LiteralPath $taskSaved)){throw 'Unexpected fixture move endpoints'}
foreach($taskFixture in @($taskRoot,$taskBaseline)){
    $taskEntry=Get-Item -LiteralPath $taskFixture
    if($taskEntry.Attributes -band [IO.FileAttributes]::ReparsePoint -or
       @(Get-ChildItem -LiteralPath $taskFixture -Recurse -Force | Where-Object {$_.Attributes -band [IO.FileAttributes]::ReparsePoint}).Count){throw 'Fixture alias refused'}
    $taskRuntime=Get-Content -LiteralPath (Join-Path $taskFixture 'manager-state/runtime.json') -Raw | ConvertFrom-Json
    if($taskRuntime.source -cne 'unresolved' -or $taskRuntime.path){throw 'Fixture runtime changed'}
}
$taskExpectedPrefs=(Get-Content -LiteralPath (Join-Path $PSScriptRoot 'preparation-readback.json') -Raw | ConvertFrom-Json).fixtureReadback[0].preferencesSha256
if((Get-FileHash -LiteralPath (Join-Path $taskRoot 'manager-state/preferences.json')).Hash.ToLowerInvariant() -cne $taskExpectedPrefs){throw 'Prepared preferences changed'}
$taskPointer=(Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop').StartupDataRootV1 | ConvertFrom-Json
$taskMetadata=Get-Content -LiteralPath (Join-Path $taskRoot 'manager-state/data-root.json') -Raw | ConvertFrom-Json
if($taskPointer.anchor -cnotin @($taskRoot,('\\?\'+$taskRoot)) -or
   $taskMetadata.structure_version -cne '1' -or $taskMetadata.active_data_root){throw 'Fixture anchor changed'}
# Both resolved absolute endpoints have been checked before this recursive move.
Move-Item -LiteralPath $taskBaseline -Destination $taskSaved
Copy-Item -LiteralPath $taskRoot -Destination $taskBaseline -Recurse
$taskCloneHash=(Get-FileHash -LiteralPath (Join-Path $taskBaseline 'manager-state/preferences.json')).Hash.ToLowerInvariant()
if($taskCloneHash -cne $taskExpectedPrefs){throw 'Cloned preferences differ; preserve all endpoints for diagnosis'}
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');nativeCount=0
    sourceRoot=$taskRoot;baselineRoot=$taskBaseline;previousBaselineRetainedAt=$taskSaved
    fixturePreferencesSha256=$taskCloneHash;originalSidecarsTouched=$false
    profileEnvironmentRedirected=$false;productStarted=$false
} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRun 'fixture-ready.json') -Encoding utf8

$ErrorActionPreference='Stop'
$taskDocs='D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop'
$taskPrep=Join-Path $taskDocs '.adg/evidence/OCX-0110-20261016-WIN-B4-FRONT-PREP'
$taskOriginalRun='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop\test\runtime\foreground-b4-pair-20261016T204338'
$taskOutput=Join-Path $PSScriptRoot 'cleanup-followup.original.json'
if(Test-Path -LiteralPath $taskOutput){throw 'Preserve the first follow-up; do not overwrite it'}
$taskNative=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'")
$taskRestoreAttempted=$false
if($taskNative.Count -eq 0 -and !(Test-Path -LiteralPath (Join-Path $taskOriginalRun 'fixture-restored.json'))){
    $taskRestoreAttempted=$true
    & (Join-Path $taskPrep 'restore-fixture.ps1') -RunDirectory $taskOriginalRun
}
$taskContract=Get-Content -LiteralPath (Join-Path $taskPrep 'contract.json') -Raw | ConvertFrom-Json
if((Get-FileHash -LiteralPath $taskContract.directoryIdentityMethod).Hash.ToLowerInvariant() -cne $taskContract.directoryIdentityMethodSha256){throw 'Frozen directory method changed'}
. $taskContract.directoryIdentityMethod
$taskRestored=Test-Path -LiteralPath (Join-Path $taskOriginalRun 'fixture-restored.json')
$taskOriginalPath=if($taskRestored){$taskContract.baselineFixtureRoot}else{Join-Path $taskOriginalRun 'baseline-previous'}
$taskOriginal=Get-RecoveryDirectoryIdentity -Path $taskOriginalPath
$taskCurrent=Get-RecoveryDirectoryIdentity -Path $taskContract.baselineFixtureRoot
$taskPointer=[string](Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop').StartupDataRootV1
$taskPointerHash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($taskPointer))).ToLowerInvariant()
if($taskOriginal.fileId -cne $taskContract.baselineOriginalDirectoryId -or
   $taskOriginal.creationFileTime -cne $taskContract.baselineOriginalDirectoryCreation -or
   $taskPointerHash -cne 'f62b7543fbb2b258dcaf15324dae91fea96c83d95c969065bfbf10428ac7012c'){throw 'Original profile identity or startup pointer changed'}
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o')
    nativeCount=$taskNative.Count
    nativeProcesses=@($taskNative | Select-Object ProcessId,ParentProcessId,ExecutablePath,CreationDate)
    originalDirectory=$taskOriginal;currentDirectory=$taskCurrent
    startupPointerSha256=$taskPointerHash;taskFileSha256=(Get-FileHash -LiteralPath (Join-Path $taskDocs '.adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json')).Hash.ToLowerInvariant()
    fixtureRestored=$taskRestored;restoreAttempted=$taskRestoreAttempted
    originalAndLateExitTimeoutResultsUnchanged=$true
    normalCandidateExitCodeStillUnknown=$true
    foregroundInputSent=$false;newProductStarted=$false
    forceTerminated=$false;dataDeleted=$false
} | ConvertTo-Json -Depth 9 | Set-Content -LiteralPath $taskOutput -Encoding utf8
Get-Content -LiteralPath $taskOutput

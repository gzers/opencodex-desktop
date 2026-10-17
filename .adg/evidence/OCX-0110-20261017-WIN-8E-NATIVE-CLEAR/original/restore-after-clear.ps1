$ErrorActionPreference='Stop'
$taskRun=$PSScriptRoot
$taskExpected='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop\test\runtime\foreground-8e-cleanup-20261017T213054'
if([IO.Path]::GetFullPath($taskRun) -cne $taskExpected){throw 'Exact owned run required'}
function Assert-Idle {if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe' OR Name='Un.exe' OR Name='uninstall.exe'").Count){throw 'Product and uninstaller must be closed'}}
Assert-Idle
$taskBefore=Get-Content -LiteralPath (Join-Path $taskRun 'before-clear.json') -Raw|ConvertFrom-Json
$taskContract=Get-Content -LiteralPath (Join-Path $taskRun 'contract.json') -Raw|ConvertFrom-Json
$taskClearRoot='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop\test\runtime\foreground-99-roots-20261017T172126\selected-root'
if($taskBefore.root -cne $taskClearRoot -or $taskContract.selectedRoot -cne $taskClearRoot){throw 'Exact previously used test root required'}
if((Test-Path -LiteralPath $taskClearRoot) -or (Test-Path -LiteralPath $taskContract.recoveryExecutable) -or (Test-Path -LiteralPath 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\OpenCodeX Desktop')){throw 'Actual uninstallation not complete'}
$taskKey='HKCU:\Software\com.gzers.opencodex.desktop'
if((Get-ItemProperty -LiteralPath $taskKey -ErrorAction SilentlyContinue).PSObject.Properties.Name -contains 'StartupDataRootV1'){throw 'Cleanup pointer still present or changed concurrently'}
$taskSaved=[IO.Path]::GetFullPath($taskBefore.preservedCopy)
if($taskSaved -cne (Join-Path $taskRun 'before-clear-data-copy')){throw 'Exact pre-cleanup saved copy required'}
foreach($taskFile in $taskBefore.files){
 $taskPath=[IO.Path]::GetFullPath((Join-Path $taskSaved $taskFile.path))
 if(!$taskPath.StartsWith($taskSaved+'\',[StringComparison]::OrdinalIgnoreCase) -or !(Test-Path -LiteralPath $taskPath -PathType Leaf) -or (Get-Item -LiteralPath $taskPath).Length -ne $taskFile.bytes -or (Get-FileHash -LiteralPath $taskPath).Hash.ToLowerInvariant() -cne $taskFile.sha256){throw 'Saved pre-cleanup bytes differ'}
}
foreach($taskFile in $taskBefore.unrelatedFiles){if((Get-FileHash -LiteralPath $taskFile.path).Hash.ToLowerInvariant() -cne $taskFile.sha256){throw 'Unrelated fixture changed'}}
$taskFirst=Get-Content -LiteralPath (Join-Path $taskRun 'custom-first-exit.json') -Raw|ConvertFrom-Json
$taskSecond=Get-Content -LiteralPath (Join-Path $taskRun 'custom-restart-exit.json') -Raw|ConvertFrom-Json
$taskSecondLaunch=Get-Content -LiteralPath (Join-Path $taskRun 'custom-restart-launch.json') -Raw|ConvertFrom-Json
$taskSecondCreated=([datetimeoffset]$taskSecondLaunch.creationTimeUtc).UtcDateTime.ToFileTimeUtc()
if($taskFirst.exitCode -ne 0 -or $taskSecond.exitCode -ne 0 -or $taskFirst.nativeCount -ne 0 -or $taskSecond.nativeCount -ne 0 -or $taskSecondCreated -lt [long]$taskFirst.kernelTimes.exitFileTime){throw 'Single native generation order/normal exit not proved'}
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskContract.candidateSource;userMessage='已卸载';humanPerformedCheckboxAndUninstall=$true;humanCompletionReported=$true;uninstallerCheckboxIndependentReadbackProved=$false;testRoot=$taskClearRoot;testRootAbsent=$true;startupPointerAbsent=$true;mainExecutableAbsent=$true;uninstallRegistrationAbsent=$true;uninstallerProcessCount=0;nativeCount=0;actualDataFilesPreservedInCopy=@($taskBefore.files).Count;allSavedCopyBytesVerified=$true;unrelatedFilesPreserved=$taskBefore.unrelatedFiles;rootClearedByActualUninstaller=$true;helperDirectlyInvokedByHarness=$false;uninstallerActualExitCode=$null;fullInstallerGatePassed=$false} | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $taskRun 'actual-uninstall-clear-result.json') -Encoding utf8
. 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261016-WIN-RECOVERY-PLATFORM-PREP\directory-identity.ps1'
$taskPlan=Get-Content -LiteralPath (Join-Path $taskRun 'profile-isolation-plan.json') -Raw|ConvertFrom-Json
$taskNewProfiles=@(foreach($taskAlias in $taskPlan.aliases){if(Test-Path -LiteralPath $taskAlias.path){[pscustomobject]@{name=$taskAlias.name;path=$taskAlias.path;identity=(Get-RecoveryDirectoryIdentity -Path $taskAlias.path)}}})
$taskNewPreserved=@(foreach($taskGroup in $taskNewProfiles|Group-Object {$_.identity.fileId}){
 $taskProfile=$taskGroup.Group[0]
 $taskSource=[IO.Path]::GetFullPath($taskProfile.path)
 if($taskSource -cnotin $taskPlan.aliases.path){throw 'Only exact approved app profiles may be preserved'}
 $taskDestination=[IO.Path]::GetFullPath((Join-Path $taskRun ('new-profile-'+$taskProfile.name)))
 if(!$taskDestination.StartsWith($taskExpected+'\',[StringComparison]::OrdinalIgnoreCase) -or (Test-Path -LiteralPath $taskDestination)){throw 'Owned fresh profile destination required'}
 foreach($taskPath in @($taskSource,$taskDestination)){$taskCursor=$taskPath;while($taskCursor){if(Test-Path -LiteralPath $taskCursor){if((Get-Item -LiteralPath $taskCursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Profile path alias refused'}};$taskCursor=[IO.Path]::GetDirectoryName($taskCursor)}}
 Assert-Idle
 Move-Item -LiteralPath $taskSource -Destination $taskDestination
 $taskAfter=Get-RecoveryDirectoryIdentity -Path $taskDestination
 if($taskAfter.fileId -cne $taskProfile.identity.fileId -or $taskAfter.creationFileTime -cne $taskProfile.identity.creationFileTime){throw 'Moved test profile identity differs'}
 [ordered]@{source=$taskSource;destination=$taskDestination;before=$taskProfile.identity;after=$taskAfter;contentsPreserved=$true}
})
$taskNewPreserved|ConvertTo-Json -Depth 7|Set-Content -LiteralPath (Join-Path $taskRun 'new-profile-preservation.json') -Encoding utf8
& 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261016-WIN-RECOVERY-PLATFORM-PREP\restore-platform-roots.ps1' -RunDirectory $taskRun
$taskPrivate=Get-Content -LiteralPath (Join-Path $taskRun 'private-original-pointer.json') -Raw|ConvertFrom-Json
$taskOriginal=$taskPrivate.originalPointer
$taskPointerHash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($taskOriginal))).ToLowerInvariant()
if($taskPointerHash -cne 'f62b7543fbb2b258dcaf15324dae91fea96c83d95c969065bfbf10428ac7012c'){throw 'Preserved original pointer differs'}
Assert-Idle
if(!(Test-Path -LiteralPath $taskKey)){New-Item -Path $taskKey -Force|Out-Null}
Set-ItemProperty -LiteralPath $taskKey -Name StartupDataRootV1 -Value $taskOriginal
if((Get-ItemProperty -LiteralPath $taskKey -Name StartupDataRootV1).StartupDataRootV1 -cne $taskOriginal){throw 'Exact original pointer restore failed'}
$taskPreference='C:\Users\15119\AppData\Roaming\com.gzers.opencodex.desktop\manager-state\preferences.json'
$taskPreferenceHash=(Get-FileHash -LiteralPath $taskPreference).Hash.ToLowerInvariant()
if($taskPreferenceHash -cne '07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d'){throw 'Original preference bytes differ'}
$taskTaskHash=(Get-FileHash -LiteralPath 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\work\objects\task\TASK-OPENCODEX-DESKTOP-182.json').Hash.ToLowerInvariant()
if($taskTaskHash -cne 'bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93'){throw 'TASK changed'}
Assert-Idle
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');nativeCount=0;uninstallerProcessCount=0;mainExecutableAbsent=$true;testRootStillAbsent=!(Test-Path -LiteralPath $taskClearRoot);testPreCleanupCopyPreserved=$true;originalProfileNativeIdsRestored=$true;originalPointerRestoredExact=$true;originalPointerSha256=$taskPointerHash;originalPreferencesSha256=$taskPreferenceHash;taskSha256=$taskTaskHash;originalRealDataDeleted=$false;foregroundReleased=$true;nextUiRequiresNewHumanWindow=$true;normalProductExitCount=2;ordinaryRestartReadbackPassed=$true;fullWindowsGatePassed=$false;releaseAccepted=$false} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $taskRun 'final-protection.json') -Encoding utf8
Get-Content -LiteralPath (Join-Path $taskRun 'final-protection.json') -Raw
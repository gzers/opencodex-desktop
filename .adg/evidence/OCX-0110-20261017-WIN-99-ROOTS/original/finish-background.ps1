$ErrorActionPreference='Stop'
$taskRun=$PSScriptRoot
$taskAllowed='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop\test\runtime\foreground-99-roots-20261017T172126'
if([IO.Path]::GetFullPath($taskRun) -cne $taskAllowed){throw 'Exact owned run required'}
function Assert-NativeZero {if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native0 required'}}
function Get-Sha([string]$Value){[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($Value))).ToLowerInvariant()}
Assert-NativeZero
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');userMessage='已退出';nativeCount=0;retainedObserverOutcome='deadline expired before exit was observed';actualExitCode=$null;actualExitTimeUtc=$null;normalExit0Proved=$false;forceTerminated=$false;additionalProductLaunch=$false;foregroundExpired=$true} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRun 'replacement-close-action.json') -Encoding utf8
$taskContract=Get-Content -LiteralPath (Join-Path $taskRun 'contract.json') -Raw|ConvertFrom-Json
$taskPrivate=Get-Content -LiteralPath (Join-Path $taskRun 'recovery-private-originals.json') -Raw|ConvertFrom-Json
$taskInstall=[IO.Path]::GetFullPath($taskContract.recoveryDirectory)
if($taskInstall -cne (Join-Path $taskRun 'install with spaces') -or $taskPrivate.directory -cne $taskInstall){throw 'Exact installation required'}
$taskCursor=Get-Item -LiteralPath $taskInstall
while($taskCursor){if($taskCursor.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Reparse ancestor refused'};$taskCursor=$taskCursor.Parent}
if((Get-FileHash -LiteralPath $taskContract.recoveryExecutable).Hash.ToLowerInvariant() -cne $taskContract.candidateExeSha256){throw 'Installed executable changed'}
$taskAcl=Get-Acl -LiteralPath $taskInstall
$taskAcl.SetSecurityDescriptorSddlForm($taskPrivate.originalSddl,[Security.AccessControl.AccessControlSections]::Access)
Set-Acl -LiteralPath $taskInstall -AclObject $taskAcl
if((Get-Acl -LiteralPath $taskInstall).GetSecurityDescriptorSddlForm([Security.AccessControl.AccessControlSections]::Access) -cne $taskPrivate.originalSddl){throw 'Original ACL not restored'}
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');scope=$taskInstall;originalAccessSddlRestoredExact=$true;rawSddlPublished=$false;nativeCount=0} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRun 'acl-restored.json') -Encoding utf8
$taskKey='HKCU:\Software\com.gzers.opencodex.desktop'
$taskCurrentPointer=(Get-ItemProperty -LiteralPath $taskKey -Name StartupDataRootV1).StartupDataRootV1
$taskExpectedPointer=(Get-Content -LiteralPath (Join-Path $taskRun 'selected-startup-pointer.original.json') -Raw).TrimEnd("`r","`n")
if($taskCurrentPointer -cne $taskExpectedPointer){throw 'Current selected pointer changed'}
$taskFiles=@(foreach($taskRootName in @('selected-root','initialize-empty-root')){
 $taskRoot=Join-Path $taskRun $taskRootName
 foreach($taskFile in Get-ChildItem -LiteralPath $taskRoot -Recurse -File -Force){
  if($taskFile.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'File alias refused'}
  [ordered]@{path=[IO.Path]::GetRelativePath($taskRun,$taskFile.FullName);bytes=$taskFile.Length;sha256=(Get-FileHash -LiteralPath $taskFile.FullName).Hash.ToLowerInvariant()}
 }
})
$taskFiles | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $taskRun 'before-uninstall-files.json') -Encoding utf8
$taskUninstaller=Join-Path $taskInstall 'uninstall.exe'
if(!(Test-Path -LiteralPath $taskUninstaller -PathType Leaf)){throw 'Uninstaller missing'}
Assert-NativeZero
$taskProcess=Start-Process -FilePath $taskUninstaller -ArgumentList '/S' -WindowStyle Hidden -PassThru
$taskWatch=[Diagnostics.Stopwatch]::StartNew()
while($taskWatch.Elapsed.TotalSeconds -lt 45 -and ((Test-Path -LiteralPath $taskContract.recoveryExecutable) -or !( $taskProcess.HasExited))){Start-Sleep -Milliseconds 250;$taskProcess.Refresh()}
if(!$taskProcess.HasExited -or $taskProcess.ExitCode -ne 0 -or (Test-Path -LiteralPath $taskContract.recoveryExecutable)){throw 'Uninstall not complete; never retry automatically'}
$taskRegistry='HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\OpenCodeX Desktop'
if(Test-Path -LiteralPath $taskRegistry){throw 'Uninstall entry still present'}
Assert-NativeZero
foreach($taskFile in $taskFiles){$taskPath=Join-Path $taskRun $taskFile.path;if(!(Test-Path -LiteralPath $taskPath -PathType Leaf) -or (Get-Item -LiteralPath $taskPath).Length -ne $taskFile.bytes -or (Get-FileHash -LiteralPath $taskPath).Hash.ToLowerInvariant() -cne $taskFile.sha256){throw 'Uninstall did not preserve actual data bytes'}}
if((Get-ItemProperty -LiteralPath $taskKey -Name StartupDataRootV1).StartupDataRootV1 -cne $taskCurrentPointer){throw 'Default uninstall changed pointer'}
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskContract.candidateSource;uninstallerParentExitCode=$taskProcess.ExitCode;mainExePresent=$false;uninstallRegistryPresent=$false;actualDataFiles=$taskFiles;allActualDataBytesPreserved=$true;selectedPointerPreservedExact=$true;explicitCleanupSelected=$false;nativeCount=0;newProductStarted=$false;fullInstallerGatePassed=$false} | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $taskRun 'actual-custom-uninstall-retention.json') -Encoding utf8
$taskOriginalHash=Get-Sha $taskPrivate.originalPointer
if($taskOriginalHash -cne 'f62b7543fbb2b258dcaf15324dae91fea96c83d95c969065bfbf10428ac7012c'){throw 'Original pointer identity differs'}
Set-ItemProperty -LiteralPath $taskKey -Name StartupDataRootV1 -Value $taskPrivate.originalPointer
if((Get-ItemProperty -LiteralPath $taskKey -Name StartupDataRootV1).StartupDataRootV1 -cne $taskPrivate.originalPointer){throw 'Pointer restore differs'}
. 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261016-WIN-RECOVERY-PLATFORM-PREP\directory-identity.ps1'
$taskPlan=Get-Content -LiteralPath (Join-Path $taskRun 'profile-isolation-plan.json') -Raw|ConvertFrom-Json
$taskNewPhysical=@(foreach($taskAlias in $taskPlan.aliases){if(Test-Path -LiteralPath $taskAlias.path){[pscustomobject]@{name=$taskAlias.name;path=$taskAlias.path;identity=(Get-RecoveryDirectoryIdentity -Path $taskAlias.path)}}})
$taskSavedNew=@(foreach($taskGroup in $taskNewPhysical|Group-Object {$_.identity.fileId}){
 $taskProfile=$taskGroup.Group[0];$taskDestination=[IO.Path]::GetFullPath((Join-Path $taskRun ('new-profile-'+$taskProfile.name)))
 if(!$taskDestination.StartsWith($taskAllowed+'\',[StringComparison]::OrdinalIgnoreCase) -or (Test-Path -LiteralPath $taskDestination)){throw 'Exact fresh owned destination required'}
 if($taskProfile.path -cnotin $taskPlan.aliases.path){throw 'Unknown app profile path'}
 Assert-NativeZero
 Move-Item -LiteralPath $taskProfile.path -Destination $taskDestination
 $taskAfter=Get-RecoveryDirectoryIdentity -Path $taskDestination
 if($taskAfter.fileId -cne $taskProfile.identity.fileId -or $taskAfter.creationFileTime -cne $taskProfile.identity.creationFileTime){throw 'New profile identity changed'}
 [ordered]@{source=$taskProfile.path;destination=$taskDestination;before=$taskProfile.identity;after=$taskAfter;contentsPreserved=$true}
})
$taskSavedNew | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $taskRun 'new-profile-preservation.json') -Encoding utf8
& 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261016-WIN-RECOVERY-PLATFORM-PREP\restore-platform-roots.ps1' -RunDirectory $taskRun
$taskPrefs='C:\Users\15119\AppData\Roaming\com.gzers.opencodex.desktop\manager-state\preferences.json'
$taskPrefHash=(Get-FileHash -LiteralPath $taskPrefs).Hash.ToLowerInvariant()
if($taskPrefHash -cne '07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d'){throw 'Restored preference bytes differ'}
Assert-NativeZero
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');nativeCount=0;mainExePresent=$false;originalPointerRestoredExact=$true;originalPointerSha256=$taskOriginalHash;originalPreferencesSha256=$taskPrefHash;originalProfileNativeIdsRestored=$true;originalInstallAclRestoredExact=$true;testProfilesPreserved=$true;realDataDeleted=$false;foregroundReleased=$true;furtherNativeUiRequiresNewHumanWindow=$true;restartReadbackPassed=$false;replacementExit0Proved=$false} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRun 'final-protection.json') -Encoding utf8
Get-Content -LiteralPath (Join-Path $taskRun 'final-protection.json') -Raw
# Restore owned prerequisites even after window expiry; no UI or process input.
param([Parameter(Mandatory)][string]$RunDirectory)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'run-binding.ps1')
$taskRun=Assert-PreparedIdleRun -RunDirectory $RunDirectory
$taskContract=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'contract.json') -Raw | ConvertFrom-Json
$taskSaved=Get-Content -LiteralPath (Join-Path $taskRun 'recovery-private-originals.json') -Raw | ConvertFrom-Json
if($taskSaved.source -cne $taskContract.candidateSource -or
   $taskSaved.directory -cne $taskContract.recoveryDirectory){throw 'Private restoration scope differs'}
$null=Assert-PreparedRecoveryDirectory -Directory $taskSaved.directory
$taskAcl=Get-Acl -LiteralPath $taskSaved.directory
$taskAcl.SetSecurityDescriptorSddlForm($taskSaved.originalSddl,[Security.AccessControl.AccessControlSections]::Access)
Set-Acl -LiteralPath $taskSaved.directory -AclObject $taskAcl
$taskKey='HKCU:\Software\com.gzers.opencodex.desktop'
$taskCurrent=(Get-ItemProperty -LiteralPath $taskKey).StartupDataRootV1
if($taskCurrent){
    $taskCurrentRoot=($taskCurrent | ConvertFrom-Json).anchor
    if($taskCurrentRoot -cnotin @($taskContract.fixtureRoot,('\\?\'+$taskContract.fixtureRoot))){throw 'Different current choice preserved; review before pointer restoration'}
}
Set-ItemProperty -LiteralPath $taskKey -Name StartupDataRootV1 -Value $taskSaved.originalPointer
$taskRestored=(Get-ItemProperty -LiteralPath $taskKey).StartupDataRootV1
$taskRestoredSddl=(Get-Acl -LiteralPath $taskSaved.directory).GetSecurityDescriptorSddlForm([Security.AccessControl.AccessControlSections]::Access)
if($taskRestored -cne $taskSaved.originalPointer -or $taskRestoredSddl -cne $taskSaved.originalSddl){throw 'Prerequisite restoration readback failed'}
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');aclRestored=$true;pointerRestoredExact=$true
    dataMovedOrDeleted=$false;rawSddlOrSidPublished=$false;processesStopped=$false;uiInputInjected=$false} |
    ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRun 'recovery-restored.json') -Encoding utf8

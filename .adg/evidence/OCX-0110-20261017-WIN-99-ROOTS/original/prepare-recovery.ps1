# Deferred, task-owned prerequisites. A new human window is mandatory.
param([Parameter(Mandatory)][string]$RunDirectory)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'run-binding.ps1')
$taskRun=Assert-PreparedIdleRun -RunDirectory $RunDirectory -RequireCurrentHumanWindow
$taskContract=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'contract.json') -Raw | ConvertFrom-Json
$taskHuman=Get-Content -LiteralPath (Join-Path $taskRun 'human-window.json') -Raw | ConvertFrom-Json
if((([datetimeoffset]$taskHuman.expiresAtUtc)-[datetimeoffset]::UtcNow).TotalSeconds -lt 180){throw 'Keep recovery pending when fewer than 180 seconds remain'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Close the bound prior instance normally before recovery'}
$taskDir=Assert-PreparedRecoveryDirectory -Directory $taskContract.recoveryDirectory
if((Get-FileHash -LiteralPath $taskContract.recoveryExecutable).Hash.ToLowerInvariant() -cne $taskContract.candidateExeSha256){throw 'Recovery image changed'}
$taskKey='HKCU:\Software\com.gzers.opencodex.desktop'
$taskValue=(Get-ItemProperty -LiteralPath $taskKey -Name StartupDataRootV1).StartupDataRootV1
$taskPointer=$taskValue | ConvertFrom-Json
if($taskPointer.anchor -cnotin @($taskContract.fixtureRoot,('\\?\'+$taskContract.fixtureRoot))){throw 'Existing manager anchor changed; preserve it'}
if(Test-Path -LiteralPath (Join-Path $taskDir 'data')){throw 'Recovery default is no longer fresh'}
$taskPrivate=Join-Path $taskRun 'recovery-private-originals.json'
if(Test-Path -LiteralPath $taskPrivate){throw 'Recovery preparation already attempted; do not overwrite'}
$taskAcl=Get-Acl -LiteralPath $taskDir
[ordered]@{source=$taskContract.candidateSource;directory=$taskDir;originalPointer=$taskValue
    originalSddl=$taskAcl.GetSecurityDescriptorSddlForm([Security.AccessControl.AccessControlSections]::Access)} |
    ConvertTo-Json | Set-Content -LiteralPath $taskPrivate -Encoding utf8
# The private SDDL contains raw SIDs and stays in ignored runtime, never public evidence.
$taskSid=[Security.Principal.WindowsIdentity]::GetCurrent().User
$taskRights=[Security.AccessControl.FileSystemRights]::CreateFiles -bor [Security.AccessControl.FileSystemRights]::CreateDirectories
$taskRule=[Security.AccessControl.FileSystemAccessRule]::new($taskSid,$taskRights,
    [Security.AccessControl.InheritanceFlags]::None,[Security.AccessControl.PropagationFlags]::None,
    [Security.AccessControl.AccessControlType]::Deny)
$taskAcl.SetAccessRule($taskRule)
Set-Acl -LiteralPath $taskDir -AclObject $taskAcl
Remove-ItemProperty -LiteralPath $taskKey -Name StartupDataRootV1
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskContract.candidateSource;directory=$taskDir
    privateOriginalsSha256=(Get-FileHash -LiteralPath $taskPrivate).Hash.ToLowerInvariant()
    scope='owned install-d only; deny file/directory creation on this directory without inheritance'
    pointerTemporarilyUnbound=$true;aclApplied=$true;rawSddlOrSidPublished=$false
    executableSha256=$taskContract.candidateExeSha256;productStarted=$false
} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRun 'recovery-prepared.json') -Encoding utf8

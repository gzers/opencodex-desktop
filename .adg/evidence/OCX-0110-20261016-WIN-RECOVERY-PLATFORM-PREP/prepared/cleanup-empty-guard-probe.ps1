param([string]$Preparation,[string]$Methods)
$ErrorActionPreference='Stop'
. (Join-Path $Methods 'directory-identity.ps1')
$taskRoot='C:\Users\15119\AppData\Roaming\com.gzers.opencodex.desktop'
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Require native0'}
$taskIdentity=Get-RecoveryDirectoryIdentity -Path $taskRoot
$taskPlan=Get-Content -LiteralPath (Join-Path $Preparation 'profile-isolation-plan.json') -Raw | ConvertFrom-Json
if($taskIdentity.fileId -cin @($taskPlan.physicalProfiles.fileId)){throw 'Never remove an original saved directory'}
if(@(Get-ChildItem -LiteralPath $taskRoot -Force).Count){throw 'Probe root has contents; preserve it'}
if([IO.Path]::GetFullPath($taskRoot) -cne $taskRoot){throw 'Require exact probe path'}
Remove-Item -LiteralPath $taskRoot
if(Test-Path -LiteralPath $taskRoot){throw 'Empty probe cleanup failed'}
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');stage='guard-check helper scope, after creating empty probe'
    error='Get-RecoveryDirectoryIdentity not in caller scope';emptyProbeIdentity=$taskIdentity
    emptyProbeRemovedNonrecursively=$true;originalSavedDirectoriesPreserved=$true
    productStarted=$false;pointerChanged=$false;profileContentsDeleted=$false} | ConvertTo-Json -Depth 6 |
    Set-Content -LiteralPath (Join-Path $Preparation 'guard-validation-initial-failure.json') -Encoding utf8

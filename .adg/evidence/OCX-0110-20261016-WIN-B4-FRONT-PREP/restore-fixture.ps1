param([Parameter(Mandatory)][string]$RunDirectory)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'run-binding.ps1')
$taskRun=Assert-PreparedIdleRun -RunDirectory $RunDirectory
$taskContract=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'contract.json') -Raw | ConvertFrom-Json
$taskReady=Get-Content -LiteralPath (Join-Path $taskRun 'fixture-ready.json') -Raw | ConvertFrom-Json
$taskBaseline=[IO.Path]::GetFullPath($taskContract.baselineFixtureRoot)
$taskSaved=[IO.Path]::GetFullPath((Join-Path $taskRun 'baseline-previous'))
$taskMeasured=[IO.Path]::GetFullPath((Join-Path $taskRun 'baseline-measured'))
if($taskBaseline -cne 'C:\Users\15119\AppData\Roaming\com.gzers.opencodex.desktop' -or
   $taskReady.baselineRoot -cne $taskBaseline -or $taskReady.previousBaselineRetainedAt -cne $taskSaved -or
   !$taskSaved.StartsWith($taskRun+'\',[StringComparison]::OrdinalIgnoreCase) -or
   !$taskMeasured.StartsWith($taskRun+'\',[StringComparison]::OrdinalIgnoreCase) -or
   (Test-Path -LiteralPath $taskMeasured)){throw 'Exact restoration endpoints required'}
foreach($taskPath in @($taskBaseline,$taskSaved,$taskMeasured)){
    $taskCursor=$taskPath
    while($taskCursor){
        if((Test-Path -LiteralPath $taskCursor) -and
           ((Get-Item -LiteralPath $taskCursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Restoration alias refused'}
        $taskCursor=[IO.Path]::GetDirectoryName($taskCursor)
    }
}
. $taskContract.directoryIdentityMethod
$taskClone=Get-RecoveryDirectoryIdentity -Path $taskBaseline
$taskOriginal=Get-RecoveryDirectoryIdentity -Path $taskSaved
if($taskClone.fileId -cne $taskReady.cloneDirectory.fileId -or
   $taskClone.creationFileTime -cne $taskReady.cloneDirectory.creationFileTime -or
   $taskOriginal.fileId -cne $taskReady.originalDirectory.fileId -or
   $taskOriginal.creationFileTime -cne $taskReady.originalDirectory.creationFileTime){throw 'Restoration directory generation differs'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Require native0'}
# All resolved absolute endpoints above are the exact app/test directories.
Move-Item -LiteralPath $taskBaseline -Destination $taskMeasured
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native appeared; preserve both endpoints and stop'}
Move-Item -LiteralPath $taskSaved -Destination $taskBaseline
$taskAfter=Get-RecoveryDirectoryIdentity -Path $taskBaseline
if($taskAfter.fileId -cne $taskOriginal.fileId -or
   $taskAfter.creationFileTime -cne $taskOriginal.creationFileTime){throw 'Restored profile identity differs'}
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');originalDirectoryRestored=$true
    measuredClonePreservedAt=$taskMeasured;dataDeleted=$false;registryOrAclChanged=$false
    productStarted=$false;uiInputInjected=$false} | ConvertTo-Json |
    Set-Content -LiteralPath (Join-Path $taskRun 'fixture-restored.json') -Encoding utf8

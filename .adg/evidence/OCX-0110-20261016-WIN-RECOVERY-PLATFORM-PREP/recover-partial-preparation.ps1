# Undo only the two moves actually completed by the first preparation.
param([Parameter(Mandatory)][string]$RunDirectory)
$ErrorActionPreference='Stop'
$taskExact='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop\test\runtime\recovery-platform-preparation-20261016T1606'
if([IO.Path]::GetFullPath($RunDirectory) -cne $taskExact){throw 'Exact failed attempt required'}
if(Test-Path -LiteralPath (Join-Path $RunDirectory 'partial-restoration-result.json')){throw 'Partial restoration already attempted'}
$taskFailure=Get-Content -LiteralPath (Join-Path $RunDirectory 'profile-isolation-failure.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskPlan=Get-Content -LiteralPath (Join-Path $RunDirectory 'profile-isolation-plan.json') -Raw -Encoding utf8 | ConvertFrom-Json
if($taskFailure.completedMoves.Count -ne 2 -or
   @($taskFailure.completedMoves.name | Sort-Object) -join ',' -cne 'ordinary-local,ordinary-roaming'){throw 'Unexpected partial move set'}
. (Join-Path $PSScriptRoot 'directory-identity.ps1')
$taskItems=[Collections.Generic.List[object]]::new()
foreach($taskMove in $taskFailure.completedMoves){
    $taskExpected=if($taskMove.name -ceq 'ordinary-roaming'){'C:\Users\15119\AppData\Roaming\com.gzers.opencodex.desktop'}else{'C:\Users\15119\AppData\Local\com.gzers.opencodex.desktop'}
    $taskSaved=[IO.Path]::GetFullPath((Join-Path $RunDirectory ('saved-'+$taskMove.name)))
    if($taskMove.source -cne $taskExpected -or $taskMove.destination -cne $taskSaved -or
       !$taskSaved.StartsWith($taskExact+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Partial restore scope differs'}
    if(Test-Path -LiteralPath $taskExpected){throw 'Preserve state created after the failed preparation'}
    $taskBefore=Get-RecoveryDirectoryIdentity -Path $taskSaved
    if($taskBefore.creationFileTime -cne $taskMove.originalCreationFileTime){throw 'Saved directory creation differs'}
    $taskItems.Add(@{move=$taskMove;before=$taskBefore})
}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Require native0'}
foreach($taskItem in $taskItems){
    # The exact source and destination were validated above; no recursive deletion.
    Move-Item -LiteralPath $taskItem.move.destination -Destination $taskItem.move.source
    $taskAfter=Get-RecoveryDirectoryIdentity -Path $taskItem.move.source
    if($taskAfter.fileId -cne $taskItem.before.fileId -or
       $taskAfter.creationFileTime -cne $taskItem.before.creationFileTime){throw 'Restored directory handle identity differs'}
    $taskItem.after=$taskAfter
}
$taskRoots=@($taskPlan.profiles | ForEach-Object {Get-RecoveryDirectoryIdentity -Path $_.path})
$taskPointer=(Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop').StartupDataRootV1
$taskPointerSha=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($taskPointer))).ToLowerInvariant()
if($taskPointerSha -cne $taskPlan.startupPointerSha256){throw 'Pointer changed during profile-only work'}
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');restoredMoves=$taskItems.ToArray()
    rootsAfterRestoration=$taskRoots;uniqueNativeDirectoryCount=@($taskRoots.fileId | Sort-Object -Unique).Count
    startupPointerUnchangedExact=$true;dataDeleted=$false;productStarted=$false;uiInputInjected=$false} |
    ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $RunDirectory 'partial-restoration-result.json') -Encoding utf8

param([string]$Method,[string]$RunDirectory)
$ErrorActionPreference='Stop'
. 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\smoke\windows-process-token.ps1'
$taskToken=Get-WindowsHarnessToken
Assert-WindowsProcessMode -Observation $taskToken -Mode ordinary
$taskPlan=Join-Path $RunDirectory 'execution-plan.json'
$taskBefore=(Get-FileHash -LiteralPath $taskPlan).Hash.ToLowerInvariant()
$taskRejected=$false
$taskException=$null
try {& $Method -RunDirectory $RunDirectory -Execute}catch{
    if($_.Exception.Message -cne 'Application process token does not match requested mode: administrator'){throw}
    $taskRejected=$true
    $taskException=$_.Exception.GetType().FullName
}
if(!$taskRejected){throw 'Ordinary token unexpectedly admitted execution'}
if((Test-Path -LiteralPath (Join-Path $RunDirectory 'execution-attempt.json')) -or
   (Test-Path -LiteralPath (Join-Path $RunDirectory 'execution-result.json')) -or
   (Test-Path -LiteralPath (Join-Path $RunDirectory 'install with spaces')) -or
   (Test-Path -LiteralPath (Join-Path $RunDirectory 'human-window.json'))){throw 'Refusal created an execution side effect'}
if((Get-FileHash -LiteralPath $taskPlan).Hash.ToLowerInvariant() -cne $taskBefore){throw 'Preparation plan changed'}
$taskDaily='D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe'
$taskOld=Get-ItemProperty -LiteralPath 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\{7069DA97-F262-4585-98CA-AE416E6833A5}'
if($taskOld.DisplayVersion -cne '0.1.9' -or
   (Get-FileHash -LiteralPath $taskDaily).Hash.ToLowerInvariant() -cne '43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67'){throw 'Prior installation changed'}
$taskNative=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count
if($taskNative){throw 'Native product exists'}
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');actualHarnessToken=$taskToken
    executionRejectedForOrdinaryToken=$true;exceptionType=$taskException
    executionAttemptCreated=$false;executionResultCreated=$false;humanWindowCreated=$false;installDirectoryCreated=$false
    preparedPlanSha256=$taskBefore;preparedPlanUnchanged=$true;priorRegistered019Unchanged=$true
    priorExecutableSha256=(Get-FileHash -LiteralPath $taskDaily).Hash.ToLowerInvariant()
    nativeCount=$taskNative;installerExecuted=$false;productStarted=$false;administratorExecutionTested=$false
    actualApplicationRootBindingTested=$false;fullInstallerGatePassed=$false} | ConvertTo-Json -Depth 7 |
    Set-Content -LiteralPath (Join-Path $RunDirectory 'ordinary-execution-refusal.original.json') -Encoding utf8

param([Parameter(Mandatory)][ValidateSet('custom-first','custom-restart')][string]$Phase)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'run-binding.ps1')
$taskRun=Assert-PreparedIdleRun -RunDirectory $PSScriptRoot -RequireCurrentHumanWindow
if((Test-Path -LiteralPath (Join-Path $taskRun ($Phase+'-launch.json'))) -or
   @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Fresh phase/native0 required'}
$taskContract=Get-Content -LiteralPath (Join-Path $taskRun 'contract.json') -Raw | ConvertFrom-Json
. 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\smoke\windows-process-token.ps1'
. $taskContract.sidMethodPath
. (Join-Path $taskRun 'retained-process-times.ps1')
$taskInfo=[Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName=$taskContract.recoveryExecutable
$taskInfo.WorkingDirectory=$taskContract.recoveryDirectory
$taskInfo.UseShellExecute=$false
foreach($taskName in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY','OPENCODEX_WINDOWS_SMOKE_CDP_PORT','OPENCODEX_WINDOWS_SMOKE_SCALE','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','WEBVIEW2_USER_DATA_FOLDER')){[void]$taskInfo.Environment.Remove($taskName)}
$taskProcess=[Diagnostics.Process]::Start($taskInfo)
try{
    [void]$taskProcess.SafeHandle
    $taskToken=Get-AcceptanceProcessSidEvidence -Process $taskProcess -ExpectedExecutable $taskInfo.FileName
    Assert-WindowsProcessMode -Observation $taskToken -Mode ordinary
    $taskCreated=([datetimeoffset]$taskToken.creationTimeUtc).UtcDateTime.ToFileTimeUtc()
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');phase=$Phase;source=$taskContract.candidateSource;executable=$taskInfo.FileName;exeSha256=$taskContract.candidateExeSha256;pid=$taskProcess.Id;creationTimeUtc=$taskToken.creationTimeUtc;actualToken=$taskToken;sandboxUsed=$false;cdpUsed=$false;retainedHandle=$true;uiInputInjected=$false} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskRun ($Phase+'-launch.json')) -Encoding utf8
    $taskHuman=Get-Content -LiteralPath (Join-Path $taskRun 'human-window.json') -Raw | ConvertFrom-Json
    $taskDuration=(([datetimeoffset]$taskHuman.expiresAtUtc).AddSeconds(60)-[datetimeoffset]::UtcNow).TotalMilliseconds
    if($taskDuration -le 0){throw 'Window expired'}
    if(!$taskProcess.WaitForExit([int]$taskDuration)){throw 'Normal close not observed; do not terminate or retry'}
    $taskTimes=Get-WindowsRetainedTimesEvidence -Process $taskProcess -ExpectedCreationFileTime $taskCreated -RequireExited
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');phase=$Phase;pid=$taskProcess.Id;kernelTimes=$taskTimes;exitCode=$taskProcess.ExitCode;nativeCount=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count;forcedTermination=$false;uiInputInjected=$false} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskRun ($Phase+'-exit.json')) -Encoding utf8
}finally{$taskProcess.Dispose()}

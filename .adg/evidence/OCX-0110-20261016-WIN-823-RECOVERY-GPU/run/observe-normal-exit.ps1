param([string]$RunDirectory)
$ErrorActionPreference='Stop'
$taskLaunch=Get-Content -LiteralPath (Join-Path $RunDirectory 'recovery-launch.json') -Raw | ConvertFrom-Json
. 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261016-WIN-823-FRONT-PREP\retained-process-times.ps1'
$taskProcess=[Diagnostics.Process]::GetProcessById([int]$taskLaunch.pid)
try {
    [void]$taskProcess.SafeHandle
    if($taskProcess.MainModule.FileName -cne $taskLaunch.executable){throw 'Observer image differs'}
    $taskCreated=([datetimeoffset]$taskLaunch.creationTimeUtc).UtcDateTime.ToFileTimeUtc()
    $taskBefore=Get-WindowsRetainedTimesEvidence -Process $taskProcess -ExpectedCreationFileTime $taskCreated
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');launchSha256=(Get-FileHash -LiteralPath (Join-Path $RunDirectory 'recovery-launch.json')).Hash.ToLowerInvariant();before=$taskBefore;productInputInjected=$false} | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $RunDirectory 'normal-exit-observer-ready.json') -Encoding utf8
    if(!$taskProcess.WaitForExit(300000)){throw 'Bound normal close not observed in 300 seconds'}
    $taskExit=Get-WindowsRetainedTimesEvidence -Process $taskProcess -ExpectedCreationFileTime $taskCreated -RequireExited
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');kernelTimes=$taskExit;exitCode=$taskProcess.ExitCode;nativeCount=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count;forceTerminated=$false;uiInputInjected=$false} | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $RunDirectory 'normal-exit.original.json') -Encoding utf8
} catch {
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');exceptionType=$_.Exception.GetType().FullName;message=$_.Exception.Message;forceTerminated=$false} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $RunDirectory 'normal-exit-observer.failure.json') -Encoding utf8
    throw
} finally {$taskProcess.Dispose()}

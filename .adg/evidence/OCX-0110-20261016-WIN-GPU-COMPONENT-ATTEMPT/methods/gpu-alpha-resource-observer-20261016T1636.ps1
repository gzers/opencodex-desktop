param([int]$BrowserProcessId,[string]$BrowserExecutable,[string]$OutputDirectory)
$ErrorActionPreference='Stop'
$taskBoundary='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime'
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
if(!$taskOutput.StartsWith($taskBoundary+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned output required'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Require native0'}
. 'C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop\test\smoke\windows-gpu-resource.ps1'
. 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261016-WIN-823-FRONT-PREP\retained-process-times.ps1'
$taskBrowser=[Diagnostics.Process]::GetProcessById($BrowserProcessId)
try {
    [void]$taskBrowser.SafeHandle
    if($taskBrowser.MainModule.FileName -cne $BrowserExecutable){throw 'Headless root image differs'}
    $taskLaunch=[ordered]@{pid=$taskBrowser.Id;creationTimeUtc=$taskBrowser.StartTime.ToUniversalTime().ToString('o')
        executable=$BrowserExecutable;exeSha256=(Get-FileHash -LiteralPath $BrowserExecutable).Hash.ToLowerInvariant()
        productStarted=$false;controlledHeadlessBrowserOnly=$true}
    $taskLaunch | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskOutput 'launch.original.json') -Encoding utf8
    $taskSnapshot=Get-WindowsBoundGpuResourceSnapshot -Launch $taskLaunch -Mode ordinary
    $taskSnapshot | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $taskOutput 'gpu-snapshot.original.json') -Encoding utf8
    $taskCreated=([datetimeoffset]$taskLaunch.creationTimeUtc).UtcDateTime.ToFileTimeUtc()
    $taskBefore=Get-WindowsRetainedTimesEvidence -Process $taskBrowser -ExpectedCreationFileTime $taskCreated
    $taskBefore | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskOutput 'exit-observer-ready.original.json') -Encoding utf8
    'ready' | Set-Content -LiteralPath (Join-Path $taskOutput 'snapshot-ready') -Encoding utf8
    if(!$taskBrowser.WaitForExit(45000)){throw 'Headless root did not close within 45s'}
    $taskExit=Get-WindowsRetainedTimesEvidence -Process $taskBrowser -ExpectedCreationFileTime $taskCreated -RequireExited
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');kernelTimes=$taskExit;exitCode=$taskBrowser.ExitCode
        nativeCount=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count
        productStarted=$false;observerStartedOrStoppedProcesses=$false} | ConvertTo-Json -Depth 8 |
        Set-Content -LiteralPath (Join-Path $taskOutput 'normal-exit.original.json') -Encoding utf8
}catch{
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');exceptionType=$_.Exception.GetType().FullName
        message=$_.Exception.Message;productStarted=$false;originalLibraryExecuted=$false} | ConvertTo-Json |
        Set-Content -LiteralPath (Join-Path $taskOutput 'observer-failure.json') -Encoding utf8
    throw
}finally{$taskBrowser.Dispose()}

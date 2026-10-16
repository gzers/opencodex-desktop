param([int]$BrowserProcessId,[string]$BrowserExecutable,[string]$OutputDirectory)
$ErrorActionPreference='Stop'
$taskAllowed='C:\Users\15119\.codex\worktrees\windows-working-set\opencodex-desktop\test\runtime\memory-trace-method-20261016T2252\controlled'
if([IO.Path]::GetFullPath($OutputDirectory) -cne $taskAllowed){throw 'Exact controlled output required'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Require native0'}
. 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261016-WIN-B4-FRONT-PREP\retained-process-times.ps1'
$taskProcess=[Diagnostics.Process]::GetProcessById($BrowserProcessId)
try{
    [void]$taskProcess.SafeHandle
    if($taskProcess.MainModule.FileName -cne $BrowserExecutable){throw 'Controlled root image differs'}
    $taskCreation=$taskProcess.StartTime.ToUniversalTime().ToFileTimeUtc()
    [ordered]@{pid=$BrowserProcessId;creationTimeUtc=$taskProcess.StartTime.ToUniversalTime().ToString('o')
        executable=$BrowserExecutable;exeSha256=(Get-FileHash -LiteralPath $BrowserExecutable).Hash.ToLowerInvariant()
        kernel=Get-WindowsRetainedTimesEvidence -Process $taskProcess -ExpectedCreationFileTime $taskCreation
        productStarted=$false;observerBoundBeforeTrace=$true} |
        ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $OutputDirectory 'root-ready.original.json') -Encoding utf8
    [Console]::Out.WriteLine('ready')
    if(!$taskProcess.WaitForExit(90000)){throw 'Controlled root exit observation expired'}
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$BrowserProcessId
        exitCode=$taskProcess.ExitCode;kernel=Get-WindowsRetainedTimesEvidence -Process $taskProcess -ExpectedCreationFileTime $taskCreation -RequireExited
        nativeCount=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count
        forcedTermination=$false;productStarted=$false} | ConvertTo-Json -Depth 7 |
        Set-Content -LiteralPath (Join-Path $OutputDirectory 'root-exit.original.json') -Encoding utf8
}finally{$taskProcess.Dispose()}

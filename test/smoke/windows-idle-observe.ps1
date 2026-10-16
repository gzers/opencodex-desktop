param(
    [Parameter(Mandatory)][string]$LaunchPath,
    [Parameter(Mandatory)][string]$ReadinessPath,
    [Parameter(Mandatory)][string]$OutputDirectory
)
$ErrorActionPreference='Stop'
if(!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7){throw 'Windows PowerShell 7 required'}
$taskRepo=(Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../..')).Path
$taskRuntime=[IO.Path]::GetFullPath((Join-Path $taskRepo 'test/runtime'))
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
if(!$taskOutput.StartsWith($taskRuntime+'\',[StringComparison]::OrdinalIgnoreCase) -or
   (Test-Path -LiteralPath $taskOutput)){throw 'Use a fresh task-owned runtime output directory'}
foreach($taskInput in @($LaunchPath,$ReadinessPath)){
    if(!(Test-Path -LiteralPath $taskInput -PathType Leaf) -or
       (Get-Item -LiteralPath $taskInput).Attributes -band [IO.FileAttributes]::ReparsePoint){
        throw 'An existing plain observation input is required'
    }
}
$taskLaunch=Get-Content -LiteralPath $LaunchPath -Raw -Encoding utf8 | ConvertFrom-Json
$taskReady=Get-Content -LiteralPath $ReadinessPath -Raw -Encoding utf8 | ConvertFrom-Json
if($taskLaunch.source -notmatch '^[0-9a-f]{40}$' -or
   $taskLaunch.exeSha256 -notmatch '^[0-9a-f]{64}$' -or
   (Get-FileHash -LiteralPath $taskLaunch.executable).Hash.ToLowerInvariant() -cne $taskLaunch.exeSha256){
    throw 'Explicit source and exact executable digest required'
}
if($taskLaunch.sandboxUsed -or $taskLaunch.cdpUsed){throw 'This idle subset excludes sandbox/CDP instrumentation'}
if($taskReady.accessibility.tree -notmatch '尚未接入 OpenCodex' -or
   ([datetimeoffset]$taskReady.observedAtUtc-[datetimeoffset]$taskLaunch.creationTimeUtc).TotalSeconds -lt 60){
    throw 'Observed preparation scene and 60 real seconds of settling required'
}
$taskProcess=Get-Process -Id ([int]$taskLaunch.pid)
if($taskProcess.StartTime.ToUniversalTime().Ticks -ne ([datetimeoffset]$taskLaunch.creationTimeUtc).UtcTicks){
    throw 'Root process generation changed'
}
. (Join-Path $PSScriptRoot 'windows-process-token.ps1')
$taskToken=Get-WindowsProcessToken -Process $taskProcess -ExpectedExecutable $taskLaunch.executable
Assert-WindowsProcessMode -Observation $taskToken -Mode ordinary
Add-Type -TypeDefinition @'
using System;using System.Runtime.InteropServices;
public static class IdleObservationWindow {
 [StructLayout(LayoutKind.Sequential)]public struct Rect{public int Left,Top,Right,Bottom;}
 [DllImport("user32.dll")]public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")]public static extern bool IsWindowVisible(IntPtr w);
 [DllImport("user32.dll")]public static extern bool IsIconic(IntPtr w);
 [DllImport("user32.dll")]public static extern uint GetWindowThreadProcessId(IntPtr w,out uint p);
 [DllImport("user32.dll")]public static extern bool GetClientRect(IntPtr w,out Rect r);
}
'@
$taskWindow=[IntPtr]$taskReady.window.id
$taskOwner=[uint32]0
[void][IdleObservationWindow]::GetWindowThreadProcessId($taskWindow,[ref]$taskOwner)
$taskRect=[IdleObservationWindow+Rect]::new()
if($taskOwner -ne $taskProcess.Id -or
   ![IdleObservationWindow]::GetClientRect($taskWindow,[ref]$taskRect) -or
   $taskRect.Right-$taskRect.Left -ne 2360 -or $taskRect.Bottom-$taskRect.Top -ne 1520){
    throw 'Observed window owner/client dimensions differ'
}
. (Join-Path $PSScriptRoot 'windows-idle-family.ps1')
New-Item -ItemType Directory -Path $taskOutput | Out-Null
$taskFailurePath=Join-Path $taskOutput 'observation-failures.original.jsonl'
$taskOnFailure={
    param($Record)
    [IO.File]::AppendAllText($taskFailurePath,($Record | ConvertTo-Json -Depth 7 -Compress)+"`n",
        [Text.UTF8Encoding]::new($false))
}
$taskState=New-WindowsIdleFamily -RootProcess $taskProcess -OnFailure $taskOnFailure
$taskSelf=[Diagnostics.Process]::GetCurrentProcess()
$taskCpuBefore=$taskSelf.TotalProcessorTime.TotalSeconds
$taskSamples=0
$taskFirst=$null
$taskLast=$null
$taskComplete=$false
$taskTimer=[Diagnostics.Stopwatch]::StartNew()
$taskStream=[IO.File]::Open((Join-Path $taskOutput 'samples.original.jsonl'),
    [IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
$taskWriter=[IO.StreamWriter]::new($taskStream,[Text.UTF8Encoding]::new($false))
[ordered]@{
    source=$taskLaunch.source;exeSha256=$taskLaunch.exeSha256;pid=$taskProcess.Id
    creationTimeUtc=$taskLaunch.creationTimeUtc;actualToken=$taskToken
    launchInputSha256=(Get-FileHash -LiteralPath $LaunchPath).Hash.ToLowerInvariant()
    readinessInputSha256=(Get-FileHash -LiteralPath $ReadinessPath).Hash.ToLowerInvariant()
    moduleSha256=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'windows-idle-family.ps1')).Hash.ToLowerInvariant()
    methodSha256=(Get-FileHash -LiteralPath $PSCommandPath).Hash.ToLowerInvariant()
    minimumCompletedCounterSpanSeconds=600;scene='overview-preparation'
    clientWidth=2360;clientHeight=1520;zeroReadFailuresRequired=$true
    shortLivedChildrenFullyCovered=$false;sharedResidentPagesDeduplicated=$false
    productStartedByCollector=$false;uiInputInjected=$false;forcedTerminationAllowed=$false
} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $taskOutput 'identity.json') -Encoding utf8
try{
    do{
        $taskIteration=[Diagnostics.Stopwatch]::StartNew()
        try {
            $taskRows=@(Get-CimInstance -Query 'SELECT ProcessId,ParentProcessId,CreationDate,Name FROM Win32_Process')
        } catch {
            Stop-WindowsIdleObservation -State $taskState -Stage 'process-snapshot-read' -Failure $_.Exception
        }
        $taskFrame=@{
            Foreground=[IdleObservationWindow]::GetForegroundWindow() -eq $taskWindow
            Visible=[IdleObservationWindow]::IsWindowVisible($taskWindow)
            Minimized=[IdleObservationWindow]::IsIconic($taskWindow)
            NativeCount=@($taskRows | Where-Object Name -in @('opencodex-desktop.exe','ocxd.exe')).Count
        }
        Assert-WindowsIdleFrame -State $taskState @taskFrame
        $taskCounters=Get-WindowsIdleFamilySample -State $taskState -Rows $taskRows
        # A focus change while reading counters must invalidate this iteration,
        # rather than leave a passing sample with an old frame observation.
        $taskFrame.Foreground=[IdleObservationWindow]::GetForegroundWindow() -eq $taskWindow
        $taskFrame.Visible=[IdleObservationWindow]::IsWindowVisible($taskWindow)
        $taskFrame.Minimized=[IdleObservationWindow]::IsIconic($taskWindow)
        Assert-WindowsIdleFrame -State $taskState @taskFrame
        $taskLast=$taskTimer.Elapsed.TotalSeconds
        if($null -eq $taskFirst){$taskFirst=$taskLast}
        $taskWriter.WriteLine(([ordered]@{
            atUtc=[datetimeoffset]::UtcNow.ToString('o');elapsedSeconds=$taskLast
            cpuSeconds=$taskCounters.cpuSeconds;workingSetBytes=$taskCounters.workingSetBytes
            members=$taskCounters.members;foreground=$taskFrame.Foreground
            visible=$taskFrame.Visible;minimized=$taskFrame.Minimized;nativeCount=$taskFrame.NativeCount
        } | ConvertTo-Json -Depth 7 -Compress))
        $taskWriter.Flush()
        $taskSamples++
        if(($taskLast-$taskFirst) -ge 600){$taskComplete=$true;break}
        if($taskSamples % 10 -eq 0){
            [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');phase='measuring'
                samples=$taskSamples;counterSpanSeconds=$taskLast-$taskFirst;observationInvalid=$false} |
                ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskOutput 'progress.json') -Encoding utf8
        }
        Start-Sleep -Milliseconds ([Math]::Max(1,1000-[int]$taskIteration.ElapsedMilliseconds))
    }while($true)
}catch{
    $_ | Out-String | Set-Content -LiteralPath (Join-Path $taskOutput 'collector-error.original.txt') -Encoding utf8
    throw
}finally{
    $taskWriter.Dispose()
    $taskStream.Dispose()
    $taskSelf.Refresh()
    # Terminal and progress agree immediately on all exits; stale periodic
    # progress can no longer be mistaken for a live or valid observation.
    try {
        Write-WindowsIdleTerminal -State $taskState -OutputDirectory $taskOutput -SampleCount $taskSamples `
            -FirstElapsed $taskFirst -LastElapsed $taskLast -Completed $taskComplete `
            -CollectorCpuSeconds ($taskSelf.TotalProcessorTime.TotalSeconds-$taskCpuBefore) | Out-Null
    } finally {
        Close-WindowsIdleFamily -State $taskState
        $taskSelf.Dispose()
        $taskProcess.Dispose()
    }
}

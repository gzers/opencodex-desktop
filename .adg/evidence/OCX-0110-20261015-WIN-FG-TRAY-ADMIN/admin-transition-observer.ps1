$ErrorActionPreference = 'Stop'
$taskIdentity = [Security.Principal.WindowsIdentity]::GetCurrent()
$taskPrincipal = [Security.Principal.WindowsPrincipal]::new($taskIdentity)
if (!$taskPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'Administrator read-only observer required.' }
$taskOutput = Join-Path $PSScriptRoot 'administrator-window-transitions.original.jsonl'
if (Test-Path -LiteralPath $taskOutput) { throw 'Transition evidence already exists.' }
try {
    & (Join-Path $PSScriptRoot 'read-state.ps1') -Phase transition-observer-start -Mode administrator `
        1> (Join-Path $PSScriptRoot 'administrator-transition-start.stdout.original.txt') `
        2> (Join-Path $PSScriptRoot 'administrator-transition-start.stderr.original.txt')
    $taskFirst = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-first-screen.json') -Raw | ConvertFrom-Json
    $taskLaunch = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-launch.json') -Raw | ConvertFrom-Json
    $taskBoundProcess = Get-Process -Id $taskLaunch.pid
    if ($taskBoundProcess.StartTime.ToUniversalTime().Ticks -ne ([datetimeoffset]$taskLaunch.creationTimeUtc).UtcDateTime.Ticks) { throw 'Bound product identity changed.' }
    $taskHandle = $taskBoundProcess.Handle
    $taskBoundWindow = [IntPtr]::new([long]$taskFirst.hwnd)
    $taskTimer = [Diagnostics.Stopwatch]::StartNew()
    $taskWriter = [IO.StreamWriter]::new($taskOutput,$false,[Text.UTF8Encoding]::new($false))
    $taskWriter.AutoFlush = $true
    $taskSamples = 0
    while ($taskTimer.Elapsed.TotalSeconds -lt 240 -and !$taskBoundProcess.HasExited) {
        $taskOwnerPid = [uint32]0
        [void][ForegroundGateReadOnly]::GetWindowThreadProcessId($taskBoundWindow,[ref]$taskOwnerPid)
        if ($taskOwnerPid -ne $taskLaunch.pid) { throw 'Bound main HWND owner changed.' }
        $taskOuter = [ForegroundGateReadOnly+Rect]::new()
        $taskClient = [ForegroundGateReadOnly+Rect]::new()
        [void][ForegroundGateReadOnly]::GetWindowRect($taskBoundWindow,[ref]$taskOuter)
        [void][ForegroundGateReadOnly]::GetClientRect($taskBoundWindow,[ref]$taskClient)
        $taskBackdrop = 0
        $taskBackdropHresult = [ForegroundGateReadOnly]::DwmGetWindowAttribute($taskBoundWindow,38,[ref]$taskBackdrop,4)
        $taskRow = [ordered]@{
            atUtc=[DateTime]::UtcNow.ToString('o'); elapsedMs=$taskTimer.Elapsed.TotalMilliseconds
            pid=$taskLaunch.pid; creationTimeUtc=$taskLaunch.creationTimeUtc; hwnd=$taskBoundWindow.ToInt64()
            visible=[ForegroundGateReadOnly]::IsWindowVisible($taskBoundWindow)
            maximized=[ForegroundGateReadOnly]::IsZoomed($taskBoundWindow)
            minimized=[ForegroundGateReadOnly]::IsIconic($taskBoundWindow)
            foreground=([ForegroundGateReadOnly]::GetForegroundWindow() -eq $taskBoundWindow)
            outerRect=$taskOuter; clientRect=$taskClient; backdropHresult=$taskBackdropHresult; backdropRawValue=$taskBackdrop
            inputInjected=$false; settingsChanged=$false
        }
        $taskWriter.WriteLine(($taskRow | ConvertTo-Json -Depth 5 -Compress))
        $taskSamples++
        Start-Sleep -Milliseconds 1000
    }
    $taskWriter.Dispose()
    [ordered]@{
        atUtc=[DateTime]::UtcNow.ToString('o'); samples=$taskSamples
        pid=$taskLaunch.pid; elapsedSeconds=$taskTimer.Elapsed.TotalSeconds
        productExited=$taskBoundProcess.HasExited; forcedTermination=$false; inputInjected=$false
        reason='bounded user-assisted administrator window transition observation'
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-transition-observer-result.json') -Encoding utf8
    $taskBoundProcess.Dispose()
} catch {
    if ($taskWriter) { $taskWriter.Dispose() }
    $_ | Out-String | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-transition-observer-error.original.txt') -Encoding utf8
} finally {
    $taskIdentity.Dispose()
}

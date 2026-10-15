param([Parameter(Mandatory=$true)][ValidatePattern('^[a-z][a-z0-9-]{0,63}$')][string]$Phase, [ValidateSet('ordinary','administrator')][string]$Mode='ordinary')
$ErrorActionPreference = 'Stop'
$taskLaunch = Get-Content -LiteralPath (Join-Path $PSScriptRoot "$Mode-launch.json") -Raw | ConvertFrom-Json
$taskProcess = Get-Process -Id $taskLaunch.pid
if ($taskProcess.StartTime.ToUniversalTime().Ticks -ne ([datetimeoffset]$taskLaunch.creationTimeUtc).UtcDateTime.Ticks) {
    throw 'PID reuse; do not read or act on the new process'
}
. 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\smoke\windows-process-token.ps1'
$taskToken = Get-WindowsProcessToken -Process $taskProcess -ExpectedExecutable $taskLaunch.executable
if (([datetimeoffset]$taskToken.creationTimeUtc).UtcDateTime.Ticks -ne
    ([datetimeoffset]$taskLaunch.applicationToken.creationTimeUtc).UtcDateTime.Ticks) {
    throw 'Retained native process creation time changed'
}
if (!('ForegroundGateReadOnly' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class ForegroundGateReadOnly {
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left,Top,Right,Bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr window,out Rect rect);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr window,out Rect rect);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr window);
    [DllImport("user32.dll")] public static extern bool IsZoomed(IntPtr window);
    [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr window);
    [DllImport("user32.dll")] public static extern IntPtr GetMenu(IntPtr window);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window,out uint pid);
    [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr window,StringBuilder text,int size);
    [DllImport("user32.dll")] public static extern int GetSystemMetrics(int index);
    [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr window,uint attribute,out int value,int size);
}
'@
}
$taskProcess.Refresh()
$taskWindow = $taskProcess.MainWindowHandle
if (Test-Path -LiteralPath (Join-Path $PSScriptRoot "$Mode-first-screen.json")) {
    # When the main window is hidden, Process.MainWindowHandle can select a
    # tiny visible tray-helper HWND instead. Keep the previously bound main
    # HWND and still verify its owning PID below.
    $taskPrior = Get-Content -LiteralPath (Join-Path $PSScriptRoot "$Mode-first-screen.json") -Raw | ConvertFrom-Json
    $taskWindow = [IntPtr]::new([long]$taskPrior.hwnd)
}
$taskWindowPid = [uint32]0
[void][ForegroundGateReadOnly]::GetWindowThreadProcessId($taskWindow,[ref]$taskWindowPid)
if ($taskWindow -eq [IntPtr]::Zero -or $taskWindowPid -ne $taskProcess.Id) {
    throw 'The observed HWND is missing or does not belong to the bound process'
}
$taskOuter = [ForegroundGateReadOnly+Rect]::new()
$taskInner = [ForegroundGateReadOnly+Rect]::new()
$taskTitle = [Text.StringBuilder]::new(512)
$taskBackdrop = 0; $taskDarkMode = 0
$taskRectValid = [ForegroundGateReadOnly]::GetWindowRect($taskWindow,[ref]$taskOuter)
$taskClientValid = [ForegroundGateReadOnly]::GetClientRect($taskWindow,[ref]$taskInner)
[void][ForegroundGateReadOnly]::GetWindowTextW($taskWindow,$taskTitle,512)
$taskBackdropResult = [ForegroundGateReadOnly]::DwmGetWindowAttribute($taskWindow,38,[ref]$taskBackdrop,4)
$taskDarkResult = [ForegroundGateReadOnly]::DwmGetWindowAttribute($taskWindow,20,[ref]$taskDarkMode,4)
$taskState = [ordered]@{
    atUtc = [DateTime]::UtcNow.ToString('o'); phase = $Phase
    pid = $taskProcess.Id; creationTimeUtc = $taskLaunch.creationTimeUtc
    applicationToken = $taskToken; hwnd = $taskWindow.ToInt64()
    title = $taskTitle.ToString(); outerRectValid = $taskRectValid; outerRect = $taskOuter
    clientRectValid = $taskClientValid; clientRect = $taskInner
    visible = [ForegroundGateReadOnly]::IsWindowVisible($taskWindow)
    maximized = [ForegroundGateReadOnly]::IsZoomed($taskWindow)
    minimized = [ForegroundGateReadOnly]::IsIconic($taskWindow)
    nativeDpi = [ForegroundGateReadOnly]::GetDpiForWindow($taskWindow)
    foreground = [ForegroundGateReadOnly]::GetForegroundWindow() -eq $taskWindow
    applicationMenuAttached = [ForegroundGateReadOnly]::GetMenu($taskWindow) -ne [IntPtr]::Zero
    remoteSessionMetric = [ForegroundGateReadOnly]::GetSystemMetrics(0x1000)
    backdropHresult = $taskBackdropResult; backdropRawValue = $taskBackdrop
    darkModeHresult = $taskDarkResult; darkModeRawValue = $taskDarkMode
    physicalDpiGateClaimed = $false; forcedScaleUsed = $false
    inputInjectedByThisReader = $false; settingsChangedByThisReader = $false
}
$taskPath = Join-Path $PSScriptRoot ("$Mode-$Phase.json")
if (Test-Path -LiteralPath $taskPath) { throw 'Phase evidence already exists; use a fresh phase identifier' }
$taskState | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $taskPath -Encoding utf8
$taskState | ConvertTo-Json -Depth 10

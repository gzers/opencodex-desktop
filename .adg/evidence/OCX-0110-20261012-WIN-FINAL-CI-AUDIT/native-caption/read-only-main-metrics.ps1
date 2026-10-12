param([long]$WindowHandle=0)
$ErrorActionPreference='Stop'
$taskBase='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\windows0110'
$taskSession=Get-Content -LiteralPath (Join-Path $taskBase 'control/current-session.json') -Raw | ConvertFrom-Json -DateKind String
if ($taskSession.status -ne 'running') { throw 'No running isolated session' }
$taskProcess=Get-Process -Id $taskSession.applicationPid -ErrorAction Stop
if ($taskProcess.Path -cne $taskSession.executable -or
    $taskProcess.StartTime.ToUniversalTime() -ne [datetime]::Parse($taskSession.applicationToken.creationTimeUtc).ToUniversalTime()) {
    throw 'Application process identity differs from the retained supervisor'
}
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class OcxReadOnlyWindow {
 [StructLayout(LayoutKind.Sequential)] public struct Rect {public int Left,Top,Right,Bottom;}
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr window,out Rect rectangle);
 [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr window);
 [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr window);
 [DllImport("user32.dll")] public static extern bool IsZoomed(IntPtr window);
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr window);
 [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr window);
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window,out uint process);
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr window,uint attribute,out int value,int size);
}
'@
$taskWindow=if($WindowHandle){[IntPtr]$WindowHandle}else{$taskProcess.MainWindowHandle}
if (!$taskWindow) { throw 'Isolated application exposes no main window' }
$taskWindowProcess=0u
if (![OcxReadOnlyWindow]::IsWindow($taskWindow) -or
    ![OcxReadOnlyWindow]::GetWindowThreadProcessId($taskWindow,[ref]$taskWindowProcess) -or
    $taskWindowProcess -ne $taskProcess.Id) {throw 'Window is not owned by the retained application'}
$taskRect=[OcxReadOnlyWindow+Rect]::new()
if (![OcxReadOnlyWindow]::GetWindowRect($taskWindow,[ref]$taskRect)) {throw 'Window geometry unavailable'}
$taskBackdrop=0
$taskDwmStatus=[OcxReadOnlyWindow]::DwmGetWindowAttribute($taskWindow,38,[ref]$taskBackdrop,4)
[ordered]@{
    collectedAtUtc=[datetime]::UtcNow.ToString('o');sourceCommit=$taskSession.sourceCommit
    applicationPid=$taskProcess.Id;creationTimeUtc=$taskProcess.StartTime.ToUniversalTime().ToString('o')
    executableSha256=$taskSession.executableSha256;hwnd=$taskWindow.ToInt64()
    windowIdentitySource=if($WindowHandle){'explicit previously selected main HWND; owner PID verified'}else{'Process.MainWindowHandle heuristic; verify geometry before acceptance'}
    dpi=[OcxReadOnlyWindow]::GetDpiForWindow($taskWindow)
    visible=[OcxReadOnlyWindow]::IsWindowVisible($taskWindow)
    minimized=[OcxReadOnlyWindow]::IsIconic($taskWindow)
    maximized=[OcxReadOnlyWindow]::IsZoomed($taskWindow)
    foreground=[OcxReadOnlyWindow]::GetForegroundWindow().ToInt64() -eq $taskWindow.ToInt64()
    windowBounds=$taskRect;backdropAttribute=$taskBackdrop;dwmResult=$taskDwmStatus
    scope='read-only native geometry and DWM observation; no UI Automation or input'
} | ConvertTo-Json -Depth 5

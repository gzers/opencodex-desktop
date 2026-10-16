Add-Type -TypeDefinition @"
using System;using System.Runtime.InteropServices;
public static class TaskObserverReadRect {
[StructLayout(LayoutKind.Sequential)]public struct R {public int Left,Top,Right,Bottom;}
[DllImport("user32.dll")]public static extern bool GetClientRect(IntPtr w,out R r);
[DllImport("user32.dll")]public static extern uint GetDpiForWindow(IntPtr w);
[DllImport("user32.dll")]public static extern IntPtr GetThreadDpiAwarenessContext();
[DllImport("user32.dll")]public static extern int GetAwarenessFromDpiAwarenessContext(IntPtr v);
}
"@
$taskRect=[TaskObserverReadRect+R]::new()
[void][TaskObserverReadRect]::GetClientRect([IntPtr]3084160,[ref]$taskRect)
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');width=$taskRect.Right-$taskRect.Left;height=$taskRect.Bottom-$taskRect.Top;targetDpi=[TaskObserverReadRect]::GetDpiForWindow([IntPtr]3084160);readerAwareness=[TaskObserverReadRect]::GetAwarenessFromDpiAwarenessContext([TaskObserverReadRect]::GetThreadDpiAwarenessContext());compatLayer=$env:__COMPAT_LAYER;uiInputInjected=$false} | ConvertTo-Json

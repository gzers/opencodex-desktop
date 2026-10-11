# Read the retained process handle, never the harness identity or a reopened PID.
function Get-WindowsProcessToken {
    param(
        [Parameter(Mandatory=$true)][System.Diagnostics.Process]$Process,
        [Parameter(Mandatory=$true)][string]$ExpectedExecutable
    )
    if (!$IsWindows) { throw 'Process-token evidence requires Windows and PowerShell 7' }
    if (!('OpenCodexProcessToken' -as [type])) {
        Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;
public sealed class OpenCodexTokenObservation {
    public uint ProcessId { get; set; }
    public string Executable { get; set; }
    public DateTime CreationTimeUtc { get; set; }
    public bool Elevated { get; set; }
    public int ElevationType { get; set; }
}
public static class OpenCodexProcessToken {
    [DllImport("advapi32.dll", SetLastError=true)]
    private static extern bool OpenProcessToken(IntPtr process, uint access, out IntPtr token);
    [DllImport("advapi32.dll", SetLastError=true)]
    private static extern bool GetTokenInformation(IntPtr token, int kind, out int value, int size, out int returned);
    [DllImport("kernel32.dll", SetLastError=true)]
    private static extern bool CloseHandle(IntPtr handle);
    [DllImport("kernel32.dll", SetLastError=true)]
    private static extern uint GetProcessId(IntPtr process);
    [DllImport("kernel32.dll", SetLastError=true)]
    private static extern uint WaitForSingleObject(IntPtr handle, uint milliseconds);
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, ExactSpelling=true, SetLastError=true)]
    private static extern bool QueryFullProcessImageNameW(IntPtr process, uint flags, StringBuilder name, ref uint size);
    [DllImport("kernel32.dll", SetLastError=true)]
    private static extern bool GetProcessTimes(IntPtr process, out long created, out long exited, out long kernel, out long user);
    private static void RequireLive(IntPtr process) {
        uint state = WaitForSingleObject(process, 0);
        if (state == 0xffffffff) throw new Win32Exception(Marshal.GetLastWin32Error());
        if (state != 258) throw new InvalidOperationException("Target process has exited");
    }
    private static int Read(IntPtr token, int kind) {
        int value, returned;
        if (!GetTokenInformation(token, kind, out value, sizeof(int), out returned))
            throw new Win32Exception(Marshal.GetLastWin32Error());
        if (returned != sizeof(int)) throw new InvalidOperationException("Unexpected token information size");
        return value;
    }
    public static OpenCodexTokenObservation Observe(Process process) {
        SafeProcessHandle retained = process.SafeHandle;
        bool referenced = false;
        IntPtr token = IntPtr.Zero;
        try {
            retained.DangerousAddRef(ref referenced);
            IntPtr handle = retained.DangerousGetHandle();
            RequireLive(handle);
            uint pid = GetProcessId(handle);
            if (pid == 0) throw new Win32Exception(Marshal.GetLastWin32Error());
            var name = new StringBuilder(32768);
            uint length = (uint)name.Capacity;
            if (!QueryFullProcessImageNameW(handle, 0, name, ref length))
                throw new Win32Exception(Marshal.GetLastWin32Error());
            long created, exited, kernel, user;
            if (!GetProcessTimes(handle, out created, out exited, out kernel, out user))
                throw new Win32Exception(Marshal.GetLastWin32Error());
            if (!OpenProcessToken(handle, 0x0008, out token))
                throw new Win32Exception(Marshal.GetLastWin32Error());
            int elevated = Read(token, 20), type = Read(token, 18);
            if ((elevated != 0 && elevated != 1) || type < 1 || type > 3)
                throw new InvalidOperationException("Unexpected token elevation values");
            RequireLive(handle);
            return new OpenCodexTokenObservation {
                ProcessId = pid, Executable = name.ToString(),
                CreationTimeUtc = DateTime.FromFileTimeUtc(created),
                Elevated = elevated != 0, ElevationType = type
            };
        } finally {
            if (token != IntPtr.Zero) CloseHandle(token);
            if (referenced) retained.DangerousRelease();
        }
    }
}
'@
    }
    $observation = [OpenCodexProcessToken]::Observe($Process)
    $expected = [IO.Path]::GetFullPath($ExpectedExecutable)
    if (![StringComparer]::OrdinalIgnoreCase.Equals($observation.Executable, $expected)) {
        throw 'Observed process executable does not match the expected executable'
    }
    if ($observation.ProcessId -ne $Process.Id) { throw 'Retained process identity mismatch' }
    [ordered]@{
        schemaVersion = 1; status = 'observed'; processId = $observation.ProcessId
        executable = $observation.Executable; creationTimeUtc = $observation.CreationTimeUtc.ToString('o')
        elevated = $observation.Elevated; elevationType = $observation.ElevationType
        collectedAtUtc = [datetime]::UtcNow.ToString('o')
    }
}

function Get-WindowsHarnessToken {
    $harness = [Diagnostics.Process]::GetCurrentProcess()
    try { Get-WindowsProcessToken -Process $harness -ExpectedExecutable $harness.MainModule.FileName }
    finally { $harness.Dispose() }
}

function Assert-WindowsProcessMode {
    param(
        [Parameter(Mandatory=$true)]$Observation,
        [ValidateSet('any','ordinary','administrator')][string]$Mode = 'any'
    )
    if ($Observation.schemaVersion -ne 1 -or $Observation.status -ne 'observed' -or
        $Observation.processId -le 0 -or $Observation.elevated -isnot [bool] -or
        $Observation.elevationType -notin @(1,2,3)) {
        throw 'Application token evidence is missing or invalid'
    }
    if ($Mode -ne 'any' -and (($Mode -eq 'administrator') -ne $Observation.elevated)) {
        throw "Application process token does not match requested mode: $Mode"
    }
}

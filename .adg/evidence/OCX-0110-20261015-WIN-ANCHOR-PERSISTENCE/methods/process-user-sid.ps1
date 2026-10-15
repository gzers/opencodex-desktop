# Read-only companion to test/smoke/windows-process-token.ps1. Dot-source that
# admitted helper first; this method does not launch an application or inspect
# registry/configuration paths. The SID digest comes from the target token.
function Get-AcceptanceProcessSidEvidence {
    param(
        [Parameter(Mandatory=$true)][Diagnostics.Process]$Process,
        [Parameter(Mandatory=$true)][string]$ExpectedExecutable
    )
    $taskMode = Get-WindowsProcessToken -Process $Process -ExpectedExecutable $ExpectedExecutable
    if (!('OpenCodexAcceptanceProcessUser' -as [type])) {
        Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Security.Principal;
using System.Text;
using Microsoft.Win32.SafeHandles;
public sealed class OpenCodexProcessUserObservation {
    public uint ProcessId { get; set; }
    public DateTime CreationTimeUtc { get; set; }
    public string SidSha256 { get; set; }
}
public static class OpenCodexAcceptanceProcessUser {
    [DllImport("advapi32.dll", SetLastError=true)]
    private static extern bool OpenProcessToken(IntPtr process, uint access, out IntPtr token);
    [DllImport("advapi32.dll", SetLastError=true)]
    private static extern bool GetTokenInformation(IntPtr token, int kind, IntPtr data, uint size, out uint needed);
    [DllImport("advapi32.dll")]
    private static extern uint GetLengthSid(IntPtr sid);
    [DllImport("kernel32.dll", SetLastError=true)]
    private static extern bool CloseHandle(IntPtr handle);
    [DllImport("kernel32.dll", SetLastError=true)]
    private static extern uint GetProcessId(IntPtr process);
    [DllImport("kernel32.dll", SetLastError=true)]
    private static extern uint WaitForSingleObject(IntPtr process, uint milliseconds);
    [DllImport("kernel32.dll", SetLastError=true)]
    private static extern bool GetProcessTimes(IntPtr process, out long created, out long exited, out long kernel, out long user);
    private static void RequireLive(IntPtr process) {
        uint state = WaitForSingleObject(process, 0);
        if (state == 0xffffffff) throw new Win32Exception(Marshal.GetLastWin32Error());
        if (state != 258) throw new InvalidOperationException("Target process has exited");
    }
    public static OpenCodexProcessUserObservation Observe(Process process) {
        SafeProcessHandle retained = process.SafeHandle;
        bool referenced = false;
        IntPtr token = IntPtr.Zero, buffer = IntPtr.Zero;
        try {
            retained.DangerousAddRef(ref referenced);
            IntPtr handle = retained.DangerousGetHandle();
            RequireLive(handle);
            uint pid = GetProcessId(handle);
            if (pid == 0) throw new Win32Exception(Marshal.GetLastWin32Error());
            long created, exited, kernel, user;
            if (!GetProcessTimes(handle, out created, out exited, out kernel, out user))
                throw new Win32Exception(Marshal.GetLastWin32Error());
            if (!OpenProcessToken(handle, 0x0008, out token))
                throw new Win32Exception(Marshal.GetLastWin32Error());
            uint size;
            // TokenUser=1. Query size before allocating bounded native memory.
            GetTokenInformation(token, 1, IntPtr.Zero, 0, out size);
            if (size < (uint)IntPtr.Size || size > 65536)
                throw new InvalidOperationException("TokenUser size is invalid");
            buffer = Marshal.AllocHGlobal((int)size);
            uint returned;
            if (!GetTokenInformation(token, 1, buffer, size, out returned))
                throw new Win32Exception(Marshal.GetLastWin32Error());
            if (returned > size || returned < (uint)IntPtr.Size)
                throw new InvalidOperationException("TokenUser changed size");
            IntPtr sid = Marshal.ReadIntPtr(buffer);
            long offset = sid.ToInt64() - buffer.ToInt64();
            if (offset < IntPtr.Size || offset > returned - 8)
                throw new InvalidOperationException("TokenUser SID is outside the returned buffer");
            uint length = GetLengthSid(sid);
            if (length < 8 || length > 256 || offset + length > returned)
                throw new InvalidOperationException("TokenUser SID length is invalid");
            string value = new SecurityIdentifier(sid).Value;
            string digest = Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(value))).ToLowerInvariant();
            RequireLive(handle);
            return new OpenCodexProcessUserObservation {
                ProcessId=pid, CreationTimeUtc=DateTime.FromFileTimeUtc(created), SidSha256=digest
            };
        } finally {
            if (buffer != IntPtr.Zero) Marshal.FreeHGlobal(buffer);
            if (token != IntPtr.Zero) CloseHandle(token);
            if (referenced) retained.DangerousRelease();
        }
    }
}
'@
    }
    $taskUser = [OpenCodexAcceptanceProcessUser]::Observe($Process)
    if ($taskUser.ProcessId -ne $taskMode.processId -or
        $taskUser.CreationTimeUtc.Ticks -ne ([datetimeoffset]$taskMode.creationTimeUtc).UtcDateTime.Ticks -or
        $taskUser.SidSha256 -cnotmatch '^[0-9a-f]{64}$') {
        throw 'Target token and process-generation observations do not match'
    }
    [ordered]@{
        schemaVersion=1;status='observed';processId=$taskMode.processId
        creationTimeUtc=$taskMode.creationTimeUtc;executable=$taskMode.executable
        elevated=$taskMode.elevated;elevationType=$taskMode.elevationType
        sidSha256=$taskUser.SidSha256;sidSource='GetTokenInformation(TokenUser), retained target process handle'
        rawSidPublished=$false;collectedAtUtc=[datetimeoffset]::UtcNow.ToString('o')
        sourceMethodSha256=(Get-FileHash -LiteralPath $PSCommandPath).Hash.ToLowerInvariant()
        nativeRootAcceptanceClaimed=$false;launchPermissionGranted=$false
    }
}

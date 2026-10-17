# Read-only kernel times for a retained process generation; never reopen its PID.
$script:taskRetainedTimesMethodPath=$PSCommandPath
if(!('OpenCodexRetainedTimesV1' -as [type])){
Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;
public sealed class OpenCodexRetainedTimesObservation {
    public uint ProcessId { get; set; }
    public long CreationFileTime { get; set; }
    public long ExitFileTime { get; set; }
    public bool HasExited { get; set; }
}
public static class OpenCodexRetainedTimesV1 {
    [DllImport("kernel32.dll", SetLastError=true)]
    private static extern uint GetProcessId(IntPtr process);
    [DllImport("kernel32.dll", SetLastError=true)]
    private static extern uint WaitForSingleObject(IntPtr process,uint milliseconds);
    [DllImport("kernel32.dll", SetLastError=true)]
    private static extern bool GetProcessTimes(IntPtr process,out long created,out long exited,out long kernel,out long user);
    public static OpenCodexRetainedTimesObservation Observe(Process process) {
        SafeProcessHandle retained=process.SafeHandle;
        bool referenced=false;
        try {
            retained.DangerousAddRef(ref referenced);
            IntPtr handle=retained.DangerousGetHandle();
            uint pid=GetProcessId(handle);
            if(pid==0)throw new Win32Exception(Marshal.GetLastWin32Error());
            if(pid!=process.Id)throw new InvalidOperationException("Retained identity changed");
            uint state=WaitForSingleObject(handle,0);
            if(state==0xffffffff)throw new Win32Exception(Marshal.GetLastWin32Error());
            if(state!=0 && state!=258)throw new InvalidOperationException("Unexpected wait state");
            long created,exited,kernel,user;
            if(!GetProcessTimes(handle,out created,out exited,out kernel,out user))
                throw new Win32Exception(Marshal.GetLastWin32Error());
            bool hasExited=state==0;
            if(created<=0 || (hasExited && exited<created))
                throw new InvalidOperationException("Kernel lifetime cannot establish ordering");
            return new OpenCodexRetainedTimesObservation {
                ProcessId=pid,CreationFileTime=created,ExitFileTime=exited,HasExited=hasExited
            };
        } finally {
            if(referenced)retained.DangerousRelease();
        }
    }
}
'@
}
function Get-WindowsRetainedTimesEvidence {
    param(
        [Parameter(Mandatory)][Diagnostics.Process]$Process,
        [Parameter(Mandatory)][long]$ExpectedCreationFileTime,
        [switch]$RequireExited
    )
    $taskObserved=[OpenCodexRetainedTimesV1]::Observe($Process)
    if($taskObserved.CreationFileTime -ne $ExpectedCreationFileTime){throw 'Retained process generation mismatch'}
    if($RequireExited -and !$taskObserved.HasExited){throw 'Process is not yet terminated'}
    [ordered]@{
        observedAtUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskObserved.ProcessId
        creationFileTime=[string]$taskObserved.CreationFileTime
        exitFileTime=if($taskObserved.HasExited){[string]$taskObserved.ExitFileTime}else{$null}
        exited=$taskObserved.HasExited;retainedHandle=$true
        timeSource='GetProcessTimes and WaitForSingleObject on the same retained handle'
        int64TimesSerializedAsDecimalStrings=$true
        sourceMethodSha256=(Get-FileHash -LiteralPath $script:taskRetainedTimesMethodPath).Hash.ToLowerInvariant()
        productAcceptanceClaimed=$false
    }
}

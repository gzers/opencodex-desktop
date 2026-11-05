# Candidate replacement for the published Windows desktop list query only.
# $taskInstallRoot is supplied as UTF-16 hex by the adapter, not shell source.
$ErrorActionPreference='Stop'
if(!$taskInstallRoot -or ![IO.Path]::IsPathRooted($taskInstallRoot)){throw 'Absolute package root required'}
Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Security.Principal;
using System.Text;
public static class OcxDesktopOwnerV1 {
    [StructLayout(LayoutKind.Sequential)] struct NativeFileTime {public uint Low,High;}
    [DllImport("kernel32.dll",SetLastError=true)] static extern IntPtr OpenProcess(uint access,bool inherit,int pid);
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool CloseHandle(IntPtr handle);
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetProcessTimes(IntPtr h,out NativeFileTime created,out NativeFileTime exited,out NativeFileTime kernel,out NativeFileTime user);
    [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool QueryFullProcessImageName(IntPtr h,int flags,StringBuilder name,ref int size);
    [DllImport("advapi32.dll",SetLastError=true)] static extern bool OpenProcessToken(IntPtr h,uint access,out IntPtr token);
    [DllImport("advapi32.dll",SetLastError=true)] static extern bool GetTokenInformation(IntPtr token,int type,IntPtr buffer,int length,out int needed);
    public sealed class Identity {public string Path; public string Sid; public long CreatedUtcTicks;}
    public static Identity Read(int pid) {
        IntPtr process=OpenProcess(0x1000,false,pid),token=IntPtr.Zero,data=IntPtr.Zero;
        if(process==IntPtr.Zero){int code=Marshal.GetLastWin32Error();if(code==87)return null;throw new Win32Exception(code,"Process identity unavailable");}
        try {
            NativeFileTime c,e,k,u;
            if(!GetProcessTimes(process,out c,out e,out k,out u))throw new Win32Exception(Marshal.GetLastWin32Error());
            long created=DateTime.FromFileTimeUtc(((long)c.High<<32)|c.Low).Ticks;
            var name=new StringBuilder(32768);int size=name.Capacity;
            if(!QueryFullProcessImageName(process,0,name,ref size))throw new Win32Exception(Marshal.GetLastWin32Error());
            if(!OpenProcessToken(process,8,out token))throw new Win32Exception(Marshal.GetLastWin32Error());
            int needed;
            bool first=GetTokenInformation(token,1,IntPtr.Zero,0,out needed);
            if(first || Marshal.GetLastWin32Error()!=122 || needed< IntPtr.Size || needed>65536)throw new InvalidOperationException("Invalid token-user size");
            data=Marshal.AllocHGlobal(needed);
            if(!GetTokenInformation(token,1,data,needed,out needed))throw new Win32Exception(Marshal.GetLastWin32Error());
            string sid=new SecurityIdentifier(Marshal.ReadIntPtr(data)).Value;
            return new Identity {Path=name.ToString(),Sid=sid,CreatedUtcTicks=created};
        } finally {if(data!=IntPtr.Zero)Marshal.FreeHGlobal(data);if(token!=IntPtr.Zero)CloseHandle(token);CloseHandle(process);}
    }
}
'@
$taskOwnerSid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$taskRoot=[IO.Path]::GetFullPath($taskInstallRoot).TrimEnd('\','/')+[IO.Path]::DirectorySeparatorChar
$taskOwn=@()
# One CIM snapshot obtains parent IDs. Native handles establish path, owner and
# creation on the same generation; GetOwner RPC is not repeated per process.
foreach($taskP in @(Get-CimInstance Win32_Process -Filter "Name='ChatGPT.exe'" -ErrorAction Stop)){
    if(!$taskP.ExecutablePath){
        if($taskP.SessionId -eq [Diagnostics.Process]::GetCurrentProcess().SessionId){throw 'Current-session image unavailable'}
        continue
    }
    if(!$taskP.ExecutablePath.Replace('/','\').StartsWith($taskRoot,[StringComparison]::OrdinalIgnoreCase)){continue}
    $taskNative=[OcxDesktopOwnerV1]::Read([int]$taskP.ProcessId)
    if(!$taskNative){continue}
    if($taskNative.Sid -cne $taskOwnerSid){continue}
    $taskImage=$taskNative.Path
    if($taskImage.StartsWith('\\?\')){$taskImage=$taskImage.Substring(4)}
    if(!$taskImage.StartsWith($taskRoot,[StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetFileName($taskImage) -ine 'ChatGPT.exe'){throw 'Native package image changed'}
    $taskDelta=$taskNative.CreatedUtcTicks-$taskP.CreationDate.ToUniversalTime().Ticks
    if($taskDelta -lt 0 -or $taskDelta -gt 9){throw 'Process generation changed during enumeration'}
    $taskCreated=[datetime]::new($taskNative.CreatedUtcTicks,[DateTimeKind]::Utc).ToString('o')
    $taskOwn += "$($taskP.ProcessId) $($taskP.ParentProcessId) $taskCreated $taskImage"
}
# No partial stdout is published if an identity check fails halfway through.
$taskOwn|Write-Output

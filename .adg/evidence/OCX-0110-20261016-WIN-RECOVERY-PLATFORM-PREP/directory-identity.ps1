# Read-only identity from the actual directory handle, including package redirection.
if(!('OpenCodexDirectoryIdentityV1' -as [type])){
Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;
public sealed class OpenCodexDirectoryIdentityRecord {
    public string FileId {get;set;}
    public string FinalPath {get;set;}
    public string CreationFileTime {get;set;}
}
public static class OpenCodexDirectoryIdentityV1 {
    [StructLayout(LayoutKind.Sequential)]
    struct Info {
        public uint Attributes;
        public System.Runtime.InteropServices.ComTypes.FILETIME Creation;
        public System.Runtime.InteropServices.ComTypes.FILETIME Access;
        public System.Runtime.InteropServices.ComTypes.FILETIME Write;
        public uint Volume,SizeHigh,SizeLow,Links,IndexHigh,IndexLow;
    }
    [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)]
    static extern SafeFileHandle CreateFileW(string name,uint access,uint share,IntPtr security,uint disposition,uint flags,IntPtr template);
    [DllImport("kernel32.dll",SetLastError=true)]
    static extern bool GetFileInformationByHandle(SafeFileHandle handle,out Info info);
    [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)]
    static extern uint GetFinalPathNameByHandleW(SafeFileHandle handle,StringBuilder path,uint size,uint flags);
    public static OpenCodexDirectoryIdentityRecord Read(string path) {
        using(var handle=CreateFileW(path,0,7,IntPtr.Zero,3,0x02200000,IntPtr.Zero)) {
            if(handle.IsInvalid)throw new Win32Exception(Marshal.GetLastWin32Error());
            Info info;
            if(!GetFileInformationByHandle(handle,out info))throw new Win32Exception(Marshal.GetLastWin32Error());
            if((info.Attributes&0x10)==0 || (info.Attributes&0x400)!=0)throw new InvalidOperationException("Ordinary directory required");
            var text=new StringBuilder(32768);
            uint length=GetFinalPathNameByHandleW(handle,text,(uint)text.Capacity,0);
            if(length==0 || length>=text.Capacity)throw new Win32Exception(Marshal.GetLastWin32Error());
            ulong created=((ulong)(uint)info.Creation.dwHighDateTime<<32)|(uint)info.Creation.dwLowDateTime;
            return new OpenCodexDirectoryIdentityRecord {
                FileId=info.Volume.ToString("x8")+"-"+info.IndexHigh.ToString("x8")+info.IndexLow.ToString("x8"),
                FinalPath=text.ToString(),CreationFileTime=created.ToString()
            };
        }
    }
}
'@
}
function Get-RecoveryDirectoryIdentity {
    param([Parameter(Mandatory)][string]$Path)
    $taskRecord=[OpenCodexDirectoryIdentityV1]::Read($Path)
    [ordered]@{path=$Path;fileId=$taskRecord.FileId;finalPath=$taskRecord.FinalPath
        creationFileTime=$taskRecord.CreationFileTime;source='GetFileInformationByHandle/GetFinalPathNameByHandle'}
}

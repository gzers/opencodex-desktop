$ErrorActionPreference='Stop'
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Security.Principal;
public static class OcxWtsOwnRead {
 [StructLayout(LayoutKind.Sequential)] struct Row {public uint SessionId,ProcessId;public IntPtr Name,Sid;}
 [DllImport("wtsapi32.dll",CharSet=CharSet.Unicode,ExactSpelling=true,SetLastError=true)] static extern bool WTSEnumerateProcessesW(IntPtr server,uint reserved,uint version,out IntPtr rows,out uint count);
 [DllImport("wtsapi32.dll")] static extern void WTSFreeMemory(IntPtr p);
 public static object[] Read(){IntPtr rows;uint count;if(!WTSEnumerateProcessesW(IntPtr.Zero,0,1,out rows,out count))throw new Win32Exception();
  try {if(count>100000)throw new Exception("count");var values=new List<object>();for(int i=0;i<count;i++){var r=(Row)Marshal.PtrToStructure(IntPtr.Add(rows,i*Marshal.SizeOf(typeof(Row))),typeof(Row));string name=Marshal.PtrToStringUni(r.Name);if(name=="node.exe"||name=="bun.exe")values.Add(new {pid=r.ProcessId,session=r.SessionId,sid=r.Sid==IntPtr.Zero?null:new SecurityIdentifier(r.Sid).Value});}return values.ToArray();}finally{WTSFreeMemory(rows);}
 }
}
'@
@{sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value;rows=@([OcxWtsOwnRead]::Read());accepted=$false} | ConvertTo-Json -Depth 4 | Tee-Object -FilePath (Join-Path $PSScriptRoot 'wts-own-processes.json')

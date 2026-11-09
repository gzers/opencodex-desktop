$ErrorActionPreference='Stop'
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Security.Principal;
public static class OcxProcessReadAccess {
 [DllImport("kernel32.dll",SetLastError=true)] static extern IntPtr OpenProcess(uint access,bool inherit,uint pid);
 [DllImport("advapi32.dll",SetLastError=true)] static extern bool OpenProcessToken(IntPtr process,uint access,out IntPtr token);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetExitCodeProcess(IntPtr process,out uint code);
 [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr handle);
 public static object Observe(uint pid,uint access){
  IntPtr process=OpenProcess(access,false,pid);int error=Marshal.GetLastWin32Error();
  if(process==IntPtr.Zero)return new {pid,access,opened=false,error};
  try {uint code;bool observed=GetExitCodeProcess(process,out code);IntPtr token;
   if(!OpenProcessToken(process,8,out token))return new {pid,access,opened=true,liveObserved=observed,exitCode=code,tokenOpened=false,error=Marshal.GetLastWin32Error()};
   try {using(var identity=new WindowsIdentity(token)){return new {pid,access,opened=true,liveObserved=observed,exitCode=code,tokenOpened=true,sid=identity.User.Value};}}finally{CloseHandle(token);}
  }finally{CloseHandle(process);}
 }
}
'@
$result=@(foreach($p in Get-CimInstance Win32_Process | Where-Object {$_.Name -in @('node.exe','bun.exe','ocx.exe','opencodex.exe')}){
 foreach($access in @(0x101000,0x1000)){[OcxProcessReadAccess]::Observe([uint32]$p.ProcessId,[uint32]$access)}
})
$result | ConvertTo-Json -Depth 4 | Tee-Object -FilePath (Join-Path $PSScriptRoot 'process-read-access.json')

$ErrorActionPreference='Stop'
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;
public static class L4AncestorOpenProbe {
 [StructLayout(LayoutKind.Sequential)] public struct Info {
  public uint attrs,createdLow,createdHigh,accessLow,accessHigh,writeLow,writeHigh;
  public uint volume,sizeHigh,sizeLow,links,indexHigh,indexLow;
 }
 public sealed class Result { public string path,access; public uint flags=0x02200000,share=3; public bool openOk,infoOk; public int openError,infoError; public uint attributes,links,type; }
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,ExactSpelling=true,SetLastError=true)] static extern SafeFileHandle CreateFileW(string name,uint access,uint share,IntPtr security,uint disposition,uint flags,IntPtr template);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetFileInformationByHandle(SafeFileHandle h,out Info info);
 [DllImport("kernel32.dll",SetLastError=true)] static extern uint GetFileType(SafeFileHandle h);
 public static Result Probe(string path,uint access) {
  var r=new Result{path=path,access="0x"+access.ToString("x8")};
  using(var h=CreateFileW(path,access,3,IntPtr.Zero,3,0x02200000,IntPtr.Zero)) {
   r.openOk=!h.IsInvalid;r.openError=r.openOk?0:Marshal.GetLastWin32Error();
   if(r.openOk) {Info info;r.infoOk=GetFileInformationByHandle(h,out info);r.infoError=r.infoOk?0:Marshal.GetLastWin32Error();if(r.infoOk){r.attributes=info.attrs;r.links=info.links;}r.type=GetFileType(h);}
  }
  return r;
 }
}
'@
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
$results=@()
foreach($path in @(('D:'+[char]92),[Environment]::GetFolderPath('UserProfile'),('C:'+[char]92))) {
 $path=$path.Replace([string][char]0,'')
 foreach($mask in @([uint32]2147614720,[uint32]2147483648,[uint32]0x00000080,[uint32]0x00020080)) {
  $results += [L4AncestorOpenProbe]::Probe($path,$mask)
 }
}
$result=[ordered]@{utc=[datetime]::UtcNow.ToString('o');identity=$identity.Name;sid=$identity.User.Value;profile=[Environment]::GetFolderPath('UserProfile');mode='read-only existing directories; OPEN_EXISTING; share=READ|WRITE; BACKUP_SEMANTICS|OPEN_REPARSE_POINT';results=$results}
$result|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $PSScriptRoot 'local-probe.json') -Encoding utf8
$result|ConvertTo-Json -Depth 5

param([Parameter(Mandatory)][string]$Controller,[Parameter(Mandatory)][string]$Output)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $Output){throw 'fresh diagnosis evidence required'}
$c=Get-Content -LiteralPath $Controller -Raw|ConvertFrom-Json
if(!$c.created -or !$c.administratorScenario -or $c.name -notmatch '^ocxw1d[0-9a-f]{9}$' -or [IO.Path]::GetFileName($c.profile) -cne $c.name -or [IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($c.profile)) -ine (Join-Path $env:SystemDrive 'Users')){throw 'exact disposable test Profile required'}
Add-Type -TypeDefinition @'
using System;
using System.Text;
using System.ComponentModel;
using System.Runtime.InteropServices;
public static class OcxProfileLockReadback {
 [StructLayout(LayoutKind.Sequential)] public struct Unique { public uint Pid; public System.Runtime.InteropServices.ComTypes.FILETIME Started; }
 [StructLayout(LayoutKind.Sequential,CharSet=CharSet.Unicode)] public struct Info {
  public Unique Process;
  [MarshalAs(UnmanagedType.ByValTStr,SizeConst=256)] public string App;
  [MarshalAs(UnmanagedType.ByValTStr,SizeConst=64)] public string Service;
  public uint Type; public uint Status; public uint Session;
  [MarshalAs(UnmanagedType.Bool)] public bool Restartable;
 }
 [DllImport("rstrtmgr.dll",CharSet=CharSet.Unicode)] static extern uint RmStartSession(out uint session,uint flags,StringBuilder key);
 [DllImport("rstrtmgr.dll",CharSet=CharSet.Unicode)] static extern uint RmRegisterResources(uint session,uint count,[MarshalAs(UnmanagedType.LPArray,ArraySubType=UnmanagedType.LPWStr)] string[] files,uint apps,IntPtr processes,uint services,IntPtr names);
 [DllImport("rstrtmgr.dll")] static extern uint RmGetList(uint session,out uint needed,ref uint count,[In,Out] Info[] infos,out uint reasons);
 [DllImport("rstrtmgr.dll")] static extern uint RmEndSession(uint session);
 public static object Read(string[] files) {
  uint session; uint start=RmStartSession(out session,0,new StringBuilder(33));
  if(start!=0) throw new Win32Exception((int)start);
  try {
   uint registered=RmRegisterResources(session,(uint)files.Length,files,0,IntPtr.Zero,0,IntPtr.Zero);
   if(registered!=0) throw new Win32Exception((int)registered);
   uint needed,count=0,reasons; uint result=RmGetList(session,out needed,ref count,null,out reasons);
   Info[] infos=new Info[0];
   if(result==234) { if(needed>64) throw new Exception("bounded exact-resource owner list exceeded"); infos=new Info[needed]; count=needed; result=RmGetList(session,out needed,ref count,infos,out reasons); }
   if(result!=0) throw new Win32Exception((int)result);
   return new { result=result,rebootReasons=reasons,count=count,owners=infos };
  } finally { RmEndSession(session); }
 }
}
'@
$r=[ordered]@{scope='read-only Restart Manager query for two exact owned Profile hive files; no RmShutdown/RmRestart';sid=$c.sid;profile=$c.profile}
try{
 $files=@((Join-Path $c.profile 'NTUSER.DAT'),(Join-Path $c.profile 'AppData/Local/Microsoft/Windows/UsrClass.dat'))
 $r.files=$files
 $r.native=[OcxProfileLockReadback]::Read($files)
 $r.profileService=Get-CimInstance Win32_UserProfile -Filter ("SID='"+$c.sid+"'")|Select-Object SID,Loaded,LocalPath
}catch{$r.error=$_.Exception.ToString()}
finally{$r|ConvertTo-Json -Depth 8|Set-Content -LiteralPath $Output -Encoding utf8}

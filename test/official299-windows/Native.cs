// Test-only retained-handle launcher. No production imports or legacy lifetime claims.
using System;
using System.Collections;
using System.Collections.Generic;
using System.ComponentModel;
using System.Diagnostics;
using System.Linq;
using System.Runtime.InteropServices;
using System.Security.Principal;
using System.Text;

namespace Official299Proof {
public static class Native {
  [StructLayout(LayoutKind.Sequential)] public struct SI {
    public int cb; public IntPtr reserved, desktop, title; public int x,y,xs,ys,xc,yc,fill,flags;
    public short show, reserved2; public IntPtr reserved3, input, output, error;
  }
  [StructLayout(LayoutKind.Sequential)] public struct SIX { public SI si; public IntPtr attributes; }
  [StructLayout(LayoutKind.Sequential)] public struct PI { public IntPtr process, thread; public uint pid, tid; }
  [StructLayout(LayoutKind.Sequential)] public struct SA { public int length; public IntPtr descriptor; public int inherit; }
  [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)] public struct PROFILE {
    public int size, flags; public string name, path, defaultPath, server, policy; public IntPtr handle;
  }
  [DllImport("kernel32", SetLastError=true, CharSet=CharSet.Unicode)] public static extern bool CreateProcessW(string app, StringBuilder command, IntPtr pa, IntPtr ta, bool inherit, uint flags, IntPtr env, string cwd, ref SIX si, out PI pi);
  [DllImport("advapi32", SetLastError=true, CharSet=CharSet.Unicode)] public static extern bool CreateProcessWithLogonW(string user,string domain,string password,uint logon,string app,StringBuilder command,uint flags,IntPtr env,string cwd,ref SI si,out PI pi);
  [DllImport("advapi32", SetLastError=true, CharSet=CharSet.Unicode)] public static extern bool LogonUserW(string user,string domain,string password,int type,int provider,out IntPtr token);
  [DllImport("userenv", SetLastError=true, CharSet=CharSet.Unicode)] public static extern bool LoadUserProfileW(IntPtr token, ref PROFILE profile);
  [DllImport("userenv", SetLastError=true)] public static extern bool UnloadUserProfile(IntPtr token,IntPtr profile);
  [DllImport("userenv", CharSet=CharSet.Unicode)] public static extern int CreateProfile(string sid,string user,StringBuilder path,uint length);
  [DllImport("kernel32", SetLastError=true)] public static extern bool CloseHandle(IntPtr h);
  [DllImport("kernel32", SetLastError=true)] public static extern uint WaitForSingleObject(IntPtr h,uint ms);
  [DllImport("kernel32", SetLastError=true)] public static extern bool GetExitCodeProcess(IntPtr h,out uint code);
  [DllImport("kernel32", SetLastError=true)] public static extern bool GetProcessTimes(IntPtr h,out long created,out long exit,out long kernel,out long user);
  [DllImport("kernel32", SetLastError=true)] public static extern uint GetProcessId(IntPtr h);
  [DllImport("kernel32", SetLastError=true)] public static extern bool ProcessIdToSessionId(uint pid,out uint session);
  [DllImport("kernel32", SetLastError=true, CharSet=CharSet.Unicode)] public static extern bool QueryFullProcessImageNameW(IntPtr h,uint flags,StringBuilder path,ref uint length);
  [DllImport("advapi32", SetLastError=true)] public static extern bool OpenProcessToken(IntPtr h,uint rights,out IntPtr token);
  [DllImport("advapi32", SetLastError=true)] public static extern bool GetTokenInformation(IntPtr token,int kind,IntPtr buffer,int size,out int needed);
  [DllImport("kernel32", SetLastError=true, CharSet=CharSet.Unicode)] public static extern IntPtr CreateJobObjectW(ref SA sa,string name);
  [DllImport("kernel32", SetLastError=true)] public static extern bool AssignProcessToJobObject(IntPtr job,IntPtr process);
  [DllImport("kernel32", SetLastError=true)] public static extern bool IsProcessInJob(IntPtr process,IntPtr job,out bool yes);
  [DllImport("kernel32", SetLastError=true)] public static extern bool QueryInformationJobObject(IntPtr job,int kind,IntPtr buffer,int size,out int needed);
  [DllImport("kernel32", SetLastError=true)] public static extern bool TerminateJobObject(IntPtr job,uint code);
  [DllImport("kernel32", SetLastError=true)] public static extern bool TerminateProcess(IntPtr process,uint code);
  [DllImport("kernel32", SetLastError=true)] public static extern uint ResumeThread(IntPtr thread);
  [DllImport("kernel32", SetLastError=true, CharSet=CharSet.Unicode)] public static extern IntPtr CreateFileW(string path,uint access,uint sharing,ref SA sa,uint creation,uint flags,IntPtr template);
  [DllImport("kernel32", SetLastError=true)] public static extern bool InitializeProcThreadAttributeList(IntPtr list,int count,uint flags,ref IntPtr size);
  [DllImport("kernel32", SetLastError=true)] public static extern bool UpdateProcThreadAttribute(IntPtr list,uint flags,IntPtr attribute,IntPtr value,IntPtr size,IntPtr old,IntPtr returned);
  [DllImport("kernel32")] public static extern void DeleteProcThreadAttributeList(IntPtr list);
  [DllImport("advapi32", SetLastError=true, CharSet=CharSet.Unicode)] public static extern bool ConvertStringSecurityDescriptorToSecurityDescriptorW(string text,uint rev,out IntPtr sd,out uint length);
  [DllImport("kernel32")] public static extern IntPtr LocalFree(IntPtr h);
  [DllImport("advapi32", SetLastError=true)] static extern bool GetSecurityDescriptorDacl(IntPtr sd,out bool present,out IntPtr dacl,out bool defaulted);
  [DllImport("advapi32", CharSet=CharSet.Unicode)] static extern uint SetNamedSecurityInfoW(string path,int kind,uint info,IntPtr owner,IntPtr group,IntPtr dacl,IntPtr sacl);
  public static void RestoreSyntheticDacl(string path,string sddl) { SetFixtureDacl(path,sddl,false); }
  public static void SetFixtureDacl(string path,string sddl,bool protect) {
    IntPtr sd;uint len;Check(ConvertStringSecurityDescriptorToSecurityDescriptorW(sddl,1,out sd,out len));
    try { bool present,def;IntPtr acl;Check(GetSecurityDescriptorDacl(sd,out present,out acl,out def));if(!present||acl==IntPtr.Zero)throw new InvalidOperationException("null DACL refused");uint rc=SetNamedSecurityInfoW(path,1,protect?0x80000004u:0x20000004u,IntPtr.Zero,IntPtr.Zero,acl,IntPtr.Zero);if(rc!=0)throw new Win32Exception((int)rc); }
    finally {LocalFree(sd);}
  }
  [DllImport("iphlpapi", SetLastError=true)] static extern uint GetExtendedTcpTable(IntPtr table,ref int size,bool order,uint family,int kind,uint reserved);
  public static uint[] Listeners(int port) {
    var ids=new List<uint>();
    foreach(uint family in new uint[]{2,23}) {
      int size=0; uint rc=GetExtendedTcpTable(IntPtr.Zero,ref size,false,family,3,0);
      if(rc!=122 && rc!=0) throw new Win32Exception((int)rc);
      var b=Marshal.AllocHGlobal(Math.Max(size,4));
      try { rc=GetExtendedTcpTable(b,ref size,false,family,3,0); if(rc!=0) throw new Win32Exception((int)rc);
        int count=Marshal.ReadInt32(b), stride=family==2?24:56;
        if(count<0 || 4L+count*(long)stride>size) throw new InvalidOperationException("incomplete TCP table");
        for(int i=0;i<count;i++) { int row=4+i*stride,p=family==2?row+8:row+20,state=family==2?row:row+48;
          int actual=(Marshal.ReadByte(b,p)<<8)|Marshal.ReadByte(b,p+1);
          if(actual==port && Marshal.ReadInt32(b,state)==2) ids.Add(unchecked((uint)Marshal.ReadInt32(b,row+stride-4)));
        }
      } finally {Marshal.FreeHGlobal(b);}
    }
    return ids.ToArray();
  }
  [StructLayout(LayoutKind.Sequential)] public struct LUID { public uint low; public int high; }
  [StructLayout(LayoutKind.Sequential)] public struct TP { public int count; public LUID luid; public uint attributes; }
  [DllImport("advapi32", SetLastError=true, CharSet=CharSet.Unicode)] public static extern bool LookupPrivilegeValueW(string system,string name,out LUID luid);
  [DllImport("advapi32", SetLastError=true)] public static extern bool AdjustTokenPrivileges(IntPtr token,bool disable,ref TP tp,int size,IntPtr old,IntPtr returned);
  public static void Check(bool ok) { if(!ok) throw new Win32Exception(Marshal.GetLastWin32Error()); }
  public static void EnableProfilePrivileges() {
    IntPtr t; Check(OpenProcessToken(Process.GetCurrentProcess().Handle,0x28,out t));
    try { foreach(string name in new[]{"SeBackupPrivilege","SeRestorePrivilege"}) { LUID id; Check(LookupPrivilegeValueW(null,name,out id)); var p=new TP{count=1,luid=id,attributes=2}; Check(AdjustTokenPrivileges(t,false,ref p,0,IntPtr.Zero,IntPtr.Zero)); if(Marshal.GetLastWin32Error()!=0) throw new Win32Exception(Marshal.GetLastWin32Error()); } }
    finally { CloseHandle(t); }
  }
  public static IntPtr EnvironmentBlock(IDictionary env) {
    var pairs=new List<string>(); foreach(DictionaryEntry p in env) pairs.Add(p.Key+"="+p.Value);
    pairs.Sort(StringComparer.OrdinalIgnoreCase); return Marshal.StringToHGlobalUni(string.Join("\0",pairs)+"\0\0");
  }
  public static Dictionary<string,object> Observe(IntPtr h) {
    long c,e,k,u; Check(GetProcessTimes(h,out c,out e,out k,out u));
    uint pid=GetProcessId(h),session; Check(pid!=0); Check(ProcessIdToSessionId(pid,out session));
    var path=new StringBuilder(32768); uint n=(uint)path.Capacity; Check(QueryFullProcessImageNameW(h,0,path,ref n));
    IntPtr t; Check(OpenProcessToken(h,8,out t));
    try {
      int need; GetTokenInformation(t,1,IntPtr.Zero,0,out need); var b=Marshal.AllocHGlobal(need);
      string sid; try { Check(GetTokenInformation(t,1,b,need,out need)); sid=new SecurityIdentifier(Marshal.ReadIntPtr(b)).Value; } finally { Marshal.FreeHGlobal(b); }
      var q=Marshal.AllocHGlobal(4); int elevated,type;
      try { Check(GetTokenInformation(t,20,q,4,out need)); elevated=Marshal.ReadInt32(q); Check(GetTokenInformation(t,18,q,4,out need)); type=Marshal.ReadInt32(q); } finally { Marshal.FreeHGlobal(q); }
      return new Dictionary<string,object>{{"pid",pid},{"filetime",c.ToString()},{"sid",sid},{"session",session},{"image",path.ToString()},{"elevated",elevated!=0},{"elevationType",type}};
    } finally { CloseHandle(t); }
  }
}

public sealed class Guard : IDisposable {
  [StructLayout(LayoutKind.Sequential)] struct INFO { public uint attrs,cl,ch,al,ah,wl,wh,volume,sh,sl,links,ih,il; }
  [DllImport("kernel32", SetLastError=true)] static extern bool GetFileInformationByHandle(IntPtr h,out INFO info);
  IntPtr handle; public string Identity {get;private set;}
  public Guard(string path,bool directory) {
    var sa=new Native.SA{length=Marshal.SizeOf<Native.SA>()};
    handle=Native.CreateFileW(path,0x80000000,3,ref sa,3,0x02200000,IntPtr.Zero);
    Native.Check(handle!=new IntPtr(-1));
    try { INFO f;Native.Check(GetFileInformationByHandle(handle,out f));
      if((f.attrs&0x400)!=0 || ((f.attrs&0x10)!=0)!=directory || (!directory && f.links!=1)) throw new InvalidOperationException("linked or wrong-kind proof path");
      Identity=f.volume+":"+f.ih+":"+f.il;
    } catch {Dispose();throw;}
  }
  public void Dispose() {if(handle!=IntPtr.Zero && handle!=new IntPtr(-1)){Native.CloseHandle(handle);handle=IntPtr.Zero;}}
}

public sealed class Owned : IDisposable {
  public IntPtr Job {get;private set;} public IntPtr Process {get;private set;} public IntPtr Thread {get;private set;}
  public bool Resumed {get;private set;} public Dictionary<string,object> Identity {get;private set;}
  public Owned(string exe,string command,string cwd,IDictionary env,string stdout,string stderr,string sid) {
    IntPtr sd=IntPtr.Zero,eb=IntPtr.Zero,list=IntPtr.Zero,handles=IntPtr.Zero; var files=new List<IntPtr>();
    try {
      uint len; Native.Check(Native.ConvertStringSecurityDescriptorToSecurityDescriptorW("O:"+sid+"D:P(A;;GA;;;"+sid+")(A;;GA;;;SY)",1,out sd,out len));
      var js=new Native.SA{length=Marshal.SizeOf<Native.SA>(),descriptor=sd}; Job=Native.CreateJobObjectW(ref js,null); Native.Check(Job!=IntPtr.Zero);
      var sa=new Native.SA{length=Marshal.SizeOf<Native.SA>(),inherit=1};
      foreach(string f in new[]{"NUL",stdout,stderr}) { var h=Native.CreateFileW(f,f=="NUL"?0x80000000u:0x40000000u,1,ref sa,f=="NUL"?3u:1u,0x80,IntPtr.Zero); Native.Check(h!=new IntPtr(-1)); files.Add(h); }
      IntPtr size=IntPtr.Zero; Native.InitializeProcThreadAttributeList(IntPtr.Zero,1,0,ref size); list=Marshal.AllocHGlobal(size); Native.Check(Native.InitializeProcThreadAttributeList(list,1,0,ref size));
      handles=Marshal.AllocHGlobal(3*IntPtr.Size); for(int i=0;i<3;i++) Marshal.WriteIntPtr(handles,i*IntPtr.Size,files[i]);
      Native.Check(Native.UpdateProcThreadAttribute(list,0,new IntPtr(0x20002),handles,new IntPtr(3*IntPtr.Size),IntPtr.Zero,IntPtr.Zero));
      var si=new Native.SIX{si=new Native.SI{cb=Marshal.SizeOf<Native.SIX>(),flags=0x100,input=files[0],output=files[1],error=files[2]},attributes=list};
      eb=Native.EnvironmentBlock(env); Native.PI pi;
      Native.Check(Native.CreateProcessW(exe,new StringBuilder(command),IntPtr.Zero,IntPtr.Zero,true,0x08080404,eb,cwd,ref si,out pi));
      Process=pi.process; Thread=pi.thread; Identity=Native.Observe(Process);
      if((string)Identity["sid"]!=sid || (bool)Identity["elevated"]) throw new InvalidOperationException("ordinary TokenUser required before resume");
      Native.Check(Native.AssignProcessToJobObject(Job,Process)); bool member; Native.Check(Native.IsProcessInJob(Process,Job,out member)); if(!member) throw new InvalidOperationException("Job assignment missing before resume");
      if(Active()!=1) throw new InvalidOperationException("unexpected initial Job accounting");
    } catch { if(Process!=IntPtr.Zero) { Native.Check(Native.TerminateProcess(Process,241)); Native.WaitForSingleObject(Process,3000); } Dispose(); throw; }
    finally { foreach(var f in files) Native.CloseHandle(f); if(list!=IntPtr.Zero) {Native.DeleteProcThreadAttributeList(list);Marshal.FreeHGlobal(list);} if(handles!=IntPtr.Zero) Marshal.FreeHGlobal(handles); if(eb!=IntPtr.Zero) Marshal.FreeHGlobal(eb); if(sd!=IntPtr.Zero) Native.LocalFree(sd); }
  }
  public void Resume() { if(Resumed) throw new InvalidOperationException("single resume only"); if(Native.ResumeThread(Thread)!=1) throw new Win32Exception(Marshal.GetLastWin32Error()); Resumed=true; }
  public bool Exited() { uint v=Native.WaitForSingleObject(Process,0); if(v==0xffffffff) throw new Win32Exception(Marshal.GetLastWin32Error()); return v==0; }
  public uint ExitCode() { if(!Exited()) throw new InvalidOperationException("original handle not signalled"); uint code; Native.Check(Native.GetExitCodeProcess(Process,out code)); return code; }
  public uint Active() { var b=Marshal.AllocHGlobal(48); try { int n; Native.Check(Native.QueryInformationJobObject(Job,1,b,48,out n)); return unchecked((uint)Marshal.ReadInt32(b,40)); } finally {Marshal.FreeHGlobal(b);} }
  public uint Limits() { var b=Marshal.AllocHGlobal(144);try{int n;Native.Check(Native.QueryInformationJobObject(Job,9,b,144,out n));return unchecked((uint)Marshal.ReadInt32(b,16));}finally{Marshal.FreeHGlobal(b);} }
  public uint[] Members() { int size=65536; var b=Marshal.AllocHGlobal(size); try { int n; Native.Check(Native.QueryInformationJobObject(Job,3,b,size,out n)); int assigned=Marshal.ReadInt32(b),count=Marshal.ReadInt32(b,4); if(assigned!=count || count<0 || count>(size-8)/IntPtr.Size) throw new InvalidOperationException("incomplete Job PID list"); var ids=new uint[count];for(int i=0;i<count;i++) ids[i]=unchecked((uint)Marshal.ReadIntPtr(b,8+i*IntPtr.Size).ToInt64());return ids; } finally {Marshal.FreeHGlobal(b);} }
  public void ForceOwnedCleanup() { Native.Check(Native.TerminateJobObject(Job,242)); }
  public void Dispose() { if(Thread!=IntPtr.Zero){Native.CloseHandle(Thread);Thread=IntPtr.Zero;} if(Process!=IntPtr.Zero){Native.CloseHandle(Process);Process=IntPtr.Zero;} if(Job!=IntPtr.Zero){Native.CloseHandle(Job);Job=IntPtr.Zero;} }
}
}

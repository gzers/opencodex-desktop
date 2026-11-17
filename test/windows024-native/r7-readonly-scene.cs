using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Security.Principal;
using System.Text;
using Microsoft.Win32.SafeHandles;

// Query-only r7 observation. No terminate, resume, Job, mutation or restore API.
public static class R7ReadOnlyScene {
 const string Sid = "S-1-5-21-587408064-856152790-2583328096-1003";
 const string Base = @"C:\Users\marenben\ocx024-r6-existing-20261117-r2";
 [StructLayout(LayoutKind.Sequential)] struct Info {
  public uint attributes,creationLow,creationHigh,accessLow,accessHigh,writeLow,writeHigh;
  public uint volume,sizeHigh,sizeLow,links,indexHigh,indexLow;
 }
 [DllImport("kernel32.dll",SetLastError=true)] static extern IntPtr OpenProcess(uint access,bool inherit,uint pid);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetProcessTimes(IntPtr p,out long c,out long e,out long k,out long u);
 [DllImport("kernel32.dll",SetLastError=true)] static extern uint GetProcessId(IntPtr p);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool QueryFullProcessImageName(IntPtr p,uint flags,StringBuilder path,ref uint size);
 [DllImport("kernel32.dll",SetLastError=true)] static extern uint WaitForSingleObject(IntPtr p,uint ms);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetExitCodeProcess(IntPtr p,out uint code);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool CloseHandle(IntPtr p);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool IsProcessInJob(IntPtr p,IntPtr job,out bool member);
 [DllImport("advapi32.dll",SetLastError=true)] static extern bool OpenProcessToken(IntPtr p,uint access,out IntPtr token);
 [DllImport("advapi32.dll",SetLastError=true)] static extern bool GetTokenInformation(IntPtr token,int kind,IntPtr buffer,int size,out int needed);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern SafeFileHandle CreateFile(string path,uint access,uint share,IntPtr security,uint creation,uint flags,IntPtr template);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetFileInformationByHandle(SafeFileHandle file,out Info info);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern uint GetFinalPathNameByHandle(SafeFileHandle file,StringBuilder path,uint size,uint flags);
 [DllImport("shell32.dll")] static extern int SHGetKnownFolderPath(ref Guid id,uint flags,IntPtr token,out IntPtr path);
 static void Check(bool ok) {if(!ok)throw new Win32Exception(Marshal.GetLastWin32Error());}
 static void Require(bool ok,string reason) {if(!ok)throw new InvalidDataException(reason);}
 static bool Same(string a,string b) {return String.Equals(Path.GetFullPath(a).TrimEnd((char)92),Path.GetFullPath(b).TrimEnd((char)92),StringComparison.OrdinalIgnoreCase);}
 public sealed class TokenFacts {public string sid;public int session,elevationType,elevated;}
 static IntPtr TokenValue(IntPtr token,int kind) {
  int n;GetTokenInformation(token,kind,IntPtr.Zero,0,out n);
  Require(n>0 && n<=65536,"bounded native token information");
  IntPtr b=Marshal.AllocHGlobal(n);
  try {Check(GetTokenInformation(token,kind,b,n,out n));return b;}catch{Marshal.FreeHGlobal(b);throw;}
 }
 static int TokenInt(IntPtr token,int kind) {IntPtr b=TokenValue(token,kind);try{return Marshal.ReadInt32(b);}finally{Marshal.FreeHGlobal(b);}}
 static TokenFacts Token(IntPtr p) {
  IntPtr t;Check(OpenProcessToken(p,8,out t));
  try {IntPtr b=TokenValue(t,1);string sid;try{sid=new SecurityIdentifier(Marshal.ReadIntPtr(b)).Value;}finally{Marshal.FreeHGlobal(b);}
   return new TokenFacts {sid=sid,session=TokenInt(t,12),elevationType=TokenInt(t,18),elevated=TokenInt(t,20)};
  }finally{Check(CloseHandle(t));}
 }
 static bool InJob(IntPtr p) {bool member;Check(IsProcessInJob(p,IntPtr.Zero,out member));return member;}
 static Info Identity(SafeFileHandle file,string expected,bool directory) {
  Info i;Check(GetFileInformationByHandle(file,out i));
  Require((i.attributes & 0x400)==0 && ((i.attributes & 0x10)!=0)==directory,"reparse/wrong-kind image or ancestor refused");
  if(!directory)Require(i.links==1,"multiple-link task file refused");
  StringBuilder path=new StringBuilder(32768);uint n=GetFinalPathNameByHandle(file,path,32768,0);
  Require(n>0 && n<32768,"exact native final path unavailable");
  string value=path.ToString();if(value.StartsWith(@"\\?\"))value=value.Substring(4);
  Require(Same(value,expected),"native final path changed");return i;
 }
 static string Key(Info i) {return i.volume.ToString(CultureInfo.InvariantCulture)+":"+(((ulong)i.indexHigh<<32)|i.indexLow).ToString(CultureInfo.InvariantCulture);}
 internal sealed class Pinned : IDisposable {
  public readonly string path,key,sha;public readonly long bytes;
  readonly FileStream stream;readonly List<SafeFileHandle> ancestors=new List<SafeFileHandle>();
  public Pinned(string path,string sha,long expectedBytes,bool liveWriter) {
   this.path=Path.GetFullPath(path);this.sha=sha;
   try {
    for(string parent=Path.GetDirectoryName(this.path);parent!=null;parent=Path.GetDirectoryName(parent)) {
     SafeFileHandle pin=CreateFile(parent,0x80,7,IntPtr.Zero,3,0x02200000,IntPtr.Zero);
     if(pin.IsInvalid){pin.Dispose();throw new Win32Exception(Marshal.GetLastWin32Error());}
     ancestors.Add(pin);Identity(pin,parent,true);
    }
    SafeFileHandle file=CreateFile(this.path,0x80000000,7u,IntPtr.Zero,3,0x00200000,IntPtr.Zero);
    if(file.IsInvalid){file.Dispose();throw new Win32Exception(Marshal.GetLastWin32Error());}
    try{key=Key(Identity(file,this.path,false));stream=new FileStream(file,FileAccess.Read);}catch{file.Dispose();throw;}
    bytes=stream.Length;Require(expectedBytes<0 || bytes==expectedBytes,"pinned file length differs");Recheck();
   }catch{Dispose();throw;}
  }
  public void Recheck() {
   Require(Key(Identity(stream.SafeFileHandle,path,false))==key && stream.Length==bytes,"pinned file identity/length changed");
   stream.Position=0;string actual;using(SHA256 hash=SHA256.Create()){actual=BitConverter.ToString(hash.ComputeHash(stream)).Replace("-","").ToLowerInvariant();}
   Require(actual==sha,"pinned file SHA256 differs");
  }
  public void Dispose(){if(stream!=null)stream.Dispose();foreach(SafeFileHandle pin in ancestors)pin.Dispose();}
 }
 public sealed class ProcessFacts {
  public uint pid;public string filetime,image,imageKey,imageSha256;public TokenFacts token;public bool inJob,exited;public int threadCount;public string threadState,threadWaitReason;
 }
 internal sealed class Target : IDisposable {
  readonly IntPtr handle;readonly uint pid;readonly string filetime,image,sid;readonly int session;readonly bool ordinary,noJob,suspended;readonly Pinned pin;
  public ProcessFacts facts;
  public Target(uint pid,string filetime,string image,string sha,string sid,int session,bool ordinary,bool noJob,bool suspended) {
   this.pid=pid;this.filetime=filetime;this.image=image;this.sid=sid;this.session=session;this.ordinary=ordinary;this.noJob=noJob;this.suspended=suspended;
   handle=OpenProcess(0x00101000,false,pid);if(handle==IntPtr.Zero)throw new Win32Exception(Marshal.GetLastWin32Error());
   try{pin=new Pinned(image,sha,-1,false);Recheck();}catch{Dispose();throw;}
  }
  public void Recheck() {
   long c,e,k,u;Check(GetProcessTimes(handle,out c,out e,out k,out u));
   Require(GetProcessId(handle)==pid && c.ToString(CultureInfo.InvariantCulture)==filetime,"exact FILETIME/PID differs");
   Require(WaitForSingleObject(handle,0)==258,"exact target already exited/unknown");
   uint size=32768;StringBuilder path=new StringBuilder((int)size);Check(QueryFullProcessImageName(handle,0,path,ref size));
   Require(Same(path.ToString(),image),"exact process image path differs");
   TokenFacts t=Token(handle);Require(t.sid==sid && t.session==session,"exact TokenUser/session differs");
   Require(!ordinary || t.elevated==0 && t.elevationType==1,"original ordinary non-elevated token required");
   bool job=InJob(handle);Require(!noJob || !job,"original broker inherited Job differs");pin.Recheck();
   ProcessFacts f=new ProcessFacts {pid=pid,filetime=filetime,image=path.ToString(),imageKey=pin.key,imageSha256=pin.sha,token=t,inJob=job,exited=false};
   if(suspended) {
    using(Process p=Process.GetProcessById((int)pid)) {
     f.threadCount=p.Threads.Count;Require(f.threadCount==1,"never-resumed launcher thread set differs");
     ProcessThread thread=p.Threads[0];try {f.threadState=thread.ThreadState.ToString();
      Require(thread.ThreadState==System.Diagnostics.ThreadState.Wait,"launcher thread is not waiting");
      f.threadWaitReason=thread.WaitReason.ToString();Require(thread.WaitReason==ThreadWaitReason.Suspended,"launcher is not still suspended");
     }finally{thread.Dispose();}
    }
   }
   Check(GetProcessTimes(handle,out c,out e,out k,out u));Require(c.ToString(CultureInfo.InvariantCulture)==filetime && WaitForSingleObject(handle,0)==258,"target changed during native observation");facts=f;
  }
  public void Dispose(){if(pin!=null)pin.Dispose();if(handle!=IntPtr.Zero)Check(CloseHandle(handle));}
 }

 [StructLayout(LayoutKind.Sequential,CharSet=CharSet.Unicode)] struct StreamInfo {
  public long size;[MarshalAs(UnmanagedType.ByValTStr,SizeConst=296)]public string name;
 }
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr FindFirstStreamW(string path,int level,out StreamInfo info,uint flags);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool FindNextStreamW(IntPtr search,out StreamInfo info);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool FindClose(IntPtr search);
 public sealed class FileFacts {public string path,key;public uint attributes,links;public bool directory;public string[] streams;public long bytes;}
 public static FileFacts ObserveFile(string path,bool directory) {
  path=Path.GetFullPath(path);
  using(SafeFileHandle file=CreateFile(path,0x80,7,IntPtr.Zero,3,0x00200000u|(directory?0x02000000u:0u),IntPtr.Zero)) {
   if(file.IsInvalid)throw new Win32Exception(Marshal.GetLastWin32Error());
   Info before=Identity(file,path,directory);List<string> names=new List<string>();StreamInfo stream;
   IntPtr search=FindFirstStreamW(path,0,out stream,0);
   if(search==new IntPtr(-1)){int error=Marshal.GetLastWin32Error();if(!(directory && error==38))throw new Win32Exception(error);}
   else try {
    while(true) {Require(names.Count<32,"stream enumeration bound");names.Add(stream.name);
     Require(stream.name=="::$DATA","alternate or unknown stream refused: "+stream.name);
     if(!FindNextStreamW(search,out stream)){int error=Marshal.GetLastWin32Error();Require(error==38,"stream enumeration incomplete raw="+error);break;}
    }
   }finally{Check(FindClose(search));}
   Info after=Identity(file,path,directory);Require(Key(before)==Key(after),"archive metadata identity changed");
   return new FileFacts{path=path,key=Key(before),attributes=before.attributes,links=before.links,directory=directory,streams=names.ToArray(),bytes=(long)(((ulong)before.sizeHigh<<32)|before.sizeLow)};
  }
 }
 public sealed class Pair : IDisposable {
  readonly Target[] targets;internal Pair(Target[] targets){this.targets=targets;}
  public ProcessFacts[] Snapshot(){foreach(Target target in targets)target.Recheck();return new ProcessFacts[]{targets[0].facts,targets[1].facts};}
  public void Dispose(){foreach(Target target in targets)target.Dispose();}
 }
 [DllImport("shell32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr CommandLineToArgvW(string command,out int count);
 [DllImport("kernel32.dll")] static extern IntPtr LocalFree(IntPtr value);
 public sealed class LineageData {
  public uint pid,parentPid;public string nativeImage,optionalWmiImage;public string[] arguments;
  public bool optionalWmiImageUnavailable,consistent,nativeAuthority=false;
 }
 // Compare already observed DATA only. The caller must still retain/recheck
 // Pair; this does not open a process or confer cleanup/recovery authority.
 public static LineageData CompareLineageData(ProcessFacts[] observed,uint pid,uint parent,string optionalWmiImage,string command) {
  Require(observed!=null && observed.Length==2,"exact native pair observations required");
  ProcessFacts target=null;foreach(ProcessFacts p in observed){if(p.pid==pid){Require(target==null,"duplicate process data");target=p;}}
  Require(target!=null && !target.exited && (pid==21500 || pid==75744),"exact task process data required");
  string expected=Base+(pid==21500?@"\I\runtime\versions\2.99.0\dependencies\node\node.exe":@"\I\ocxd.exe");
  Require(Same(target.image,expected),"native query image data differs");
  Require(String.IsNullOrEmpty(optionalWmiImage) || Same(optionalWmiImage,target.image),"WMI contradicts native image data");
  Require(!String.IsNullOrEmpty(command) && command.Length<32768,"bounded real command line required");
  foreach(char c in command)Require(!Char.IsControl(c),"command control character refused");
  int count;IntPtr raw=CommandLineToArgvW(command,out count);Require(raw!=IntPtr.Zero,"native argv query failed");
  string[] args;try{Require(count>0 && count<=32,"bounded argv required");args=new string[count];for(int i=0;i<count;i++)args[i]=Marshal.PtrToStringUni(Marshal.ReadIntPtr(raw,i*IntPtr.Size));}finally{LocalFree(raw);}
  Require(Same(args[0],target.image),"command image contradicts exact native image data");
  if(pid==21500){
   Require(parent==75744 && args.Length==5 && Same(args[1],Base+@"\I\runtime\versions\2.99.0\opencodex\node_modules\@bitkyc08\opencodex\bin\ocx.mjs") && args[2]=="start" && args[3]=="--port" && args[4]=="0","exact Node command/parent differs");
  }else{Require(args.Length==2 && args[1]=="--internal-runtime-owner024","exact broker command differs");}
  return new LineageData{pid=pid,parentPid=parent,nativeImage=target.image,optionalWmiImage=optionalWmiImage,optionalWmiImageUnavailable=String.IsNullOrEmpty(optionalWmiImage),arguments=args,consistent=true};
 }
 public static void RequireContext() {
  using(Process p=Process.GetCurrentProcess()) {TokenFacts t=Token(p.Handle);
   Require(t.sid==Sid && t.session==7 && t.elevated==0 && t.elevationType==1 && !InJob(p.Handle),"exact ordinary SID1003/session7/noJob required");}
  Guid id=new Guid("5E6C858F-0E22-4760-9AFE-EA3317B67173");IntPtr value;int rc=SHGetKnownFolderPath(ref id,0,IntPtr.Zero,out value);
  if(rc!=0)Marshal.ThrowExceptionForHR(rc);try{Require(Same(Marshal.PtrToStringUni(value),@"C:\Users\marenben"),"actual OS Profile differs");}finally{Marshal.FreeCoTaskMem(value);}
 }
 public static Pair ObserveExactPair() {
  RequireContext();List<Target> targets=new List<Target>();
  try {
   targets.Add(new Target(21500,"134393657318681068",Base+@"\I\runtime\versions\2.99.0\dependencies\node\node.exe","5cba0ea928508c65ddadefc0681d2fb6d95f1f3beea4428f04397631140ee8e5",Sid,7,true,false,true));
   targets.Add(new Target(75744,"134393656723346634",Base+@"\I\ocxd.exe","568f9c7916a4d3c70ad8966cc54bcac263b2faa99fdb56c994d48dd42642bec0",Sid,7,true,true,false));
   return new Pair(targets.ToArray());
  }catch{foreach(Target t in targets)t.Dispose();throw;}
 }
 public static ProcessFacts ProbeProcess(int pid,string expectedHash,string expectedFiletime) {
  using(Process p=Process.GetProcessById(pid)) {
   long c,e,k,u;Check(GetProcessTimes(p.Handle,out c,out e,out k,out u));
   TokenFacts token=Token(p.Handle);
   using(Target t=new Target((uint)p.Id,String.IsNullOrEmpty(expectedFiletime)?c.ToString(CultureInfo.InvariantCulture):expectedFiletime,p.MainModule.FileName,expectedHash,token.sid,token.session,false,false,false)){return t.facts;}
  }
 }
}

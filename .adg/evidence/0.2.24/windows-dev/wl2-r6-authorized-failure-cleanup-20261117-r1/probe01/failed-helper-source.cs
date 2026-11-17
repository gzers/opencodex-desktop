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

// Explicitly authorized external FAILURE cleanup. No original Job/Owner,
// NormalExit, restore, Closed, resume or file-deletion authority exists here.
public static class R6AuthorizedFailureCleanup {
 const string Sid = "S-1-5-21-587408064-856152790-2583328096-1003";
 const string Base = @"C:\Users\marenben\ocx024-r6-existing-20261117-r1";
 const string Archive = @"C:\Users\marenben\ocx024-r6-private-failure-archive-20261117-r3";
 const string PublicArchive = @"C:\Users\Public\ocx-r6-failure-archive-20261117-r3";
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
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool TerminateProcess(IntPtr p,uint code);
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
 static bool Same(string a,string b) {return String.Equals(Path.GetFullPath(a).TrimEnd('\'),Path.GetFullPath(b).TrimEnd('\'),StringComparison.OrdinalIgnoreCase);}
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
     SafeFileHandle pin=CreateFile(parent,0x80,3,IntPtr.Zero,3,0x02200000,IntPtr.Zero);
     if(pin.IsInvalid){pin.Dispose();throw new Win32Exception(Marshal.GetLastWin32Error());}
     ancestors.Add(pin);Identity(pin,parent,true);
    }
    SafeFileHandle file=CreateFile(this.path,0x80000000,liveWriter?3u:1u,IntPtr.Zero,3,0x00200000,IntPtr.Zero);
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
   handle=OpenProcess(0x00101001,false,pid);if(handle==IntPtr.Zero)throw new Win32Exception(Marshal.GetLastWin32Error());
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
  public uint Force(Stopwatch action) {
   Require(action.ElapsedMilliseconds<10000,"original external cleanup window expired");
   Check(TerminateProcess(handle,242));
   uint left=(uint)Math.Max(0,10000-action.ElapsedMilliseconds);
   Require(left>0 && WaitForSingleObject(handle,left)==0,"same-handle forced exit readback timed out");
   uint code;Check(GetExitCodeProcess(handle,out code));Require(code==242,"forced raw exit differs");return code;
  }
  public void Dispose(){if(pin!=null)pin.Dispose();if(handle!=IntPtr.Zero)Check(CloseHandle(handle));}
 }
 public sealed class Report {
  public string scope="explicit two-target external forced failure cleanup; not original Job/NormalStop";
  public ProcessFacts[] before;public List<uint> terminatedPids=new List<uint>(),rawExits=new List<uint>();
  public List<string> pinnedSourceFiles=new List<string>();
  public bool bothValidated,attempted,completed,forced,normalStop=false,closed=false,filesDeleted=false,originalJobAdopted=false,allCompanionsProven=false,publicStartOpen=false;
  public string firstError;public long elapsedMs;
 }
 public sealed class Prepared : IDisposable {
  readonly Target[] targets;readonly List<Pinned> source;readonly bool production;
  public readonly Report report=new Report();
  internal Prepared(Target[] targets,List<Pinned> source,bool production) {this.targets=targets;this.source=source;this.production=production;report.before=new ProcessFacts[]{targets[0].facts,targets[1].facts};foreach(Pinned p in source)report.pinnedSourceFiles.Add(p.path+"|"+p.key+"|"+p.sha);report.bothValidated=true;}
  public Report CommitOnce() {
   Require(!report.attempted,"immutable cleanup already attempted");
   try {
    if(production)RequireContext();foreach(Pinned p in source)p.Recheck();foreach(Target t in targets)t.Recheck();
    report.attempted=true;Stopwatch action=Stopwatch.StartNew();
    try {foreach(Target t in targets){t.Recheck();report.rawExits.Add(t.Force(action));report.terminatedPids.Add(t.facts.pid);report.forced=true;}
     foreach(Pinned p in source)p.Recheck();report.completed=true;
    }finally{report.elapsedMs=action.ElapsedMilliseconds;}
    return report;
   }catch(Exception e){report.firstError=e.ToString();throw;}
  }
  public void Dispose(){foreach(Target t in targets)t.Dispose();foreach(Pinned p in source)p.Dispose();}
 }
 static void RequireContext() {
  using(Process p=Process.GetCurrentProcess()) {TokenFacts t=Token(p.Handle);
   Require(t.sid==Sid && t.session==7 && t.elevated==0 && t.elevationType==1 && !InJob(p.Handle),"exact ordinary SID1003/session7/noJob required");}
  Guid id=new Guid("5E6C858F-0E22-4760-9AFE-EA3317B67173");IntPtr value;int rc=SHGetKnownFolderPath(ref id,0,IntPtr.Zero,out value);
  if(rc!=0)Marshal.ThrowExceptionForHR(rc);try{Require(Same(Marshal.PtrToStringUni(value),@"C:\Users\marenben"),"actual OS Profile differs");}finally{Marshal.FreeCoTaskMem(value);}
 }
#if !R6_CLEANUP_PROBE
 public static Prepared PrepareExact() {
  RequireContext();List<Pinned> files=new List<Pinned>();List<Target> targets=new List<Target>();
  try {
   files.Add(new Pinned(Archive+@"\failed-existing-root-original-bytes.zip","d7664840a85258fe49b8488e79861285d1748ab6fddb7cd18424abd3e10a0026",338702293,false));
   files.Add(new Pinned(PublicArchive+@"\out01\archive-result.json","465e946bdefbbf7ac749001ba3f74e4cc2dcce3d0dcfef5f87ed1b6e04fabd70",6171427,false));
   files.Add(new Pinned(Base+@"\D\qualification-result.json","887e4d59b4987b04fec3f4f5fe4c914af4fe567e8f62997c470f61cd8828747c",1045,false));
   files.Add(new Pinned(Base+@"\D\qualification-start1.json","8e8e5d2d109019c19ced038ff1c8a0ec4b7968ae04d1378225cf0e837b4aaabd",980,false));
   files.Add(new Pinned(Base+@"\D\qualification-events.jsonl","fe716402c8484dd111f50b07dda6452368ab496b0219be1daebd768bf2c021f6",2556,true));
   targets.Add(new Target(50580,"134393577919362322",Base+@"\I\runtime\versions\2.99.0\dependencies\node\node.exe","5cba0ea928508c65ddadefc0681d2fb6d95f1f3beea4428f04397631140ee8e5",Sid,7,true,false,true));
   targets.Add(new Target(54692,"134393577413203335",Base+@"\I\ocxd.exe","0249a1b5e55e554274f7759c3cbe99d1f581fed045dcb1080fc03a03ad0d5a0b",Sid,7,true,true,false));
   return new Prepared(targets.ToArray(),files,true);
  }catch{foreach(Target t in targets)t.Dispose();foreach(Pinned p in files)p.Dispose();throw;}
 }
#endif
#if R6_CLEANUP_PROBE
 [StructLayout(LayoutKind.Sequential,CharSet=CharSet.Unicode)] struct Startup {public uint cb;public string reserved,desktop,title;public uint x,y,w,h,xc,yc,fill,flags;public ushort show,reservedBytes;public IntPtr reservedPtr,input,output,error;}
 [StructLayout(LayoutKind.Sequential)] struct Created {public IntPtr process,thread;public uint pid,tid;}
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool CreateProcess(string app,StringBuilder command,IntPtr ps,IntPtr ts,bool inherit,uint flags,IntPtr env,string directory,ref Startup startup,out Created process);
 static Created NewSuspended(string image) {Startup s=new Startup();s.cb=(uint)Marshal.SizeOf(s);Created p;Check(CreateProcess(image,new StringBuilder("\""+image+"\" /d /c exit 0"),IntPtr.Zero,IntPtr.Zero,false,0x08000004,IntPtr.Zero,Path.GetDirectoryName(image),ref s,out p));return p;}
 static string CreatedTime(Created p) {long c,e,k,u;Check(GetProcessTimes(p.process,out c,out e,out k,out u));return c.ToString(CultureInfo.InvariantCulture);}
 static string Hash(string p) {using(FileStream f=File.OpenRead(p))using(SHA256 h=SHA256.Create()){return BitConverter.ToString(h.ComputeHash(f)).Replace("-","").ToLowerInvariant();}}
 static Prepared Pair(Created a,Created b,string image,string hash,string sid,int session,string fault) {
  List<Target> targets=new List<Target>();try {
   targets.Add(new Target(a.pid,CreatedTime(a),image,hash,sid,session,false,false,true));
   targets.Add(new Target(b.pid,fault=="filetime"?"1":CreatedTime(b),image,fault=="hash"?new string('0',64):hash,fault=="sid"?sid+"-999":sid,session,false,false,true));
   return new Prepared(targets.ToArray(),new List<Pinned>(),false);
  }catch{foreach(Target t in targets)t.Dispose();throw;}
 }
 public sealed class ProbeReport {public bool passed,ordinaryAcceptance=false,officialExecuted=false,marenbenTouched=false,resumed=false;public List<string> refused=new List<string>();public Report cleanup;public string firstError;public List<uint> originalHandleExits=new List<uint>();}
 public static ProbeReport Probe(string root) {
  Require(!Directory.Exists(root),"fresh probe directory required");Directory.CreateDirectory(root);
  string image=Path.Combine(root,"cmd.exe");File.Copy(Path.Combine(Environment.SystemDirectory,"cmd.exe"),image,false);
  string hash=Hash(image);Created a=new Created(),b=new Created();ProbeReport report=new ProbeReport();
  try {
   a=NewSuspended(image);b=NewSuspended(image);TokenFacts token;using(Process current=Process.GetCurrentProcess()){token=Token(current.Handle);}
   foreach(string fault in new string[]{"filetime","sid","hash"}) {
    bool refused=false;try{using(Prepared unexpected=Pair(a,b,image,hash,token.sid,token.session,fault)){throw new Exception("fault accepted");}}
    catch(InvalidDataException e){string expected=fault=="filetime"?"exact FILETIME/PID differs":fault=="sid"?"exact TokenUser/session differs":"pinned file SHA256 differs";Require(e.Message==expected,"wrong rejection cause: "+e.Message);refused=true;}
    Require(refused && WaitForSingleObject(a.process,0)==258 && WaitForSingleObject(b.process,0)==258,"precondition rejection affected an original process");report.refused.Add(fault);
   }
   using(Prepared valid=Pair(a,b,image,hash,token.sid,token.session,"none")) {
    Require(valid.report.before[0].threadWaitReason=="Suspended" && valid.report.before[1].threadWaitReason=="Suspended","real never-resumed thread state not proven");
    report.cleanup=valid.CommitOnce();Require(report.cleanup.completed && report.cleanup.rawExits.Count==2 && report.cleanup.rawExits[0]==242 && report.cleanup.rawExits[1]==242 && !report.cleanup.normalStop && !report.cleanup.closed,"forced-only cleanup fact differs");
    bool replay=false;try{valid.CommitOnce();}catch(InvalidDataException e){replay=e.Message=="immutable cleanup already attempted";}Require(replay,"cleanup replay accepted");report.refused.Add("same-object replay");
   }
   report.passed=true;
  }catch(Exception e){report.firstError=e.ToString();}
  finally {
   foreach(Created p in new Created[]{a,b}) {if(p.process==IntPtr.Zero)continue;try {if(WaitForSingleObject(p.process,0)==258){Check(TerminateProcess(p.process,242));Require(WaitForSingleObject(p.process,5000)==0,"own probe cleanup timed out");}uint code;Check(GetExitCodeProcess(p.process,out code));report.originalHandleExits.Add(code);}finally{Check(CloseHandle(p.thread));Check(CloseHandle(p.process));}}
  }
  return report;
 }
#endif
}

// Test-only operation compensation. A refused batch never grants write authority.
using System;
using System.IO;
using System.Linq;
using System.Text;
using System.Collections.Generic;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Security.AccessControl;
using System.Security.Principal;
using System.Security.Cryptography;
using Microsoft.Win32.SafeHandles;

namespace Official299Proof {
public sealed class FileImage {
 public string Name {get;set;} public string Id {get;set;} public string Sha256 {get;set;}
 public string Bytes {get;set;} public string Sddl {get;set;} public string Owner {get;set;}
}
public sealed class AgentImage {
 public string Root {get;set;} public string RootId {get;set;} public FileImage[] Files {get;set;}
 public string CapturedUtc {get;set;}
}
public sealed class AgentChange {
 public string Name {get;set;} public FileImage Before {get;set;} public FileImage After {get;set;}
}
public sealed class AgentPlan {
 public string TransactionId {get;set;} public string Root {get;set;} public string RootId {get;set;}
 public AgentChange[] Changes {get;set;} public string SealedUtc {get;set;}
}
public static class Recovery {
 const string Marker="generated-by: opencodex";
 [StructLayout(LayoutKind.Sequential)] struct INFO {public uint attrs,cl,ch,al,ah,wl,wh,volume,sh,sl,links,ih,il;}
 [DllImport("kernel32",SetLastError=true)] static extern bool GetFileInformationByHandle(IntPtr h,out INFO i);
 [DllImport("kernel32",SetLastError=true)] static extern bool SetFileInformationByHandle(IntPtr h,int kind,IntPtr b,uint size);
 [DllImport("advapi32")] static extern uint GetSecurityInfo(IntPtr h,int kind,uint requested,out IntPtr owner,out IntPtr group,out IntPtr dacl,out IntPtr sacl,out IntPtr sd);
 [DllImport("advapi32",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool ConvertSecurityDescriptorToStringSecurityDescriptorW(IntPtr sd,uint revision,uint info,out IntPtr text,out uint len);
 [DllImport("advapi32",SetLastError=true)] static extern bool GetSecurityDescriptorDacl(IntPtr sd,out bool present,out IntPtr dacl,out bool defaulted);
 [DllImport("advapi32")] static extern uint SetSecurityInfo(IntPtr h,int kind,uint requested,IntPtr owner,IntPtr group,IntPtr dacl,IntPtr sacl);
 internal static string Hash(byte[] b){return Convert.ToHexString(SHA256.HashData(b)).ToLowerInvariant();}
 static string FileId(IntPtr h){INFO i;Native.Check(GetFileInformationByHandle(h,out i));if((i.attrs&0x410)!=0||i.links!=1)throw new InvalidOperationException("unsafe-file: link, directory or multiple hardlinks");return i.volume+":"+i.ih+":"+i.il;}
 static string Sddl(IntPtr h,out string sid){IntPtr o,g,d,s,sd;uint rc=GetSecurityInfo(h,1,7,out o,out g,out d,out s,out sd);if(rc!=0)throw new Win32Exception((int)rc);try{sid=new SecurityIdentifier(o).Value;IntPtr text;uint n;Native.Check(ConvertSecurityDescriptorToStringSecurityDescriptorW(sd,1,7,out text,out n));try{return Marshal.PtrToStringUni(text);}finally{Native.LocalFree(text);}}finally{Native.LocalFree(sd);}}
 static void Acl(IntPtr h,string sddl){IntPtr sd;uint n;Native.Check(Native.ConvertStringSecurityDescriptorToSecurityDescriptorW(sddl,1,out sd,out n));try{bool p,z;IntPtr a;Native.Check(GetSecurityDescriptorDacl(sd,out p,out a,out z));if(!p||a==IntPtr.Zero)throw new InvalidOperationException("null-dacl");var raw=new RawSecurityDescriptor(sddl);uint bits=4|((raw.ControlFlags&ControlFlags.DiscretionaryAclProtected)!=0?0x80000000u:0x20000000u);uint rc=SetSecurityInfo(h,1,bits,IntPtr.Zero,IntPtr.Zero,a,IntPtr.Zero);if(rc!=0)throw new Win32Exception((int)rc);}finally{Native.LocalFree(sd);}}
 internal sealed class Locked : IDisposable {
  public IntPtr H; public FileStream Stream; public string Name; public bool Reserved;
  public Locked(string path,bool write,bool create=false){Name=Path.GetFileName(path);var sa=new Native.SA{length=Marshal.SizeOf<Native.SA>()};H=Native.CreateFileW(path,write?0xC00F0000u:0x80020000u,write?0u:1u,ref sa,create?1u:3u,0x00200080,IntPtr.Zero);Native.Check(H!=new IntPtr(-1));try{FileId(H);Stream=new FileStream(new SafeFileHandle(H,false),write?FileAccess.ReadWrite:FileAccess.Read);Reserved=create;}catch{Native.CloseHandle(H);H=IntPtr.Zero;throw;}}
  public FileImage Image(){string owner;var acl=Sddl(H,out owner);if(owner!=WindowsIdentity.GetCurrent().User.Value)throw new InvalidOperationException("wrong-owner");Stream.Position=0;if(Stream.Length>1048576)throw new InvalidOperationException("agent-too-large");byte[] b=new byte[(int)Stream.Length];Stream.ReadExactly(b);return new FileImage{Name=Name,Id=FileId(H),Bytes=Convert.ToBase64String(b),Sha256=Hash(b),Sddl=acl,Owner=owner};}
  public void Write(FileImage im){var b=Convert.FromBase64String(im.Bytes);Stream.Position=0;Stream.Write(b);Stream.SetLength(b.Length);Stream.Flush(true);Acl(H,im.Sddl);var now=Image();if(now.Sha256!=im.Sha256||now.Sddl!=im.Sddl||now.Owner!=im.Owner)throw new InvalidOperationException("restored-bytes-or-acl-differ");}
  public void Delete(bool yes){var p=Marshal.AllocHGlobal(1);try{Marshal.WriteByte(p,yes?(byte)1:(byte)0);Native.Check(SetFileInformationByHandle(H,4,p,1));}finally{Marshal.FreeHGlobal(p);}}
  public void Dispose(){if(Stream!=null){Stream.Dispose();Stream=null;}if(H!=IntPtr.Zero){Native.CloseHandle(H);H=IntPtr.Zero;}}
 }
 internal static bool Same(FileImage a,FileImage b){return a==null?b==null:b!=null&&a.Id==b.Id&&a.Sha256==b.Sha256&&a.Sddl==b.Sddl&&a.Owner==b.Owner;}
 static bool Generated(FileImage f){return f!=null&&Encoding.UTF8.GetString(Convert.FromBase64String(f.Bytes)).Contains(Marker,StringComparison.Ordinal);}
 static void Name(string n){if(n!=Path.GetFileName(n)||n.Contains(':')||!n.StartsWith("ocx-",StringComparison.Ordinal)||!n.EndsWith(".md",StringComparison.Ordinal))throw new InvalidOperationException("not-official-operation-path");}
 internal static List<Guard> Ancestors(string root){var gs=new List<Guard>();try{string p=Path.GetFullPath(root);while(!string.IsNullOrEmpty(p)){gs.Add(new Guard(p,true));var next=Path.GetDirectoryName(p);if(next==p)break;p=next;}return gs;}catch{foreach(var g in gs)g.Dispose();throw;}}
 public static AgentImage Capture(string root){root=Path.GetFullPath(root);var gs=Ancestors(root);try{var files=new List<FileImage>();foreach(var p in Directory.GetFileSystemEntries(root).OrderBy(p=>p,StringComparer.Ordinal)){using(var f=new Locked(p,false))files.Add(f.Image());}return new AgentImage{Root=root,RootId=gs[0].Identity,Files=files.ToArray(),CapturedUtc=DateTime.UtcNow.ToString("o")};}finally{foreach(var g in gs)g.Dispose();}}
 public static string RemoveExactFixtureFile(string path,string expectedId){using(var f=new Locked(path,true)){var im=f.Image();if(im.Id!=expectedId)throw new InvalidOperationException("cleanup-file-generation-changed");f.Delete(true);return im.Sha256;}}
 public static AgentPlan Plan(AgentImage before,AgentImage after,string transaction){
  if(before.Root!=after.Root||before.RootId!=after.RootId||string.IsNullOrEmpty(transaction))throw new InvalidOperationException("transaction-root-identity-differs");
  var changes=new List<AgentChange>();var b=before.Files.ToDictionary(x=>x.Name,StringComparer.OrdinalIgnoreCase);var a=after.Files.ToDictionary(x=>x.Name,StringComparer.OrdinalIgnoreCase);
  foreach(var n in b.Keys.Union(a.Keys,StringComparer.OrdinalIgnoreCase).OrderBy(x=>x,StringComparer.Ordinal)){FileImage x,y;b.TryGetValue(n,out x);a.TryGetValue(n,out y);if(x!=null&&y!=null&&x.Sha256==y.Sha256&&x.Sddl==y.Sddl)continue;Name(n);if((x!=null&&!Generated(x))||(y!=null&&!Generated(y)))throw new InvalidOperationException("unowned-change-refused");changes.Add(new AgentChange{Name=n,Before=x,After=y});}
  return new AgentPlan{TransactionId=transaction,Root=before.Root,RootId=before.RootId,Changes=changes.ToArray(),SealedUtc=after.CapturedUtc};
 }
 // Test hook runs after ALL existing targets are held exclusively, before reservation/write.
 // A CREATE_NEW collision rolls back reservations without overwriting the competing file.
 public static string[] Restore(AgentPlan plan,int failAfterWrites,Action preWriteHook){
  var gs=Ancestors(plan.Root);var locks=new Dictionary<string,Locked>(StringComparer.OrdinalIgnoreCase);var operations=new List<string>();var altered=new List<AgentChange>();bool committed=false;int writes=0;
  try{
   if(gs[0].Identity!=plan.RootId)throw new InvalidOperationException("directory-generation-differs");
   foreach(var c in plan.Changes){Name(c.Name);string p=Path.Combine(plan.Root,c.Name);if(c.After!=null){var f=new Locked(p,true);locks.Add(c.Name,f);if(!Same(c.After,f.Image()))throw new InvalidOperationException("concurrent-postimage-change");}else {try{File.GetAttributes(p);throw new InvalidOperationException("deleted-path-reappeared");}catch(FileNotFoundException){}}}
   preWriteHook?.Invoke();
   // Reserve all originally deleted paths before any existing file content is changed.
   foreach(var c in plan.Changes.Where(x=>x.After==null)){var f=new Locked(Path.Combine(plan.Root,c.Name),true,true);locks.Add(c.Name,f);operations.Add("reserve:"+c.Name);}
   foreach(var c in plan.Changes){var f=locks[c.Name];altered.Add(c);if(c.Before!=null){f.Write(c.Before);operations.Add("restore:"+c.Name);}else{f.Delete(true);operations.Add("delete-owned-postimage:"+c.Name);}writes++;if(failAfterWrites>0&&writes==failAfterWrites)throw new IOException("injected-transaction-failure");}
   committed=true;operations.Add("commit:"+plan.TransactionId);return operations.ToArray();
  }catch{
   foreach(var c in altered.AsEnumerable().Reverse()){var f=locks[c.Name];if(c.After!=null){f.Delete(false);f.Write(c.After);}else{f.Delete(true);}}
   foreach(var c in plan.Changes.Where(x=>x.After==null))if(locks.TryGetValue(c.Name,out var f))f.Delete(true);
   throw;
  }finally{foreach(var f in locks.Values){if(!committed&&f.Reserved)f.Delete(true);f.Dispose();}foreach(var g in gs)g.Dispose();}
 }
}
}

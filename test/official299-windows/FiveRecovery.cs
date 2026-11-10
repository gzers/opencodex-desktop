// Revision4 test fixture only. No credential/path discovery and no stable-lock deletion.
using System;
using System.IO;
using System.Linq;
using System.Collections.Generic;
using System.Text;
using System.Text.Json;
using System.ComponentModel;
using System.Security.Principal;
using System.Security.AccessControl;

namespace Official299Proof {
public sealed class Artifact {
 public string Path {get;set;} public string State {get;set;} public FileImage Image {get;set;}
 public string FailureType {get;set;} public int Error {get;set;} public string Stage {get;set;}
 public string CapturedUtc {get;set;} public long Length {get;set;}
}
public sealed class FivePlan {
 public string TransactionId {get;set;} public string Root {get;set;} public string RootId {get;set;}
 public string CodexId {get;set;} public string GrokId {get;set;} public string SourceSha256 {get;set;}
 public Artifact[] Before {get;set;} public Artifact[] After {get;set;} public string ProducerPhase {get;set;}
}
public static class FiveRecovery {
 public static readonly string[] Paths={"codex/config.toml","codex/opencodex-catalog.json","codex/models_cache.json","codex/opencodex-catalog-audit.jsonl","grok/config.toml.bak-opencodex"};
 static readonly string[] Collectable=Paths.Concat(new[]{"codex/auth.json","claude/settings.json","grok/config.toml"}).ToArray();
 public static string Hash(string p){return Recovery.Hash(File.ReadAllBytes(p));}
 static string Full(string root,string rel){if(!Collectable.Contains(rel,StringComparer.Ordinal))throw new InvalidOperationException("unregistered-synthetic-object");return System.IO.Path.Combine(root,rel.Replace('/',System.IO.Path.DirectorySeparatorChar));}
 static bool Absent(Exception e){return e is FileNotFoundException||e is DirectoryNotFoundException||e is Win32Exception w&&(w.NativeErrorCode==2||w.NativeErrorCode==3);}
 public static Artifact[] Capture(string root,string stage,string[] selected){
  foreach(var rel in selected)Full(root,rel);var list=new List<Artifact>();
  foreach(var rel in selected){var row=new Artifact{Path=rel,Stage=stage,CapturedUtc=DateTime.UtcNow.ToString("o")};var gs=new List<Guard>();
   try{gs=Recovery.Ancestors(System.IO.Path.GetDirectoryName(Full(root,rel)));using(var file=new Recovery.Locked(Full(root,rel),false)){row.Image=file.Image();row.Length=Convert.FromBase64String(row.Image.Bytes).LongLength;row.State="file";}}
   catch(Exception e){row.State=Absent(e)?"absent":"unknown";row.FailureType=e.GetType().FullName;row.Error=e is Win32Exception w?w.NativeErrorCode:e.HResult;}
   finally{foreach(var g in gs)g.Dispose();}list.Add(row);
  }return list.ToArray();
 }
 static bool Content(Artifact a,Artifact b,bool identity){return a.State==b.State&&(a.State=="absent"||(a.State=="file"&&a.Image.Sha256==b.Image.Sha256&&a.Image.Sddl==b.Image.Sddl&&a.Image.Owner==b.Image.Owner&&(!identity||a.Image.Id==b.Image.Id)));}
 public static bool Equal(Artifact[] a,Artifact[] b,bool identity){return a.Length==b.Length&&a.All(x=>b.Any(y=>x.Path==y.Path&&Content(x,y,identity)));}
 static void Valid(Artifact x){if(x.State=="absent")return;if(x.State!="file"||x.Image==null)throw new InvalidOperationException("unknown-is-not-authority");var bytes=Convert.FromBase64String(x.Image.Bytes);var sid=WindowsIdentity.GetCurrent().User.Value;if(bytes.Length>1048576||bytes.LongLength!=x.Length||Recovery.Hash(bytes)!=x.Image.Sha256||x.Image.Owner!=sid)throw new InvalidOperationException("preimage-or-postimage-integrity");var acl=new RawSecurityDescriptor(x.Image.Sddl);if(acl.DiscretionaryAcl==null)throw new InvalidOperationException("missing-private-DACL");foreach(GenericAce ace in acl.DiscretionaryAcl){if(ace is not CommonAce a||a.AceQualifier!=AceQualifier.AccessAllowed||(a.SecurityIdentifier.Value!=sid&&a.SecurityIdentifier.Value!="S-1-5-18"))throw new InvalidOperationException("nonprivate-preimage-or-postimage-DACL");}}
 public static FivePlan Plan(string root,Artifact[] before,Artifact[] after,string txn,string source,string phase){
  foreach(var a in new[]{before,after}){if(a.Length!=5||Paths.Any(p=>a.Count(x=>x.Path==p)!=1))throw new InvalidOperationException("exact-five-targets-required");foreach(var x in a)Valid(x);}
  if(string.IsNullOrEmpty(txn)||source.Length!=64||string.IsNullOrEmpty(phase))throw new InvalidOperationException("missing-transaction-producer-binding");
  using(var r=new Guard(root,true))using(var c=new Guard(System.IO.Path.Combine(root,"codex"),true))using(var g=new Guard(System.IO.Path.Combine(root,"grok"),true))return new FivePlan{Root=System.IO.Path.GetFullPath(root),RootId=r.Identity,CodexId=c.Identity,GrokId=g.Identity,Before=before,After=after,TransactionId=txn,SourceSha256=source,ProducerPhase=phase};
 }
 public static string SavePlan(string path,FivePlan plan){var bytes=JsonSerializer.SerializeToUtf8Bytes(plan);using(var f=new FileStream(path,FileMode.CreateNew,FileAccess.Write,FileShare.Read)){f.Write(bytes);f.Flush(true);}return Recovery.Hash(bytes);}
 static FivePlan Load(string path,string digest){var b=File.ReadAllBytes(path);if(Recovery.Hash(b)!=digest)throw new InvalidOperationException("plan-integrity-changed");var plan=JsonSerializer.Deserialize<FivePlan>(b);if(plan==null)throw new InvalidOperationException("missing-plan");return plan;}
 static void Append(string journal,object record){var b=Encoding.UTF8.GetBytes(JsonSerializer.Serialize(record)+"\n");using(var f=new FileStream(journal,FileMode.Append,FileAccess.Write,FileShare.Read)){f.Write(b);f.Flush(true);}}
 // Protected expected plan SHA is an independent input, not read from the mutable journal.
 public static string[] Apply(string planFile,string digest,string journal,bool recover,int failAfter,int crashAfter,Action preWrite){
  using(var protection=new FileStream(planFile,FileMode.Open,FileAccess.Read,FileShare.Read)){
   var plan=Load(planFile,digest);foreach(var x in plan.Before.Concat(plan.After))Valid(x);
   if(plan.Before.Length!=5||plan.After.Length!=5||Paths.Any(p=>plan.Before.Count(x=>x.Path==p)!=1||plan.After.Count(x=>x.Path==p)!=1))throw new InvalidOperationException("plan-targets-changed");
   var gs=Recovery.Ancestors(plan.Root);var locked=new Dictionary<string,Recovery.Locked>();var current=new Dictionary<string,Artifact>();var modified=new List<string>();bool commit=false;var ops=new List<string>();
   try{
    using(var c=new Guard(System.IO.Path.Combine(plan.Root,"codex"),true))using(var g=new Guard(System.IO.Path.Combine(plan.Root,"grok"),true)){
     if(gs[0].Identity!=plan.RootId||c.Identity!=plan.CodexId||g.Identity!=plan.GrokId)throw new InvalidOperationException("root-generation-changed");
     foreach(var rel in Paths){var target=Full(plan.Root,rel);Artifact now;
      try{var f=new Recovery.Locked(target,true);locked.Add(rel,f);now=new Artifact{Path=rel,State="file",Image=f.Image()};}
      catch(Exception e){if(!Absent(e))throw;now=new Artifact{Path=rel,State="absent"};}
      var after=plan.After.Single(x=>x.Path==rel);var before=plan.Before.Single(x=>x.Path==rel);
      if(!recover&&!Content(after,now,true))throw new InvalidOperationException("late-postimage-conflict");
      if(recover&&!Content(after,now,false)&&!Content(before,now,false))throw new InvalidOperationException("recovery-third-state-refused");
      if(recover&&now.State=="file"&&now.Image.Id!=after.Image?.Id&&now.Image.Id!=before.Image?.Id)throw new InvalidOperationException("recovery-file-generation-unknown");
      current[rel]=now;
     }
     preWrite?.Invoke();
     // Reserve all required absent destinations before touching existing content.
     foreach(var rel in Paths){var desired=(recover?plan.After:plan.Before).Single(x=>x.Path==rel);if(current[rel].State=="absent"&&desired.State=="file")locked.Add(rel,new Recovery.Locked(Full(plan.Root,rel),true,true));}
     Append(journal,new {state=recover?"recovery-intent":"apply-intent",planSha256=digest,transactionId=plan.TransactionId,utc=DateTime.UtcNow.ToString("o")});
     foreach(var rel in Paths){var desired=(recover?plan.After:plan.Before).Single(x=>x.Path==rel);if(Content(current[rel],desired,false))continue;
      modified.Add(rel);Append(journal,new{state="before-operation",path=rel,expected=current[rel].State,want=desired.State});
      var f=locked[rel];if(desired.State=="file")f.Write(desired.Image);else f.Delete(true);
      ops.Add((recover?"recover:":"restore:")+rel);Append(journal,new{state="after-operation",path=rel,want=desired.State});
      if(crashAfter>0&&modified.Count==crashAfter)Environment.Exit(91);
      if(failAfter>0&&modified.Count==failAfter)throw new IOException("injected-second-write-failure");
     }
     Append(journal,new{state=recover?"rolled-back-to-sealed-postimage":"committed-originals",planSha256=digest});commit=true;
    }
   }catch{
    foreach(var rel in modified.AsEnumerable().Reverse()){var was=current[rel];var f=locked[rel];if(was.State=="file"){f.Delete(false);f.Write(was.Image);}else f.Delete(true);}
    foreach(var f in locked.Values)if(f.Reserved)f.Delete(true);
    Append(journal,new{state="failed-reverted-or-refused",modified=modified.ToArray(),planSha256=digest});throw;
   }finally{foreach(var f in locked.Values){if(!commit&&f.Reserved)f.Delete(true);f.Dispose();}foreach(var g in gs)g.Dispose();}
   var final=Capture(plan.Root,"readback",Paths);var expected=recover?plan.After:plan.Before;if(!Equal(final,expected,false))throw new InvalidOperationException("final-readback-differs");return ops.ToArray();
  }
 }
}
}

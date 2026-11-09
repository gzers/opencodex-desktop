param([Parameter(Mandatory=$true)][string]$Directory)
$ErrorActionPreference='Stop'
. 'C:\Users\15119\.codex\worktrees\ocxd-023-windows-r1\code\test\smoke\windows-process-token.ps1'
$expectedSid='S-1-5-21-587408064-856152790-2583328096-1018'
$token=Get-WindowsHarnessToken;Assert-WindowsProcessMode $token administrator
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Security.Principal;
[StructLayout(LayoutKind.Sequential)] public struct OcxTailLuid {public uint Low;public int High;}
[StructLayout(LayoutKind.Sequential)] public struct OcxTailLsaString {public ushort Length;public ushort MaximumLength;public IntPtr Buffer;}
// Read only the documented prefix through LogonTime; later profile/credential fields are not read.
[StructLayout(LayoutKind.Sequential)] public struct OcxTailLogonPrefix {public uint Size;public OcxTailLuid LogonId;public OcxTailLsaString UserName;public OcxTailLsaString Domain;public OcxTailLsaString Package;public uint LogonType;public uint Session;public IntPtr Sid;public long LogonTime;}
public sealed class OcxTailLogonMatch {public string Sid;public uint LogonType;public uint Session;public string LogonUtc;}
public sealed class OcxTailLogonSnapshot {public bool EnumerationSucceeded;public int Status;public uint Enumerated;public int ReadFailures;public int NoSid;public List<OcxTailLogonMatch> Matches=new List<OcxTailLogonMatch>();}
public static class OcxTailLsa {
 [DllImport("secur32.dll",ExactSpelling=true)] static extern int LsaEnumerateLogonSessions(out uint count,out IntPtr sessions);
 [DllImport("secur32.dll",ExactSpelling=true)] static extern int LsaGetLogonSessionData(ref OcxTailLuid luid,out IntPtr data);
 [DllImport("secur32.dll",ExactSpelling=true)] static extern int LsaFreeReturnBuffer(IntPtr buffer);
 public static OcxTailLogonSnapshot Snapshot(string expected) {
  uint count;IntPtr list;var r=new OcxTailLogonSnapshot();r.Status=LsaEnumerateLogonSessions(out count,out list);
  if(r.Status!=0)return r;
  try {
   if(count>65536||(count>0&&list==IntPtr.Zero))throw new InvalidOperationException("Invalid LSA enumeration shape");
   r.EnumerationSucceeded=true;r.Enumerated=count;
   for(int i=0;i<count;i++){
    var luid=Marshal.PtrToStructure<OcxTailLuid>(IntPtr.Add(list,checked(i*8)));IntPtr data;int status=LsaGetLogonSessionData(ref luid,out data);
    if(status!=0){r.ReadFailures++;continue;}
    if(data==IntPtr.Zero){r.NoSid++;continue;}
    try{
     if((uint)Marshal.ReadInt32(data)<Marshal.SizeOf<OcxTailLogonPrefix>()){r.ReadFailures++;continue;}
     var value=Marshal.PtrToStructure<OcxTailLogonPrefix>(data);
     if(value.Sid==IntPtr.Zero){r.NoSid++;continue;}
     string sid;try{sid=new SecurityIdentifier(value.Sid).Value;}catch(ArgumentException){r.ReadFailures++;continue;}
     if(String.Equals(sid,expected,StringComparison.Ordinal)){
      r.Matches.Add(new OcxTailLogonMatch {Sid=sid,LogonType=value.LogonType,Session=value.Session,LogonUtc=value.LogonTime>0?DateTime.FromFileTimeUtc(value.LogonTime).ToString("o"):null});
     }
    }finally{if(LsaFreeReturnBuffer(data)!=0)throw new InvalidOperationException("LSA session buffer release failed");}
   }
   return r;
  }finally{if(list!=IntPtr.Zero&&LsaFreeReturnBuffer(list)!=0)throw new InvalidOperationException("LSA list buffer release failed");}
 }
}
'@
$clock=[Diagnostics.Stopwatch]::StartNew();$r=[OcxTailLsa]::Snapshot($expectedSid)
$p=Get-CimInstance Win32_UserProfile -Filter ("SID='"+$expectedSid+"'")
@{observedUtc=[datetime]::UtcNow.ToString('o');expectedSid=$expectedSid;scope='Readonly documented LSA logon-session prefix; output only matching synthetic SID/types/session/time and aggregate unknown counts; no credentials, token handles, other usernames or logon IDs';token=$token;elapsedMilliseconds=$clock.ElapsedMilliseconds;lsa=$r;profile=$(if($p){@{sid=$p.SID;path=$p.LocalPath;loaded=$p.Loaded;special=$p.Special}}else{$null});noProfileLoadUnloadOrLogoff=$true}|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $Directory 'query-result.json') -Encoding utf8

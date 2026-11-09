param([Parameter(Mandatory=$true)][string]$Directory)
$ErrorActionPreference='Stop'
$expectedSid='S-1-5-21-587408064-856152790-2583328096-1018'
@{pid=$PID;sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value;atUtc=[datetime]::UtcNow.ToString('o');scope='This read-only helper only'}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $Directory 'query-self.json')
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Security.Principal;
[StructLayout(LayoutKind.Sequential)]
public struct OcxTailWtsProcess { public uint SessionId; public uint ProcessId; public IntPtr Name; public IntPtr Sid; }
public sealed class OcxTailRow { public uint Pid; public uint Session; public string Image; public string Sid; public bool OwnerKnown; }
public sealed class OcxTailSnapshot { public uint RequestedSession; public bool Succeeded; public int Win32Error; public uint Count; public int UnknownOwnerCount; public List<OcxTailRow> Matches=new List<OcxTailRow>(); public List<OcxTailRow> Unknown=new List<OcxTailRow>(); }
public static class OcxTailWts {
 [DllImport("wtsapi32.dll",SetLastError=true,ExactSpelling=true)] static extern bool WTSEnumerateProcessesExW(IntPtr server,ref uint level,uint session,out IntPtr info,out uint count);
 [DllImport("wtsapi32.dll",SetLastError=true,ExactSpelling=true)] static extern bool WTSFreeMemoryExW(int kind,IntPtr info,uint count);
 public static OcxTailSnapshot Snapshot(uint session,string expectedSid) {
  var result=new OcxTailSnapshot {RequestedSession=session}; uint level=0,count=0; IntPtr buffer=IntPtr.Zero;
  if(!WTSEnumerateProcessesExW(IntPtr.Zero,ref level,session,out buffer,out count)){result.Win32Error=Marshal.GetLastWin32Error();return result;}
  try {
   if(level!=0||count>65536||(count>0&&buffer==IntPtr.Zero))throw new InvalidOperationException("Invalid WTS buffer shape");
   result.Succeeded=true;result.Count=count;int size=Marshal.SizeOf<OcxTailWtsProcess>();
   for(int i=0;i<count;i++){
    var p=Marshal.PtrToStructure<OcxTailWtsProcess>(IntPtr.Add(buffer,checked(i*size)));
    var row=new OcxTailRow {Pid=p.ProcessId,Session=p.SessionId,Image=p.Name==IntPtr.Zero?"unknown":Marshal.PtrToStringUni(p.Name)};
    try{if(p.Sid!=IntPtr.Zero){row.Sid=new SecurityIdentifier(p.Sid).Value;row.OwnerKnown=true;}}catch(ArgumentException){}
    if(!row.OwnerKnown){result.UnknownOwnerCount++;if(result.Unknown.Count<64)result.Unknown.Add(row);}
    if(row.OwnerKnown&&String.Equals(row.Sid,expectedSid,StringComparison.Ordinal))result.Matches.Add(row);
   }
   return result;
  } finally {if(buffer!=IntPtr.Zero&&!WTSFreeMemoryExW(0,buffer,count))throw new InvalidOperationException("WTS buffer release failed");}
 }
}
'@
$watch=[Diagnostics.Stopwatch]::StartNew()
$currentSession=[Diagnostics.Process]::GetCurrentProcess().SessionId
$observations=@([OcxTailWts]::Snapshot([uint32]$currentSession,$expectedSid),[OcxTailWts]::Snapshot([uint32]4294967294,$expectedSid))
$profile=Get-CimInstance Win32_UserProfile -Filter ("SID='"+$expectedSid+"'")
$events=@();$eventFailure=$null
try{
 $events=@(Get-WinEvent -FilterHashtable @{LogName='Application';StartTime=[datetime]'2026-11-09T23:30:00';Id=1508,1517,1518,1530,1533,1534} -MaxEvents 100 -ErrorAction Stop|Where-Object {$_.ToXml().Contains($expectedSid) -or $_.Message.Contains('ocxaccu16625a3be3')}|ForEach-Object {@{timeUtc=$_.TimeCreated.ToUniversalTime().ToString('o');id=$_.Id;provider=$_.ProviderName;message=$_.Message}})
}catch{$eventFailure=$_.FullyQualifiedErrorId}
@{observedUtc=[datetime]::UtcNow.ToString('o');expectedSid=$expectedSid;queryProcess=$PID;querySession=$currentSession;elapsedMilliseconds=$watch.ElapsedMilliseconds;scope='Readonly WTS snapshots plus exact disposable Profile and bounded application events; no identity admission for termination';wts=$observations;profile=$(if($profile){@{sid=$profile.SID;path=$profile.LocalPath;loaded=$profile.Loaded;special=$profile.Special}}else{$null});unloadEvents=$events;eventQueryFailure=$eventFailure;unknownIsNotAbsent=$true;noProcessOrRegistryMutation=$true}|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $Directory 'query-result.json') -Encoding utf8

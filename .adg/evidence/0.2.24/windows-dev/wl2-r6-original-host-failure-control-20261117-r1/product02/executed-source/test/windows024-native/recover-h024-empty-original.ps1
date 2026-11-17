param([Parameter(Mandatory)][string]$Evidence)
$ErrorActionPreference='Stop'
$sid='S-1-5-21-587408064-856152790-2583328096-1003'
$id=[Security.Principal.WindowsIdentity]::GetCurrent()
if($id.User.Value -cne $sid -or [Diagnostics.Process]::GetCurrentProcess().SessionId -ne 7 -or ([Security.Principal.WindowsPrincipal]::new($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'original ordinary marenben session7 only'}
$base=Join-Path $env:USERPROFILE 'ocx024-h-13ae0590-60a3-4d5d-accc-bb6329181394'
$expectedEvidence=[IO.Path]::GetFullPath('C:/Users/Public/ocx-h024-repaired-20261115-r1/out01')
if([IO.Path]::GetFullPath($Evidence) -cne $expectedEvidence){throw 'exact new task evidence root required'}
$receipt=Join-Path $Evidence 'original-empty-host-recovery.json'
if(Test-Path -LiteralPath $receipt){throw 'immutable original empty-host recovery already attempted'}
$result=[ordered]@{scope='only original empty diagnostic host; not official service Stop or native Owner adoption';accepted=$false;originalBase=$base;forced=$false;originalACLChanged=$false;nativeWaitConfirmed=$false}
function Digest([byte[]]$Bytes){$h=[Security.Cryptography.SHA256]::Create();try{return ([BitConverter]::ToString($h.ComputeHash($Bytes))).Replace('-','').ToLowerInvariant()}finally{$h.Dispose()}}
function Snapshot([string]$Path){
 $f=[IO.File]::Open($Path,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
 try{$n=$f.Length;if($n -gt 1048576){throw 'original task snapshot bound'};$b=New-Object byte[] ([int]$n);$done=0;while($done -lt $b.Length){$k=$f.Read($b,$done,$b.Length-$done);if($k -le 0){throw 'truncated original snapshot'};$done+=$k};if($f.Length -ne $n){throw 'original snapshot length drift'};return ,$b}finally{$f.Dispose()}
}
function PrivateNewDirectory([string]$Path){
 if(Test-Path -LiteralPath $Path){throw 'new recovery locator directory already exists'}
 $acl=New-Object Security.AccessControl.DirectorySecurity
 $acl.SetOwner([Security.Principal.SecurityIdentifier]::new($sid));$acl.SetAccessRuleProtection($true,$false)
 foreach($who in @($sid,'S-1-5-18')){$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($who),[Security.AccessControl.FileSystemRights]::FullControl,([Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [Security.AccessControl.InheritanceFlags]::ObjectInherit),[Security.AccessControl.PropagationFlags]::None,[Security.AccessControl.AccessControlType]::Allow))}
 [void][IO.Directory]::CreateDirectory($Path,$acl)
}
function PrivateNewFile([string]$Path,[byte[]]$Bytes){
 $acl=New-Object Security.AccessControl.FileSecurity
 $acl.SetOwner([Security.Principal.SecurityIdentifier]::new($sid));$acl.SetAccessRuleProtection($true,$false)
 foreach($who in @($sid,'S-1-5-18')){$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($who),[Security.AccessControl.FileSystemRights]::FullControl,[Security.AccessControl.AccessControlType]::Allow))}
 $f=New-Object IO.FileStream($Path,[IO.FileMode]::CreateNew,[Security.AccessControl.FileSystemRights]::FullControl,[IO.FileShare]::Read,4096,[IO.FileOptions]::None,$acl)
 try{$f.Write($Bytes,0,$Bytes.Length);$f.Flush($true)}finally{$f.Dispose()}
}
Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Text;
using System.Runtime.InteropServices;
using System.Security.Principal;
public sealed class H024ReadOnlyWitness : IDisposable {
 [DllImport("kernel32.dll",SetLastError=true)] static extern IntPtr OpenProcess(uint access,bool inherit,uint pid);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetProcessTimes(IntPtr h,out long c,out long e,out long k,out long u);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool QueryFullProcessImageNameW(IntPtr h,uint flags,StringBuilder b,ref uint n);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool IsProcessInJob(IntPtr h,IntPtr job,out bool inside);
 [DllImport("kernel32.dll",SetLastError=true)] static extern uint WaitForSingleObject(IntPtr h,uint ms);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetExitCodeProcess(IntPtr h,out uint code);
 [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
 [DllImport("advapi32.dll",SetLastError=true)] static extern bool OpenProcessToken(IntPtr h,uint access,out IntPtr token);
 [DllImport("advapi32.dll",SetLastError=true)] static extern bool GetTokenInformation(IntPtr h,int kind,out uint value,uint size,out uint needed);
 IntPtr handle;
 public readonly string Image; public readonly long Creation; public readonly uint Session; public readonly string Sid;
 public H024ReadOnlyWitness(uint pid,long expectedCreation,string expectedImage,string expectedSid,uint expectedSession) {
  handle=OpenProcess(0x101000,false,pid);if(handle==IntPtr.Zero)throw new Win32Exception();
  try {long c,e,k,u;if(!GetProcessTimes(handle,out c,out e,out k,out u))throw new Win32Exception();Creation=c;
   var b=new StringBuilder(32768);uint n=32768;if(!QueryFullProcessImageNameW(handle,0,b,ref n))throw new Win32Exception();Image=b.ToString();
   IntPtr token;if(!OpenProcessToken(handle,8,out token))throw new Win32Exception();
   try{using(var w=new WindowsIdentity(token)){Sid=w.User.Value;}uint value,needed;if(!GetTokenInformation(token,12,out value,4,out needed))throw new Win32Exception();Session=value;
    if(!GetTokenInformation(token,20,out value,4,out needed))throw new Win32Exception();if(value!=0)throw new InvalidOperationException("elevated original host refused");
   }finally{CloseHandle(token);}
   var normalized=Image.StartsWith(@"\\?\")?Image.Substring(4):Image;
   if(Creation!=expectedCreation||!String.Equals(normalized,expectedImage,StringComparison.OrdinalIgnoreCase)||Sid!=expectedSid||Session!=expectedSession)throw new InvalidOperationException("observed original host identity mismatch");
   bool inside;if(!IsProcessInJob(handle,IntPtr.Zero,out inside))throw new Win32Exception();if(inside)throw new InvalidOperationException("original host enclosing Job unknown");
  }catch{Dispose();throw;}
 }
 public bool AlreadyExited(){return WaitForSingleObject(handle,0)==0;}
 public uint WaitExit(uint millis){if(WaitForSingleObject(handle,millis)!=0)throw new InvalidOperationException("original observational handle exit unconfirmed");uint code;if(!GetExitCodeProcess(handle,out code))throw new Win32Exception();return code;}
 public void Dispose(){if(handle!=IntPtr.Zero){CloseHandle(handle);handle=IntPtr.Zero;}}
}
'@
$witness=$null
try{
 $expected=@{'original-broker.json'='';'rendezvous.json'=''}
 # These are byte-for-byte locator copies from the now-archived failed scene,
 # not a second live registration, process capability, phase or new Owner.
 $export=Get-Content -LiteralPath 'C:/Users/Public/ocx-h024-desktop-20261114-r1/out01/same-sid-export02.json' -Raw|ConvertFrom-Json
 if(!$export.complete -or $export.sid -cne $sid -or $export.base -cne $base){throw 'complete original sameSID failure archive required before recovery'}
 foreach($name in @('original-broker.json','rendezvous.json')){
  $row=@($export.files|Where-Object {$_.relative -ceq ('D/'+$name) -or $_.relative -ceq ('D'+[char]92+$name)})
  if($row.Count -ne 1){throw 'original archive locator missing/duplicated'}
  $expected[$name]=$row[0].sha256
 }
 $original=Snapshot (Join-Path $base 'D/original-broker.json');$locator=Snapshot (Join-Path $base 'D/rendezvous.json')
 if((Digest $original) -cne $expected['original-broker.json'] -or (Digest $locator) -cne $expected['rendezvous.json']){throw 'original locator bytes drift; no recovery request'}
 $native=[Text.Encoding]::UTF8.GetString($original)|ConvertFrom-Json
 if($native.pid -ne 748 -or $native.creationFiletime -cne '134391474259863429' -or $native.sid -cne $sid -or $native.session -ne 7){throw 'exact original failed empty-host identity required'}
 $exe=Join-Path $base 'I/opencodex-desktop.exe'
 if((Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant() -cne '24f201211140560dfbe5ed558d2462805cadee80fecc517d3df7d1a1705347a7'){throw 'original pinned fixture manager changed'}
 $witness=[H024ReadOnlyWitness]::new(748,134391474259863429,(Join-Path $base 'I/ocxd.exe'),$sid,7)
 $result.originalWitness=@{pid=748;filetime=$witness.Creation.ToString();image=$witness.Image;sid=$witness.Sid;session=$witness.Session;noJob=$true;rights='QUERY_LIMITED_INFORMATION|SYNCHRONIZE; no termination rights';nativeOwnerAdopted=$false}
 $recovery=Join-Path $env:USERPROFILE ('ocx024-h-'+[guid]::NewGuid().ToString())
 PrivateNewDirectory $recovery;PrivateNewDirectory (Join-Path $recovery 'D')
 PrivateNewFile (Join-Path $recovery 'D/original-broker.json') $original
 PrivateNewFile (Join-Path $recovery 'D/rendezvous.json') $locator
 $result.lookupSnapshotBase=$recovery;$result.lookupSnapshotOnly=$true
 # Execute the exact old, still-pinned manager image so the old original
 # server verifies its own allowed image fileID/SID/session and fresh challenge.
 # Copying only its nonsecret locator avoids the documented old read/write
 # sharing defect; the copy grants no authority. Original record/ACL untouched.
 $env:OCXD024_H_BASE=$recovery;$env:OCXD024_H_MODE='finish'
 $psi=New-Object Diagnostics.ProcessStartInfo;$psi.FileName=$exe
 $psi.Arguments='--exact infrastructure::windows_owner_host024::desktop_tests::isolated_h024_manager --ignored --test-threads=1 --nocapture'
 $psi.UseShellExecute=$false;$psi.CreateNoWindow=$true;$psi.RedirectStandardOutput=$true;$psi.RedirectStandardError=$true
 $process=New-Object Diagnostics.Process;$process.StartInfo=$psi
 $out=[IO.File]::Open((Join-Path $Evidence 'original-recovery-manager.stdout.log'),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 $err=[IO.File]::Open((Join-Path $Evidence 'original-recovery-manager.stderr.log'),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try{
  if(!$process.Start()){throw 'original fixture manager did not start'}
  $a=$process.StandardOutput.BaseStream.CopyToAsync($out);$b=$process.StandardError.BaseStream.CopyToAsync($err)
  $process.WaitForExit();$a.GetAwaiter().GetResult();$b.GetAwaiter().GetResult();$result.managerRawExit=$process.ExitCode
 }finally{$out.Dispose();$err.Dispose();$process.Dispose()}
 if($result.managerRawExit -ne 0){throw 'old manager explicit empty-host exit refused; no force fallback'}
 $result.originalHostExitCode=$witness.WaitExit(5000);$result.nativeWaitConfirmed=$true
 if($result.originalHostExitCode -ne 0){throw 'old empty-host exit was not zero'}
 $rows=@()
 foreach($f in Get-ChildItem -LiteralPath (Join-Path $base 'D') -File -Force){$bytes=Snapshot $f.FullName;$rows+=@{relative=('original-D/'+$f.Name);bytes=$bytes.Length;sha256=(Digest $bytes);base64=[Convert]::ToBase64String($bytes)}}
 foreach($f in Get-ChildItem -LiteralPath (Join-Path $recovery 'D') -File -Force){$bytes=Snapshot $f.FullName;$rows+=@{relative=('lookup-only-D/'+$f.Name);bytes=$bytes.Length;sha256=(Digest $bytes);base64=[Convert]::ToBase64String($bytes)}}
 $result.files=$rows;$result.originalFailureSceneRetained=$true;$result.lookupSnapshotRetained=$true
 $result.emptyDiagnosticRecoveryPassed=$true
}catch{$result.error=@{message=$_.Exception.ToString();line=$_.InvocationInfo.ScriptLineNumber};$result.emptyDiagnosticRecoveryPassed=$false}
finally{
 if($witness){$witness.Dispose()}
 $bytes=[Text.Encoding]::UTF8.GetBytes(($result|ConvertTo-Json -Depth 10))
 $file=[IO.File]::Open($receipt,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try{$file.Write($bytes,0,$bytes.Length);$file.Flush()}finally{$file.Dispose()}
}
if(!$result.emptyDiagnosticRecoveryPassed){throw 'original empty-host recovery incomplete; inspect original receipt, do not retry same conditions'}

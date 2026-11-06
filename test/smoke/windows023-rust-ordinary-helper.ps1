# Test-only support. Library mode performs no provisioning or process launch.
param([ValidateSet('Library','Child')][string]$Mode='Library',[string]$ConfigPath)
function Assert-RustOrdinaryEphemeral($ParentContext=$null) {
 if($null -ne $ParentContext){
  # RunAs may drop custom environment even with -Environment. This is an
  # explicit parent observation, never a claim those fields were inherited.
  if(!$IsWindows -or $ParentContext.kind -cne 'ordinary-runner-parent-observation-v1' -or $ParentContext.githubActions -cne 'true' -or $ParentContext.runnerEnvironment -cne 'github-hosted' -or [string]::IsNullOrWhiteSpace($ParentContext.runnerTemp) -or [string]::IsNullOrWhiteSpace($ParentContext.imageOS)){throw 'Parent-observed ephemeral runner context is incomplete'}
  return
 }
 if(!$IsWindows -or $env:GITHUB_ACTIONS -cne 'true' -or $env:RUNNER_ENVIRONMENT -cne 'github-hosted' -or [string]::IsNullOrWhiteSpace($env:RUNNER_TEMP) -or [string]::IsNullOrWhiteSpace($env:ImageOS)){throw 'Only a fresh GitHub-hosted ephemeral Windows runner is admitted'}
}
function Get-RustOrdinaryBuildArguments([string[]]$CargoArguments,[string]$ManifestPath) {
 $single=@('--workspace','--no-fail-fast','--locked','--offline','--release','--all-features','--no-default-features')
 $pairs=@('--features','--target','--target-dir','--jobs','--profile')
 if('--workspace' -notin $CargoArguments -or '--no-fail-fast' -notin $CargoArguments){throw 'Complete workspace and no-fail-fast required'}
 for($i=0;$i -lt $CargoArguments.Count;$i++){
  $arg=$CargoArguments[$i]
  if($arg -in $single){continue}
  if($arg -in $pairs -and $i+1 -lt $CargoArguments.Count -and !$CargoArguments[$i+1].StartsWith('-')){$i++;continue}
  throw 'Unsupported Cargo build argument: filters, excludes, doctest-only and runner overrides are not admitted'
 }
 @('test','--manifest-path',$ManifestPath)+$CargoArguments+@('--no-run','--message-format=json')
}
function Assert-RustOrdinaryTestArguments([string[]]$TestArguments) {
 $workers=0
 foreach($arg in $TestArguments){
  if($arg -match '^--test-threads=([1-9][0-9]*)$'){$workers++;continue}
  if($arg -notin @('--nocapture','--show-output','--quiet','--include-ignored')){throw 'Test filters/skips and deadline changes are not admitted'}
 }
 if($workers -gt 1){throw 'At most one explicit original libtest worker count is permitted'}
}
function Get-RustOrdinaryArtifacts([string[]]$Lines,$Metadata) {
 $packages=@{};foreach($p in $Metadata.packages){$packages[$p.id]=$p}
 $seen=[Collections.Generic.Dictionary[string,string]]::new([StringComparer]::OrdinalIgnoreCase);$results=[Collections.Generic.List[object]]::new()
 foreach($line in $Lines){
  if([string]::IsNullOrWhiteSpace($line)){continue}
  $m=$line|ConvertFrom-Json -ErrorAction Stop
  if($m.reason -cne 'compiler-artifact' -or $m.profile.test -ne $true -or !$m.executable){continue}
  if(!$packages.ContainsKey($m.package_id)){throw 'Test artifact not in exact workspace metadata'}
  $identity=[string]$m.package_id+'|'+[string]$m.target.name+'|'+(@($m.target.kind)-join ',')
  if($seen.ContainsKey([string]$m.executable)){if($seen[[string]$m.executable] -cne $identity){throw 'Different test artifacts collided on one executable'};continue}
  $seen[[string]$m.executable]=$identity
  if($m.executable -notmatch '(?i)\.exe$' -or ([string]$m.executable).Replace([char]92,[char]47) -notmatch '^[a-zA-Z]:/'){throw 'Exact Windows test executable path required'}
  $package=$packages[$m.package_id]
  $results.Add([ordered]@{executable=[string]$m.executable;packageId=$m.package_id;target=$m.target.name;kinds=@($m.target.kind);workingDirectory=(Split-Path -Parent $package.manifest_path);manifestPath=$package.manifest_path;packageName=$package.name;packageVersion=$package.version})
 }
 if(!$results.Count){throw 'Cargo emitted no exact test executables; never use newest/glob fallback'}
 $results.ToArray()
}
function Get-RustOrdinaryHarnessAdmission($Metadata,[scriptblock]$ReadManifest={param($path) Get-Content -LiteralPath $path -Raw}) {
 $expected=[Collections.Generic.List[object]]::new();$members=@($Metadata.workspace_members)
 if(!$members.Count){throw 'Exact workspace membership is required'}
 foreach($package in $Metadata.packages){
  if($package.id -notin $members){continue}
  $text=& $ReadManifest $package.manifest_path
  # Cargo metadata lacks harness: refuse ambiguous manifests instead of omitting custom tests.
  if($text -match '(?i)harness' -or $text.Contains(([string][char]92+'u')) -or $text.Contains(([string][char]92+'U'))){throw 'Custom/ambiguous Cargo harness unsupported; no silent omission'}
  foreach($target in $package.targets){
   if($target.test -isnot [bool]){throw 'Cargo target.test metadata missing'}
   if(!$target.test){continue}
   if($target.'required-features'){throw 'Conditional test target coverage unsupported; no silent omission'}
   $expected.Add(@{packageId=$package.id;target=$target.name;kinds=@($target.kind)})
  }
 }
 if(!$expected.Count){throw 'No default libtest targets in workspace metadata'}
 $expected.ToArray()
}
function Assert-RustOrdinaryArtifactCoverage($Expected,$Artifacts) {
 $keys={param($x) [string]$x.packageId+'|'+[string]$x.target+'|'+(@($x.kinds|Sort-Object)-join ',')}
 $a=@($Expected|ForEach-Object {& $keys $_}|Sort-Object -Unique)
 $b=@($Artifacts|ForEach-Object {& $keys $_}|Sort-Object -Unique)
 if(($a -join [char]10) -cne ($b -join [char]10)){throw 'Cargo no-run did not emit every expected default libtest executable'}
}
function Get-RustOrdinarySuiteExit([object[]]$ExitCodes,[int]$ExpectedBinaries) {
 if($ExpectedBinaries -le 0 -or $ExitCodes.Count -ne $ExpectedBinaries){throw 'Incomplete binary suite cannot pass'}
 foreach($code in $ExitCodes){if($code -isnot [int]){throw 'Native test process exit code is missing; no null-to-zero conversion permitted'}}
 foreach($code in $ExitCodes){if($code -ne 0){return $code}}
 0
}
function Get-RustOrdinaryBootstrap([string]$OrdinaryScript) {
 $parts=Get-TwoUserReuseParts $OrdinaryScript
 $sanitizer=@'
# No inherited parent GitHub credentials, package tokens or user configuration.
$keep=@('SystemRoot','WINDIR','COMSPEC','COMPUTERNAME','NUMBER_OF_PROCESSORS','PROCESSOR_ARCHITECTURE','PROCESSOR_IDENTIFIER','GITHUB_ACTIONS','RUNNER_ENVIRONMENT','RUNNER_TEMP','ImageOS','USERNAME','USERDOMAIN','USERPROFILE','HOMEDRIVE','HOMEPATH','APPDATA','LOCALAPPDATA','TEMP','TMP','OCX_CI_LOCAL_PROFILE_INITIALIZED')
foreach($name in @([Environment]::GetEnvironmentVariables('Process').Keys)){if($name -notin $keep){[Environment]::SetEnvironmentVariable($name,$null,'Process')}}
$env:PATH=$PSHOME+';'+(Join-Path $env:SystemRoot 'System32')
'@
 # The original bootstrap initializes only this newly registered SID's folders.
 $text=$parts.Bootstrap.Replace("param([string]"+'$InputFile)',"param([string]"+'$InputFile)'+[Environment]::NewLine+$sanitizer)
 $marker='$env:LOCALAPPDATA=Join-Path $registered '+"'AppData/Local'"
 $addition=$marker+[Environment]::NewLine+@'
 $env:TEMP=Join-Path $registered 'AppData/Local/Temp';$env:TMP=$env:TEMP
 New-Item -ItemType Directory -Force $env:TEMP|Out-Null
'@
 if(!$text.Contains($marker)){throw 'Verified Profile TEMP bootstrap boundary changed'}
 $text=$text.Replace($marker,$addition)
 [pscustomobject]@{ProfileType=$parts.ProfileType;DesktopType=$parts.DesktopType;Bootstrap=$text}
}
function Add-RustOrdinaryReadAccess([string]$Path,[string]$Sid,$Changes) {
 Assert-TwoUserNoReparseAncestors $Path
 $item=Get-Item -LiteralPath $Path -Force -ErrorAction Stop
 $acl=Get-Acl -LiteralPath $item.FullName
 $Changes.Add([pscustomobject]@{path=$item.FullName;accessSddl=$acl.GetSecurityDescriptorSddlForm([Security.AccessControl.AccessControlSections]::Access)})
 # Exact object only; no /T, inheritance, owner change or existing fixture repair.
 $rule=[Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($Sid),[Security.AccessControl.FileSystemRights]::ReadAndExecute,[Security.AccessControl.InheritanceFlags]::None,[Security.AccessControl.PropagationFlags]::None,[Security.AccessControl.AccessControlType]::Allow)
 $acl.AddAccessRule($rule);Set-Acl -LiteralPath $item.FullName -AclObject $acl
}
# Suspend ONLY our newly created test process until its real token/identity is
# recorded. This removes the fast zero-test-binary observation race, not tests.
function Restore-RustOrdinaryReadAccess($Change) {
 $diagnostic=[ordered]@{path=$Change.path;stage='read-before';originalAccessSddl=$Change.accessSddl;readbackAccessSddl=$null;readbackDiffers=$null;restorationKind='unverified';exact=$false}
 $Change|Add-Member -NotePropertyName aclRestoreDiagnostic -NotePropertyValue $diagnostic -Force
 try{
  $acl=Get-Acl -LiteralPath $Change.path
  $diagnostic.stage='set-original-access'
  $acl.SetSecurityDescriptorSddlForm($Change.accessSddl,[Security.AccessControl.AccessControlSections]::Access)
  $diagnostic.stage='write-access';Set-Acl -LiteralPath $Change.path -AclObject $acl
  $diagnostic.stage='readback';$actual=Get-Acl -LiteralPath $Change.path
  $diagnostic.readbackAccessSddl=$actual.GetSecurityDescriptorSddlForm([Security.AccessControl.AccessControlSections]::Access)
  $diagnostic.stage='compare'
  $diagnostic.readbackDiffers=$diagnostic.readbackAccessSddl -cne $Change.accessSddl
  if($diagnostic.readbackDiffers){
   # preflight4: Windows Set-Acl adds SE_DACL_AUTO_INHERITED (AI, 0x0400),
   # recording automatic inheritance processing, not a protection/access rule.
   # Admit ONLY original D:(... -> D:AI(... with the complete ordered ACE text
   # unchanged. Never remove P/AR, sort ACEs, expand masks or normalize SID text.
   $original=[string]$Change.accessSddl
   if(!$original.StartsWith('D:(',[StringComparison]::Ordinal) -or
      ![StringComparer]::Ordinal.Equals($diagnostic.readbackAccessSddl,('D:AI'+$original.Substring(2)))){throw 'Exact original read-ACL restoration readback differs'}
   $diagnostic.allowedControlDelta='DiscretionaryAclAutoInherited(0x0400)'
   $diagnostic.orderedAceTextUnchanged=$true
   $diagnostic.protectionUnchanged=$true
   $diagnostic.nativeDescriptorVerified=$false
   if($IsWindows){
    $before=[Security.AccessControl.RawSecurityDescriptor]::new($original)
    $after=[Security.AccessControl.RawSecurityDescriptor]::new($diagnostic.readbackAccessSddl)
    $controlDelta=[int]$before.ControlFlags -bxor [int]$after.ControlFlags
    $protected=[int][Security.AccessControl.ControlFlags]::DiscretionaryAclProtected
    if($controlDelta -ne 0x0400 -or ([int]$before.ControlFlags -band 0x0400) -ne 0 -or
       ([int]$after.ControlFlags -band 0x0400) -eq 0 -or
       ([int]$before.ControlFlags -band $protected) -ne ([int]$after.ControlFlags -band $protected) -or
       $null -eq $before.DiscretionaryAcl -or $null -eq $after.DiscretionaryAcl){throw 'Exact original read-ACL restoration readback differs'}
    $oldAcl=[byte[]]::new($before.DiscretionaryAcl.BinaryLength)
    $newAcl=[byte[]]::new($after.DiscretionaryAcl.BinaryLength)
    $before.DiscretionaryAcl.GetBinaryForm($oldAcl,0);$after.DiscretionaryAcl.GetBinaryForm($newAcl,0)
    if([Convert]::ToHexString($oldAcl) -cne [Convert]::ToHexString($newAcl)){throw 'Exact original read-ACL restoration readback differs'}
    $diagnostic.nativeDescriptorVerified=$true
    $diagnostic.originalControlFlags=[int]$before.ControlFlags
    $diagnostic.readbackControlFlags=[int]$after.ControlFlags
   }
   $diagnostic.restorationKind='normalized-auto-inherited-only'
  }else{
   $diagnostic.restorationKind='exact';$diagnostic.exact=$true
  }
  $diagnostic.stage='verified'
 }catch{
  # DACL-only SDDL and stage contain no password/token. Main may archive this
  # object from Changes or Exception.Data without serializing raw exceptions.
  $_.Exception.Data['aclRestoreDiagnostic']=$diagnostic
  throw
 }
}
function Initialize-RustOrdinaryLaunch {
 if(!('OcxRustOrdinaryLaunch' -as [type])){Add-Type -TypeDefinition @'
using System;
using System.Text;
using System.Diagnostics;
using System.IO;
using System.ComponentModel;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;
public sealed class OcxRustOrdinaryIdentity {
 public int ProcessId; public string Executable, Sid;
 public DateTime CreationTimeUtc; public bool Elevated; public int ElevationType;
}
public sealed class OcxRustOrdinaryPending : IDisposable {
 public Process Process;
 readonly SafeProcessHandle native; readonly SafeWaitHandle thread; bool resumed;
 public OcxRustOrdinaryPending(IntPtr process,IntPtr primaryThread){native=new SafeProcessHandle(process,true);thread=new SafeWaitHandle(primaryThread,true);}
 public void Attach(int id){Process=System.Diagnostics.Process.GetProcessById(id);}
 void Live(){uint state=WaitForSingleObject(native,0);if(state!=258)throw new InvalidOperationException("Created native process is not live");}
 public OcxRustOrdinaryIdentity Observe(){
  Live();uint pid=GetProcessId(native);if(pid==0)throw new Win32Exception(Marshal.GetLastWin32Error());
  var name=new StringBuilder(32768);uint size=(uint)name.Capacity;
  if(!QueryFullProcessImageNameW(native,0,name,ref size))throw new Win32Exception(Marshal.GetLastWin32Error());
  long created,exited,kernel,user;if(!GetProcessTimes(native,out created,out exited,out kernel,out user))throw new Win32Exception(Marshal.GetLastWin32Error());
  IntPtr rawToken; if(!OpenProcessToken(native,8,out rawToken))throw new Win32Exception(Marshal.GetLastWin32Error());
  using(var token=new SafeAccessTokenHandle(rawToken)){
   uint needed;GetTokenInformation(token,1,IntPtr.Zero,0,out needed);
   if(needed<8||needed>65536)throw new InvalidOperationException("TokenUser buffer size invalid");
   IntPtr buffer=Marshal.AllocHGlobal((int)needed),sidText=IntPtr.Zero;
   try{
    if(!GetTokenInformation(token,1,buffer,needed,out needed))throw new Win32Exception(Marshal.GetLastWin32Error());
    if(!ConvertSidToStringSidW(Marshal.ReadIntPtr(buffer),out sidText))throw new Win32Exception(Marshal.GetLastWin32Error());
    string sid=Marshal.PtrToStringUni(sidText);if(sid==null||sid.Length>512)throw new InvalidOperationException("TokenUser SID invalid");
    int elevated=ReadInt(token,20),type=ReadInt(token,18);
    if((elevated!=0&&elevated!=1)||type<1||type>3)throw new InvalidOperationException("Token elevation invalid");
    Live();return new OcxRustOrdinaryIdentity{ProcessId=(int)pid,Executable=name.ToString(),Sid=sid,CreationTimeUtc=DateTime.FromFileTimeUtc(created),Elevated=elevated!=0,ElevationType=type};
   }finally{if(sidText!=IntPtr.Zero)LocalFree(sidText);Marshal.FreeHGlobal(buffer);}
  }
 }
 static int ReadInt(SafeAccessTokenHandle token,uint kind){
  IntPtr value=Marshal.AllocHGlobal(4);try{uint needed;
   if(!GetTokenInformation(token,kind,value,4,out needed))throw new Win32Exception(Marshal.GetLastWin32Error());
   if(needed!=4)throw new InvalidOperationException("Token integer size invalid");return Marshal.ReadInt32(value);
  }finally{Marshal.FreeHGlobal(value);}
 }
 public void Resume(){Live();uint previous=ResumeThread(thread);if(previous==0xffffffff)throw new Win32Exception(Marshal.GetLastWin32Error());if(previous!=1)throw new InvalidOperationException("Created thread suspend count changed");resumed=true;}
 public int ExitCode(){
  if(WaitForSingleObject(native,0)!=0)throw new InvalidOperationException("Created native process has not exited");
  uint code;if(!GetExitCodeProcess(native,out code))throw new Win32Exception(Marshal.GetLastWin32Error());return unchecked((int)code);
 }
 public void Dispose(){
  // No reopened PID/name authority: this is precisely CreateProcess's handle.
  // If observation/ledger persistence failed, do not orphan our suspended child.
  try{if(!resumed&&!native.IsClosed&&!native.IsInvalid){
   uint state=WaitForSingleObject(native,0);
   if(state==258){if(!TerminateProcess(native,1))throw new Win32Exception(Marshal.GetLastWin32Error());if(WaitForSingleObject(native,10000)!=0)throw new InvalidOperationException("Created suspended process cleanup incomplete");}
   else if(state!=0)throw new Win32Exception(Marshal.GetLastWin32Error());
  }}finally{thread.Dispose();native.Dispose();}
 }
 [DllImport("kernel32.dll",SetLastError=true)] static extern uint ResumeThread(SafeWaitHandle handle);
 [DllImport("kernel32.dll",SetLastError=true)] static extern uint WaitForSingleObject(SafeProcessHandle handle,uint milliseconds);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool TerminateProcess(SafeProcessHandle handle,uint exitCode);
 [DllImport("kernel32.dll",SetLastError=true)] static extern uint GetProcessId(SafeProcessHandle handle);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetExitCodeProcess(SafeProcessHandle handle,out uint code);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,ExactSpelling=true,SetLastError=true)] static extern bool QueryFullProcessImageNameW(SafeProcessHandle handle,uint flags,StringBuilder name,ref uint size);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetProcessTimes(SafeProcessHandle handle,out long created,out long exited,out long kernel,out long user);
 [DllImport("advapi32.dll",SetLastError=true)] static extern bool OpenProcessToken(SafeProcessHandle handle,uint access,out IntPtr token);
 [DllImport("advapi32.dll",SetLastError=true)] static extern bool GetTokenInformation(SafeAccessTokenHandle token,uint kind,IntPtr buffer,uint size,out uint needed);
 [DllImport("advapi32.dll",CharSet=CharSet.Unicode,ExactSpelling=true,SetLastError=true)] static extern bool ConvertSidToStringSidW(IntPtr sid,out IntPtr text);
 [DllImport("kernel32.dll")] static extern IntPtr LocalFree(IntPtr handle);
}
public static class OcxRustOrdinaryLaunch {
 [StructLayout(LayoutKind.Sequential,CharSet=CharSet.Unicode)] struct Startup {
  public int cb; public string reserved,desktop,title;
  public uint x,y,xsize,ysize,xchars,ychars,fill,flags;
  public short show,reservedSize; public IntPtr reservedPtr,input,output,error;
 }
 [StructLayout(LayoutKind.Sequential)] struct Info {public IntPtr process,thread; public uint pid,tid;}
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,ExactSpelling=true,SetLastError=true)]
 static extern bool CreateProcessW(string app,StringBuilder command,IntPtr processAttributes,IntPtr threadAttributes,bool inherit,uint flags,IntPtr environment,string cwd,ref Startup startup,out Info info);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool DuplicateHandle(IntPtr sourceProcess,IntPtr source,IntPtr targetProcess,out IntPtr target,uint access,bool inherit,uint options);
 [DllImport("kernel32.dll")] static extern IntPtr GetCurrentProcess();
 [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr handle);
 public static string Quote(string value){
  if(value.IndexOf('\0')>=0)throw new ArgumentException("NUL argv");
  var b=new StringBuilder("\"");int slashes=0;
  foreach(char c in value){if(c=='\\'){slashes++;continue;}
   if(c=='"'){b.Append('\\',slashes*2+1);b.Append(c);}else{b.Append('\\',slashes);b.Append(c);}slashes=0;}
  b.Append('\\',slashes*2);return b.Append('"').ToString();
 }
 public static string[] CimContradictions(OcxRustOrdinaryIdentity native,int? pid,string sid,string image,long? creationTicks){
  var differences=new System.Collections.Generic.List<string>();
  if(pid.HasValue&&pid.Value!=native.ProcessId)differences.Add("pid");
  if(!String.IsNullOrEmpty(sid)&&!String.Equals(sid,native.Sid,StringComparison.Ordinal))differences.Add("sid");
  if(!String.IsNullOrEmpty(image)&&!String.Equals(image,native.Executable,StringComparison.OrdinalIgnoreCase))differences.Add("image");
  if(creationTicks.HasValue){long delta=native.CreationTimeUtc.Ticks-creationTicks.Value;if(delta<0||delta>9)differences.Add("creation");}
  return differences.ToArray();
 }
 public static OcxRustOrdinaryPending Start(string exe,string[] args,string cwd,FileStream stdout,FileStream stderr){
  IntPtr output=IntPtr.Zero,error=IntPtr.Zero;Info info=new Info();OcxRustOrdinaryPending pending=null;
  try {
   var current=GetCurrentProcess();
   if(!DuplicateHandle(current,stdout.SafeFileHandle.DangerousGetHandle(),current,out output,0,true,2)||
      !DuplicateHandle(current,stderr.SafeFileHandle.DangerousGetHandle(),current,out error,0,true,2))throw new Win32Exception(Marshal.GetLastWin32Error());
   var command=new StringBuilder(Quote(exe));foreach(string arg in args)command.Append(' ').Append(Quote(arg));
   var startup=new Startup();startup.cb=Marshal.SizeOf<Startup>();startup.flags=0x100;startup.output=output;startup.error=error;
   // CREATE_SUSPENDED | CREATE_NO_WINDOW, environment inherited from scrubbed child.
   if(!CreateProcessW(exe,command,IntPtr.Zero,IntPtr.Zero,true,0x08000004,IntPtr.Zero,cwd,ref startup,out info))throw new Win32Exception(Marshal.GetLastWin32Error());
   pending=new OcxRustOrdinaryPending(info.process,info.thread);info.process=IntPtr.Zero;info.thread=IntPtr.Zero;
   pending.Attach((int)info.pid);return pending;
  } catch {if(pending!=null){try{pending.Dispose();}finally{if(pending.Process!=null)pending.Process.Dispose();}}throw;
  } finally {if(output!=IntPtr.Zero)CloseHandle(output);if(error!=IntPtr.Zero)CloseHandle(error);if(info.process!=IntPtr.Zero)CloseHandle(info.process);if(info.thread!=IntPtr.Zero)CloseHandle(info.thread);}
 }
}
'@}
}
function Start-RustOrdinaryCaptured([string]$Executable,[string[]]$Arguments,[string]$WorkingDirectory,[string]$Stdout,[string]$Stderr,[string]$Sid,$Ledger,[int]$DeadlineSeconds,[string]$LedgerPath,$ParentContext=$null) {
 Assert-RustOrdinaryEphemeral $ParentContext
 if([Security.Principal.WindowsIdentity]::GetCurrent().User.Value -cne $Sid){throw 'Launch token differs from admitted ordinary SID'}
 Initialize-RustOrdinaryLaunch
 $outFile=[IO.File]::Open($Stdout,'CreateNew','Write','ReadWrite');$errFile=[IO.File]::Open($Stderr,'CreateNew','Write','ReadWrite');$pending=$null
 $diagnostic=[ordered]@{stage='create-suspended';identityAuthority='retained-CreateProcess-handle';kernel=$null;cim=$null;resumed=$false;failureType=$null;failureHResult=$null}
 try{
  $pending=[OcxRustOrdinaryLaunch]::Start($Executable,$Arguments,$WorkingDirectory,$outFile,$errFile);$p=$pending.Process
  $diagnostic.stage='observe-created-handle'
  $native=$pending.Observe()
  $created=$native.CreationTimeUtc.Ticks
  $token=[ordered]@{schemaVersion=1;status='observed';processId=$native.ProcessId;executable=$native.Executable;creationTimeUtc=$native.CreationTimeUtc.ToString('o');elevated=$native.Elevated;elevationType=$native.ElevationType;tokenUser=$native.Sid;authority='retained-CreateProcess-handle'}
  $diagnostic.kernel=$token
  Assert-WindowsProcessMode -Observation $token -Mode ordinary
  if($native.ProcessId -ne $p.Id -or $native.Sid -cne $Sid -or ![StringComparer]::OrdinalIgnoreCase.Equals($native.Executable,[IO.Path]::GetFullPath($Executable))){throw 'Suspended native process TokenUser/image/creation differs'}
  # CIM is supplementary before loader startup. Missing values are explicitly
  # absent, never recorded as agreement; any supplied contradiction still fails.
  $diagnostic.stage='supplementary-cim'
  $cim=[ordered]@{rowPresent=$false;pidProvided=$false;pid=$null;sidProvided=$false;sid=$null;imageProvided=$false;image=$null;creationProvided=$false;creationTicks=$null;ownerReturnValue=$null;queryFailureType=$null;queryHResult=$null;contradictions=@()}
  $cimPid=$null;$cimSid=$null;$cimImage=$null;$cimCreated=$null;$session=$null
  try{
   $row=Get-CimInstance Win32_Process -Filter ('ProcessId='+$native.ProcessId) -ErrorAction Stop
   if($row){
    $cim.rowPresent=$true
    if($null -ne $row.ProcessId){$cimPid=[int]$row.ProcessId;$cim.pidProvided=$true;$cim.pid=$cimPid}
    if(![string]::IsNullOrEmpty($row.ExecutablePath)){$cimImage=[string]$row.ExecutablePath;if($cimImage.Length -gt 32768){throw 'Bounded CIM image required'};$cim.imageProvided=$true;$cim.image=$cimImage}
    if($row.CreationDate){$cimCreated=$row.CreationDate.ToUniversalTime().Ticks;$cim.creationProvided=$true;$cim.creationTicks=[string]$cimCreated}
    $session=$row.SessionId
    try{
     $owner=Invoke-CimMethod -InputObject $row -MethodName GetOwnerSid -ErrorAction Stop
     $cim.ownerReturnValue=$owner.ReturnValue
     if(![string]::IsNullOrEmpty($owner.Sid)){$cimSid=[string]$owner.Sid;if($cimSid.Length -gt 512){throw 'Bounded CIM SID required'};$cim.sidProvided=$true;$cim.sid=$cimSid}
    }catch{$cim.queryFailureType=$_.Exception.GetType().FullName;$cim.queryHResult=[int]$_.Exception.HResult}
   }
  }catch{$cim.queryFailureType=$_.Exception.GetType().FullName;$cim.queryHResult=[int]$_.Exception.HResult}
  $cim.contradictions=@([OcxRustOrdinaryLaunch]::CimContradictions($native,$cimPid,$cimSid,$cimImage,$cimCreated))
  $diagnostic.cim=$cim
  if($cim.contradictions.Count){throw 'Supplementary CIM contradicts retained created-process identity'}
  $fact=[pscustomobject]@{pid=$native.ProcessId;sid=$native.Sid;path=$native.Executable;creationTicks=[string]$created;sessionId=$session;identityAuthority='retained-CreateProcess-handle'}
  $Ledger[[int]$fact.pid]=$fact
  # Persist exact identity before allowing ANY test code to execute.
  $diagnostic.stage='persist-ledger'
  Write-TwoUserJson $LedgerPath @($Ledger.Values)
  $pending.Resume()
  $diagnostic.resumed=$true;$diagnostic.stage='running'
  Write-TwoUserJson ($Stdout+'.launch.json') $diagnostic
  $until=[datetime]::UtcNow.AddSeconds($DeadlineSeconds)
  while(!$p.HasExited){
   try{Add-TwoUserDescendants $fact $Ledger}catch{if(!$p.HasExited){throw}}
   Write-TwoUserJson $LedgerPath @($Ledger.Values)
   if([datetime]::UtcNow -gt $until){throw 'Original suite envelope exhausted; no test deadline was enlarged'}
   Start-Sleep -Milliseconds 100
  }
  $outFile.Flush($true);$errFile.Flush($true)
  $diagnostic.stage='exited';Write-TwoUserJson ($Stdout+'.launch.json') $diagnostic
  $nativeExit=$pending.ExitCode()
  [ordered]@{process=$fact;token=$token;cim=$cim;exitCode=$nativeExit;exitAuthority='retained-CreateProcess-handle';arguments=$Arguments;stdout=$Stdout;stderr=$Stderr;workingDirectory=$WorkingDirectory;identityObservedBeforeResume=$true}
 }catch{
  $diagnostic.failureType=$_.Exception.GetType().FullName;$diagnostic.failureHResult=[int]$_.Exception.HResult
  Write-TwoUserJson ($Stdout+'.launch.json') $diagnostic
  throw
 }finally{
  try{if($pending){try{$pending.Dispose()}finally{if($pending.Process){$pending.Process.Dispose()}}}}finally{$outFile.Dispose();$errFile.Dispose()}
 }
}
function Invoke-RustOrdinaryChild([string]$InputFile) {
 $c=Get-Content -LiteralPath $InputFile -Raw|ConvertFrom-Json
 . (Join-Path $c.code 'windows023-two-user-native-helper.ps1')
 . (Join-Path $c.code 'windows-process-token.ps1')
 $identity=[Security.Principal.WindowsIdentity]::GetCurrent()
 $sid=$identity.User.Value
 $tokenOwner=$identity.Owner.Value
 if($tokenOwner -cne $sid){throw 'Ordinary current TokenOwner differs from TokenUser; no owner repair is permitted'}
 if($sid -cne $c.accountSid -or $c.ephemeralGuard -ne $true){throw 'Ordinary suite real SID/guard differs'}
 $principal=[Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
 if($principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Ordinary suite account is administrator'}
 if(!(Test-TwoUserSamePath $env:USERPROFILE $c.profile) -or (Split-Path -Leaf $c.profile) -notlike 'ocxacc*'){throw 'Actual registered new Profile required'}
 Assert-RustOrdinaryEphemeral $c.parentRunnerObservation
 if($c.commit -cnotmatch '^[0-9a-f]{40}$' -or !(Test-TwoUserSamePath (Split-Path -Parent $InputFile) $c.evidence)){throw 'Original parent observation is not bound to this child input'}
 $null=Assert-TwoUserUnder $c.parentRunnerObservation.runnerTemp $c.evidence
 $null=Assert-TwoUserUnder $c.profile $env:TEMP
 if(!(Test-TwoUserSamePath $env:TEMP (Join-Path $env:LOCALAPPDATA 'Temp')) -or !(Test-TwoUserSamePath $env:LOCALAPPDATA (Join-Path $c.profile 'AppData/Local'))){throw 'Actual token Profile TEMP/KnownFolder differs'}
 if(!(Test-Path -LiteralPath $env:TEMP -PathType Container)){throw 'Own registered TEMP missing'}
 # HOME is derived only after the original bootstrap's real Profile/known-folder proof.
 $env:HOME=$env:USERPROFILE;$env:OPENCODEX_TEST_NODE=$c.node
 $accountParts=$identity.Name.Split([char]92);$env:USERNAME=$accountParts[-1]
 if($accountParts.Count -gt 1){$env:USERDOMAIN=$accountParts[0]}
 if((Get-FileHash -LiteralPath $c.node -Algorithm SHA256).Hash -ine $c.nodeSha256){throw 'Real fixture Node bytes changed'}
 $env:RUST_BACKTRACE='1';$env:PATH=$PSHOME+';'+(Split-Path -Parent $c.node)+';'+(Join-Path $env:SystemRoot 'System32')
 $ledger=[Collections.Generic.Dictionary[int,object]]::new();$results=[Collections.Generic.List[object]]::new();$exitCode=0
 $report=[ordered]@{schemaVersion=1;result='fail';scope='binaries';fullWorkspacePassed=$false;nativeExecuted=$true;runnerAdmissionSource='explicit-parent-observation-after-parent-CI-guard';accountSid=$sid;tokenUser=$sid;tokenOwner=$tokenOwner;profile=$env:USERPROFILE;home=$env:HOME;temp=$env:TEMP;expectedCommit=$c.commit;testArguments=@($c.testArguments);docTestsExecuted=$false;allBinariesAttempted=$false;cleanup='pending';exitCode=1;tests=@()}
 try{
  $probe=Join-Path $env:TEMP ('rust-ordinary-owner-'+[guid]::NewGuid().ToString('N'))
  New-Item -ItemType Directory -Path $probe|Out-Null
  $owner=(Get-Acl -LiteralPath $probe).GetOwner([Security.Principal.SecurityIdentifier]).Value
  if($owner -cne $sid){throw 'Fresh std-style fixture owner is not the ordinary TokenUser'}
  Remove-Item -LiteralPath $probe
  $report.freshDirectoryOwner=$owner
  $until=[datetime]::UtcNow.AddSeconds($c.suiteTimeoutSeconds)
  $index=0
  foreach($artifact in $c.artifacts){
   $index++;$remaining=[int][Math]::Floor(($until-[datetime]::UtcNow).TotalSeconds)
   if($remaining -le 0){throw 'Original suite envelope exhausted before all test binaries'}
   foreach($input in @(@($artifact)+@($c.runtimeDlls)|Where-Object {$_})){if((Get-FileHash -LiteralPath $input.executable -Algorithm SHA256).Hash -ine $input.sha256){throw 'Test executable/DLL changed after parent no-run compilation'}}
   $env:CARGO_MANIFEST_DIR=$artifact.workingDirectory;$env:CARGO_MANIFEST_PATH=$artifact.manifestPath
   $env:CARGO_PKG_NAME=$artifact.packageName;$env:CARGO_PKG_VERSION=$artifact.packageVersion
   $result=Start-RustOrdinaryCaptured $artifact.executable @($c.testArguments) $artifact.workingDirectory (Join-Path $c.evidence ('test-'+$index+'.stdout.log')) (Join-Path $c.evidence ('test-'+$index+'.stderr.log')) $sid $ledger $remaining (Join-Path $c.evidence 'owned-processes.json') $c.parentRunnerObservation
   if($result.exitCode -isnot [int]){throw 'Native test process exit code is missing; no null-to-zero conversion permitted'}
   $result.sha256=$artifact.sha256;$result.packageId=$artifact.packageId;$result.target=$artifact.target
   $results.Add($result)
   Write-TwoUserJson (Join-Path $c.evidence 'owned-processes.json') @($ledger.Values)
   Write-TwoUserJson (Join-Path $c.evidence ('test-'+$index+'.json')) $result
   if($exitCode -eq 0 -and $result.exitCode -ne 0){$exitCode=[int]$result.exitCode}
   # Deliberately continue every Cargo-emitted test binary after a failed binary.
  }
  $exitCode=Get-RustOrdinarySuiteExit @($results|ForEach-Object {$_.exitCode}) $c.artifacts.Count
  $report.allBinariesAttempted=$results.Count -eq $c.artifacts.Count;$report.exitCode=$exitCode
  $report.result=if($exitCode -eq 0){'pass-ordinary-rust-binaries'}else{'test-failed'}
 }catch{$report.failureCode='ordinary_suite_execution_or_identity_failed';$report.failureType=$_.Exception.GetType().FullName;$report.failureLine=$_.InvocationInfo.ScriptLineNumber;$report.exitCode=1}
 finally{
  $cleanup=@();foreach($fact in @($ledger.Values|Sort-Object {[long]$_.creationTicks} -Descending)){
   try{Stop-TwoUserTracked $fact;$cleanup+=@{process=$fact;result='stopped-or-exited'}}catch{$cleanup+=@{process=$fact;result='retained-unproven'};$report.result='cleanup-failed';$report.exitCode=1}
  }
  $report.tests=$results.ToArray();$report.cleanup=$cleanup
  Write-TwoUserJson (Join-Path $c.evidence 'owned-processes.json') @($ledger.Values)
  Write-TwoUserJson (Join-Path $c.evidence 'ordinary-suite.json') $report
 }
 exit ([int]$report.exitCode)
}
if($Mode -ceq 'Child'){
 try{Invoke-RustOrdinaryChild $ConfigPath}catch{
  # Early identity/admission failures occur before the suite report exists.
  # Retain only controlled diagnostics, never raw exception text or environment.
  $failure=[ordered]@{result='child-admission-failed';failureType=$_.Exception.GetType().FullName;failureLine=$_.InvocationInfo.ScriptLineNumber;failureHResult=[int]$_.Exception.HResult;sourceLeaf=[IO.Path]::GetFileName($_.InvocationInfo.ScriptName)}
  $failure.ephemeralFacts=@{isWindows=[bool]$IsWindows;githubActions=($env:GITHUB_ACTIONS -ceq 'true');hosted=($env:RUNNER_ENVIRONMENT -ceq 'github-hosted');runnerTempPresent=![string]::IsNullOrWhiteSpace($env:RUNNER_TEMP);imagePresent=![string]::IsNullOrWhiteSpace($env:ImageOS)}
  $known=@('Only a fresh GitHub-hosted ephemeral Windows runner is admitted','Ordinary current TokenOwner differs from TokenUser; no owner repair is permitted','Ordinary suite real SID/guard differs','Ordinary suite account is administrator','Actual registered new Profile required','Actual token Profile TEMP/KnownFolder differs','Own registered TEMP missing','Real fixture Node bytes changed','Local absolute non-device path required','Path escaped tracked fixture')
  if($_.Exception.Message -cin $known){$failure.guard=$_.Exception.Message}
  $path=Join-Path (Split-Path -Parent $ConfigPath) 'child-admission-failure.json'
  $failure|ConvertTo-Json|Set-Content -LiteralPath $path -Encoding utf8
  [Console]::Error.WriteLine(($failure|ConvertTo-Json -Compress));exit 1
 }
}

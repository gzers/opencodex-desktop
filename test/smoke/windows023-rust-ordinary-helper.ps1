# Test-only support. Library mode performs no provisioning or process launch.
param([ValidateSet('Library','Child')][string]$Mode='Library',[string]$ConfigPath)
function Assert-RustOrdinaryEphemeral {
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
function Get-RustOrdinarySuiteExit([int[]]$ExitCodes,[int]$ExpectedBinaries) {
 if($ExpectedBinaries -le 0 -or $ExitCodes.Count -ne $ExpectedBinaries){throw 'Incomplete binary suite cannot pass'}
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
 $acl=Get-Acl -LiteralPath $Change.path
 $acl.SetSecurityDescriptorSddlForm($Change.accessSddl,[Security.AccessControl.AccessControlSections]::Access)
 Set-Acl -LiteralPath $Change.path -AclObject $acl
 $readback=(Get-Acl -LiteralPath $Change.path).GetSecurityDescriptorSddlForm([Security.AccessControl.AccessControlSections]::Access)
 if($readback -cne $Change.accessSddl){throw 'Exact original read-ACL restoration readback differs'}
}
function Initialize-RustOrdinaryLaunch {
 if(!('OcxRustOrdinaryLaunch' -as [type])){Add-Type -TypeDefinition @'
using System;
using System.Text;
using System.Diagnostics;
using System.IO;
using System.ComponentModel;
using System.Runtime.InteropServices;
public sealed class OcxRustOrdinaryPending : IDisposable {
 public Process Process;
 IntPtr thread;
 public OcxRustOrdinaryPending(Process process,IntPtr handle){Process=process;thread=handle;}
 public void Resume(){if(ResumeThread(thread)==0xffffffff)throw new Win32Exception(Marshal.GetLastWin32Error());}
 public void Dispose(){if(thread!=IntPtr.Zero){CloseHandle(thread);thread=IntPtr.Zero;}}
 [DllImport("kernel32.dll",SetLastError=true)] static extern uint ResumeThread(IntPtr handle);
 [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr handle);
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
 public static OcxRustOrdinaryPending Start(string exe,string[] args,string cwd,FileStream stdout,FileStream stderr){
  IntPtr output=IntPtr.Zero,error=IntPtr.Zero;Info info=new Info();
  try {
   var current=GetCurrentProcess();
   if(!DuplicateHandle(current,stdout.SafeFileHandle.DangerousGetHandle(),current,out output,0,true,2)||
      !DuplicateHandle(current,stderr.SafeFileHandle.DangerousGetHandle(),current,out error,0,true,2))throw new Win32Exception(Marshal.GetLastWin32Error());
   var command=new StringBuilder(Quote(exe));foreach(string arg in args)command.Append(' ').Append(Quote(arg));
   var startup=new Startup();startup.cb=Marshal.SizeOf<Startup>();startup.flags=0x100;startup.output=output;startup.error=error;
   // CREATE_SUSPENDED | CREATE_NO_WINDOW, environment inherited from scrubbed child.
   if(!CreateProcessW(exe,command,IntPtr.Zero,IntPtr.Zero,true,0x08000004,IntPtr.Zero,cwd,ref startup,out info))throw new Win32Exception(Marshal.GetLastWin32Error());
   var process=Process.GetProcessById((int)info.pid);var result=new OcxRustOrdinaryPending(process,info.thread);info.thread=IntPtr.Zero;
   // Retain the actual kernel process handle before closing CreateProcess's handle.
   var retained=process.Handle;return result;
  } finally {if(output!=IntPtr.Zero)CloseHandle(output);if(error!=IntPtr.Zero)CloseHandle(error);if(info.process!=IntPtr.Zero)CloseHandle(info.process);if(info.thread!=IntPtr.Zero)CloseHandle(info.thread);}
 }
}
'@}
}
function Start-RustOrdinaryCaptured([string]$Executable,[string[]]$Arguments,[string]$WorkingDirectory,[string]$Stdout,[string]$Stderr,[string]$Sid,$Ledger,[int]$DeadlineSeconds,[string]$LedgerPath) {
 Assert-RustOrdinaryEphemeral
 Initialize-RustOrdinaryLaunch
 $outFile=[IO.File]::Open($Stdout,'CreateNew','Write','ReadWrite');$errFile=[IO.File]::Open($Stderr,'CreateNew','Write','ReadWrite');$pending=$null
 try{
  $pending=[OcxRustOrdinaryLaunch]::Start($Executable,$Arguments,$WorkingDirectory,$outFile,$errFile);$p=$pending.Process
  # QueryFullProcessImageName/GetProcessTimes from retained native handle work
  # before the loader runs; Process.MainModule is not reliable while suspended.
  $token=Get-WindowsProcessToken -Process $p -ExpectedExecutable $Executable
  Assert-WindowsProcessMode -Observation $token -Mode ordinary
  $row=Get-CimInstance Win32_Process -Filter ('ProcessId='+$p.Id) -ErrorAction Stop
  $owner=Invoke-CimMethod -InputObject $row -MethodName GetOwnerSid -ErrorAction Stop
  $created=[datetime]::Parse($token.creationTimeUtc).ToUniversalTime().Ticks
  $delta=$created-$row.CreationDate.ToUniversalTime().Ticks
  if($owner.ReturnValue -ne 0 -or $owner.Sid -cne $Sid -or $row.ExecutablePath -ine $token.executable -or $delta -lt 0 -or $delta -gt 9){throw 'Suspended native process TokenUser/image/creation differs'}
  $fact=[pscustomobject]@{pid=$p.Id;sid=$owner.Sid;path=$token.executable;creationTicks=[string]$created;sessionId=$row.SessionId}
  $Ledger[[int]$fact.pid]=$fact
  # Persist exact identity before allowing ANY test code to execute.
  Write-TwoUserJson $LedgerPath @($Ledger.Values)
  $pending.Resume()
  $until=[datetime]::UtcNow.AddSeconds($DeadlineSeconds)
  while(!$p.HasExited){
   try{Add-TwoUserDescendants $fact $Ledger}catch{if(!$p.HasExited){throw}}
   Write-TwoUserJson $LedgerPath @($Ledger.Values)
   if([datetime]::UtcNow -gt $until){throw 'Original suite envelope exhausted; no test deadline was enlarged'}
   Start-Sleep -Milliseconds 100
  }
  $outFile.Flush($true);$errFile.Flush($true)
  [ordered]@{process=$fact;token=$token;exitCode=$p.ExitCode;arguments=$Arguments;stdout=$Stdout;stderr=$Stderr;workingDirectory=$WorkingDirectory;identityObservedBeforeResume=$true}
 }finally{if($pending){$pending.Dispose();$pending.Process.Dispose()};$outFile.Dispose();$errFile.Dispose()}
}
function Invoke-RustOrdinaryChild([string]$InputFile) {
 Assert-RustOrdinaryEphemeral
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
 $report=[ordered]@{schemaVersion=1;result='fail';scope='binaries';fullWorkspacePassed=$false;nativeExecuted=$true;accountSid=$sid;tokenUser=$sid;tokenOwner=$tokenOwner;profile=$env:USERPROFILE;home=$env:HOME;temp=$env:TEMP;expectedCommit=$c.commit;testArguments=@($c.testArguments);docTestsExecuted=$false;allBinariesAttempted=$false;cleanup='pending';exitCode=1;tests=@()}
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
   $result=Start-RustOrdinaryCaptured $artifact.executable @($c.testArguments) $artifact.workingDirectory (Join-Path $c.evidence ('test-'+$index+'.stdout.log')) (Join-Path $c.evidence ('test-'+$index+'.stderr.log')) $sid $ledger $remaining (Join-Path $c.evidence 'owned-processes.json')
   $result.sha256=$artifact.sha256;$result.packageId=$artifact.packageId;$result.target=$artifact.target
   $results.Add($result)
   Write-TwoUserJson (Join-Path $c.evidence 'owned-processes.json') @($ledger.Values)
   Write-TwoUserJson (Join-Path $c.evidence ('test-'+$index+'.json')) $result
   if($exitCode -eq 0 -and $result.exitCode -ne 0){$exitCode=[int]$result.exitCode}
   # Deliberately continue every Cargo-emitted test binary after a failed binary.
  }
  $exitCode=Get-RustOrdinarySuiteExit @($results|ForEach-Object {[int]$_.exitCode}) $c.artifacts.Count
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
if($Mode -ceq 'Child'){Invoke-RustOrdinaryChild $ConfigPath}

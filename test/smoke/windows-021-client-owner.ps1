param(
 [Parameter(Mandatory=$true)][string]$Node,
 [Parameter(Mandatory=$true)][string]$NpmCli,
 [Parameter(Mandatory=$true)][string]$OutputDirectory,
 [Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-f]{40}$')][string]$ExpectedProductTree,
 [string]$TestExecutable=''
)
$ErrorActionPreference='Stop'
if(!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7 -or $env:GITHUB_ACTIONS -cne 'true'){throw 'Disposable GitHub Windows runner and PowerShell 7 required'}
$taskRepo=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$taskNode=(Resolve-Path -LiteralPath $Node).Path
$taskNpm=(Resolve-Path -LiteralPath $NpmCli).Path
$taskProduct=(& git -C $taskRepo rev-parse 'HEAD:apps/desktop/tauri/src').Trim()
if($LASTEXITCODE -ne 0 -or $taskProduct -cne $ExpectedProductTree){throw 'Exact frozen product source tree mismatch'}
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $taskOutput){throw 'Fresh evidence output required'}
New-Item -ItemType Directory -Path $taskOutput | Out-Null
$taskRoot=Join-Path ([IO.Path]::GetTempPath()) ('ocx021-win-owner-'+[guid]::NewGuid().ToString('N'))
foreach($taskPart in @('home','codex-home','official','work','temp','installation','home/AppData/Roaming','home/AppData/Local')){New-Item -ItemType Directory -Force -Path (Join-Path $taskRoot $taskPart)|Out-Null}
$taskEnvironment=@{}
foreach($taskKey in @('SystemRoot','WINDIR','COMSPEC','PATHEXT','PROCESSOR_ARCHITECTURE','NUMBER_OF_PROCESSORS')){if([Environment]::GetEnvironmentVariable($taskKey)){$taskEnvironment[$taskKey]=[Environment]::GetEnvironmentVariable($taskKey)}}
$taskEnvironment.HOME=Join-Path $taskRoot 'home'
$taskEnvironment.USERPROFILE=$taskEnvironment.HOME
$taskEnvironment.CODEX_HOME=Join-Path $taskRoot 'codex-home'
$taskEnvironment.OPENCODEX_HOME=Join-Path $taskRoot 'official'
$taskEnvironment.APPDATA=Join-Path $taskEnvironment.HOME 'AppData/Roaming'
$taskEnvironment.LOCALAPPDATA=Join-Path $taskEnvironment.HOME 'AppData/Local'
$taskEnvironment.TEMP=Join-Path $taskRoot 'temp';$taskEnvironment.TMP=$taskEnvironment.TEMP
$taskEnvironment.PATH=(Join-Path $taskRoot 'installation/node_modules/.bin')+';'+(Split-Path -Parent $taskNode)+';'+(Join-Path $env:SystemRoot 'System32')+';'+$env:SystemRoot
$taskEnvironment.OCX_WINDOWS_ACTUAL_OWNER='1'
$taskEnvironment.OCX_WINDOWS_OWNER_NODE=$taskNode
$taskEnvironment.OPENCODEX_SANDBOX='1';$taskEnvironment.OPENCODEX_SANDBOX_ROOT=$taskRoot
$taskEnvironment.DO_NOT_TRACK='1';$taskEnvironment.BUN_CONFIG_NO_CLEAR_TERMINAL='1'
$taskEnvironment.npm_config_registry='https://registry.npmjs.org/'
$taskEnvironment.npm_config_userconfig=Join-Path $taskRoot 'npmrc'
$taskEnvironment.npm_config_globalconfig=Join-Path $taskRoot 'global-npmrc'
$taskEnvironment.npm_config_cache=Join-Path $taskRoot 'npm-cache'
[IO.File]::WriteAllText($taskEnvironment.npm_config_userconfig,'registry=https://registry.npmjs.org/'+[Environment]::NewLine)
[IO.File]::WriteAllText($taskEnvironment.npm_config_globalconfig,'')
[IO.File]::WriteAllText((Join-Path $taskEnvironment.CODEX_HOME 'config.toml'),@'
cli_auth_credentials_store = "file"
check_for_update_on_startup = false
web_search = "disabled"
[analytics]
enabled = false
[feedback]
enabled = false
'@)
Add-Type -TypeDefinition @'
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Threading;
public sealed class Ocx021OwnerJob : IDisposable {
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr CreateJobObjectW(IntPtr a,string n);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool AssignProcessToJobObject(IntPtr j,IntPtr p);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool TerminateJobObject(IntPtr j,uint c);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool QueryInformationJobObject(IntPtr j,int c,IntPtr b,uint l,IntPtr r);
 [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
 IntPtr job=CreateJobObjectW(IntPtr.Zero,null);
 public Ocx021OwnerJob(){if(job==IntPtr.Zero)throw new System.ComponentModel.Win32Exception();}
 public void Attach(Process p){if(!AssignProcessToJobObject(job,p.Handle))throw new System.ComponentModel.Win32Exception();}
 public bool KillAndWait(){if(!TerminateJobObject(job,1))throw new System.ComponentModel.Win32Exception();IntPtr b=Marshal.AllocHGlobal(48);try{for(int i=0;i<100;i++){if(!QueryInformationJobObject(job,1,b,48,IntPtr.Zero))throw new System.ComponentModel.Win32Exception();if(Marshal.ReadInt32(b,40)==0)return true;Thread.Sleep(100);}return false;}finally{Marshal.FreeHGlobal(b);}}
 public void Dispose(){if(job!=IntPtr.Zero){TerminateJobObject(job,1);CloseHandle(job);job=IntPtr.Zero;}}
}
'@
$taskProcesses=@();$taskJobs=@();$taskReport=[ordered]@{schemaVersion=1;result='fail';phase='binding-only';sourceCommit=(& git -C $taskRepo rev-parse HEAD).Trim();productTree=$taskProduct;root=$taskRoot;platform='Windows';officialVersion='2.84.0';codexVersion='0.166.0';bindingMode='default-npm-PATH';forcedNativeVariables=$false;processes=@();providerStarted=$false;turnSent=$false;toolsExecuted=$false}
$taskBinding=Join-Path $taskRoot 'default-settings.json'
function Invoke-Owned([string]$Name,[string]$Executable,[string[]]$Arguments,[bool]$Gate,[int]$TimeoutSeconds){
 $taskInfo=[Diagnostics.ProcessStartInfo]::new();$taskInfo.FileName=$Executable;$taskInfo.UseShellExecute=$false;$taskInfo.WorkingDirectory=Join-Path $taskRoot 'work'
 $taskInfo.RedirectStandardOutput=$true;$taskInfo.RedirectStandardError=$true;$taskInfo.RedirectStandardInput=$true;$taskInfo.CreateNoWindow=$true
 $taskInfo.Environment.Clear();foreach($taskKey in $taskEnvironment.Keys){$taskInfo.Environment[$taskKey]=$taskEnvironment[$taskKey]}
 foreach($taskArgument in $Arguments){$taskInfo.ArgumentList.Add($taskArgument)}
 $taskJob=[Ocx021OwnerJob]::new();$script:taskJobs+=,$taskJob
 $taskProcess=[Diagnostics.Process]::Start($taskInfo);$script:taskProcesses+=,$taskProcess
 $taskOut=$taskProcess.StandardOutput.ReadToEndAsync();$taskErr=$taskProcess.StandardError.ReadToEndAsync()
 try{
  $taskJob.Attach($taskProcess)
  if($Gate){$taskProcess.StandardInput.WriteLine('owned-job-ready');$taskProcess.StandardInput.Flush()}
  $taskProcess.StandardInput.Close()
  if(!$taskProcess.WaitForExit($TimeoutSeconds*1000)){throw "$Name exceeded its bounded deadline"}
  $taskExit=$taskProcess.ExitCode
 }finally{
  $taskCleaned=$taskJob.KillAndWait();if(!$taskProcess.HasExited){[void]$taskProcess.WaitForExit(5000)}
  [IO.File]::WriteAllText((Join-Path $taskOutput ($Name+'.stdout')),$taskOut.GetAwaiter().GetResult())
  [IO.File]::WriteAllText((Join-Path $taskOutput ($Name+'.stderr')),$taskErr.GetAwaiter().GetResult())
  $script:taskReport.processes+=@{name=$Name;pid=$taskProcess.Id;executable=$Executable;exitCode=$taskProcess.ExitCode;ownedJobEmpty=$taskCleaned;probeStartedOnlyAfterJobAssignment=$Gate}
  if(!$taskCleaned){throw "$Name owned process cleanup not confirmed"}
 }
 if($taskExit -ne 0){throw "$Name exited $taskExit"}
}
try{
 # Fixed public package download only; no lifecycle scripts or global install.
 Invoke-Owned 'npm-public-packages' $taskNode @($taskNpm,'install','--prefix',(Join-Path $taskRoot 'installation'),'--ignore-scripts','--no-audit','--no-fund','--no-update-notifier','@bitkyc08/opencodex@2.84.0','@openai/codex@0.166.0','@oven/bun-windows-x64@1.4.2') $false 300
 $taskOfficial=Join-Path $taskRoot 'installation/node_modules/@bitkyc08/opencodex'
 $taskBun=Join-Path $taskRoot 'installation/node_modules/@oven/bun-windows-x64/bin/bun.exe'
 if(!(Test-Path -LiteralPath $taskBun -PathType Leaf)){throw 'Published fixed Bun platform executable missing'}
 Copy-Item -LiteralPath (Join-Path $taskRoot 'installation/package-lock.json') -Destination (Join-Path $taskOutput 'public-packages.lock.json')
 Invoke-Owned 'official-default-settings' $taskBun @((Join-Path $PSScriptRoot 'windows-021-client-owner.mjs'),$taskRoot,$taskOfficial,$taskBinding) $true 90
 Copy-Item -LiteralPath $taskBinding -Destination (Join-Path $taskOutput 'default-settings.json')
 $taskFacts=Get-Content -LiteralPath $taskBinding -Raw|ConvertFrom-Json
 $taskReport.codexRuntime=$taskFacts.codexRuntime;$taskReport.nativeOwnerAdmitted=$taskFacts.admission.admitted
 $taskReport.productBlocked=!$taskFacts.admission.admitted
 $taskReport.bindingEvidence='default-settings.json';$taskReport.result='binding-captured'
 if($TestExecutable){
  $taskTest=(Resolve-Path -LiteralPath $TestExecutable).Path
  if([IO.Path]::GetExtension($taskTest) -ine '.exe'){throw 'Actual Windows Rust test EXE required'}
  $taskEnvironment.OCX_WINDOWS_OWNER_ROOT=$taskRoot
  $taskEnvironment.OCX_WINDOWS_OWNER_WORK=Join-Path $taskRoot 'work'
  $taskEnvironment.OCX_WINDOWS_OWNER_BINDING=$taskBinding
  $taskEnvironment.OCX_WINDOWS_OWNER_EVIDENCE=$taskOutput
  $taskReport.ownerTestExecutable=$taskTest
  $taskReport.ownerTestSha256=(Get-FileHash -LiteralPath $taskTest -Algorithm SHA256).Hash.ToLowerInvariant()
  Invoke-Owned 'actual-native-owner' $taskTest @('actual::windows_actual_','--ignored','--test-threads=1','--nocapture') $false 180
  foreach($taskReceipt in @('windows-actual-owner-dual.json','windows-actual-owner-external-refusal.json')){
   $taskOwner=Get-Content -LiteralPath (Join-Path $taskOutput $taskReceipt) -Raw|ConvertFrom-Json
   if($taskOwner.result -cne 'pass'){throw 'Actual native owner case did not pass'}
  }
  $taskReport.result='native-owner-passed';$taskReport.nativeOwnerIntegration=$true
 }
}catch{$taskReport.error=$_.Exception.Message;throw}
finally{
 if(Test-Path -LiteralPath $taskBinding){
  Copy-Item -LiteralPath $taskBinding -Destination (Join-Path $taskOutput 'default-settings.json') -Force
  $taskObserved=Get-Content -LiteralPath $taskBinding -Raw|ConvertFrom-Json
  $taskReport.codexRuntime=$taskObserved.codexRuntime
  $taskReport.admission=$taskObserved.admission
  $taskReport.machineObservation=$taskObserved.machineObservation
  $taskReport.bindingEvidence='default-settings.json'
 }
 foreach($taskJob in $taskJobs){$taskJob.Dispose()}
 foreach($taskProcess in $taskProcesses){$taskProcess.Dispose()}
 $taskReport.finishedAtUtc=[datetime]::UtcNow.ToString('o')
 $taskReport|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $taskOutput 'receipt.json') -Encoding utf8
 Remove-Item -LiteralPath $taskRoot -Recurse -Force -ErrorAction SilentlyContinue
}

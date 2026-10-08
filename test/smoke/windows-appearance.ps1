param(
 [Parameter(Mandatory=$true)][string]$Executable,
 [Parameter(Mandatory=$true)][string]$Node,
 [Parameter(Mandatory=$true)][string]$OutputDirectory,
 [ValidateSet('ordinary','administrator')][string]$Mode='ordinary'
)
$ErrorActionPreference='Stop'
$taskExe=(Resolve-Path -LiteralPath $Executable).Path
$taskNode=(Resolve-Path -LiteralPath $Node).Path
$taskOutput=[System.IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Path $taskOutput -Force|Out-Null
if(!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7){throw 'Windows PowerShell 7 required'}
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.ComponentModel;
public static class AppearanceNative {
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
 [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr window,int action);
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window,out uint process);
 [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
 [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint first,uint second,bool attach);
 [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr window);
 public static bool Activate(IntPtr window) {
  uint ignored; uint current=GetCurrentThreadId();
  uint foreground=GetWindowThreadProcessId(GetForegroundWindow(),out ignored);
  uint target=GetWindowThreadProcessId(window,out ignored);
  bool a=foreground!=0 && foreground!=current && AttachThreadInput(current,foreground,true);
  bool b=target!=0 && target!=current && target!=foreground && AttachThreadInput(current,target,true);
  try { BringWindowToTop(window); return SetForegroundWindow(window); }
  finally { if(b)AttachThreadInput(current,target,false);if(a)AttachThreadInput(current,foreground,false); }
 }
 [DllImport("advapi32.dll",SetLastError=true)]
 static extern bool GetTokenInformation(IntPtr token,int kind,out int value,int size,out int returned);
 public static int Token(IntPtr token,int kind) {
  if(!GetTokenInformation(token,kind,out int value,4,out int returned))throw new Win32Exception(Marshal.GetLastWin32Error());
  return value;
 }
}
'@
$taskIdentity=[System.Security.Principal.WindowsIdentity]::GetCurrent()
$taskElevated=[bool][AppearanceNative]::Token($taskIdentity.Token,20)
$taskElevationType=[AppearanceNative]::Token($taskIdentity.Token,18)
$taskIdentity.Dispose()
if(($Mode -eq 'administrator') -ne $taskElevated){throw 'Requested and actual permission token differ'}
$taskCommit=(& git rev-parse HEAD)
$taskPrevious=[AppearanceNative]::GetForegroundWindow()
function Focus-Candidate([IntPtr]$Window,[int]$ProcessId) {
 [void][AppearanceNative]::ShowWindow($Window,5)
 [void][AppearanceNative]::ShowWindow($Window,9)
 $taskActivation=New-Object -ComObject WScript.Shell
 try {
  for($taskTry=0;$taskTry -lt 4;$taskTry++){
   [void]$taskActivation.AppActivate($ProcessId)
   [void][AppearanceNative]::Activate($Window)
   Start-Sleep -Milliseconds 250
   if([AppearanceNative]::GetForegroundWindow() -eq $Window){return}
  }
  throw 'Candidate could not acquire the native foreground window'
 } finally { [void][System.Runtime.InteropServices.Marshal]::ReleaseComObject($taskActivation) }
}
function Invoke-AppearancePhase([string]$Phase,[string]$Destination,[string]$Log,[bool]$KeepForeground=$true) {
 $taskPhaseInfo=[System.Diagnostics.ProcessStartInfo]::new()
 $taskPhaseInfo.FileName=$taskNode
 $taskPhaseInfo.UseShellExecute=$false;$taskPhaseInfo.CreateNoWindow=$true
 $taskPhaseInfo.RedirectStandardOutput=$true;$taskPhaseInfo.RedirectStandardError=$true
 foreach($taskArgument in @((Join-Path $PSScriptRoot 'windows-appearance.mjs'),"http://127.0.0.1:$taskPort",$Destination,$Phase,$taskScale)){
  $taskPhaseInfo.ArgumentList.Add($taskArgument)
 }
 $taskChild=[System.Diagnostics.Process]::Start($taskPhaseInfo)
 $taskStdout=$taskChild.StandardOutput.ReadToEndAsync();$taskStderr=$taskChild.StandardError.ReadToEndAsync()
 $taskReacquired=0
 try {
  while(!$taskChild.WaitForExit(50)){
   if($KeepForeground -and [AppearanceNative]::GetForegroundWindow() -ne $taskWindow){
    Focus-Candidate $taskWindow $taskProcess.Id
    $taskReacquired++
   }
  }
  $taskStdout.Result+$taskStderr.Result|Set-Content -LiteralPath $Log -Encoding utf8
  @{phase=$Phase;exitCode=$taskChild.ExitCode;foregroundReacquisitions=$taskReacquired}|ConvertTo-Json|Set-Content -LiteralPath ($Log+'.json') -Encoding utf8
  if($taskChild.ExitCode -ne 0){throw "Appearance phase failed: $Phase ($($taskChild.ExitCode))"}
 } finally {
  if(!$taskChild.HasExited){$taskChild.Kill($true);[void]$taskChild.WaitForExit(5000)}
  $taskChild.Dispose()
 }
}
$taskSummary=[ordered]@{
 mode=$Mode;tokenElevated=$taskElevated;tokenElevationType=$taskElevationType
 powershell=$PSVersionTable.PSVersion.ToString();expectedCommit=$taskCommit
 exeSha256=(Get-FileHash -LiteralPath $taskExe -Algorithm SHA256).Hash.ToLower()
 startedAtUtc=[datetime]::UtcNow.ToString('o');result='fail';scales=@()
 display=@(Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion,CurrentRefreshRate,CurrentHorizontalResolution,CurrentVerticalResolution)
}
$taskFailure=$null
try {
 foreach($taskScale in @('1','1.25','1.5','2')){
  $taskGroup=Join-Path $taskOutput ('scale-'+$taskScale)
  New-Item -ItemType Directory -Path $taskGroup -Force|Out-Null
  $taskSandbox=Join-Path $taskGroup ('sandbox-'+[guid]::NewGuid().ToString('N'))
  $taskListener=[System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback,0)
  $taskListener.Start();$taskPort=$taskListener.LocalEndpoint.Port;$taskListener.Stop()
  $taskInfo=[System.Diagnostics.ProcessStartInfo]::new()
  $taskInfo.FileName=$taskExe;$taskInfo.WorkingDirectory=Split-Path -LiteralPath $taskExe
  $taskInfo.UseShellExecute=$false;$taskInfo.CreateNoWindow=$true
  $taskInfo.RedirectStandardOutput=$true;$taskInfo.RedirectStandardError=$true
  $taskInfo.Environment['OPENCODEX_SANDBOX']='1'
  $taskInfo.Environment['OPENCODEX_SANDBOX_ROOT']=$taskSandbox
  $taskInfo.Environment['OPENCODEX_WINDOWS_SMOKE_CDP_PORT']=[string]$taskPort
  $taskInfo.Environment['OPENCODEX_WINDOWS_SMOKE_SCALE']=$taskScale
  [void]$taskInfo.Environment.Remove('HOME')
  [void]$taskInfo.Environment.Remove('WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS')
  [void]$taskInfo.Environment.Remove('WEBVIEW2_USER_DATA_FOLDER')
  $taskProcess=$null;$taskErrorRead=$null;$taskOutRead=$null
  $taskRun=[ordered]@{scale=$taskScale;sandbox=$taskSandbox;port=$taskPort;result='fail'}
  try {
   $taskProcess=[System.Diagnostics.Process]::Start($taskInfo)
   $taskErrorRead=$taskProcess.StandardError.ReadToEndAsync();$taskOutRead=$taskProcess.StandardOutput.ReadToEndAsync()
   Start-Sleep -Seconds 2;$taskProcess.Refresh()
   if($taskProcess.HasExited){throw 'Candidate exited before first screen'}
   $taskRun.pid=$taskProcess.Id;$taskWindow=$taskProcess.MainWindowHandle
   if(!$taskWindow){throw 'Native main window missing'}
   $taskLog=Join-Path $taskSandbox 'logs\app.log'
   if(!(Get-Content -LiteralPath $taskLog -Raw).Contains("commit=$taskCommit")){throw 'EXE source provenance mismatch'}
   Focus-Candidate $taskWindow $taskProcess.Id
   $taskRun.foregroundWindow=[AppearanceNative]::GetForegroundWindow().ToInt64()
   & $taskNode (Join-Path $PSScriptRoot 'windows-webview.mjs') "http://127.0.0.1:$taskPort" $taskGroup *> (Join-Path $taskGroup 'firstscreen.log')
   if($LASTEXITCODE -ne 0){throw "First-screen assertions failed: $LASTEXITCODE"}
   Invoke-AppearancePhase visual $taskGroup (Join-Path $taskGroup 'visual.log')
   Invoke-AppearancePhase menu $taskGroup (Join-Path $taskGroup 'menu.log')
   Invoke-AppearancePhase performance $taskGroup (Join-Path $taskGroup 'performance.log')
   foreach($taskTier in @('high','mid')){
    Invoke-AppearancePhase ('prepare-'+$taskTier) $taskGroup (Join-Path $taskGroup ('prepare-'+$taskTier+'.log'))
    foreach($taskVisibility in @('minimized','hidden')){
    $taskAction=if($taskVisibility -eq 'minimized'){6}else{0}
    [void][AppearanceNative]::ShowWindow($taskWindow,$taskAction);Start-Sleep -Milliseconds 200
    $taskStateOutput=Join-Path $taskGroup ($taskTier+'-'+$taskVisibility)
    Invoke-AppearancePhase background $taskStateOutput (Join-Path $taskGroup ($taskTier+'-'+$taskVisibility+'.log')) $false
    Focus-Candidate $taskWindow $taskProcess.Id
    Invoke-AppearancePhase recovery $taskStateOutput (Join-Path $taskGroup ($taskTier+'-'+$taskVisibility+'-recovery.log'))
   }
   }
   Invoke-AppearancePhase reload $taskGroup (Join-Path $taskGroup 'reload.log')
   $taskRun.result='pass'
  } catch {
   $taskRun.error=$_.Exception.Message
   throw
  } finally {
   if(Test-Path -LiteralPath $taskLog){Copy-Item -LiteralPath $taskLog -Destination (Join-Path $taskGroup 'app.log')}
   if($taskProcess){
    if(!$taskProcess.HasExited){$taskProcess.Kill($true);[void]$taskProcess.WaitForExit(10000)}
    if($taskErrorRead -and $taskErrorRead.Wait(5000)){$taskErrorRead.Result|Set-Content -LiteralPath (Join-Path $taskGroup 'stderr.log') -Encoding utf8}
    if($taskOutRead -and $taskOutRead.Wait(5000)){$taskOutRead.Result|Set-Content -LiteralPath (Join-Path $taskGroup 'stdout.log') -Encoding utf8}
    $taskProcess.Dispose()
   }
   $taskSummary.scales+= $taskRun
   $taskRun|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $taskGroup 'run.json') -Encoding utf8
  }
 }
 $taskSummary.result='pass'
}catch{$taskFailure=$_;$taskSummary.error=$_.Exception.Message}
finally{
 [void][AppearanceNative]::SetForegroundWindow($taskPrevious)
 $taskSummary.finishedAtUtc=[datetime]::UtcNow.ToString('o')
 $taskSummary|ConvertTo-Json -Depth 7|Set-Content -LiteralPath (Join-Path $taskOutput 'matrix.json') -Encoding utf8
}
if($taskFailure){throw $taskFailure}

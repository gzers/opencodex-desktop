param([Parameter(Mandatory)][string]$Evidence)
$ErrorActionPreference='Stop'
$evidencePath=[IO.Path]::GetFullPath($Evidence)
if($evidencePath -cne 'C:\Users\15119\.codex\worktrees\official299-windows-evidence\docs-governance\.adg\evidence\0.2.24\windows-dev\wl2-r6-official-source-paths-20261117-r1\archive-probe02'){throw 'exact fresh directed proof output required'}
if(Test-Path -LiteralPath $evidencePath){throw 'immutable native probe already exists'}
[void][IO.Directory]::CreateDirectory($evidencePath)
Add-Type -Path (Join-Path $PSScriptRoot 'archive-r6-process-facts.cs')
Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;
public static class R6ArchiveOwnedProbe {
 [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)]
 public struct Startup { public uint cb; public string reserved, desktop, title; public uint x,y,width,height,xchars,ychars,fill,flags; public ushort show,reservedBytes; public IntPtr reservedPointer,input,output,error; }
 [StructLayout(LayoutKind.Sequential)]
 public struct Process { public IntPtr process,thread; public uint pid,tid; }
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)]
 static extern bool CreateProcess(string app,StringBuilder command,IntPtr processSecurity,IntPtr threadSecurity,bool inherit,uint flags,IntPtr environment,string directory,ref Startup startup,out Process process);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetProcessTimes(IntPtr p,out long c,out long e,out long k,out long u);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool TerminateProcess(IntPtr process,uint code);
 [DllImport("kernel32.dll",SetLastError=true)] static extern uint WaitForSingleObject(IntPtr process,uint timeout);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetExitCodeProcess(IntPtr process,out uint code);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool CloseHandle(IntPtr handle);
 public static Process Create(string image) {Startup s=new Startup();s.cb=(uint)Marshal.SizeOf(s);Process p;if(!CreateProcess(image,new StringBuilder("\""+image+"\" /d /c exit 0"),IntPtr.Zero,IntPtr.Zero,false,0x08000004,IntPtr.Zero,System.IO.Path.GetDirectoryName(image),ref s,out p))throw new Win32Exception(Marshal.GetLastWin32Error());return p;}
 public static string Creation(Process p){long c,e,k,u;if(!GetProcessTimes(p.process,out c,out e,out k,out u))throw new Win32Exception(Marshal.GetLastWin32Error());return c.ToString(System.Globalization.CultureInfo.InvariantCulture);}
 public static uint CleanupOriginal(Process p){if(!TerminateProcess(p.process,242))throw new Win32Exception(Marshal.GetLastWin32Error());if(WaitForSingleObject(p.process,5000)!=0)throw new Exception("original owned probe forced cleanup did not finish in5s");uint code;if(!GetExitCodeProcess(p.process,out code))throw new Win32Exception(Marshal.GetLastWin32Error());return code;}
 public static void CloseOriginal(Process p){if(!CloseHandle(p.thread))throw new Win32Exception(Marshal.GetLastWin32Error());if(!CloseHandle(p.process))throw new Win32Exception(Marshal.GetLastWin32Error());}
}
'@
$image=Join-Path $env:SystemRoot 'System32/cmd.exe'
$result=[ordered]@{scope='controller-owned never-resumed harmless cmd only; no official/marenben process, Owner or normalStop authority';passed=$false;source=[ordered]@{image=$image;helperSha256=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'archive-r6-process-facts.cs') -Algorithm SHA256).Hash.ToLowerInvariant()};ordinaryAcceptance=$false;officialExecuted=$false;resumed=$false;forcedCleanup=$false;rawExit=$null}
$created=$false;$closed=$false
try {
 $p=[R6ArchiveOwnedProbe]::Create($image);$created=$true;$result.pid=$p.pid;$result.creation=[R6ArchiveOwnedProbe]::Creation($p)
 $facts=[R6ArchiveProcessFacts]::Observe($p.pid)
 $result.actualFacts=$facts
 if($facts.filetime -cne $result.creation -or ![string]::Equals($facts.image.Replace([char]47,[char]92),$image.Replace([char]47,[char]92),[StringComparison]::OrdinalIgnoreCase) -or $facts.session -ne [Diagnostics.Process]::GetCurrentProcess().SessionId -or $facts.exited){throw 'query-only kernel image/times/session differs from retained original suspended creation'}
 try{$module=Get-Process -Id $p.pid -ErrorAction Stop;try{$result.moduleEnumerationPath=$module.MainModule.FileName}finally{$module.Dispose()}}catch{$result.moduleEnumerationError=$_.Exception.Message}
 $result.rawExit=[R6ArchiveOwnedProbe]::CleanupOriginal($p);$result.forcedCleanup=$true
 if($result.rawExit -ne 242){throw 'original forced cleanup raw exit changed'}
 $result.passed=$true
}catch{$result.firstError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
finally {
 if($created){if(!$result.forcedCleanup){try{$result.rawExit=[R6ArchiveOwnedProbe]::CleanupOriginal($p);$result.forcedCleanup=$true}catch{$result.cleanupError=$_.Exception.Message}};[R6ArchiveOwnedProbe]::CloseOriginal($p);$closed=$true}
 $result.originalHandlesClosed=$closed
 $result|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $evidencePath 'result.json') -Encoding utf8
}
if(!$result.passed){throw 'directed query-only archive observer failed; raw proof retained'}

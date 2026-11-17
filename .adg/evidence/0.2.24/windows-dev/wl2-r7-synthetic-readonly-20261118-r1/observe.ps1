$ErrorActionPreference='Stop'
$taskTargetPid=27960
$taskExpectedPath='C:\Users\15119\AppData\Local\Temp\.tmpMgOtNr\never-resumed-fixture.exe'
$taskExpectedHash='4699667b79d3aebd6e1e90c8b063f80a95b3f32c5c4a69ec49bd4c70217e4a38'
Add-Type -TypeDefinition @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class Ocx024OwnReadonly {
 [DllImport("kernel32.dll",SetLastError=true)] public static extern IntPtr OpenProcess(uint access,bool inherit,uint pid);
 [DllImport("kernel32.dll",SetLastError=true)] public static extern bool CloseHandle(IntPtr handle);
 [DllImport("kernel32.dll",SetLastError=true)] public static extern bool GetProcessTimes(IntPtr p,out long created,out long exited,out long kernel,out long user);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] public static extern bool QueryFullProcessImageName(IntPtr p,uint flags,StringBuilder name,ref uint size);
 [DllImport("kernel32.dll",SetLastError=true)] public static extern bool IsProcessInJob(IntPtr p,IntPtr job,out bool inJob);
 [DllImport("kernel32.dll",SetLastError=true)] public static extern bool ProcessIdToSessionId(uint pid,out uint session);
 [DllImport("kernel32.dll",SetLastError=true)] public static extern bool GetExitCodeProcess(IntPtr p,out uint code);
}
'@
$taskResult=[ordered]@{observedAt=(Get-Date).ToString('o');targetPid=$taskTargetPid;expectedImage=$taskExpectedPath;expectedImageSha256=$taskExpectedHash;readOnly=$true;processAccess='PROCESS_QUERY_LIMITED_INFORMATION 0x1000 only';originalCreationHandleRetained=$false;originalJobHandleRetained=$false;historicalNeverResumeProof='Unknown; no assertion from currently suspended state';terminationAuthorized=$false;terminationExecuted=$false;currentR2Touched=$false;errors=@()}
$taskHandle=[Ocx024OwnReadonly]::OpenProcess(0x1000,$false,$taskTargetPid)
if($taskHandle -eq [IntPtr]::Zero){$taskResult.errors+=@{stage='OpenProcess';raw=[Runtime.InteropServices.Marshal]::GetLastWin32Error()}}
else {
 try {
  [long]$created=0;[long]$exited=0;[long]$kernel=0;[long]$user=0
  if([Ocx024OwnReadonly]::GetProcessTimes($taskHandle,[ref]$created,[ref]$exited,[ref]$kernel,[ref]$user)){
   $taskResult.creationFiletime=$created.ToString();$taskResult.creationUtc=[DateTime]::FromFileTimeUtc($created).ToString('o')
  }else{$taskResult.errors+=@{stage='GetProcessTimes';raw=[Runtime.InteropServices.Marshal]::GetLastWin32Error()}}
  $name=[Text.StringBuilder]::new(32768);[uint32]$size=32768
  if([Ocx024OwnReadonly]::QueryFullProcessImageName($taskHandle,0,$name,[ref]$size)){
   $taskResult.imagePath=$name.ToString();$taskResult.exactTaskPath=$taskResult.imagePath -ieq $taskExpectedPath
  }else{$taskResult.errors+=@{stage='QueryFullProcessImageName';raw=[Runtime.InteropServices.Marshal]::GetLastWin32Error()}}
  [bool]$inJob=$false
  if([Ocx024OwnReadonly]::IsProcessInJob($taskHandle,[IntPtr]::Zero,[ref]$inJob)){$taskResult.inSomeJob=$inJob}else{$taskResult.errors+=@{stage='IsProcessInJob';raw=[Runtime.InteropServices.Marshal]::GetLastWin32Error()}}
  [uint32]$session=0
  if([Ocx024OwnReadonly]::ProcessIdToSessionId($taskTargetPid,[ref]$session)){$taskResult.sessionId=$session}else{$taskResult.errors+=@{stage='ProcessIdToSessionId';raw=[Runtime.InteropServices.Marshal]::GetLastWin32Error()}}
  [uint32]$exitCode=0
  if([Ocx024OwnReadonly]::GetExitCodeProcess($taskHandle,[ref]$exitCode)){$taskResult.exitCodeRead=$exitCode}else{$taskResult.errors+=@{stage='GetExitCodeProcess';raw=[Runtime.InteropServices.Marshal]::GetLastWin32Error()}}
  $taskResult.cim=Get-CimInstance Win32_Process -Filter "ProcessId=$taskTargetPid" | Select-Object ProcessId,ParentProcessId,Name,CreationDate,ExecutablePath
  $taskResult.threads=@(Get-CimInstance Win32_Thread -Filter "ProcessHandle='$taskTargetPid'" | Select-Object Handle,ProcessHandle,ThreadState,ThreadWaitReason,ExecutionState)
  if(Test-Path -LiteralPath $taskExpectedPath){$taskResult.taskFileSha256=(Get-FileHash -LiteralPath $taskExpectedPath -Algorithm SHA256).Hash.ToLowerInvariant();$taskResult.taskFileHashMatches=$taskResult.taskFileSha256 -eq $taskExpectedHash}
  [long]$createdAfter=0
  if([Ocx024OwnReadonly]::GetProcessTimes($taskHandle,[ref]$createdAfter,[ref]$exited,[ref]$kernel,[ref]$user)){$taskResult.sameHeldQueryHandleCreation=$createdAfter -eq $created}else{$taskResult.errors+=@{stage='GetProcessTimesAfter';raw=[Runtime.InteropServices.Marshal]::GetLastWin32Error()}}
 }catch{$taskResult.errors+=@{stage='readonlyCollector';error=$_.Exception.Message}}
 finally{$taskResult.queryHandleClosed=[Ocx024OwnReadonly]::CloseHandle($taskHandle)}
}
$path=Join-Path $PSScriptRoot 'observation.json'
if(Test-Path -LiteralPath $path){throw 'refuse overwrite prior observation'}
[IO.File]::WriteAllText($path,($taskResult|ConvertTo-Json -Depth 10),[Text.UTF8Encoding]::new($false))
$taskResult|ConvertTo-Json -Depth 10

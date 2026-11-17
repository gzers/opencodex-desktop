param([Parameter(Mandatory)][string]$Root)
$ErrorActionPreference='Stop'
$rootPath=[IO.Path]::GetFullPath($Root)
if($rootPath -cne 'C:\Users\Public\ocx-r6-official-existing-20261117-r2'){throw 'exact prepared qualification root required'}
$base='C:\Users\marenben\ocx024-r6-existing-20261117-r2'
$out=Join-Path $rootPath 'out01'
$receipt=Join-Path $out 'launch-result.json'
if(Test-Path -LiteralPath $receipt){throw 'immutable actual round already ran; no same premise retry'}
$id=[Security.Principal.WindowsIdentity]::GetCurrent()
$admin=([Security.Principal.WindowsPrincipal]::new($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
$session=[Diagnostics.Process]::GetCurrentProcess().SessionId
$result=[ordered]@{sid=$id.User.Value;session=$session;elevated=$admin;scope='actual isolated existing-root fixed official native lifecycle qualification; public Start remains refused';publicStartOpen=$false;allAccepted=$false;brokerRawExit=$null;brokerRetained=$false}
function NewPrivateTaskDirectory([string]$Path) {
 if($Path -notin @($base,(Join-Path $base 'I'),(Join-Path $base 'D'),(Join-Path $base 'H'))){throw 'exact fresh task directory only'}
 if(Test-Path -LiteralPath $Path){throw 'fresh task object already exists; no replace/adopt/retry'}
 $security=New-Object Security.AccessControl.DirectorySecurity
 $security.SetSecurityDescriptorSddlForm(('O:{0}D:P(A;OICI;FA;;;{0})(A;OICI;FA;;;SY)' -f $id.User.Value))
 [void][IO.Directory]::CreateDirectory($Path,$security)
 if((Get-Item -LiteralPath $Path -Force).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'new task directory reparse refused'}
}
function PathFacts {
 @('User','Machine')|ForEach-Object {
  $value=[Environment]::GetEnvironmentVariable('Path',$_);$digest=$null
  if($null -ne $value){$alg=[Security.Cryptography.SHA256]::Create();try{$digest=([BitConverter]::ToString($alg.ComputeHash([Text.Encoding]::Unicode.GetBytes($value)))).Replace('-','').ToLowerInvariant()}finally{$alg.Dispose()}}
  @{scope=$_;present=$null -ne $value;sha256=$digest}
 }|ConvertTo-Json -Compress
}
$before=PathFacts
try {
 if($id.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or $session -ne 7 -or $admin -or $env:USERPROFILE -cne 'C:\Users\marenben'){throw 'original ordinary marenben session7 actual Profile required'}
 $manifest=Get-Content -LiteralPath (Join-Path $rootPath 'input-manifest.json') -Raw|ConvertFrom-Json
 if($manifest.scope -cne 'isolated-r6-existing-official-qualification' -or $manifest.expectedSid -cne $id.User.Value -or $manifest.expectedSession -ne $session){throw 'fixed prepared scope/SID/session mismatch'}
 foreach($file in $manifest.files){
  $path=[IO.Path]::GetFullPath((Join-Path (Join-Path $rootPath 'input') $file.name))
  if([IO.Path]::GetDirectoryName($path) -cne (Join-Path $rootPath 'input')){throw 'input escapes exact task'}
  $item=Get-Item -LiteralPath $path -Force
  if(($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $item.Length -ne $file.bytes -or (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $file.sha256){throw 'pinned input bytes differ'}
 }
 $oldCleanup='C:/Users/Public/ocx-r6-authorized-cleanup-20261117-r1/out01/cleanup-result.json'
 if((Get-FileHash -LiteralPath $oldCleanup -Algorithm SHA256).Hash.ToLowerInvariant() -cne '7a8c90139cf67184853837c699d4182464c9800b1a0999f975166daf7067cf0e'){throw 'exact accepted old cleanup receipt changed'}
 if(Test-Path -LiteralPath $base){throw 'independent task base already exists; no same failed transaction retry'}
 @{base=$base;state='Absent';oldFailureRetained=$true;source='independent fresh r2 I/D/H only';oldCleanupReceiptSha256='7a8c90139cf67184853837c699d4182464c9800b1a0999f975166daf7067cf0e';nativeLifecycleAuthority=$false}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $out 'fresh-task-before.json') -Encoding UTF8
 NewPrivateTaskDirectory $base
 foreach($directoryName in @('I','D','H')){NewPrivateTaskDirectory (Join-Path $base $directoryName)}
 foreach($name in @('ocxd.exe','opencodex-desktop.exe','ocx.exe')){
  $target=Join-Path (Join-Path $base 'I') $name
  if(Test-Path -LiteralPath $target){throw 'original I image already exists; no replace/adopt'}
  $src=[IO.File]::OpenRead((Join-Path (Join-Path $rootPath 'input') $name));$dst=[IO.File]::Open($target,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
  try{$src.CopyTo($dst);$dst.Flush()}finally{$src.Dispose();$dst.Dispose()}
  $expected=@($manifest.files|Where-Object {$_.name -ceq $name})
  if($expected.Count -ne 1 -or (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant() -cne $expected[0].sha256){throw 'exact task I image copy differs'}
 }
 $bootstrap=@{schema=1;scope=$manifest.scope;expectedSid=$id.User.Value;expectedSession=$session;profile=$env:USERPROFILE;installation=(Join-Path $base 'I');data=(Join-Path $base 'D');home=(Join-Path $base 'H');brokerSha256=(@($manifest.files|Where-Object {$_.name -ceq 'ocxd.exe'})[0].sha256);managerSha256=(@($manifest.files|Where-Object {$_.name -ceq 'opencodex-desktop.exe'})[0].sha256)}
 $payload=[Text.Encoding]::UTF8.GetBytes(($bootstrap|ConvertTo-Json -Compress))
 if($payload.Length -le 0 -or $payload.Length -gt 65536){throw 'bounded bootstrap'}
 $psi=New-Object Diagnostics.ProcessStartInfo
 $psi.FileName=Join-Path $base 'I/ocxd.exe';$psi.Arguments='--internal-runtime-owner024';$psi.WorkingDirectory=$base
 $psi.UseShellExecute=$false;$psi.CreateNoWindow=$true;$psi.RedirectStandardInput=$true;$psi.RedirectStandardOutput=$true;$psi.RedirectStandardError=$true
 foreach($name in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY')){[void]$psi.EnvironmentVariables.Remove($name)}
 $process=New-Object Diagnostics.Process;$process.StartInfo=$psi
 $stdout=[IO.File]::Open((Join-Path $out 'broker.stdout.log'),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 $stderr=[IO.File]::Open((Join-Path $out 'broker.stderr.log'),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try {
  if(!$process.Start()){throw 'original ordinary broker creation failed'}
  $result.brokerPid=$process.Id;$result.brokerCreation=$process.StartTime.ToString('o')
  $copyOut=$process.StandardOutput.BaseStream.CopyToAsync($stdout);$copyErr=$process.StandardError.BaseStream.CopyToAsync($stderr)
  $stream=$process.StandardInput.BaseStream;$length=[BitConverter]::GetBytes([uint32]$payload.Length);$stream.Write($length,0,4);$stream.Write($payload,0,$payload.Length);$stream.Flush();$process.StandardInput.Close()
  $watch=[Diagnostics.Stopwatch]::StartNew()
  $nativeResult=Join-Path $base 'D/qualification-result.json'
  while($watch.Elapsed.TotalSeconds -lt 180 -and !$process.HasExited -and !(Test-Path -LiteralPath $nativeResult)){Start-Sleep -Milliseconds 250}
  $result.observerElapsedMs=$watch.ElapsedMilliseconds
  if($process.HasExited){$process.WaitForExit();$copyOut.GetAwaiter().GetResult();$copyErr.GetAwaiter().GetResult();$result.brokerRawExit=$process.ExitCode}
  else{$result.brokerRetained=$true}
  $result.nativeResultPresent=Test-Path -LiteralPath $nativeResult
  if($result.nativeResultPresent){$native=Get-Content -LiteralPath $nativeResult -Raw -Encoding UTF8|ConvertFrom-Json;$result.actualProductionExecuted=$native.actualProductionExecuted;$result.nativeProductionSequencePassed=$native.nativeProductionSequencePassed;$result.nativeFirstError=$native.firstError}
 }finally{$stdout.Flush();$stderr.Flush();if(!$process.HasExited){$result.brokerRetained=$true};$process.Dispose()}
}catch{$result.firstError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber};$_|Out-String|Set-Content -LiteralPath (Join-Path $out 'launcher-error.log') -Encoding utf8}
finally {
 $after=PathFacts;$result.dailyPathBefore=ConvertFrom-Json $before;$result.dailyPathAfter=ConvertFrom-Json $after;$result.dailyPathUnchanged=$before -ceq $after
 try{& (Join-Path $rootPath 'export-r6-official-existing-r2.ps1') -Root $rootPath}catch{$result.exportError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
 $result.taskAndClientRootsRetained=$true
 $result|ConvertTo-Json -Depth 14|Set-Content -LiteralPath $receipt -Encoding utf8
}

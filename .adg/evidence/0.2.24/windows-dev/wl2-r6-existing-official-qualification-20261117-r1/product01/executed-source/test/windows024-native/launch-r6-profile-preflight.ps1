param([Parameter(Mandatory)][string]$Root)
$ErrorActionPreference='Stop'
$rootPath=[IO.Path]::GetFullPath($Root)
if($rootPath -cne 'C:\Users\Public\ocx-r6-profile-preflight-20261117-r1'){throw 'exact prepared task root required'}
$out=Join-Path $rootPath 'out01'
$receipt=Join-Path $out 'launch-result.json'
if(Test-Path -LiteralPath $receipt){throw 'immutable round already ran; no same-premise retry'}
$id=[Security.Principal.WindowsIdentity]::GetCurrent()
$admin=([Security.Principal.WindowsPrincipal]::new($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
$session=[Diagnostics.Process]::GetCurrentProcess().SessionId
function PathFacts {
 @('User','Machine')|ForEach-Object {
  $value=[Environment]::GetEnvironmentVariable('Path',$_)
  $hash=$null
  if($null -ne $value){$alg=[Security.Cryptography.SHA256]::Create();try{$hash=([BitConverter]::ToString($alg.ComputeHash([Text.Encoding]::Unicode.GetBytes($value)))).Replace('-','').ToLowerInvariant()}finally{$alg.Dispose()}}
  @{scope=$_;present=$null -ne $value;sha256=$hash}
 }|ConvertTo-Json -Compress
}
$before=PathFacts
$result=[ordered]@{sid=$id.User.Value;session=$session;elevated=$admin;scope='synthetic credential-free existing client roots preparation only, not official initialization or production';actualProductionExecuted=$false;startOpen=$false;preparedExistingRoots=$false}
try {
 if($id.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or $session -ne 7 -or $admin){throw 'original ordinary marenben session7 required; no elevation/impersonation'}
 $manifest=Get-Content -LiteralPath (Join-Path $rootPath 'input-manifest.json') -Raw|ConvertFrom-Json
 if($manifest.scope -cne 'r6-ordinary-existing-baseline-preparation' -or $manifest.expectedSid -cne $id.User.Value -or $manifest.expectedSession -ne $session){throw 'prepared exact scope mismatch'}
 foreach($file in $manifest.files){
  $path=[IO.Path]::GetFullPath((Join-Path (Join-Path $rootPath 'input') $file.name))
  if([IO.Path]::GetDirectoryName($path) -cne (Join-Path $rootPath 'input')){throw 'input escapes exact directory'}
  $item=Get-Item -LiteralPath $path
  if($item.Attributes -band [IO.FileAttributes]::ReparsePoint -or $item.Length -ne $file.bytes -or (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $file.sha256){throw 'immutable input differs'}
 }
 $env:OCXD024_R6_PREFLIGHT_PUBLIC=$rootPath
 foreach($name in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY')){[Environment]::SetEnvironmentVariable($name,$null,'Process')}
 $psi=New-Object Diagnostics.ProcessStartInfo
 $psi.FileName=Join-Path $rootPath 'input/r6-profile-tests.exe'
 $psi.Arguments='--exact infrastructure::windows_client_originals024::desktop_preflight_tests::isolated_r6_existing_root_preflight --ignored --test-threads=1 --nocapture'
 $psi.UseShellExecute=$false;$psi.CreateNoWindow=$true;$psi.RedirectStandardOutput=$true;$psi.RedirectStandardError=$true
 $process=New-Object Diagnostics.Process;$process.StartInfo=$psi
 $stdout=[IO.File]::Open((Join-Path $out 'native.stdout.log'),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 $stderr=[IO.File]::Open((Join-Path $out 'native.stderr.log'),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try {
  if(!$process.Start()){throw 'ordinary native preflight creation failed'}
  $result.observerPid=$process.Id;$result.observerCreation=$process.StartTime.ToString('o')
  $copyOut=$process.StandardOutput.BaseStream.CopyToAsync($stdout);$copyErr=$process.StandardError.BaseStream.CopyToAsync($stderr)
  $process.WaitForExit();$copyOut.GetAwaiter().GetResult();$copyErr.GetAwaiter().GetResult()
  $result.rawTestExit=$process.ExitCode
  if($process.ExitCode -eq 0){$native=Get-Content -LiteralPath (Join-Path $out 'preparation-result.json') -Raw|ConvertFrom-Json;$result.preparedExistingRoots=$native.preparedExistingBaseline -eq $true}
 }finally{$stdout.Dispose();$stderr.Dispose();$process.Dispose()}
}catch {
 $result.firstError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}
 $_|Out-String|Set-Content -LiteralPath (Join-Path $out 'launcher-error.log') -Encoding utf8
}finally {
 $after=PathFacts
 $result.dailyPathUnchanged=$before -ceq $after
 $result.dailyPathBefore=ConvertFrom-Json $before
 $result.dailyPathAfter=ConvertFrom-Json $after
 $result.taskAndClientRootsRetained=$true
 try{& (Join-Path $rootPath 'export-r6-profile-preflight.ps1') -Root $rootPath}
 catch{$result.exportError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
 $result|ConvertTo-Json -Depth 10|Set-Content -LiteralPath $receipt -Encoding utf8
}

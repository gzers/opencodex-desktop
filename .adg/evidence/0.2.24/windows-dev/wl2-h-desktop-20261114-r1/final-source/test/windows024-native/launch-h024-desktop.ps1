param([Parameter(Mandatory)][string]$Root)
$ErrorActionPreference='Stop'
$expected='C:/Users/Public/ocx-h024-desktop-20261114-r1'
if([IO.Path]::GetFullPath($Root) -cne [IO.Path]::GetFullPath($expected)){throw 'exact prepared desktop test root required'}
$out=Join-Path $Root 'out01'
if(Test-Path -LiteralPath (Join-Path $out 'launch-result.json')){throw 'immutable round already finished; no same-condition rerun'}
$id=[Security.Principal.WindowsIdentity]::GetCurrent()
$admin=([Security.Principal.WindowsPrincipal]::new($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
$session=[Diagnostics.Process]::GetCurrentProcess().SessionId
$result=[ordered]@{sid=$id.User.Value;session=$session;elevated=$admin;scope='real desktop-started ordinary H empty-host native component only; not actual GUI or official lifecycle';accepted=$false}
try{
 if($id.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or $session -ne 7 -or $admin){throw 'marenben original ordinary interactive session7 required; no elevation or impersonation'}
 $manifest=Get-Content -LiteralPath (Join-Path $Root 'input-manifest.json') -Raw|ConvertFrom-Json
 foreach($file in $manifest.files){
  $path=Join-Path (Join-Path $Root 'input') $file.name
  if((Get-Item -LiteralPath $path).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'reparse input refused'}
  if((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $file.sha256){throw 'prepared exact test artifact digest mismatch'}
 }
 $env:OCXD024_H_INPUT=Join-Path $Root 'input'
 $env:OCXD024_H_EVIDENCE=$out
 $env:RUST_BACKTRACE='1'
 foreach($name in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT')){[Environment]::SetEnvironmentVariable($name,$null,'Process')}
 $psi=New-Object Diagnostics.ProcessStartInfo
 $psi.FileName=Join-Path $Root 'input/h-desktop-tests.exe'
 $psi.Arguments='--exact infrastructure::windows_owner_host024::desktop_tests::isolated_h024_observer --ignored --test-threads=1 --nocapture'
 $psi.UseShellExecute=$false;$psi.CreateNoWindow=$true;$psi.RedirectStandardOutput=$true;$psi.RedirectStandardError=$true
 $process=New-Object Diagnostics.Process;$process.StartInfo=$psi
 $stdout=[IO.File]::Open((Join-Path $out 'test.stdout.log'),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 $stderr=[IO.File]::Open((Join-Path $out 'test.stderr.log'),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try{
  if(!$process.Start()){throw 'observer creation failed'}
  $result.observerPid=$process.Id;$result.observerCreation=$process.StartTime.ToString('o')
  $copyOut=$process.StandardOutput.BaseStream.CopyToAsync($stdout);$copyErr=$process.StandardError.BaseStream.CopyToAsync($stderr)
  $process.WaitForExit();$copyOut.GetAwaiter().GetResult();$copyErr.GetAwaiter().GetResult()
  $result.rawTestExit=$process.ExitCode;$result.hComponentPassed=$process.ExitCode -eq 0
 }finally{$stdout.Dispose();$stderr.Dispose();$process.Dispose()}
}catch{$result.firstError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber};$_|Out-String|Set-Content -LiteralPath (Join-Path $out 'launcher-error.log') -Encoding utf8}
finally{
 # A failed observer may leave only its empty diagnostic host. Request exact
 # same-SID empty-host recovery once; no PID kill, task/service or Job fallback.
 if(!$result.hComponentPassed -and (Test-Path -LiteralPath (Join-Path $out 'task-location.json'))){
  try{
   $location=Get-Content -LiteralPath (Join-Path $out 'task-location.json') -Raw|ConvertFrom-Json
   $base=[IO.Path]::GetFullPath($location.base)
   if([IO.Path]::GetDirectoryName($base) -cne $env:USERPROFILE -or [IO.Path]::GetFileName($base) -notmatch '^ocx024-h-[0-9a-f-]{36}$'){throw 'exact original task base required for empty-host recovery'}
   $finished=Join-Path $base 'D/manager-finish.json'
   if(!(Test-Path -LiteralPath $finished)){
    $env:OCXD024_H_BASE=$base;$env:OCXD024_H_MODE='finish'
    & (Join-Path $base 'I/opencodex-desktop.exe') --exact infrastructure::windows_owner_host024::desktop_tests::isolated_h024_manager --ignored --test-threads=1 --nocapture 1> (Join-Path $out 'failure-empty-recovery.stdout.log') 2> (Join-Path $out 'failure-empty-recovery.stderr.log')
    $result.emptyRecoveryRawExit=$LASTEXITCODE
   }
   $data=Join-Path $base 'D'
   $archive=Join-Path $out 'failed-task-data'
   [void][IO.Directory]::CreateDirectory($archive)
   $rows=@()
   foreach($f in Get-ChildItem -LiteralPath $data -File -Force){
    if($f.Attributes -band [IO.FileAttributes]::ReparsePoint -or $f.Length -gt 1048576){throw 'bounded original failure archive refused'}
    $target=Join-Path $archive $f.Name;[IO.File]::Copy($f.FullName,$target,$false)
    $a=(Get-FileHash -LiteralPath $f.FullName -Algorithm SHA256).Hash
    if((Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash -cne $a){throw 'original failure archive mismatch'}
    $rows+=@{name=$f.Name;bytes=$f.Length;sha256=$a.ToLowerInvariant()}
   }
   $rows|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $out 'failed-task-data-manifest.json') -Encoding utf8
   $result.failureTaskDataArchived=$true;$result.taskBaseRetained=$true
  }catch{$result.failureArchiveError=$_.Exception.Message;$result.taskBaseRetained=$true}
 }
 $result|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $out 'launch-result.json') -Encoding utf8
}

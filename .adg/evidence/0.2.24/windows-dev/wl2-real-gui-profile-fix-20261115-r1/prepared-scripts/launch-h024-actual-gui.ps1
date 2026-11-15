param([Parameter(Mandatory)][string]$Root)
$ErrorActionPreference='Stop'
$Root=[IO.Path]::GetFullPath($Root).TrimEnd('\')
if($Root -cne 'C:\Users\Public\ocx-h024-actual-gui-fixed-20261115-r1'){throw 'exact new actual GUI task root required'}
$out=Join-Path $Root 'out01'
if(Test-Path -LiteralPath (Join-Path $out 'launch-result.json')){throw 'immutable finished round; no same-condition repeat'}
$id=[Security.Principal.WindowsIdentity]::GetCurrent()
$admin=([Security.Principal.WindowsPrincipal]::new($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
$session=[Diagnostics.Process]::GetCurrentProcess().SessionId
$result=[ordered]@{sid=$id.User.Value;session=$session;elevated=$admin;scope='actual product GUI and empty broker continuity only; no official Owner or production lifecycle';allAccepted=$false}
try{
 if($id.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or $session -ne 7 -or $admin){throw 'original ordinary marenben interactive session7 required'}
 $intent=[IO.File]::Open((Join-Path $out 'launch-intent.json'),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try{
  $bytes=[Text.Encoding]::UTF8.GetBytes((@{sid=$id.User.Value;session=$session;started=[DateTime]::Now.ToString('o');accepted=$false}|ConvertTo-Json))
  $intent.Write($bytes,0,$bytes.Length);$intent.Flush()
 }finally{$intent.Dispose()}
 $manifest=Get-Content -LiteralPath (Join-Path $Root 'input-manifest.json') -Raw|ConvertFrom-Json
 foreach($row in $manifest.files){
  if($row.name -notin @('ocxd.exe','opencodex-desktop.exe','h-gui-tests.exe')){throw 'unknown prepared artifact'}
  $path=Join-Path (Join-Path $Root 'input') $row.name
  $item=Get-Item -LiteralPath $path
  if($item.Attributes -band [IO.FileAttributes]::ReparsePoint -or $item.Length -ne $row.bytes){throw 'exact bounded regular input required'}
  if((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $row.sha256){throw 'prepared input digest differs'}
 }
 if(@($manifest.files).Count -ne 3){throw 'three prepared artifacts required'}
 $env:OCXD024_H_INPUT=Join-Path $Root 'input'
 $env:OCXD024_H_EVIDENCE=$out
 $env:RUST_BACKTRACE='1'
 foreach($name in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY','OCXD024_ACTUAL_GUI_BASE','OCXD024_ACTUAL_GUI_MODE')){
  [Environment]::SetEnvironmentVariable($name,$null,'Process')
 }
 $psi=New-Object Diagnostics.ProcessStartInfo
 $psi.FileName=Join-Path $Root 'input/h-gui-tests.exe'
 $psi.Arguments='--exact infrastructure::windows_owner_host024::actual_gui_tests::isolated_actual_gui_observer --ignored --test-threads=1 --nocapture'
 $psi.UseShellExecute=$false;$psi.CreateNoWindow=$true;$psi.RedirectStandardOutput=$true;$psi.RedirectStandardError=$true
 $p=New-Object Diagnostics.Process;$p.StartInfo=$psi
 $stdout=[IO.File]::Open((Join-Path $out 'test.stdout.log'),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 $stderr=[IO.File]::Open((Join-Path $out 'test.stderr.log'),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try{
  if(!$p.Start()){throw 'observer creation failed'}
  $result.observerPid=$p.Id;$result.observerCreation=$p.StartTime.ToString('o')
  $copyOut=$p.StandardOutput.BaseStream.CopyToAsync($stdout);$copyErr=$p.StandardError.BaseStream.CopyToAsync($stderr)
  $p.WaitForExit();$copyOut.GetAwaiter().GetResult();$copyErr.GetAwaiter().GetResult()
  $result.rawTestExit=$p.ExitCode;$result.actualGUIComponentPassed=$p.ExitCode -eq 0
 }finally{$stdout.Dispose();$stderr.Dispose();$p.Dispose()}
}catch{
 $result.firstError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}
 $_|Out-String|Set-Content -LiteralPath (Join-Path $out 'launcher-error.log') -Encoding utf8
}finally{
 $result.taskBaseRetained=$true
 $result|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $out 'launch-result.json') -Encoding utf8
 # Read under the same ordinary SID, with compatible sharing even if a failed
 # broker still owns its log. Export every independent item; never stop at the
 # first file error or change the source's ACL/owner.
 & (Join-Path $Root 'collect-h024-actual-gui.ps1') -Root $Root
}

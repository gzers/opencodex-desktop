param([Parameter(Mandatory)][string]$Evidence)
$ErrorActionPreference='Stop'
$out=[IO.Path]::GetFullPath($Evidence)
if(Test-Path -LiteralPath $out){throw 'fresh probe evidence required'}
[void][IO.Directory]::CreateDirectory($out)
Add-Type -Path (Join-Path $PSScriptRoot 'r7-readonly-scene.cs')
$result=[ordered]@{passed=$false;scope='controller-owned query-only helper tests; not ordinary desktop acceptance';refused=@();originalR2Touched=$false;processTerminated=$false;removed=$false}
try {
 $image=Join-Path $out 'cmd.exe'
 [IO.File]::Copy((Join-Path $env:SystemRoot 'System32/cmd.exe'),$image,$false)
 $start=New-Object Diagnostics.ProcessStartInfo
 $start.FileName=$image;$start.Arguments='/d /q /c set /p r7probe='
 $start.UseShellExecute=$false;$start.CreateNoWindow=$true;$start.RedirectStandardInput=$true
 $process=New-Object Diagnostics.Process;$process.StartInfo=$start
 if(!$process.Start()){throw 'own query probe process creation failed'}
 $hash=(Get-FileHash -LiteralPath $image -Algorithm SHA256).Hash.ToLowerInvariant()
 $result.current=[R7ReadOnlyScene]::ProbeProcess($process.Id,$hash,$null)
 foreach($fault in @('hash','filetime')){
  $rejected=$false
  try{if($fault -eq 'hash'){[void][R7ReadOnlyScene]::ProbeProcess($process.Id,('0'*64),$null)}else{[void][R7ReadOnlyScene]::ProbeProcess($process.Id,$hash,'1')}}
  catch{$wanted=if($fault -eq 'hash'){'pinned file SHA256 differs'}else{'exact FILETIME/PID differs'};if(!$_.Exception.ToString().Contains($wanted)){throw};$rejected=$true;$result.refused+=@{fault=$fault;error=$_.Exception.ToString()}}
  if(!$rejected -or $process.HasExited){throw 'query mismatch failed to refuse without affecting current process'}
 }
 $fixture=Join-Path $out 'fixture';[void][IO.Directory]::CreateDirectory($fixture)
 $file=Join-Path $fixture 'content.txt';[IO.File]::WriteAllText($file,'original query-only bytes')
 $result.file=[R7ReadOnlyScene]::ObserveFile($file,$false)
 $result.directory=[R7ReadOnlyScene]::ObserveFile($fixture,$true)
 Set-Content -LiteralPath $file -Stream 'r7-test' -Value 'owned alternate stream'
 $rejected=$false
 try{[void][R7ReadOnlyScene]::ObserveFile($file,$false)}catch{if(!$_.Exception.ToString().Contains('alternate or unknown stream refused')){throw};$rejected=$true;$result.refused+=@{fault='ADS';error=$_.Exception.ToString()}}
 if(!$rejected){throw 'ADS was accepted'}
 # Delete only the newly created stream on this exact owned fixture.
 Remove-Item -LiteralPath $file -Stream 'r7-test'
 [void][R7ReadOnlyScene]::ObserveFile($file,$false)
 $hardlink=Join-Path $fixture 'hardlink.txt'
 [void](New-Item -ItemType HardLink -Path $hardlink -Target $file)
 $rejected=$false
 try{[void][R7ReadOnlyScene]::ObserveFile($file,$false)}catch{if(!$_.Exception.ToString().Contains('multiple-link task file refused')){throw};$rejected=$true;$result.refused+=@{fault='hardlink';error=$_.Exception.ToString()}}
 if(!$rejected){throw 'multiple links were accepted'}
 [IO.File]::Delete($hardlink)
 [void][R7ReadOnlyScene]::ObserveFile($file,$false)
 . (Join-Path $PSScriptRoot 'archive-r6-content.ps1')
 $archive=Invoke-R6ContentArchive -Roots @(@{source=$fixture;relative='owned-fixture'}) -ExtraFiles @() -ZipPath (Join-Path $out 'probe.zip')
 if(!$archive.archiveVerified){throw 'archive helper synthetic verification failed'}
 $result.archive=$archive
 $result.passed=$true
}catch{$result.firstError=$_.Exception.ToString();$_|Out-String|Set-Content -LiteralPath (Join-Path $out 'error.log') -Encoding utf8}
finally{
 if($null -ne $process){
  if(!$process.HasExited){$process.StandardInput.WriteLine('done');$process.StandardInput.Close()}
  $result.ownProbeExited=$process.WaitForExit(5000)
  if($result.ownProbeExited){$result.ownProbeRawExit=$process.ExitCode}
  $process.Dispose()
 }
 $result|ConvertTo-Json -Depth 15|Set-Content -LiteralPath (Join-Path $out 'result.json') -Encoding utf8
}
if(!$result.passed){exit 1}

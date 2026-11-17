param([Parameter(Mandatory)][string]$Root)
$ErrorActionPreference='Stop'
if([IO.Path]::GetFullPath($Root) -cne [IO.Path]::GetFullPath('C:/Users/Public/ocx-h024-repaired-20261115-r1')){throw 'exact repaired task root required'}
$id=[Security.Principal.WindowsIdentity]::GetCurrent()
if($id.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or [Diagnostics.Process]::GetCurrentProcess().SessionId -ne 7){throw 'original ordinary task SID/session required'}
$output=Join-Path $Root 'out01';$receipt=Join-Path $output 'final-same-sid-export.json'
if(Test-Path -LiteralPath $receipt){throw 'immutable task export already exists'}
$result=[ordered]@{sid=$id.User.Value;session=7;scope='exact synthetic task originals; no daily credentials/config';files=@();fileErrors=@();complete=$false;accepted=$false}
function Add-Snapshot([string]$Path,[string]$Relative){
 try{
  $item=Get-Item -LiteralPath $Path -Force
  if($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $item.Length -gt 1048576){throw 'bounded regular original only'}
  $f=[IO.File]::Open($item.FullName,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
  try{$length=$f.Length;$bytes=New-Object byte[] ([int]$length);$done=0;while($done -lt $bytes.Length){$n=$f.Read($bytes,$done,$bytes.Length-$done);if($n -le 0){throw 'truncated snapshot'};$done+=$n};$stable=$f.Length -eq $length}finally{$f.Dispose()}
  $sha=[Security.Cryptography.SHA256]::Create();try{$digest=([BitConverter]::ToString($sha.ComputeHash($bytes))).Replace('-','').ToLowerInvariant()}finally{$sha.Dispose()}
  $result.files+=@{relative=$Relative;bytes=$bytes.Length;sha256=$digest;base64=[Convert]::ToBase64String($bytes);stableLength=$stable}
 }catch{$result.fileErrors+=@{relative=$Relative;message=$_.Exception.ToString();line=$_.InvocationInfo.ScriptLineNumber}}
}
try{
 foreach($name in @('bootstrap-admission.json','suspended-original.json','task-location.json','after-normal.json','after-hold.json','after-after-failure.json','archive-manifest.json','result.json')){
  if(Test-Path -LiteralPath (Join-Path $output $name)){Add-Snapshot (Join-Path $output $name) ('public/'+$name)}
 }
 if(Test-Path -LiteralPath (Join-Path $output 'task-location.json')){
  $location=Get-Content -LiteralPath (Join-Path $output 'task-location.json') -Raw|ConvertFrom-Json
  $base=[IO.Path]::GetFullPath($location.base)
  if([IO.Path]::GetDirectoryName($base) -cne $env:USERPROFILE -or [IO.Path]::GetFileName($base) -notmatch '^ocx024-h-[0-9a-f-]{36}$'){throw 'exact task-owned base required'}
  $result.base=$base
  foreach($f in Get-ChildItem -LiteralPath (Join-Path $base 'D') -File -Force){Add-Snapshot $f.FullName ('D/'+$f.Name)}
 }
 $result.complete=$result.fileErrors.Count -eq 0
}catch{$result.error=@{message=$_.Exception.ToString();line=$_.InvocationInfo.ScriptLineNumber}}
finally{
 $bytes=[Text.Encoding]::UTF8.GetBytes(($result|ConvertTo-Json -Depth 10))
 $file=[IO.File]::Open($receipt,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try{$file.Write($bytes,0,$bytes.Length);$file.Flush()}finally{$file.Dispose()}
}

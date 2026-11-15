param([Parameter(Mandatory)][string]$Root)
$ErrorActionPreference='Stop'
$Root=[IO.Path]::GetFullPath($Root).TrimEnd('\')
if($Root -cne 'C:\Users\Public\ocx-h024-actual-gui-fixed-20261115-r1'){throw 'exact new actual GUI task root required'}
$id=[Security.Principal.WindowsIdentity]::GetCurrent()
if($id.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or [Diagnostics.Process]::GetCurrentProcess().SessionId -ne 7){throw 'original ordinary desktop SID/session required'}
$out=Join-Path $Root 'out01'
$target=Join-Path $out 'same-sid-export01.json'
if(Test-Path -LiteralPath $target){throw 'immutable export already exists'}
$result=[ordered]@{sid=$id.User.Value;session=7;files=@();fileErrors=@();complete=$false;allAccepted=$false;taskSceneRetained=$true;scope='bounded own task evidence only; no GUI rerun, kill, recovery, ACL or Profile changes'}
function Add-TaskSnapshot([string]$Path,[string]$Relative){
 $item=Get-Item -LiteralPath $Path -Force
 if($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $item.Length -gt 1048576){throw 'bounded regular own task log required'}
 $stream=[IO.File]::Open($item.FullName,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
 try{
  $length=$stream.Length;if($length -gt 1048576){throw 'snapshot size bound'}
  $bytes=New-Object byte[] ([int]$length);$done=0
  while($done -lt $bytes.Length){$n=$stream.Read($bytes,$done,$bytes.Length-$done);if($n -le 0){throw 'truncated snapshot'};$done+=$n}
  $stable=$stream.Length -eq $length
 }finally{$stream.Dispose()}
 $sha=[Security.Cryptography.SHA256]::Create()
 try{$digest=([BitConverter]::ToString($sha.ComputeHash($bytes))).Replace('-','').ToLowerInvariant()}finally{$sha.Dispose()}
 $result.files+=@{relative=$Relative;bytes=$bytes.Length;sha256=$digest;base64=[Convert]::ToBase64String($bytes);stableLength=$stable}
}
foreach($file in Get-ChildItem -LiteralPath $out -File -Force){
 try{Add-TaskSnapshot $file.FullName ('out/'+$file.Name)}catch{$result.fileErrors+=@{relative=('out/'+$file.Name);message=$_.Exception.Message}}
}
try{
 $location=Get-Content -LiteralPath (Join-Path $out 'task-location.json') -Raw|ConvertFrom-Json
 $base=[IO.Path]::GetFullPath($location.base)
 if([IO.Path]::GetDirectoryName($base) -cne $env:USERPROFILE -or [IO.Path]::GetFileName($base) -notmatch '^ocx024-h-[0-9a-f-]{36}$'){throw 'exact original task base differs'}
 $result.base=$base
 foreach($file in Get-ChildItem -LiteralPath (Join-Path $base 'D') -File -Force){
  try{Add-TaskSnapshot $file.FullName ('D/'+$file.Name)}catch{$result.fileErrors+=@{relative=('D/'+$file.Name);message=$_.Exception.Message}}
 }
}catch{$result.fileErrors+=@{relative='task-base';message=$_.Exception.Message}}
$result.complete=$result.fileErrors.Count -eq 0
$bytes=[Text.Encoding]::UTF8.GetBytes(($result|ConvertTo-Json -Depth 10))
# This output inherits only the pre-created task output ACL. All private task
# originals remain private and unchanged; no credentials or GUI caches exported.
$file=[IO.File]::Open($target,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
try{$file.Write($bytes,0,$bytes.Length);$file.Flush()}finally{$file.Dispose()}

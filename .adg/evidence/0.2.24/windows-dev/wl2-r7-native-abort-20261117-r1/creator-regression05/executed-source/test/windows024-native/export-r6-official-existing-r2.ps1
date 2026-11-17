param([Parameter(Mandatory)][string]$Root)
$ErrorActionPreference='Stop'
$rootPath=[IO.Path]::GetFullPath($Root)
if($rootPath -cne 'C:\Users\Public\ocx-r6-official-existing-20261117-r2'){throw 'exact original qualification export root required'}
$id=[Security.Principal.WindowsIdentity]::GetCurrent()
if($id.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or [Diagnostics.Process]::GetCurrentProcess().SessionId -ne 7){throw 'original task SID/session required'}
$base='C:\Users\marenben\ocx024-r6-existing-20261117-r2'
$out=Join-Path $rootPath 'out01';$receipt=Join-Path $out 'same-sid-export.json';$exportRoot=Join-Path $out 'raw-export'
if(Test-Path -LiteralPath $receipt){throw 'immutable same-SID collection exists; no retry'}
if(Test-Path -LiteralPath $exportRoot){throw 'partial export preserved; use a new explicitly recorded snapshot round'}
[void][IO.Directory]::CreateDirectory($exportRoot)
$result=[ordered]@{sid=$id.User.Value;session=7;files=@();fileErrors=@();directories=@();complete=$false;scope='exact task D and qualification-preparation records only; no daily or official source changes';startOpen=$false}
$total=0L
function Snapshot([string]$Path,[string]$Relative) {
 try {
  $item=Get-Item -LiteralPath $Path -Force
  if($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $item.Length -gt 4194304){throw 'bounded regular task record required'}
  $stream=[IO.File]::Open($item.FullName,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
  try{$length=$stream.Length;$bytes=New-Object byte[] ([int]$length);$done=0;while($done -lt $bytes.Length){$n=$stream.Read($bytes,$done,$bytes.Length-$done);if($n -le 0){throw 'truncated raw task snapshot'};$done+=$n};if($length -ne $stream.Length){throw 'task file size changed during snapshot'}}finally{$stream.Dispose()}
  $alg=[Security.Cryptography.SHA256]::Create();try{$hash=([BitConverter]::ToString($alg.ComputeHash($bytes))).Replace('-','').ToLowerInvariant()}finally{$alg.Dispose()}
  $target=[IO.Path]::GetFullPath((Join-Path $exportRoot $Relative))
  if(!$target.StartsWith($exportRoot+[char]92,[StringComparison]::OrdinalIgnoreCase)){throw 'export relative escape'}
  [void][IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($target))
  $dest=[IO.File]::Open($target,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
  try{$dest.Write($bytes,0,$bytes.Length);$dest.Flush()}finally{$dest.Dispose()}
  $result.files+=@{source=$item.FullName;relative=$Relative;bytes=$bytes.Length;sha256=$hash;rawCopied=$true}
 }catch{$result.fileErrors+=@{relative=$Relative;message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
}
try {
 foreach($name in @('D','qualification-preparation','preparation')) {
  $source=Join-Path $base $name
  if(!(Test-Path -LiteralPath $source)){continue}
  $entries=@(Get-Item -LiteralPath $source)+@(Get-ChildItem -LiteralPath $source -Recurse -Force)
  if($entries.Count -gt 20000){throw 'bounded complete task record membership exceeded'}
  foreach($item in $entries){
   if($item.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'task record reparse refused'}
   if($item.PSIsContainer){$result.directories+=@{source=$item.FullName};continue}
   $total+=$item.Length;if($total -gt 268435456){throw 'bounded complete task evidence size exceeded'}
   Snapshot $item.FullName ($name+'/'+$item.FullName.Substring($source.Length+1).Replace([char]92,[char]47))
  }
 }
 $result.complete=$result.fileErrors.Count -eq 0
}catch{$result.firstError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
finally{$result.totalSourceBytes=$total;$result|ConvertTo-Json -Depth 12|Set-Content -LiteralPath $receipt -Encoding utf8}

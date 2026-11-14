param([Parameter(Mandatory)][string]$Root)
$ErrorActionPreference='Stop'
$expectedRoot=[IO.Path]::GetFullPath('C:/Users/Public/ocx-h024-desktop-20261114-r1')
if([IO.Path]::GetFullPath($Root) -cne $expectedRoot){throw 'exact prepared H evidence root required'}
$id=[Security.Principal.WindowsIdentity]::GetCurrent()
if($id.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or [Diagnostics.Process]::GetCurrentProcess().SessionId -ne 7){throw 'original marenben session7 required; no impersonation/elevation'}
$output=Join-Path $Root 'out01'
$receipt=Join-Path $output 'same-sid-export02.json'
if(Test-Path -LiteralPath $receipt){throw 'immutable original export already exists; no repeat'}
$result=[ordered]@{fileErrors=@();sid=$id.User.Value;session=7;scope='read only exact current H failure scene; no restart/recovery/kill/ACL/profile changes';accepted=$false;complete=$false;files=@()}
function Add-Original([string]$Path,[string]$Relative){
 $item=Get-Item -LiteralPath $Path -Force
 if($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $item.Length -gt 1048576){throw 'bounded regular task evidence only'}
 $stream=[IO.File]::Open($item.FullName,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
 try{
  $length=$stream.Length;if($length -gt 1048576){throw 'snapshot bound'}
  $bytes=New-Object byte[] ([int]$length);$done=0
  while($done -lt $bytes.Length){$n=$stream.Read($bytes,$done,$bytes.Length-$done);if($n -le 0){throw 'truncated read snapshot'};$done+=$n}
  $stableLength=$stream.Length -eq $length
 }finally{$stream.Dispose()}
 $sha=[Security.Cryptography.SHA256]::Create()
 try{$digest=([BitConverter]::ToString($sha.ComputeHash($bytes))).Replace('-','').ToLowerInvariant()}finally{$sha.Dispose()}
 $result.files+=@{relative=$Relative;bytes=$bytes.Length;sha256=$digest;base64=[Convert]::ToBase64String($bytes);stableLength=$stableLength}
}
try{
 $location=Get-Content -LiteralPath (Join-Path $output 'task-location.json') -Raw|ConvertFrom-Json
 $base=[IO.Path]::GetFullPath($location.base)
 if([IO.Path]::GetDirectoryName($base) -cne $env:USERPROFILE -or [IO.Path]::GetFileName($base) -notmatch '^ocx024-h-[0-9a-f-]{36}$'){throw 'original task-owned base differs'}
 $result.base=$base
 foreach($name in @('bootstrap-admission.json','suspended-original.json','task-location.json')){Add-Original (Join-Path $output $name) ('public/'+$name)}
 $data=Join-Path $base 'D'
 foreach($f in Get-ChildItem -LiteralPath $data -File -Force){try{Add-Original $f.FullName ('D/'+$f.Name)}catch{$result.fileErrors+=@{relative=('D/'+$f.Name);message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}}
 $result.processes=@(Get-CimInstance Win32_Process -Filter "Name='ocxd.exe' OR Name='opencodex-desktop.exe'"|Where-Object {$_.SessionId -eq 7}|Select-Object ProcessId,ParentProcessId,CreationDate,ExecutablePath,CommandLine)
 $result.complete=$result.fileErrors.Count -eq 0
}catch{$result.error=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
finally{
 # Intentionally use the already prepared output's inherited controller-readable
 # ACL for nonsecret synthetic task evidence; never change the private originals.
 $json=$result|ConvertTo-Json -Depth 10
 $bytes=[Text.Encoding]::UTF8.GetBytes($json)
 $file=[IO.File]::Open($receipt,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try{$file.Write($bytes,0,$bytes.Length);$file.Flush()}finally{$file.Dispose()}
}

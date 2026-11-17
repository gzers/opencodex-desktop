param([Parameter(Mandatory)][string]$Root)
$ErrorActionPreference='Stop'
$rootPath=[IO.Path]::GetFullPath($Root)
if($rootPath -cne 'C:\Users\Public\ocx-r6-profile-preflight-20261117-r1'){throw 'exact preflight output root required'}
$id=[Security.Principal.WindowsIdentity]::GetCurrent()
$admin=([Security.Principal.WindowsPrincipal]::new($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if($id.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or [Diagnostics.Process]::GetCurrentProcess().SessionId -ne 7 -or $admin){throw 'original ordinary marenben session7 required'}
$out=Join-Path $rootPath 'out01'
$receipt=Join-Path $out 'same-sid-export.json'
if(Test-Path -LiteralPath $receipt){throw 'immutable export exists; no retry'}
$result=[ordered]@{sid=$id.User.Value;session=7;files=@();fileErrors=@();complete=$false;actualProductionExecuted=$false;startOpen=$false;scope='only exact current credential-free preparation facts and task ledger; no client data or daily files exported'}
function Snapshot([string]$Path,[string]$Relative) {
 try {
  $item=Get-Item -LiteralPath $Path -Force
  if($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $item.Length -gt 1048576){throw 'bounded regular preparation file required'}
  $stream=[IO.File]::Open($item.FullName,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
  try {
   $length=$stream.Length;$bytes=New-Object byte[] ([int]$length);$done=0
   while($done -lt $bytes.Length){$n=$stream.Read($bytes,$done,$bytes.Length-$done);if($n -le 0){throw 'truncated task snapshot'};$done+=$n}
   if($stream.Length -ne $length){throw 'task file changed during snapshot'}
  }finally{$stream.Dispose()}
  $alg=[Security.Cryptography.SHA256]::Create()
  try{$digest=([BitConverter]::ToString($alg.ComputeHash($bytes))).Replace('-','').ToLowerInvariant()}finally{$alg.Dispose()}
  $result.files+=@{relative=$Relative;bytes=$bytes.Length;sha256=$digest;base64=[Convert]::ToBase64String($bytes)}
 }catch{$result.fileErrors+=@{relative=$Relative;message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
}
try {
 foreach($name in @('prerequisites.json','preparation-result.json','native-failure.json')){
  $path=Join-Path $out $name
  if(Test-Path -LiteralPath $path){Snapshot $path ('public/'+$name)}
 }
 $base='C:\Users\marenben\ocx024-r6-existing-20261117-r1'
 $ledger=Join-Path $base 'preparation'
 if(Test-Path -LiteralPath $ledger){
  if((Get-Item -LiteralPath $ledger).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'task ledger reparse refused'}
  foreach($file in Get-ChildItem -LiteralPath $ledger -Force -File){
   if($file.Name -cnotmatch '^(initial-absence|result|prepare-[0-9]{3}-(intent|result))\.json$'){throw 'unknown ledger file refused'}
   Snapshot $file.FullName ('preparation/'+$file.Name)
  }
 }
 $result.complete=$result.fileErrors.Count -eq 0
}catch{$result.firstError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
finally {
 $bytes=[Text.Encoding]::UTF8.GetBytes(($result|ConvertTo-Json -Depth 10))
 $file=[IO.File]::Open($receipt,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try{$file.Write($bytes,0,$bytes.Length);$file.Flush()}finally{$file.Dispose()}
}

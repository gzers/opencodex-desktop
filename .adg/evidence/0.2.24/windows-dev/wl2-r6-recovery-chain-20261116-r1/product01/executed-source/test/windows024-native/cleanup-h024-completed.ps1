$ErrorActionPreference='Stop'
$root=$PSScriptRoot
if([IO.Path]::GetFullPath($root) -cne [IO.Path]::GetFullPath('C:/Users/Public/ocx-h024-repaired-20261115-r1')){throw 'exact completed task root required'}
$sid='S-1-5-21-587408064-856152790-2583328096-1003'
$id=[Security.Principal.WindowsIdentity]::GetCurrent()
if($id.User.Value -cne $sid -or [Diagnostics.Process]::GetCurrentProcess().SessionId -ne 7){throw 'original ordinary task SID/session required'}
$output=Join-Path $root 'out01';$receipt=Join-Path $output 'completed-task-cleanup.json'
if(Test-Path -LiteralPath $receipt){throw 'immutable cleanup already attempted; no automatic retry'}
$result=[ordered]@{scope='only two completed test-owned trees, first failed scene retained';sid=$sid;accepted=$false;forcedProcessCleanup=$false;accountOrProfileDeleted=$false;originalFailedBaseRetained=$true;oldRejectedPathsRetried=$false;targets=@();complete=$false}
function Digest([byte[]]$Bytes){$h=[Security.Cryptography.SHA256]::Create();try{return ([BitConverter]::ToString($h.ComputeHash($Bytes))).Replace('-','').ToLowerInvariant()}finally{$h.Dispose()}}
function Read-Shared([string]$Path){$f=[IO.File]::Open($Path,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete));try{$b=New-Object byte[] ([int]$f.Length);$done=0;while($done -lt $b.Length){$n=$f.Read($b,$done,$b.Length-$done);if($n -le 0){throw 'truncated current task file'};$done+=$n};return ,$b}finally{$f.Dispose()}}
try{
 $plan=Get-Content -LiteralPath (Join-Path $root 'cleanup-plan.json') -Raw|ConvertFrom-Json
 foreach($item in $plan.receipts){if((Digest (Read-Shared (Join-Path $output $item.name))) -cne $item.sha256){throw 'archived completion receipt changed; no deletion'}}
 $launch=Get-Content -LiteralPath (Join-Path $output 'launch-result.json') -Raw|ConvertFrom-Json
 $old=Get-Content -LiteralPath (Join-Path $output 'original-empty-host-recovery.json') -Raw|ConvertFrom-Json
 $export=Get-Content -LiteralPath (Join-Path $output 'final-same-sid-export.json') -Raw|ConvertFrom-Json
 if(!$launch.hComponentPassed -or $launch.rawTestExit -ne 0 -or !$old.nativeWaitConfirmed -or $old.originalHostExitCode -ne 0 -or !$export.complete){throw 'original host and completed H outcome not confirmed'}
 $allowed=@('ocx024-h-128429f2-cfe8-4e01-b585-3f015675da13','ocx024-h-b9899c33-3aea-4920-9b3f-ab16d66c0b70')
 if($plan.targets.Count -ne 2){throw 'exact two completed targets required'}
 # Verify every independent target before deleting either. Unknown files,
 # directories, changed bytes, reparse points or incomplete archive refuse all.
 foreach($target in $plan.targets){
  if($target.leaf -cnotin $allowed){throw 'not an exact completed tree'}
  $path=[IO.Path]::GetFullPath((Join-Path $env:USERPROFILE $target.leaf))
  if([IO.Path]::GetDirectoryName($path) -cne $env:USERPROFILE -or [IO.Path]::GetFileName($path) -ceq 'ocx024-h-13ae0590-60a3-4d5d-accc-bb6329181394'){throw 'resolved target outside exact completed Profile subtree'}
  $resolved=(Resolve-Path -LiteralPath $path).ProviderPath
  if($resolved -cne $path -or ((Get-Item -LiteralPath $resolved -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'resolved cleanup path or type differs'}
  $expected=@{};foreach($f in $target.files){if($expected.ContainsKey($f.relative)){throw 'duplicate expected payload'};$expected[$f.relative]=$f}
  $entries=@(Get-ChildItem -LiteralPath $resolved -Recurse -Force)
  $files=@();$bytes=0L
  foreach($entry in $entries){
   if($entry.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'reparse completed tree refused'}
   $relative=$entry.FullName.Substring($resolved.Length+1).Replace([char]92,[char]47)
   if($entry.PSIsContainer){if($relative -cnotin @($target.directories)){throw 'unknown completed directory'};continue}
   if(!$expected.ContainsKey($relative) -or $entry.Length -ne $expected[$relative].bytes){throw 'unknown or changed task payload'}
   if((Get-FileHash -LiteralPath $entry.FullName -Algorithm SHA256).Hash.ToLowerInvariant() -cne $expected[$relative].sha256){throw 'task bytes changed after archive'}
   $bytes+=$entry.Length;$files+=@{relative=$relative;bytes=$entry.Length;sha256=$expected[$relative].sha256}
  }
  if($files.Count -ne $expected.Count){throw 'expected completed payload missing'}
  $result.targets+=@{path=$resolved;archiveVerified=$true;fileCount=$files.Count;bytes=$bytes;files=$files;removed=$false}
 }
 foreach($target in $result.targets){
  # Native PowerShell only; exact absolute targets were checked above. No old
  # rejected deletion, owner/ACL repair, alternate API or process kill fallback.
  Remove-Item -LiteralPath $target.path -Recurse -Force -ErrorAction Stop
  $target.removed=!(Test-Path -LiteralPath $target.path)
 }
 $result.complete=@($result.targets|Where-Object {!$_.removed}).Count -eq 0
}catch{$result.error=@{message=$_.Exception.ToString();line=$_.InvocationInfo.ScriptLineNumber}}
finally{$result|ConvertTo-Json -Depth 10|Set-Content -LiteralPath $receipt -Encoding utf8}

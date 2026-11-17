$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath($PSScriptRoot).TrimEnd('\')
if($root -cne 'C:\Users\Public\ocx-h024-gui-archive-20261115-r1'){throw 'exact new cleanup entry required'}
$sid='S-1-5-21-587408064-856152790-2583328096-1003'
$id=[Security.Principal.WindowsIdentity]::GetCurrent()
$principal=[Security.Principal.WindowsPrincipal]::new($id)
if($id.User.Value -cne $sid -or [Diagnostics.Process]::GetCurrentProcess().SessionId -ne 7 -or $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'original ordinary marenben desktop session7 required'}
$out=Join-Path $root 'out01'
$receipt=Join-Path $out 'archive-cleanup-result.json'
$intent=Join-Path $out 'archive-cleanup-intent.json'
if((Test-Path -LiteralPath $intent) -or (Test-Path -LiteralPath $receipt)){throw 'one cleanup attempt only; preserve previous refusal without retry'}
$result=[ordered]@{scope='archive successful actual GUI task only; remove exact successful Profile child after full verification';sid=$sid;session=7;accepted=$false;complete=$false;deletionAttempted=$false;forcedProcessCleanup=$false;oldRejectedPathsRetried=$false;originalFailureRootChanged=$false;accountOrProfileDeleted=$false;dailyPolicyOrPathChanged=$false;nativeMetadataCloned=$false;targets=@();archiveDirectory=$null;removedSuccessfulProfileChild=$false;publicSourceRetained=$true;brokerRawExit=$null}
function Write-NewJson([string]$Path,$Value){
 $bytes=[Text.Encoding]::UTF8.GetBytes(($Value|ConvertTo-Json -Depth 16))
 $file=[IO.File]::Open($Path,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try{$file.Write($bytes,0,$bytes.Length);$file.Flush($true)}finally{$file.Dispose()}
}
function Digest-Stream([IO.Stream]$Stream){
 $hash=[Security.Cryptography.SHA256]::Create()
 try{return ([BitConverter]::ToString($hash.ComputeHash($Stream))).Replace('-','').ToLowerInvariant()}finally{$hash.Dispose()}
}
function Digest-File([string]$Path){
 $file=[IO.File]::Open($Path,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::Read)
 try{return Digest-Stream $file}finally{$file.Dispose()}
}
function Check-RegularDirectory([string]$Path){
 $resolved=(Resolve-Path -LiteralPath $Path -ErrorAction Stop).ProviderPath
 $item=Get-Item -LiteralPath $Path -Force
 if($resolved -cne $Path -or !$item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'exact nonreparse directory required'}
}
function Inventory([string]$Path){
 Check-RegularDirectory $Path
 $pending=New-Object 'Collections.Generic.Queue[string]'
 $pending.Enqueue($Path)
 $entries=New-Object 'Collections.Generic.List[object]'
 $total=0L
 while($pending.Count -gt 0){
  $parent=$pending.Dequeue();Check-RegularDirectory $parent
  foreach($entry in Get-ChildItem -LiteralPath $parent -Force -ErrorAction Stop){
   if(($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -or !$entry.FullName.StartsWith($Path+'\',[StringComparison]::Ordinal)){throw 'reparse or escaped task entry refused'}
   $relative=$entry.FullName.Substring($Path.Length+1).Replace([char]92,[char]47)
   if($entry.PSIsContainer){
    $pending.Enqueue($entry.FullName)
    $entries.Add(@{relative=$relative;directory=$true;bytes=0L;sha256=$null})
   }else{
    if($entry.Length -gt 268435456){throw 'task file exceeds256MiB archive bound'}
    $total+=$entry.Length;if($total -gt 1073741824){throw 'task bytes exceed1GiB archive bound'}
    $entries.Add(@{relative=$relative;directory=$false;bytes=$entry.Length;sha256=(Digest-File $entry.FullName)})
   }
   if($entries.Count -gt 50000){throw 'task entry count exceeds archive bound'}
  }
 }
 return @($entries.ToArray()|Sort-Object { $_.relative })
}
function Same-Inventory($Before,$After){
 if(@($Before).Count -ne @($After).Count){throw 'task directory/file set changed'}
 for($n=0;$n -lt @($Before).Count;$n++){
  $a=$Before[$n];$b=$After[$n]
  if($a.relative -cne $b.relative -or $a.directory -ne $b.directory -or $a.bytes -ne $b.bytes -or $a.sha256 -cne $b.sha256){throw 'task entries changed after archive'}
 }
}
function Archive-Tree([string]$Path,[string]$ZipPath,$Rows){
 $file=[IO.File]::Open($ZipPath,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 $zip=$null
 try{
  $zip=[IO.Compression.ZipArchive]::new($file,[IO.Compression.ZipArchiveMode]::Create,$true)
  foreach($row in $Rows){
   $name=$row.relative;if($row.directory){$name+='/' }
   $entry=$zip.CreateEntry($name,[IO.Compression.CompressionLevel]::Optimal)
   if(!$row.directory){
    $input=[IO.File]::Open((Join-Path $Path $row.relative),[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::Read)
    $output=$null
    try{
     if($input.Length -ne $row.bytes){throw 'task file size changed before archive'}
     $output=$entry.Open();$input.CopyTo($output)
    }finally{if($output){$output.Dispose()};$input.Dispose()}
   }
  }
 }finally{if($zip){$zip.Dispose()};$file.Flush($true);$file.Dispose()}
 $file=[IO.File]::OpenRead($ZipPath);$zip=$null
 try{
  $zip=[IO.Compression.ZipArchive]::new($file,[IO.Compression.ZipArchiveMode]::Read,$true)
  if($zip.Entries.Count -ne @($Rows).Count){throw 'archive entry set differs'}
  $expected=@{};foreach($row in $Rows){$name=$row.relative;if($row.directory){$name+='/'};$expected[$name]=$row}
  $seen=@{}
  foreach($entry in $zip.Entries){
   if(!$expected.ContainsKey($entry.FullName) -or $seen.ContainsKey($entry.FullName)){throw 'unknown or duplicate archive entry'}
   $seen[$entry.FullName]=$true;$row=$expected[$entry.FullName]
   if($entry.Length -ne $row.bytes){throw 'archive uncompressed size differs'}
   if(!$row.directory){$stream=$entry.Open();try{if((Digest-Stream $stream) -cne $row.sha256){throw 'archive bytes differ'}}finally{$stream.Dispose()}}
  }
 }finally{if($zip){$zip.Dispose()};$file.Dispose()}
 Same-Inventory $Rows (Inventory $Path)
 return @{path=$Path;archive=$ZipPath;archiveSha256=(Digest-File $ZipPath);files=@($Rows|Where-Object {!$_.directory}).Count;directories=@($Rows|Where-Object {$_.directory}).Count;bytes=($Rows|ForEach-Object {[long]$_.bytes}|Measure-Object -Sum).Sum;verified=$true;sourceRetained=$true}
}
try{
 Write-NewJson $intent @{sid=$sid;session=7;started=[DateTime]::Now.ToString('o');accepted=$false;oneAttempt=$true}
 Add-Type -AssemblyName System.IO.Compression
 Add-Type -AssemblyName System.IO.Compression.FileSystem
 $planPath=Join-Path $root 'archive-plan.json'
 $plan=Get-Content -LiteralPath $planPath -Raw|ConvertFrom-Json
 if($plan.schema -ne 1 -or $plan.sid -cne $sid -or $plan.session -ne 7 -or $plan.successfulBase -cne 'C:\Users\marenben\ocx024-h-1fa5e23f-12c9-4289-84f3-bcc901642b35' -or $plan.publicRoot -cne 'C:\Users\Public\ocx-h024-actual-gui-fixed-20261115-r1'){throw 'exact successful task plan required'}
 # USERPROFILE only constrains this already fixed exact target. A spoofed value
 # refuses; it never chooses another cleanup target or grants token authority.
 $profile=[IO.Path]::GetFullPath($env:USERPROFILE).TrimEnd('\')
 if($profile -cne 'C:\Users\marenben'){throw 'original Profile differs'}
 Check-RegularDirectory $profile
 $base=[IO.Path]::GetFullPath($plan.successfulBase)
 if([IO.Path]::GetDirectoryName($base) -cne $profile -or [IO.Path]::GetFileName($base) -cne 'ocx024-h-1fa5e23f-12c9-4289-84f3-bcc901642b35'){throw 'cleanup escaped exact successful Profile child'}
 Check-RegularDirectory $base
 Check-RegularDirectory $plan.publicRoot
 foreach($row in $plan.receipts){
  if($row.relative -notmatch '^(out01/)?[a-z0-9.-]+$'){throw 'unsafe expected receipt name'}
  $path=Join-Path $plan.publicRoot $row.relative
  if((Digest-File $path) -cne $row.sha256){throw 'archived successful receipt changed'}
 }
 $export=Get-Content -LiteralPath (Join-Path $plan.publicRoot 'out01/same-sid-export01.json') -Raw|ConvertFrom-Json
 $launch=Get-Content -LiteralPath (Join-Path $plan.publicRoot 'out01/launch-result.json') -Raw|ConvertFrom-Json
 $completed=Get-Content -LiteralPath (Join-Path $plan.publicRoot 'out01/result.json') -Raw|ConvertFrom-Json
 if(!$export.complete -or @($export.fileErrors).Count -ne 0 -or $export.base -cne $base -or $launch.rawTestExit -ne 0 -or !$launch.actualGUIComponentPassed -or !$completed.actualGUI -or $completed.fixtureManager -or $completed.officialOwnerCreated -or $completed.lifecycleAccepted -or !$completed.originalCreationHandleRetained -or !$completed.brokerExitedExplicitly -or $null -ne $completed.brokerRawExit){throw 'exact successful raw/component/exit observation receipts required'}
 if(($completed.managerGenerations -join ',') -cne '1,2,3,4' -or $completed.originalBroker.pid -ne 31484 -or $completed.originalBroker.creationFiletime -cne '134392264041593579'){throw 'original successful broker or generation differs'}
 $archive=Join-Path $profile ('ocx024-gui-archive-'+[guid]::NewGuid().ToString())
 if(Test-Path -LiteralPath $archive){throw 'new private archive directory already exists'}
 $acl=New-Object Security.AccessControl.DirectorySecurity
 $acl.SetOwner([Security.Principal.SecurityIdentifier]::new($sid));$acl.SetAccessRuleProtection($true,$false)
 foreach($who in @($sid,'S-1-5-18')){$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($who),[Security.AccessControl.FileSystemRights]::FullControl,([Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [Security.AccessControl.InheritanceFlags]::ObjectInherit),[Security.AccessControl.PropagationFlags]::None,[Security.AccessControl.AccessControlType]::Allow))}
 [void][IO.Directory]::CreateDirectory($archive,$acl)
 $result.archiveDirectory=$archive
 $baseRows=@(Inventory $base);$publicRows=@(Inventory $plan.publicRoot)
 $baseArchive=Archive-Tree $base (Join-Path $archive 'successful-profile-tree.zip') $baseRows
 $publicArchive=Archive-Tree $plan.publicRoot (Join-Path $archive 'successful-public-inputs-logs.zip') $publicRows
 Write-NewJson (Join-Path $archive 'private-archive-manifest.json') @{scope='successful task bytes only; not native identity or security clone';profile=@{path=$base;entries=$baseRows;archive=$baseArchive};public=@{path=$plan.publicRoot;entries=$publicRows;archive=$publicArchive};accepted=$false}
 $result.targets=@($baseArchive,$publicArchive)
 $result.privateManifestSha256=Digest-File (Join-Path $archive 'private-archive-manifest.json')
 # All independent source/ZIP checks complete before any deletion. Recheck the
 # resolved absolute target and complete entry set immediately before removal.
 Same-Inventory $baseRows (Inventory $base)
 Same-Inventory $publicRows (Inventory $plan.publicRoot)
 Check-RegularDirectory $base
 if($base -cne 'C:\Users\marenben\ocx024-h-1fa5e23f-12c9-4289-84f3-bcc901642b35' -or [IO.Path]::GetDirectoryName($base) -cne 'C:\Users\marenben'){throw 'final exact deletion target differs'}
 $result.deletionAttempted=$true
 Remove-Item -LiteralPath $base -Recurse -Force -ErrorAction Stop
 $result.removedSuccessfulProfileChild=!(Test-Path -LiteralPath $base)
 $result.targets[0].sourceRetained=!$result.removedSuccessfulProfileChild
 $result.complete=$result.removedSuccessfulProfileChild
}catch{
 $result.error=@{message=$_.Exception.ToString();line=$_.InvocationInfo.ScriptLineNumber}
}finally{
 Write-NewJson $receipt $result
}
if(!$result.complete){throw 'archive/cleanup incomplete; preserve receipt and do not retry same conditions'}

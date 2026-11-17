param([Parameter(Mandatory)][string]$Root)
$ErrorActionPreference='Stop'
$rootPath=[IO.Path]::GetFullPath($Root)
if($rootPath -cne 'C:\Users\Public\ocx-r6-failure-archive-20261117-r2'){throw 'exact new archive output root required'}
$id=[Security.Principal.WindowsIdentity]::GetCurrent()
$admin=([Security.Principal.WindowsPrincipal]::new($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if($id.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or [Diagnostics.Process]::GetCurrentProcess().SessionId -ne 7 -or $admin){throw 'original ordinary task SID/session required'}
$out=Join-Path $rootPath 'out01';$receipt=Join-Path $out 'archive-result.json'
if(Test-Path -LiteralPath $receipt){throw 'immutable archive observation already recorded; no retry'}
$base='C:\Users\marenben\ocx024-r6-existing-20261117-r1'
$profile='C:\Users\marenben'
$archiveRoot=Join-Path $profile 'ocx024-r6-private-failure-archive-20261117-r2'
$zipPath=Join-Path $archiveRoot 'failed-existing-root-original-bytes.zip'
$result=[ordered]@{sid=$id.User.Value;session=7;scope='read-only complete first failed task+exact synthetic clients+three test coordination files; content ZIP is not NTFS security/identity clone';files=@();directories=@();fileErrors=@();archiveVerified=$false;nativeNormalStopProven=$false;forcedCleanupExecuted=$false;nativeOwnerAdopted=$false;originalObjectsRetained=$true;startOpen=$false}
function ProcessFacts([int]$PidValue,[string]$ExpectedFiletime,[string]$ExpectedImage) {
 $fact=[R6ArchiveProcessFacts]::Observe([uint32]$PidValue)
 if($fact.session -ne 7 -or $fact.filetime -cne $ExpectedFiletime -or ![string]::Equals($fact.image.Replace([char]47,[char]92),$ExpectedImage.Replace([char]47,[char]92),[StringComparison]::OrdinalIgnoreCase) -or $fact.exited){throw 'exact original process generation/image/session differs or exited'}
 $fact
}
function Sha([byte[]]$Bytes){$alg=[Security.Cryptography.SHA256]::Create();try{([BitConverter]::ToString($alg.ComputeHash($Bytes))).Replace('-','').ToLowerInvariant()}finally{$alg.Dispose()}}
function FileSha([string]$Path){(Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()}
function StreamHash([IO.Stream]$Stream){$alg=[Security.Cryptography.SHA256]::Create();try{([BitConverter]::ToString($alg.ComputeHash($Stream))).Replace('-','').ToLowerInvariant()}finally{$alg.Dispose()}}
try {
 $inputManifest=Get-Content -LiteralPath (Join-Path $rootPath 'input-manifest.json') -Raw -Encoding UTF8|ConvertFrom-Json
 foreach($row in $inputManifest.files){$p=Join-Path $rootPath $row.name;if((Get-Item -LiteralPath $p).Length -ne $row.bytes -or (FileSha $p) -cne $row.sha256){throw 'prepared archive script differs'}}
 Add-Type -Path (Join-Path $rootPath 'archive-r6-process-facts.cs')
 $native=Get-Content -LiteralPath (Join-Path $base 'D/qualification-result.json') -Raw -Encoding UTF8|ConvertFrom-Json
 if($native.actualProductionExecuted -or $native.nativeProductionSequencePassed -or !$native.nativeOwnerRetained -or $native.trace.creatorResumed -or $native.trace.originalPhaseRetained -or $native.trace.stage -cne 'created-suspended' -or $native.trace.binding.pid -ne 50580 -or $native.trace.binding.filetime -cne '134393577919362322'){throw 'exact original first failed preresume scene required'}
 $result.brokerBefore=ProcessFacts 54692 '134393577413203335' (Join-Path $base 'I/ocxd.exe')
 $result.launcherBefore=ProcessFacts 50580 '134393577919362322' (Join-Path $base 'I/runtime/versions/2.99.0/dependencies/node/node.exe')
 $result.firstError=$native.firstError
 if(Test-Path -LiteralPath $archiveRoot){throw 'fresh private archive destination required; partial archive retained'}
 [void][IO.Directory]::CreateDirectory($archiveRoot)
 $archiveDirectory=Get-Item -LiteralPath $archiveRoot -Force
 if($archiveDirectory.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'private archive root reparse refused'}
 Add-Type -AssemblyName System.IO.Compression
 Add-Type -AssemblyName System.IO.Compression.FileSystem
 $roots=@(@{source=$base;relative='failed-task'},@{source=(Join-Path $profile '.codex');relative='synthetic-codex'},@{source=(Join-Path $profile '.grok');relative='synthetic-grok'})
 $todo=@()
 foreach($rootEntry in $roots){
  $source=[IO.Path]::GetFullPath($rootEntry.source)
  $items=@(Get-Item -LiteralPath $source -Force)+@(Get-ChildItem -LiteralPath $source -Recurse -Force)
  foreach($item in $items){
   if($item.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'source reparse refused before any archive read'}
   $rel=$rootEntry.relative
   if($item.FullName -cne $source){$rel+='/'+$item.FullName.Substring($source.Length+1).Replace([char]92,[char]47)}
   if($item.PSIsContainer){$result.directories+=@{source=$item.FullName;relative=$rel};continue}
   if($item.Length -gt 536870912){throw 'original complete source file bound exceeded'}
   $todo+=@{source=$item.FullName;relative=$rel;bytes=$item.Length}
  }
 }
 $runtimeRoot=Join-Path $profile 'AppData/Local/OpenCodex/Runtime/v1/S-1-5-21-587408064-856152790-2583328096-1003'
 $clientDigest='db888acd5ce5e0ac06e0392032d3bb4646b5328a4c6f39ff8c6419ffa312a676'
 foreach($name in @(('native-write-locks/'+$clientDigest+'.sqlite'),'client-desktop-lifecycle.sqlite',('catalog-write-locks/'+$clientDigest+'.sqlite'))){
  $p=Join-Path $runtimeRoot $name;$item=Get-Item -LiteralPath $p -Force
  if($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'exact known coordinator file required'}
  $todo+=@{source=$item.FullName;relative=('synthetic-coordination/'+$name);bytes=$item.Length}
 }
 if($todo.Count -gt 20000 -or ($todo|Measure-Object -Property bytes -Sum).Sum -gt 2147483648){throw 'complete archive original bounds exceeded'}
 $file=[IO.File]::Open($zipPath,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 $zip=[IO.Compression.ZipArchive]::new($file,[IO.Compression.ZipArchiveMode]::Create,$true)
 try {
  foreach($row in $todo){
   try {
    $src=[IO.File]::Open($row.source,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
    try {
     if($src.Length -ne $row.bytes){throw 'source length changed after complete inventory'}
     $hash=StreamHash $src;$src.Position=0
     $entry=$zip.CreateEntry($row.relative,[IO.Compression.CompressionLevel]::Fastest);$dst=$entry.Open()
     try{$src.CopyTo($dst)}finally{$dst.Dispose()}
     if($src.Length -ne $row.bytes){throw 'source length changed during archive'}
     $src.Position=0;if((StreamHash $src) -cne $hash){throw 'source bytes changed during archive'}
    }finally{$src.Dispose()}
    $result.files+=@{source=$row.source;relative=$row.relative;bytes=$row.bytes;sha256=$hash}
   }catch{$result.fileErrors+=@{relative=$row.relative;error=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
  }
 }finally{$zip.Dispose();$file.Flush();$file.Dispose()}
 if($result.fileErrors.Count){throw 'one or more original archive file snapshots failed; all source/partial archive retained'}
 $verify=[IO.Compression.ZipFile]::OpenRead($zipPath)
 try {
  if($verify.Entries.Count -ne $result.files.Count){throw 'ZIP exact complete file membership mismatch'}
  foreach($row in $result.files){$entry=$verify.GetEntry($row.relative);if($null -eq $entry -or $entry.Length -ne $row.bytes){throw 'ZIP member size missing/mismatch'};$stream=$entry.Open();try{if((StreamHash $stream) -cne $row.sha256){throw 'ZIP member original byte digest mismatch'}}finally{$stream.Dispose()}}
 }finally{$verify.Dispose()}
 foreach($rootEntry in $roots){
  $source=[IO.Path]::GetFullPath($rootEntry.source)
  $after=@(Get-ChildItem -LiteralPath $source -Recurse -Force)
  if(@($after|Where-Object {$_.Attributes -band [IO.FileAttributes]::ReparsePoint}).Count){throw 'source reparse appeared after snapshot'}
  $expectedFiles=@($result.files|Where-Object {$_.source.StartsWith($source+[char]92,[StringComparison]::OrdinalIgnoreCase)}|ForEach-Object {$_.source}|Sort-Object)
  $actualFiles=@($after|Where-Object {!$_.PSIsContainer}|ForEach-Object {$_.FullName}|Sort-Object)
  if(($expectedFiles|ConvertTo-Json -Compress) -cne ($actualFiles|ConvertTo-Json -Compress)){throw 'source exact file membership changed after archive'}
  $expectedDirs=@($result.directories|Where-Object {$_.source -ceq $source -or $_.source.StartsWith($source+[char]92,[StringComparison]::OrdinalIgnoreCase)}|ForEach-Object {$_.source}|Sort-Object)
  $actualDirs=@($source)+@($after|Where-Object {$_.PSIsContainer}|ForEach-Object {$_.FullName});$actualDirs=@($actualDirs|Sort-Object)
  if(($expectedDirs|ConvertTo-Json -Compress) -cne ($actualDirs|ConvertTo-Json -Compress)){throw 'source directory membership changed after archive'}
 }
 $result.brokerAfter=ProcessFacts 54692 '134393577413203335' (Join-Path $base 'I/ocxd.exe')
 $result.launcherAfter=ProcessFacts 50580 '134393577919362322' (Join-Path $base 'I/runtime/versions/2.99.0/dependencies/node/node.exe')
 $result.archive=@{path=$zipPath;bytes=(Get-Item -LiteralPath $zipPath).Length;sha256=(FileSha $zipPath)}
 $result.archiveVerified=$true
}catch{$result.firstArchiveError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber};$_|Out-String|Set-Content -LiteralPath (Join-Path $out 'archive-error.log') -Encoding utf8}
finally {
 $result|ConvertTo-Json -Depth 14|Set-Content -LiteralPath $receipt -Encoding utf8
 if(Test-Path -LiteralPath $archiveRoot){$privateReceipt=Join-Path $archiveRoot 'archive-manifest.json';if(!(Test-Path -LiteralPath $privateReceipt)){[IO.File]::WriteAllText($privateReceipt,($result|ConvertTo-Json -Depth 14),[Text.UTF8Encoding]::new($false))}}
}

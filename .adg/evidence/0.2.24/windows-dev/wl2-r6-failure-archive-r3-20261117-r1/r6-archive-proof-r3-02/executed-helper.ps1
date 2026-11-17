# Caller owns exact token/process/scope admission. This helper only reads source
# files and writes a fresh content ZIP; never changes permissions or processes.
function Get-R6StreamDigest([IO.Stream]$Stream){
 $alg=[Security.Cryptography.SHA256]::Create()
 try{([BitConverter]::ToString($alg.ComputeHash($Stream))).Replace('-','').ToLowerInvariant()}finally{$alg.Dispose()}
}
function Get-R6ArchiveInventory([object[]]$Roots,[object[]]$ExtraFiles){
 $files=[Collections.Generic.List[object]]::new();$dirs=[Collections.Generic.List[object]]::new()
 $names=[Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
 [long]$total=0
 foreach($root in $Roots){
  $source=[IO.Path]::GetFullPath($root.source);$rootItem=Get-Item -LiteralPath $source -Force -ErrorAction Stop
  if(!$rootItem.PSIsContainer -or ($rootItem.Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'regular original archive root required'}
  $items=@($rootItem)+@(Get-ChildItem -LiteralPath $source -Recurse -Force -ErrorAction Stop)
  foreach($item in $items){
   if($item.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'source reparse refused'}
   $rel=[string]$root.relative
   if($item.FullName -cne $source){$rel+='/'+$item.FullName.Substring($source.Length+1).Replace([char]92,[char]47)}
   if(!$names.Add($rel)){throw 'duplicate archive relative name'}
   if($item.PSIsContainer){$dirs.Add(@{source=$item.FullName;relative=$rel});continue}
   if($item.Length -gt 536870912){throw 'original512MiB source file bound exceeded'}
   $total+=[long]$item.Length
   if($total -gt 2147483648 -or $files.Count -ge 20000){throw 'original2GiB/20000file archive bound exceeded'}
   $files.Add(@{source=$item.FullName;relative=$rel;bytes=[long]$item.Length})
  }
 }
 foreach($row in $ExtraFiles){
  $source=[IO.Path]::GetFullPath($row.source);$item=Get-Item -LiteralPath $source -Force -ErrorAction Stop
  if($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'regular exact extra file required'}
  if(!$names.Add([string]$row.relative)){throw 'duplicate extra relative name'}
  if($item.Length -gt 536870912){throw 'original512MiB extra file bound exceeded'}
  $total+=[long]$item.Length
  if($total -gt 2147483648 -or $files.Count -ge 20000){throw 'original2GiB/20000file archive bound exceeded'}
  $files.Add(@{source=$source;relative=[string]$row.relative;bytes=[long]$item.Length})
 }
 foreach($row in @($files.ToArray())+@($dirs.ToArray())){
  if([string]::IsNullOrEmpty($row.relative) -or $row.relative.Contains([string][char]92) -or $row.relative.Contains(':') -or $row.relative.StartsWith('/') -or @($row.relative.Split('/')|Where-Object {$_ -eq '' -or $_ -eq '.' -or $_ -eq '..'}).Count){throw 'noncanonical archive relative path'}
 }
 @{files=$files.ToArray();directories=$dirs.ToArray();bytes=$total}
}
function Invoke-R6ContentArchive([object[]]$Roots,[object[]]$ExtraFiles,[string]$ZipPath){
 $ErrorActionPreference='Stop'
 if(Test-Path -LiteralPath $ZipPath){throw 'fresh ZIP required; prior/partial archive retained'}
 $before=Get-R6ArchiveInventory $Roots $ExtraFiles
 Add-Type -AssemblyName System.IO.Compression
 Add-Type -AssemblyName System.IO.Compression.FileSystem
 $result=[ordered]@{files=@();directories=$before.directories;fileErrors=@();totalSourceBytes=$before.bytes;archiveVerified=$false;zipIsNtfsMetadataClone=$false}
 $file=[IO.File]::Open($ZipPath,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 $zip=[IO.Compression.ZipArchive]::new($file,[IO.Compression.ZipArchiveMode]::Create,$true)
 try{
  foreach($row in $before.files){
   try{
    $src=[IO.File]::Open($row.source,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
    try{
     if($src.Length -ne $row.bytes){throw 'source length changed after inventory'}
     $hash=Get-R6StreamDigest $src;$src.Position=0
     $entry=$zip.CreateEntry($row.relative,[IO.Compression.CompressionLevel]::Fastest);$dst=$entry.Open()
     try{$src.CopyTo($dst)}finally{$dst.Dispose()}
     if($src.Length -ne $row.bytes){throw 'source length changed during archive'}
     $src.Position=0;if((Get-R6StreamDigest $src) -cne $hash){throw 'source bytes changed during archive'}
    }finally{$src.Dispose()}
    $result.files+=@{source=$row.source;relative=$row.relative;bytes=$row.bytes;sha256=$hash}
   }catch{$result.fileErrors+=@{relative=$row.relative;message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
  }
 }finally{$zip.Dispose();$file.Flush();$file.Dispose()}
 if($result.fileErrors.Count){return $result}
 $verify=[IO.Compression.ZipFile]::OpenRead($ZipPath)
 try{
  if($verify.Entries.Count -ne $result.files.Count){throw 'ZIP exact member count mismatch'}
  foreach($row in $result.files){
   $entry=$verify.GetEntry($row.relative)
   if($null -eq $entry -or $entry.Length -ne $row.bytes){throw 'ZIP entry missing/size changed'}
   $stream=$entry.Open();try{if((Get-R6StreamDigest $stream) -cne $row.sha256){throw 'ZIP entry SHA256 changed'}}finally{$stream.Dispose()}
  }
 }finally{$verify.Dispose()}
 $after=Get-R6ArchiveInventory $Roots $ExtraFiles
 foreach($kind in @('files','directories')){
  $left=@($before[$kind]|ForEach-Object {$_.relative+[char]0+$_.source}|Sort-Object)
  $right=@($after[$kind]|ForEach-Object {$_.relative+[char]0+$_.source}|Sort-Object)
  if(($left|ConvertTo-Json -Compress) -cne ($right|ConvertTo-Json -Compress)){throw 'complete source membership changed after ZIP'}
 }
 foreach($row in $result.files){
  $src=[IO.File]::Open($row.source,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
  try{if($src.Length -ne $row.bytes -or (Get-R6StreamDigest $src) -cne $row.sha256){throw 'source bytes changed after ZIP'}}finally{$src.Dispose()}
 }
 $result.archive=@{path=$ZipPath;bytes=(Get-Item -LiteralPath $ZipPath).Length;sha256=(Get-FileHash -LiteralPath $ZipPath -Algorithm SHA256).Hash.ToLowerInvariant()}
 $result.archiveVerified=$true;$result
}

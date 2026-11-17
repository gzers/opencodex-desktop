param([Parameter(Mandatory)][string]$Round)
$ErrorActionPreference='Stop'
$expected=[IO.Path]::GetFullPath('C:/Users/15119/.codex/worktrees/official299-windows-evidence/docs-governance/.adg/evidence/0.2.24/windows-dev/wl2-r6-successor-predecessors-20261117-r1')
if([IO.Path]::GetFullPath($Round) -cne $expected){throw 'exact current round only'}
$receipt=Join-Path $Round 'data-scene-cleanup.json'
if(Test-Path -LiteralPath $receipt){throw 'immutable cleanup receipt already present'}
$targets=@()
foreach($run in @('library01','library02','library03')){
 $lines=[IO.File]::ReadAllLines((Join-Path $Round "$run/stdout.log"))
 $selected=@();$name=''
 foreach($line in $lines){
  if($line -match '^test ([^ ]+) \.\.\. '){$name=$Matches[1]}
  if($name -like '*::v3::predecessors::tests::wl2_r6_predecessors_*' -and $line -match '^r4-retained-synthetic-fixture=(.+)$'){$selected+= $Matches[1]}
 }
 if($selected.Count -ne 2){throw "expected exactly2 completed DATA namespace fixtures in $run"}
 $archive=Get-Content -LiteralPath (Join-Path $Round "$run-private-scenes.json") -Raw|ConvertFrom-Json
 foreach($source in $selected){
  $matchesScene=@($archive.scenes|Where-Object { [IO.Path]::GetFullPath($_.source) -ceq [IO.Path]::GetFullPath($source) })
  if($matchesScene.Count -ne 1){throw 'exact scene archive missing/duplicate'}
  $s=$matchesScene[0];$absolute=[IO.Path]::GetFullPath($source)
  $tempRoot=[IO.Path]::GetFullPath('C:/Users/15119/AppData/Local/Temp').TrimEnd([char]92)
  if([IO.Path]::GetDirectoryName($absolute) -cne $tempRoot -or [IO.Path]::GetFileName($absolute) -cnotmatch '^\.tmp[A-Za-z0-9]+$'){throw 'outside intended fixture temp root'}
  if(!(Test-Path -LiteralPath $absolute -PathType Container)){throw 'source absent; no deletion retry'}
  $all=@(Get-Item -LiteralPath $absolute)+@(Get-ChildItem -LiteralPath $absolute -Recurse -Force)
  if(@($all|Where-Object { $_.Attributes -band [IO.FileAttributes]::ReparsePoint }).Count){throw 'reparse/hardlink-like source refused'}
  $files=@($all|Where-Object {!$_.PSIsContainer})
  $dirs=@($all|Where-Object {$_.PSIsContainer -and $_.FullName -cne $absolute}|ForEach-Object {$_.FullName.Substring($absolute.Length+1).Replace([char]92,[char]47)}|Sort-Object)
  if(($dirs|ConvertTo-Json -Compress) -cne (@($s.directories|Sort-Object)|ConvertTo-Json -Compress)){throw 'directory membership changed'}
  if($files.Count -ne $s.files.Count){throw 'file membership changed'}
  foreach($f in $s.files){
   $path=[IO.Path]::GetFullPath((Join-Path $absolute $f.path))
   if(!$path.StartsWith($absolute+[char]92,[StringComparison]::OrdinalIgnoreCase)){throw 'archive relative path escapes source'}
   $item=Get-Item -LiteralPath $path
   if($item.Length -ne $f.bytes -or (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $f.sha256){throw 'source changed after archive'}
  }
  if((Get-FileHash -LiteralPath $s.archive -Algorithm SHA256).Hash.ToLowerInvariant() -cne $s.archiveSha256){throw 'archive hash changed'}
  $targets+= [ordered]@{run=$run;source=$absolute;archive=$s.archive;archiveSha256=$s.archiveSha256;files=$files.Count;removed=$false;error=$null}
 }
}
if($targets.Count -ne 6){throw 'all independent targets must be verified before deletion'}
foreach($target in $targets){
 try{Remove-Item -LiteralPath $target.source -Recurse -Force;$target.removed=!(Test-Path -LiteralPath $target.source)}
 catch{$target.error=$_.Exception.Message}
}
@{scope='Only exact6 completed synthetic predecessor locator DATA fixtures, no custody/completedNative/Owner scenes';targets=$targets;zipIsNtfsMetadataClone=$false;historicalRejectedDeletionRetried=$false;overallCleanupComplete=$false}|ConvertTo-Json -Depth 12|Set-Content -LiteralPath $receipt -Encoding utf8
$targets|Select-Object run,source,removed,error|ConvertTo-Json -Depth 4

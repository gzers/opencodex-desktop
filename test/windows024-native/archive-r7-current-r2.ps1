param([Parameter(Mandatory)][string]$Root)
$ErrorActionPreference='Stop'
$rootPath=[IO.Path]::GetFullPath($Root)
if($rootPath -cne 'C:\Users\Public\ocx-r7-r2-readonly-20261117-r1'){throw 'exact readonly collection root required'}
$out=Join-Path $rootPath 'out01'
$receipt=Join-Path $out 'archive-result.json'
if(Test-Path -LiteralPath $receipt){throw 'immutable observation already exists; do not rerun'}
$profile='C:\Users\marenben'
$base=Join-Path $profile 'ocx024-r6-existing-20261117-r2'
$private=Join-Path $profile 'ocx024-r7-r2-private-archive-20261117-r1'
$sid='S-1-5-21-587408064-856152790-2583328096-1003'
$result=[ordered]@{schema='windows-r7-r2-readonly-archive-v1';scope='full r2 content archive plus exact native observations; no repair authority';started=[DateTimeOffset]::Now.ToString('o');errors=@();archiveVerified=$false;nativeBeforePassed=$false;nativeAfterPassed=$false;metadataStable=$false;finalQuiescentSeal=$false;repairExecutionReady=$false;forced=$false;restored=$false;normalStop=$false;closed=$false;publicStartOpen=$false;zipIsNtfsIdentitySecurityClone=$false;allOriginalFilesRetained=$true}
$pair=$null
function FileSha([string]$Path){
 $stream=[IO.File]::Open($Path,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
 try{$alg=[Security.Cryptography.SHA256]::Create();try{([BitConverter]::ToString($alg.ComputeHash($stream))).Replace('-','').ToLowerInvariant()}finally{$alg.Dispose()}}finally{$stream.Dispose()}
}
function ErrorFact([string]$Stage,$ErrorRecord){
 $result.errors+=@{stage=$Stage;message=$ErrorRecord.Exception.ToString();line=$ErrorRecord.InvocationInfo.ScriptLineNumber}
}
function NativeObservation([string]$Phase){
 try{
  $previousPreference=$ErrorActionPreference
  try{
   $ErrorActionPreference='Continue'
   & (Join-Path $rootPath 'input/r7-scene-observer.exe') '--r7-r2-readonly024-v1' $Phase 1> (Join-Path $out ('native-'+$Phase+'.stdout.log')) 2> (Join-Path $out ('native-'+$Phase+'.stderr.log'))
   $code=$LASTEXITCODE
  }finally{$ErrorActionPreference=$previousPreference}
  $result[('native'+(Get-Culture).TextInfo.ToTitleCase($Phase)+'RawExit')]=$code
  $path=Join-Path $out ('native-'+$Phase+'.json')
  if(Test-Path -LiteralPath $path){$report=Get-Content -LiteralPath $path -Raw -Encoding UTF8|ConvertFrom-Json;$result[('native'+(Get-Culture).TextInfo.ToTitleCase($Phase)+'Passed')]=($code -eq 0 -and $report.passed)}
  else{throw 'native raw observation report missing'}
 }catch{ErrorFact ('native-'+$Phase) $_}
}
function InventoryMetadata($Inventory,[string]$Phase){
 $rows=[Collections.Generic.List[object]]::new();$faults=[Collections.Generic.List[object]]::new()
 foreach($kind in @('directories','files')){
  foreach($row in $Inventory[$kind]){
   try{$fact=[R7ReadOnlyScene]::ObserveFile($row.source,($kind -ceq 'directories'));$rows.Add(@{relative=$row.relative;fact=$fact})}
   catch{$faults.Add(@{relative=$row.relative;message=$_.Exception.ToString();line=$_.InvocationInfo.ScriptLineNumber})}
  }
 }
 $report=@{phase=$Phase;rows=$rows.ToArray();errors=$faults.ToArray();passed=$faults.Count -eq 0;observedAt=[DateTimeOffset]::Now.ToString('o')}
 $report|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $out ('metadata-'+$Phase+'.json')) -Encoding UTF8
 $report
}
try{
 $manifest=Get-Content -LiteralPath (Join-Path $rootPath 'input-manifest.json') -Raw -Encoding UTF8|ConvertFrom-Json
 if($manifest.schema -cne 'windows-r7-r2-readonly-input-v2' -or $manifest.expectedSid -cne $sid -or $manifest.expectedSession -ne 7 -or $manifest.collector -cne 'standalone-native-shared-inventory-section'){throw 'fixed prepared manifest scope differs'}
 foreach($row in $manifest.files){
  if($row.name.Contains('..') -or [IO.Path]::IsPathRooted($row.name)){throw 'noncanonical prepared member'}
  $path=Join-Path $rootPath $row.name
  if((Get-Item -LiteralPath $path -Force).Length -ne $row.bytes -or (FileSha $path) -cne $row.sha256){throw 'prepared input bytes differ'}
 }
 Add-Type -Path (Join-Path $rootPath 'r7-readonly-scene.cs')
 [R7ReadOnlyScene]::RequireContext()
 $result.sid=$sid;$result.session=7;$result.ordinaryNoJob=$true
 try{$pair=[R7ReadOnlyScene]::ObserveExactPair();$result.processBefore=$pair.Snapshot()}catch{ErrorFact 'native-process-before' $_}
 # Separate full native rows are attempted before archiving, preserving all conflicts.
 NativeObservation 'before'
 try{
  $cim=@(Get-CimInstance Win32_Process -Filter 'ProcessId=21500 OR ProcessId=75744' -ErrorAction Stop)
  $result.processLineage=@($cim|Select-Object ProcessId,ParentProcessId,ExecutablePath,CommandLine,CreationDate)
  if($cim.Count -ne 2 -or @($cim|Where-Object {$_.ProcessId -eq 21500 -and $_.ParentProcessId -eq 75744}).Count -ne 1 -or @($cim|Where-Object {[string]::IsNullOrEmpty($_.CommandLine) -or [string]::IsNullOrEmpty($_.ExecutablePath)}).Count){throw 'exact live command lineage unavailable/differs'}
 }catch{ErrorFact 'command-lineage' $_}
 try{
  $original=Get-Content -LiteralPath (Join-Path $base 'D/qualification-result.json') -Raw -Encoding UTF8|ConvertFrom-Json
  if($original.actualProductionExecuted -or $original.trace.creatorResumed -or $original.trace.originalPhaseRetained -or $original.trace.stage -cne 'created-suspended' -or $original.trace.binding.pid -ne 21500 -or $original.trace.binding.filetime -cne '134393657318681068' -or $original.trace.family.total -ne 1 -or $original.trace.family.active -ne 1 -or $original.trace.family.members.Count -ne 0){throw 'original creator/Job failure trace differs'}
  $result.originalFailure=$original
 }catch{ErrorFact 'original-creator-history' $_}
 $result.originalJobCurrentStatus='Unavailable: original image has cleanup command but no readonly Status client dispatch; no new-image authentication attempted'
 if(Test-Path -LiteralPath $private){throw 'fresh private archive destination required; prior prefix retained'}
 $security=New-Object Security.AccessControl.DirectorySecurity
 $security.SetSecurityDescriptorSddlForm(('O:{0}D:P(A;OICI;FA;;;{0})(A;OICI;FA;;;SY)' -f $sid))
 [void][IO.Directory]::CreateDirectory($private,$security)
 [void][R7ReadOnlyScene]::ObserveFile($private,$true)
 . (Join-Path $rootPath 'archive-r6-content.ps1')
 $roots=@(
  @{source=$base;relative='r2-task'},
  @{source=(Join-Path $profile '.codex');relative='clients/codex'},
  @{source=(Join-Path $profile '.grok');relative='clients/grok'},
  @{source=(Join-Path $profile '.ocxd024-originals-c3db7afd-959c-44fb-8743-1cca384043c8-codex');relative='custody/codex'},
  @{source=(Join-Path $profile '.ocxd024-originals-c3db7afd-959c-44fb-8743-1cca384043c8-grok');relative='custody/grok'},
  @{source='C:\Users\Public\ocx-r6-official-existing-20261117-r2';relative='original-public'},
  @{source='C:\Users\Public\ocx-r6-profile-preflight-20261117-r1';relative='original-setup-public'},
  @{source=(Join-Path $profile 'ocx024-r6-existing-20261117-r1/preparation');relative='original-setup-private'}
 )
 $runtime=Join-Path $profile ('AppData/Local/OpenCodex/Runtime/v1/'+$sid)
 $digest='db888acd5ce5e0ac06e0392032d3bb4646b5328a4c6f39ff8c6419ffa312a676'
 $extras=@()
 foreach($name in @(('native-write-locks/'+$digest+'.sqlite'),'client-desktop-lifecycle.sqlite',('catalog-write-locks/'+$digest+'.sqlite'))){$extras+=@{source=(Join-Path $runtime $name);relative=('coordination/'+$name)}}
 $before=Get-R6ArchiveInventory -Roots $roots -ExtraFiles $extras
 $metaBefore=InventoryMetadata $before 'before'
 if(!$metaBefore.passed){throw 'native archive member/ADS admission has conflicts; no stream omitted silently'}
 $result.archiveSnapshotStarted=[DateTimeOffset]::Now.ToString('o')
 $content=Invoke-R6ContentArchive -Roots $roots -ExtraFiles $extras -ZipPath (Join-Path $private 'r2-complete-content.zip')
 $result.archiveSnapshotCompleted=[DateTimeOffset]::Now.ToString('o')
 $content|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $out 'content-manifest.json') -Encoding UTF8
 $after=Get-R6ArchiveInventory -Roots $roots -ExtraFiles $extras
 $metaAfter=InventoryMetadata $after 'after'
 $left=@($metaBefore.rows|ForEach-Object {($_|ConvertTo-Json -Depth 8 -Compress)}|Sort-Object)
 $right=@($metaAfter.rows|ForEach-Object {($_|ConvertTo-Json -Depth 8 -Compress)}|Sort-Object)
 $result.metadataStable=$metaAfter.passed -and (($left|ConvertTo-Json -Compress) -ceq ($right|ConvertTo-Json -Compress))
 $result.archive=$content.archive;$result.archiveVerified=$content.archiveVerified -and $result.metadataStable
 $result.files=$content.files.Count;$result.directories=$content.directories.Count;$result.bytes=$content.totalSourceBytes
 if($null -ne $pair){try{$result.processAfter=$pair.Snapshot()}catch{ErrorFact 'native-process-after' $_}}
 NativeObservation 'after'
 if(!$result.archiveVerified){throw 'full archive or native membership changed; all originals and partial archive retained'}
}catch{ErrorFact 'collection' $_;$_|Out-String|Set-Content -LiteralPath (Join-Path $out 'archive-error.log') -Encoding UTF8}
finally{
 if($null -ne $pair){$pair.Dispose()}
 $result.completed=[DateTimeOffset]::Now.ToString('o')
 $result|ConvertTo-Json -Depth 18|Set-Content -LiteralPath $receipt -Encoding UTF8
 if(Test-Path -LiteralPath $private){
  $privateReceipt=Join-Path $private 'archive-result.json'
  if(!(Test-Path -LiteralPath $privateReceipt)){[IO.File]::WriteAllText($privateReceipt,($result|ConvertTo-Json -Depth 18),[Text.UTF8Encoding]::new($false))}
 }
}
if($result.archiveVerified -and $result.nativeBeforePassed -and $result.nativeAfterPassed -and $result.errors.Count -eq 0){exit 0}
exit 1

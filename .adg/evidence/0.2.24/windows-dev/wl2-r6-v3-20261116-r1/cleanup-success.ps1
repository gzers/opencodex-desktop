$ErrorActionPreference='Stop'
$plan=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'success-cleanup-plan.json') -Raw|ConvertFrom-Json
$results=@()
foreach($scene in $plan){
 $target=[IO.Path]::GetFullPath($scene.source)
 $resolved=(Resolve-Path -LiteralPath $target).Path
 if($resolved-cne $target -or [IO.Path]::GetDirectoryName($resolved)-cne 'C:\Users\15119\AppData\Local\Temp' -or [IO.Path]::GetFileName($resolved)-cnotmatch '^\.tmp[A-Za-z0-9]+$'){throw 'not exact own completed Temp scene'}
 if((Get-FileHash -LiteralPath $scene.archive -Algorithm SHA256).Hash.ToLowerInvariant()-cne $scene.archiveSha256){throw 'archive hash differs'}
 $items=@(Get-Item -LiteralPath $resolved)+@(Get-ChildItem -LiteralPath $resolved -Recurse -Force)
 if(@($items|Where-Object {$_.Attributes-band [IO.FileAttributes]::ReparsePoint}).Count){throw 'reparse cleanup refused'}
 $files=@($items|Where-Object {-not $_.PSIsContainer})
 if($files.Count-ne $scene.files.Count){throw 'unknown file count'}
 foreach($file in $files){
  $rel=[IO.Path]::GetRelativePath($resolved,$file.FullName).Replace('\','/')
  $expected=@($scene.files|Where-Object {$_.path-ceq $rel})
  if($expected.Count-ne 1 -or $file.Length-ne $expected[0].bytes -or (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()-cne $expected[0].sha256){throw 'unknown/changed file'}
 }
 $dirs=@($items|Where-Object {$_.PSIsContainer -and $_.FullName-cne $resolved}|ForEach-Object {[IO.Path]::GetRelativePath($resolved,$_.FullName).Replace('\','/')}|Sort-Object)
 if(Compare-Object -ReferenceObject @($scene.directories|Sort-Object) -DifferenceObject $dirs -CaseSensitive){throw 'unknown directory'}
 Remove-Item -LiteralPath $resolved -Recurse -Force
 if(Test-Path -LiteralPath $resolved){throw 'cleanup incomplete'}
 $results+=@{source=$resolved;run=$scene.run;test=$scene.test;removed=$true;files=$files.Count;archive=$scene.archive;archiveSha256=$scene.archiveSha256}
}
@{time=[datetime]::Now.ToString('o');removed=$results;failedAndPendingScenesRetained=60;crossVolumePendingRootsRetained=5;newTaskParentDirectoriesRetained=3;historicalRejectedDeletionRetried=$false;overallCleanupComplete=$false;nativeMetadataCloned=$false}|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $PSScriptRoot 'cleanup.json') -Encoding utf8

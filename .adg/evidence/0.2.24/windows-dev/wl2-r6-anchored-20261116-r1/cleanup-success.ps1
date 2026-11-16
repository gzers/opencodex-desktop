$ErrorActionPreference='Stop'
$round=$PSScriptRoot
$scenes=(Get-Content -LiteralPath (Join-Path $round 'library02-private-scenes.json') -Raw|ConvertFrom-Json).scenes
$names=@('.tmppQgP2M','.tmpoabB4K')
$results=@()
foreach($name in $names){
 $target=[IO.Path]::GetFullPath((Join-Path 'C:/Users/15119/AppData/Local/Temp' $name))
 $scene=@($scenes|Where-Object {[IO.Path]::GetFullPath($_.source)-ceq $target})
 if($scene.Count-ne 1){throw 'exact own success scene not unique'}
 $resolved=(Resolve-Path -LiteralPath $target).Path
 if($resolved-cne $target -or [IO.Path]::GetDirectoryName($resolved)-cne 'C:\Users\15119\AppData\Local\Temp'){throw 'cleanup root outside exact task Temp parent'}
 if((Get-FileHash -LiteralPath $scene[0].archive -Algorithm SHA256).Hash.ToLowerInvariant()-cne $scene[0].archiveSha256){throw 'archive hash differs'}
 $items=@(Get-Item -LiteralPath $target)+@(Get-ChildItem -LiteralPath $target -Force -Recurse)
 if(@($items|Where-Object {$_.Attributes -band [IO.FileAttributes]::ReparsePoint}).Count){throw 'reparse cleanup refused'}
 $files=@($items|Where-Object {-not $_.PSIsContainer})
 if($files.Count-ne $scene[0].files.Count){throw 'unknown file count'}
 foreach($file in $files){
  $rel=[IO.Path]::GetRelativePath($target,$file.FullName).Replace('\','/')
  $expected=@($scene[0].files|Where-Object {$_.path-ceq $rel})
  if($expected.Count-ne 1 -or $file.Length-ne $expected[0].bytes -or (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()-cne $expected[0].sha256){throw 'unknown or changed scene file'}
 }
 $dirs=@($items|Where-Object {$_.PSIsContainer -and $_.FullName-cne $target}|ForEach-Object {[IO.Path]::GetRelativePath($target,$_.FullName).Replace('\','/')}|Sort-Object)
 $expectedDirs=@('Profile','Profile/.codex','Profile/.codex/empty','Profile/.codex/nested','Profile/custody','coordination')
 if($name-ceq '.tmpoabB4K'){$expectedDirs+=@('Profile/.grok','Profile/grok-custody')}
 if(Compare-Object -ReferenceObject @($expectedDirs|Sort-Object) -DifferenceObject $dirs -CaseSensitive){throw 'unknown scene directory'}
 # Exact resolved task roots checked above. All originals were restored and
 # exact retained work generations archived before removal. No historical root.
 Remove-Item -LiteralPath $resolved -Recurse -Force
 if(Test-Path -LiteralPath $resolved){throw 'cleanup incomplete'}
 $results+=@{source=$resolved;removed=$true;files=$files.Count;directories=$dirs;archive=$scene[0].archive;archiveSha256=$scene[0].archiveSha256}
}
@{time=[datetime]::Now.ToString('o');removed=$results;failureAndPendingScenesRetained=24;historicalRejectedDeletionRetried=$false;overallCleanupComplete=$false;nativeMetadataCloned=$false}|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $round 'cleanup.json') -Encoding utf8

$ErrorActionPreference='Stop'
$plan=Get-Content -LiteralPath "$PSScriptRoot/success-cleanup-plan.json" -Raw | ConvertFrom-Json
$rows=@()
foreach($scene in $plan){
  $row=@{source=$scene.source;test=$scene.test;removed=$false;archiveVerified=$false;error=$null}
  try {
    $resolved=(Resolve-Path -LiteralPath $scene.source).Path
    $separator=[IO.Path]::DirectorySeparatorChar
    $tempRoot=[IO.Path]::GetFullPath('C:/Users/15119/AppData/Local/Temp').TrimEnd($separator)+$separator
    if(-not $resolved.StartsWith($tempRoot,[StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetFileName($resolved) -notmatch '^\.tmp[A-Za-z0-9]+$'){throw 'exact fresh scene outside intended Temp refused'}
    if((Get-Item -LiteralPath $resolved -Force).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'reparse scene refused'}
    if((Get-FileHash -LiteralPath $scene.archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $scene.archiveSha256){throw 'archive digest changed'}
    $items=@(Get-ChildItem -LiteralPath $resolved -Force -Recurse)
    foreach($item in $items){if($item.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'reparse member refused'}}
    $actualFiles=@($items | Where-Object {-not $_.PSIsContainer})
    $actualDirs=@($items | Where-Object {$_.PSIsContainer} | ForEach-Object {$_.FullName.Substring($resolved.Length+1).Replace($separator,[char]'/')} | Sort-Object)
    if($actualFiles.Count -ne @($scene.files).Count -or ($actualDirs -join [char]10) -cne (@($scene.directories | Sort-Object) -join [char]10)){throw 'scene full membership changed'}
    foreach($file in $scene.files){
      $target=[IO.Path]::GetFullPath((Join-Path $resolved $file.path))
      if(-not $target.StartsWith($resolved+$separator,[StringComparison]::OrdinalIgnoreCase)){throw 'member escape refused'}
      $item=Get-Item -LiteralPath $target -Force
      if($item.PSIsContainer -or $item.Length -ne $file.bytes -or (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant() -ne $file.sha256){throw 'scene original bytes changed'}
    }
    $row.archiveVerified=$true
    # Paths have been resolved and verified within this round's fresh Temp roots.
    Remove-Item -LiteralPath $resolved -Recurse -Force
    if(Test-Path -LiteralPath $resolved){throw 'own completed scene remains'}
    $row.removed=$true
  } catch { $row.error=$_.Exception.Message }
  $rows+=@($row)
}
@{scenes=$rows;removed=@($rows|Where-Object {$_.removed}).Count;failed=@($rows|Where-Object {-not $_.removed}).Count;historicalRejectedDeletionRetried=$false;overallCleanupComplete=$false;zipIsNativeMetadataClone=$false} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath "$PSScriptRoot/cleanup.json" -Encoding utf8
if(@($rows|Where-Object {-not $_.removed}).Count -gt 0){exit 1}

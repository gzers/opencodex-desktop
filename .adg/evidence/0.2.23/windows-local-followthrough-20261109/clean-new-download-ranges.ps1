$ErrorActionPreference='Stop'
. 'C:\Users\15119\.codex\worktrees\ocxd-023-windows-r1\code\test\smoke\windows023-two-user-native-helper.ps1'
$root='D:\ocx-win-followthrough-20261109-230145'
$targets=@(
 'D:\ocx-win-followthrough-20261109-230145\installers-build-02\build-logs.zip.parts',
 'D:\ocx-win-followthrough-20261109-230145\installers-build-02\program.zip.parts',
 'D:\ocx-win-followthrough-20261109-230145\stop-phase-build\program-build\build-logs.zip.parts',
 'D:\ocx-win-followthrough-20261109-230145\stop-phase-build\program-build\program.zip.parts'
)
$report=[ordered]@{observedUtc=[datetime]::UtcNow.ToString('o');scope='Only four new own download-range caches after verified original ZIP; earlier policy-rejected paths excluded and never retried';originalArchivesAndEvidenceRetained=$true;paths=@()}
foreach($target in $targets){
 $entry=[ordered]@{path=$target;removed=$false}
 try{
  $resolved=[IO.Path]::GetFullPath($target)
  $null=Assert-TwoUserUnder $root $resolved;Assert-TwoUserNoReparseAncestors $resolved
  if($resolved -cne $target -or !$resolved.EndsWith('.zip.parts')){throw 'Exact range cache path required'}
  $archive=$resolved.Substring(0,$resolved.Length-6)
  $download=Get-Content -LiteralPath ($archive+'.download.json') -Raw|ConvertFrom-Json
  if((Get-Item -LiteralPath $archive).Length -ne $download.bytes -or (Get-FileHash -LiteralPath $archive).Hash -ine $download.sha256){throw 'Retained original archive SHA/bytes differ'}
  $entry.archiveSha256=$download.sha256
  $entry.bytes=@(Get-ChildItem -LiteralPath $resolved -File -Recurse|Measure-Object -Property Length -Sum)[0].Sum
  Remove-Item -LiteralPath $resolved -Recurse -Force
  $entry.removed=!(Test-Path -LiteralPath $resolved)
 }catch{$entry.failure=$_.Exception.Message}
 $report.paths+= $entry
}
$report.finishedUtc=[datetime]::UtcNow.ToString('o')
$report|ConvertTo-Json -Depth 7|Set-Content -LiteralPath (Join-Path $root 'new-download-range-cleanup.json') -Encoding utf8
$report.paths|Select-Object path,removed,bytes,failure

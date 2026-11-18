$ErrorActionPreference='Stop'
$code=(Get-Location).Path
$out=Join-Path $code 'test/windows024-native/target/r7-lease-range-20261118-r1'
$template=Join-Path $code 'test/windows024-native/target/r7-final-release-timing-20261118-r1/combined-scan/run-crate-opt1.ps1'
$runText=[IO.File]::ReadAllText($template).Replace("'--config','profile.test.package.opencodex-desktop.opt-level=1',",'')
[IO.File]::WriteAllText((Join-Path $out 'run-product-test.ps1'),$runText)
$utf8=[Text.UTF8Encoding]::new($false)
$dir='apps/desktop/tauri/src/infrastructure/windows_client_originals024'
$sources=@("$dir/coordination.rs","$dir/coordination/preparation.rs","$dir/coordination/preparation/tests.rs")
[IO.File]::WriteAllText((Join-Path $out 'sources.json'),($sources|ConvertTo-Json),$utf8)
$fixes=@{}
try {
  foreach ($relative in $sources[0..1]) {
    $file=Join-Path $code $relative
    $fixes[$relative]=[IO.File]::ReadAllBytes($file)
    $original=git show "HEAD:$relative"
    if ($LASTEXITCODE -ne 0) {throw "cannot load source baseline $relative"}
    [IO.File]::WriteAllText($file,($original -join [char]10)+[char]10,$utf8)
  }
  & (Join-Path $PSHOME 'pwsh.exe') -NoProfile -File (Join-Path $out 'run-product-test.ps1') -CodeRoot $code -EvidenceRoot $out -Run baseline -Filter wl2_r7_lease_prefix_native_missing_reserved_range_revokes_authority_and_keeps_claim
  $baseline=$LASTEXITCODE
} finally {
  foreach ($relative in $fixes.Keys) {[IO.File]::WriteAllBytes((Join-Path $code $relative),$fixes[$relative])}
}
function Save-Exe([string]$Run) {
  $dir=Join-Path $out $Run
  $line=(Select-String -LiteralPath (Join-Path $dir 'stderr.log') -Pattern 'Running unittests').Line
  if ($line -notmatch '\((.*\.exe)\)') {throw "actual test artifact missing for $Run"}
  $exe=(Resolve-Path -LiteralPath $Matches[1]).Path
  $data=[ordered]@{path=$exe;bytes=(Get-Item -LiteralPath $exe).Length;sha256=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant()}
  [IO.File]::WriteAllText((Join-Path $dir 'artifact.json'),($data|ConvertTo-Json)+[Environment]::NewLine,$utf8)
}
Save-Exe baseline
& (Join-Path $PSHOME 'pwsh.exe') -NoProfile -File (Join-Path $out 'run-product-test.ps1') -CodeRoot $code -EvidenceRoot $out -Run corrected -Filter wl2_r7_lease_prefix_native_
$corrected=$LASTEXITCODE
Save-Exe corrected
Write-Output "baseline=$baseline corrected=$corrected"
exit $corrected

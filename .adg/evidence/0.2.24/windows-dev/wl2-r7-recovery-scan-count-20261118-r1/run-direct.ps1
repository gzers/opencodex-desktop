param([Parameter(Mandatory)][string]$Run)
$ErrorActionPreference='Stop'
$compile=Get-Content -LiteralPath "$Run/compile.result.json" -Raw | ConvertFrom-Json
if ($compile.raw -ne 0) {throw 'MSVC compile did not succeed'}
$artifacts=@(Get-Content -LiteralPath "$Run/compile.stdout.log" | ForEach-Object {$_ | ConvertFrom-Json} | Where-Object {$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'opencodex_desktop_lib' -and $_.profile.test -and $_.executable})
if ($artifacts.Count -ne 1) {throw 'expected one MSVC test executable'}
$exe=Join-Path (Resolve-Path -LiteralPath $Run) (Split-Path -Leaf $artifacts[0].executable)
[IO.File]::Copy($artifacts[0].executable,$exe,$false)
$exeSha=(Get-FileHash -LiteralPath $exe).Hash.ToLowerInvariant()
$env:OCX024_TEST_BUN=(Resolve-Path -LiteralPath 'test/windows024-native/target/r6-source-scan-cost-d57c32d10af74ab888fcde86e6ec97cd/I/runtime/versions/2.99.0/dependencies/bun/bun.exe').ProviderPath
$bunSha=(Get-FileHash -LiteralPath $env:OCX024_TEST_BUN).Hash.ToLowerInvariant()
if ($bunSha -ne 'cb1d8e6564e655e49d30cead359750a1cbbd12f42651caa4b62244d7bd93c244') {throw 'Bun digest mismatch'}
$dataRoot='D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/test/windows024-native/target'
$rows=@(foreach ($test in @(
  @{name='whole-chain'; filter='wl2_r6_batch_native_complete_initial_and_recovery_phases_keep_exact_originals_and_claim'},
  @{name='raw5'; filter='wl2_r7_final_release_native_partial_unlock_keeps_original_claim_pending'}
)) {
    $name=$test.name
    $before=@(Get-ChildItem -LiteralPath $dataRoot -Filter 'r6-driver-*' -Directory -Name)
    $tempBefore=@(Get-ChildItem -LiteralPath ([IO.Path]::GetTempPath()) -Filter '.tmp*' -Directory -Name)
    $watch=[Diagnostics.Stopwatch]::StartNew()
    & $exe $test.filter --test-threads=1 --nocapture > "$Run/$name.stdout.log" 2> "$Run/$name.stderr.log"
    $raw=$LASTEXITCODE
    $watch.Stop()
    $after=@(Get-ChildItem -LiteralPath $dataRoot -Filter 'r6-driver-*' -Directory -Name | Where-Object {$_ -notin $before})
    $tempAfter=@(Get-ChildItem -LiteralPath ([IO.Path]::GetTempPath()) -Filter '.tmp*' -Directory -Name | Where-Object {$_ -notin $tempBefore})
    @{name=$name;filter=$test.filter;raw=$raw;elapsedMs=$watch.ElapsedMilliseconds;newDataRoots=$after;newTempRoots=$tempAfter}
})
@{tests=$rows;exe=$exe;exeSha256=$exeSha;exeBytes=(Get-Item -LiteralPath $exe).Length;bun=$env:OCX024_TEST_BUN;bunSha256=$bunSha;dataRoot=$dataRoot;tempRoot=[IO.Path]::GetTempPath();currentR2Touched=$false} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath "$Run/tests.result.json" -Encoding utf8NoBOM
$rows | Format-Table name,raw,elapsedMs


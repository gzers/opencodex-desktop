$ErrorActionPreference='Stop'
$root='C:\Users\Public\ocx-r8-marenben-20261122-r1'
$out=Join-Path $root 'out01'
$utf8=[Text.UTF8Encoding]::new($false)
$result=[ordered]@{startedUtc=(Get-Date).ToUniversalTime().ToString('o');sid=$null;session=$null;elevated=$null;raw=$null;passed=$false;firstError=$null}
try {
  $id=[Security.Principal.WindowsIdentity]::GetCurrent()
  $result.sid=$id.User.Value
  $result.session=[Diagnostics.Process]::GetCurrentProcess().SessionId
  $result.elevated=([Security.Principal.WindowsPrincipal]::new($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
  if($id.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003'){throw 'exact ordinary test SID -1003 required'}
  if($result.elevated){throw 'elevated token; ordinary run required'}
  $exe=Join-Path $root 'bin/opencodex_desktop_lib.exe'
  $bundle=Join-Path $root 'input/runtime-bundle-0.2.24-win32-x64.tar.gz'
  $manifest=Join-Path $root 'input/runtime-bundle.manifest.json'
  $hashes=[ordered]@{
    exe=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant()
    bundle=(Get-FileHash -LiteralPath $bundle -Algorithm SHA256).Hash.ToLowerInvariant()
    manifest=(Get-FileHash -LiteralPath $manifest -Algorithm SHA256).Hash.ToLowerInvariant()
  }
  $result.hashes=$hashes
  if($hashes.exe -cne 'f08edf9a872463d98541885e86db9f922cfc5ad1b1d3a45bd0d99e1fb9585e6b'){throw 'EXE digest differs'}
  if($hashes.bundle -cne '14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7'){throw 'bundle digest differs'}
  if($hashes.manifest -cne '42539a22dde96977d7575077d3e27e4537fe2794a46c07c565a73cedb7a11a1a'){throw 'manifest digest differs'}
  $env:OCXD024_NATIVE_INPUT=(Join-Path $root 'input')
  $env:OCXD024_NATIVE_EVIDENCE=$out
  $stdout=Join-Path $out 'test.stdout.log'
  $stderr=Join-Path $out 'test.stderr.log'
  $p=Start-Process -FilePath $exe -ArgumentList @('--exact','infrastructure::windows_official_lifecycle024::tests::r2_boundary_single_user','--ignored','--nocapture','--test-threads=1') -WorkingDirectory $root -WindowStyle Hidden -RedirectStandardOutput $stdout -RedirectStandardError $stderr -PassThru -Wait
  $result.raw=$p.ExitCode
  $result.passed=($p.ExitCode -eq 0)
  if(-not $result.passed){
    $err=Get-Content -LiteralPath $stderr -Tail 3 -ErrorAction SilentlyContinue
    $result.firstError=($err -join ' ')
  }
} catch {
  $result.firstError=$_.Exception.Message
} finally {
  $result.finishedUtc=(Get-Date).ToUniversalTime().ToString('o')
  $result.aggregatePresent=(Test-Path -LiteralPath (Join-Path $env:USERPROFILE '.codex-desktop'))
  $result | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $out 'result.json') -Encoding utf8NoBOM
  Get-Content -LiteralPath (Join-Path $out 'result.json')
  if($result.passed){ Write-Host '' ; Write-Host 'MARENBEN RESULT: PASS' } else { Write-Host '' ; Write-Host ('MARENBEN RESULT: FAIL - ' + $result.firstError) }
}
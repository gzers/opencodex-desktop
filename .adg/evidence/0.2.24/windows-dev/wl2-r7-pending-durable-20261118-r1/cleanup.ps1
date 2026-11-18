param([Parameter(Mandatory)][string]$Run)
$ErrorActionPreference = 'Stop'
$runRoot = (Resolve-Path -LiteralPath $Run).ProviderPath
$fixtureRows = Get-Content -LiteralPath "$runRoot/fixture-state.json" -Raw | ConvertFrom-Json
$rows = @(foreach ($row in $fixtureRows) {
    [ordered]@{path=$row.path; suite=$row.suite; suiteRaw=$row.suiteRaw;
        absent=!(Test-Path -LiteralPath $row.path); fixtureReportedAbsent=$row.absent}
})
$results = Get-Content -LiteralPath "$runRoot/direct-results.json" -Raw | ConvertFrom-Json
$newRoots = @(foreach ($result in $results) { foreach ($path in $result.newRoots) {
    [ordered]@{path=$path; absent=!(Test-Path -LiteralPath $path); retained=$true}
} })
$relative = 'apps/desktop/tauri/gen/schemas/windows-schema.json'
$path = [IO.Path]::GetFullPath($relative)
$resolved = (Resolve-Path -LiteralPath $path).ProviderPath
if ($resolved -cne $path) { throw 'schema resolved path changed' }
if (git ls-files -- $relative) { throw 'generated schema is tracked' }
if ($LASTEXITCODE -ne 0) { throw 'schema ownership check failed' }
if ((Get-Item -LiteralPath $resolved -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'schema is a reparse point' }
$archive = Join-Path $runRoot 'generated-windows-schema.json'
[IO.File]::Copy($resolved, $archive, $false)
$sha = (Get-FileHash -LiteralPath $resolved).Hash.ToLowerInvariant()
if ($sha -ne (Get-FileHash -LiteralPath $archive).Hash.ToLowerInvariant()) { throw 'schema archive digest differs' }
Remove-Item -LiteralPath $resolved
$schema = [ordered]@{path=$resolved; archive=$archive; sha256=$sha; absent=!(Test-Path -LiteralPath $resolved)}
[ordered]@{fixtures=$rows; unexpectedNewRoots=$newRoots; schema=$schema; priorDeniedPathsRetried=$false; currentR2Touched=$false} |
    ConvertTo-Json -Depth 8 | Set-Content -LiteralPath "$runRoot/cleanup.json" -Encoding utf8NoBOM
"fixtures=$($rows.Count) remaining=$(@($rows | Where-Object { !$_.absent }).Count) newRoots=$($newRoots.Count) schemaAbsent=$($schema.absent)"

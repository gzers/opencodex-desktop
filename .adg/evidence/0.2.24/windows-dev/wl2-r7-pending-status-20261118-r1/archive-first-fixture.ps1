$ErrorActionPreference = 'Stop'
$run = $PSScriptRoot
$first = (Resolve-Path -LiteralPath (Join-Path $run '../r7-pending-status-20261118-r1')).ProviderPath
$originalLog = Join-Path $first 'direct-00/stdout.log'
$pattern = '(?m)^.*r7-pending-attachment-failure-retained=(?<path>[^\r\n]+)?$'
$matchesFound = [regex]::Matches([IO.File]::ReadAllText($originalLog), $pattern)
if ($matchesFound.Count -ne 1) { throw 'expected one retained fixture' }
$root = $matchesFound[0].Groups['path'].Value
if ($root -cne 'C:\Users\15119\AppData\Local\Temp\.tmpGJQlrt' -or
    (Resolve-Path -LiteralPath $root).ProviderPath -cne $root -or
    (Split-Path -Parent $root) -cne (Resolve-Path -LiteralPath ([IO.Path]::GetTempPath())).ProviderPath.TrimEnd('\')) {
    throw 'fixture root identity mismatch'
}
$data = Join-Path $root 'D'
$action = Join-Path $data 'before-resume-pending-774dea26-d533-45d4-9694-2291184b5269'
$recordNames = @('v2-record-000001.json', 'v2-record-000002.json')
function Assert-Children($Parent, $Expected) {
    $item = Get-Item -LiteralPath $Parent -Force
    if (!$item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw 'not an original ordinary fixture directory'
    }
    $actual = @(Get-ChildItem -LiteralPath $Parent -Force | Sort-Object Name | ForEach-Object Name)
    if (($actual -join '|') -cne ($Expected -join '|')) { throw 'fixture membership differs' }
}
function Assert-Membership {
    Assert-Children $root @('D')
    Assert-Children $data @((Split-Path -Leaf $action))
    Assert-Children $action $recordNames
}
Assert-Membership
$archive = Join-Path $run 'first-fixture-archive'
New-Item -ItemType Directory -Path $archive -ErrorAction Stop | Out-Null
$rows = @(foreach ($name in $recordNames) {
    $source = Join-Path $action $name
    $item = Get-Item -LiteralPath $source -Force
    if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'unexpected file type' }
    $streams = @(Get-Item -LiteralPath $source -Stream * | Select-Object -ExpandProperty Stream)
    if (($streams -join '|') -cne ':$DATA') { throw 'additional stream' }
    $target = Join-Path $archive $name
    [IO.File]::Copy($source, $target, $false)
    $digest = (Get-FileHash -LiteralPath $source).Hash.ToLowerInvariant()
    if ((Get-FileHash -LiteralPath $target).Hash.ToLowerInvariant() -cne $digest) { throw 'archive bytes differ' }
    [ordered]@{path=$source; archive=$name; bytes=$item.Length; sha256=$digest; attributes=[string]$item.Attributes}
})
$report = [ordered]@{root=$root; directoryPaths=@($root,$data,$action); files=$rows; originalLog=$originalLog; archiveVerified=$true}
$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $archive 'archive-plan.json') -Encoding utf8NoBOM
Assert-Membership
foreach ($row in $rows) {
    if ((Get-FileHash -LiteralPath $row.path).Hash.ToLowerInvariant() -cne $row.sha256) { throw 'source changed before cleanup' }
}
foreach ($row in $rows) { Remove-Item -LiteralPath $row.path }
foreach ($directory in @($action,$data,$root)) {
    Assert-Children $directory @()
    Remove-Item -LiteralPath $directory
}
$report['rootAbsent'] = !(Test-Path -LiteralPath $root)
$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $archive 'cleanup.json') -Encoding utf8NoBOM
if (!$report.rootAbsent) { throw 'fixture remains' }
'first fixture archived and absent'

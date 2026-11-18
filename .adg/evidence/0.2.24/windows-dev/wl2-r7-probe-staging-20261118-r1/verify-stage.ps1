$ErrorActionPreference='Stop'
$expected=Get-Content -Raw -LiteralPath (Join-Path $PSScriptRoot 'controller-session/staging.json') | ConvertFrom-Json
$root='C:\Users\Public\ocx-r7-isolated-owner-range-r1'
if ($expected.publicRoot -cne $root) { throw 'Different staged root' }
$sections=[Security.AccessControl.AccessControlSections]::Access
$paths=[ordered]@{root=$root;input=(Join-Path $root 'input');output=(Join-Path $root 'out01')}
foreach ($key in $paths.Keys) {
    $item=Get-Item -LiteralPath $paths[$key] -Force
    if (!$item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw "Unexpected directory: $key" }
    if ((Get-Acl -LiteralPath $item.FullName).GetSecurityDescriptorSddlForm($sections) -cne $expected.acl.$key) { throw "Changed ACL: $key" }
}
$children=@(Get-ChildItem -LiteralPath @($paths.root,$paths.input,$paths.output) -Force)
$seen=@($children | ForEach-Object { $_.FullName.Substring($root.Length+1).Replace('\','/') } | Sort-Object)
if (($seen -join '|') -cne (($expected.members | Sort-Object) -join '|')) { throw 'Public membership changed' }
$readback=@(foreach($file in $expected.files) {
    $actual=Get-Item -LiteralPath $file.path -Force
    if ($actual.PSIsContainer -or ($actual.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $actual.Length -ne $file.bytes -or
        (Get-Acl -LiteralPath $file.path).GetSecurityDescriptorSddlForm($sections) -cne $file.acl) { throw "Staged file metadata differs: $($file.path)" }
    $sha=(Get-FileHash -LiteralPath $file.path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($sha -cne $file.sha256) { throw "Staged file bytes differ: $($file.path)" }
    @{path=$file.path;bytes=$actual.Length;sha256=$sha}
})
$profileFixturePresent=$true
try { [void](Get-Item -LiteralPath $expected.profileFixture -Force -ErrorAction Stop) }
catch [System.Management.Automation.ItemNotFoundException] { $profileFixturePresent=$false }
if ($profileFixturePresent) { throw 'Profile fixture exists before dispatch' }
$legacy=@(Get-ChildItem -LiteralPath 'C:\Users\Public\ocx-r7-isolated-owner-nine-r2\out01' -Force | Select-Object Name,Length)
$prior=Get-Content -Raw -LiteralPath 'C:\Users\Public\ocx-r7-isolated-owner-nine-r1\out01\Host-fixture-result.json' | ConvertFrom-Json
$selected=Get-Content -Raw -LiteralPath 'C:\Users\Public\ocx-r7-isolated-owner-nine-r1\out01\Host-result.json' | ConvertFrom-Json
$prefix='r7-initial-image-scans='
$rows=@(Select-String -LiteralPath 'C:\Users\Public\ocx-r7-isolated-owner-nine-r1\out01\Host.stdout.log' -SimpleMatch $prefix)
if (!$rows.Count) { throw 'Archived nine-r1 scan receipt missing' }
$scans=@(foreach($row in $rows) {
    ($row.Line.Substring($row.Line.IndexOf($prefix)+$prefix.Length) | ConvertFrom-Json).scans
})
$live=@(Get-Process -Id 21500,75744,95724 -ErrorAction SilentlyContinue | Select-Object Id,ProcessName,SessionId)
@{
    readUtc=[DateTime]::UtcNow.ToString('o');files=$readback;members=$seen;profileFixturePresent=$false
    nineR2Receipts=$legacy;rangeReceipts=@(Get-ChildItem -LiteralPath $paths.output -Force | Select-Object Name,Length)
    nineR1=@{elapsedMs=$prior.elapsedMs;firstError=$selected.firstError;initialScanCount=$scans.Count;pid=$selected.pid}
    observedProcesses=$live;processCreationTimeVerified=$false;nativeOrdinaryAdmissionVerified=$false;structuralNoResumeReviewed=$true
} | ConvertTo-Json -Depth 10
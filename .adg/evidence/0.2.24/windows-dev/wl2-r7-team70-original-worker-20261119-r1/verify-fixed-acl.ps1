$ErrorActionPreference = 'Stop'
$root = 'C:\Users\Public\ocx-r7-isolated-owner-range-r1'
$baseline = '.adg/evidence/0.2.24/windows-dev/wl2-r7-team68-readback-stage-20261119-r1'
$expected = Get-Content -Raw -LiteralPath (Join-Path $baseline 'stage.stdout.json') | ConvertFrom-Json
$access = [Security.AccessControl.AccessControlSections]::Access
function Plain([string]$path, [bool]$directory) {
    $member = Get-Item -LiteralPath $path -Force
    if ($member.FullName -cne $path -or $member.PSIsContainer -ne $directory -or ($member.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw "Unexpected path or type at $path" }
    $member
}
function Members([string]$path) {
    @(Get-ChildItem -LiteralPath $path -Force | Select-Object -ExpandProperty FullName | Sort-Object)
}
if ($expected.root -cne $root -or @($expected.after).Count -ne 7) { throw 'Unexpected staging baseline.' }
$dirs = [ordered]@{ root=$root; input=(Join-Path $root 'input'); output=(Join-Path $root 'out01') }
$acls = [ordered]@{}
foreach ($part in $dirs.Keys) {
    $null = Plain $dirs[$part] $true
    $acls[$part] = (Get-Acl -LiteralPath $dirs[$part]).GetSecurityDescriptorSddlForm($access)
    if ($acls[$part] -cne $expected.directoryAcl.$part) { throw "Directory ACL mismatch at $part" }
}
foreach ($ancestor in 'C:\','C:\Users','C:\Users\Public') { $null = Plain $ancestor $true }
foreach ($part in 'root','input') {
    $expectedPaths = @($expected.after | Where-Object { (Split-Path -Parent $_.path) -ceq $dirs[$part] } | Select-Object -ExpandProperty path)
    if ($part -ceq 'root') { $expectedPaths += @($dirs.input, $dirs.output) }
    if (Compare-Object @($expectedPaths | Sort-Object) @(Members $dirs[$part]) -CaseSensitive) { throw "Unexpected member set in $part" }
}
$matched = foreach ($row in $expected.after) {
    $file = Plain $row.path $false
    $sha = (Get-FileHash -LiteralPath $row.path -Algorithm SHA256).Hash.ToLowerInvariant()
    $acl = (Get-Acl -LiteralPath $row.path).GetSecurityDescriptorSddlForm($access)
    if ($file.Length -ne $row.bytes -or $sha -cne $row.sha256 -or $acl -cne $row.acl) { throw "Byte or ACL mismatch at $($row.path)" }
    [ordered]@{path=$file.FullName; bytes=$file.Length; sha256=$sha; acl=$acl}
}
$outputMembers = @(Members $dirs.output)
foreach ($path in $outputMembers) { $null = Plain $path $false }
if ($outputMembers | Where-Object { (Split-Path -Leaf $_) -cmatch '^(Explicit|DiagnosticAfter|Readback)' }) { throw 'A subsequent action already has output.' }
$pair = & powershell.exe -NoProfile -File (Join-Path $baseline 'verify-deployed-pair.ps1')
$pairRaw = $LASTEXITCODE
if ($pairRaw -ne 0) { throw "Fixed pair check failed with $pairRaw" }
$pairResult = $pair | ConvertFrom-Json
if ($pairResult.actualPairAccepted -ne $true -or $pairResult.executedDesktopAction -ne $false) { throw 'Unexpected pair check result.' }
[ordered]@{ matched=$true; root=$root; reparseAbsent=$true; observedUtc=[DateTime]::UtcNow.ToString('o'); directoryAcl=$acls; files=@($matched); outputMembers=$outputMembers; subsequentArtifactsAbsent=$true; pairRaw=$pairRaw; pair=$pairResult } | ConvertTo-Json -Depth 8

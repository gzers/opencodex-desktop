$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath('C:/Users/Public/ocx-r7-isolated-owner-range-r1')
$expected = Get-Content -Raw -LiteralPath '.adg/evidence/0.2.24/windows-dev/wl2-r7-probe-staging-20261118-r1/controller-session/staging.json' | ConvertFrom-Json
$source = Resolve-Path -LiteralPath '.adg/evidence/0.2.24/windows-dev/wl2-r7-readback-timing-observer-20261118-r1/observe-r7-readback-timing.ps1'
$sha = '5567494d2174270d77bac6f862ab789c0d43eb3aaab7689e4e5a20e771a2f426'
$name = 'observe-r7-readback-timing.ps1'
$target = Join-Path $root $name
if ($root -cne $expected.publicRoot -or (Test-Path -LiteralPath $target) -or (Get-FileHash -LiteralPath $source.Path).Hash -ine $sha) { throw 'Fixed timing stage precondition differs.' }
$sections = [Security.AccessControl.AccessControlSections]::Access
$dirs = [ordered]@{root=$root;input=(Join-Path $root 'input');output=(Join-Path $root 'out01')}
function ReadFixedInventory([bool]$WithTiming) {
    foreach ($key in $dirs.Keys) {
        $item = Get-Item -LiteralPath $dirs[$key] -Force
        if (!$item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or
            (Get-Acl -LiteralPath $item.FullName).GetSecurityDescriptorSddlForm($sections) -cne $expected.acl.$key) { throw 'Fixed folder or ACL differs.' }
    }
    $expectedPaths = @($expected.members)
    if ($WithTiming) { $expectedPaths += $name }
    $actualPaths = @(Get-ChildItem -LiteralPath @($dirs.root,$dirs.input,$dirs.output) -Force | ForEach-Object {
        $_.FullName.Substring($root.Length+1).Replace([IO.Path]::DirectorySeparatorChar, [char]'/')
    })
    if ((($actualPaths | Sort-Object) -join '|') -cne (($expectedPaths | Sort-Object) -join '|')) { throw 'Fixed placement membership differs.' }
    foreach ($file in $expected.files) {
        $item = Get-Item -LiteralPath $file.path -Force
        $actualSha = (Get-FileHash -LiteralPath $file.path).Hash.ToLowerInvariant()
        if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $item.Length -ne $file.bytes -or
            $actualSha -cne $file.sha256 -or (Get-Acl -LiteralPath $file.path).GetSecurityDescriptorSddlForm($sections) -cne $file.acl) {
            throw "Fixed file bytes or ACL differ: $($file.path)"
        }
        [ordered]@{ path=$file.path; bytes=$item.Length; sha256=$actualSha; acl=$file.acl }
    }
}
$before = @(ReadFixedInventory $false)
[IO.File]::Copy($source.Path, $target, $false)
$after = @(ReadFixedInventory $true)
$item = Get-Item -LiteralPath $target -Force
$acl = (Get-Acl -LiteralPath $target).GetSecurityDescriptorSddlForm($sections)
$launcher = $after | Where-Object { $_.path -ieq (Join-Path $root 'launch-r7-partial-owner.ps1') }
if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or
    $item.Length -ne (Get-Item -LiteralPath $source.Path).Length -or (Get-FileHash -LiteralPath $target).Hash -ine $sha -or
    $acl -cne $launcher.acl -or $before.Count -ne $after.Count) { throw 'Timing observer readback differs.' }
[ordered]@{ governance='bebdfa503f374d6115b9fa313b9c62b213009278'; root=$root; originalFiles=$after;
    added = [ordered]@{ path=$target; bytes=$item.Length; sha256=$sha; acl=$acl }; observedUtc=[DateTime]::UtcNow.ToString('o') } | ConvertTo-Json -Depth 10

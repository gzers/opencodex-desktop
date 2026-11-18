param([switch]$Apply)
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..\..\..\..')).Path
$root = 'C:\Users\Public\ocx-r7-partial-owner-r1'
$profile = 'C:\Users\ocxr7owner'
$source = '18833e7b65c4cb48316f37ee15a3dc584650877e'
$publicParent = Split-Path -Parent $root
$inputDirectory = Join-Path $root 'input'
$outputDirectory = Join-Path $root 'out01'
$controller = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$utf8 = [Text.UTF8Encoding]::new($false)

function RequirePlain([string]$Path, [bool]$Directory) {
    $item = Get-Item -LiteralPath $Path -Force
    if ([bool]$item.PSIsContainer -ne $Directory -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "Unexpected object at $Path"
    }
    return $item
}

function IsAbsent([string]$Path) {
    try { [void][IO.File]::GetAttributes($Path); return $false }
    catch [IO.FileNotFoundException] { return $true }
    catch [IO.DirectoryNotFoundException] { return $true }
}

function Sha([string]$Path) {
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function MakeDirectory([string]$Path, [string]$Sddl) {
    if (!(IsAbsent $Path)) { throw "Staging path already exists: $Path" }
    $security = [Security.AccessControl.DirectorySecurity]::new()
    $security.SetSecurityDescriptorSddlForm($Sddl)
    [void][IO.Directory]::CreateDirectory($Path, $security)
    [void](RequirePlain $Path $true)
    $sections = [Security.AccessControl.AccessControlSections]::Owner -bor [Security.AccessControl.AccessControlSections]::Access
    $actual = Get-Acl -LiteralPath $Path
    if ($actual.GetSecurityDescriptorSddlForm($sections) -cne $security.GetSecurityDescriptorSddlForm($sections)) {
        throw "Staging directory owner or access rules differ: $Path"
    }
}

function CopyExact($Item) {
    $reader = [IO.File]::Open($Item.from, 'Open', 'Read', 'Read')
    try {
        $writer = [IO.File]::Open($Item.to, 'CreateNew', 'Write', 'Read')
        try { $reader.CopyTo($writer); $writer.Flush($true) } finally { $writer.Dispose() }
    } finally { $reader.Dispose() }
    if ((RequirePlain $Item.to $false).Length -ne $Item.bytes -or (Sha $Item.to) -cne $Item.sha256) {
        throw "Staged input differs: $($Item.to)"
    }
}

$test = @((Get-Content -Raw -LiteralPath (Join-Path $PSScriptRoot 'compile-test-05\artifacts.json') | ConvertFrom-Json))
if ($test.Count -ne 1 -or $test[0].sha256 -cne '523bdc65235a90d85ac77da0f2b55266194f3dbb86077e91721729714f8df0d5') {
    throw 'The final MSVC test artifact record differs.'
}
$testPath = Join-Path $PSScriptRoot 'compile-test-05\opencodex_desktop_lib-bdc9ec5a3f66e096.exe'
if ($test[0].path -cne $testPath) { throw 'The final test artifact path differs.' }
$bundle = 'C:\Users\Public\ocx-r6-official-existing-20261117-r2\input\runtime-bundle-0.2.24-win32-x64.tar.gz'
$staging = @(
    @{from=$testPath; to=(Join-Path $inputDirectory 'fixture-test.exe'); expected=$test[0].sha256},
    @{from=$bundle; to=(Join-Path $inputDirectory 'runtime-bundle-0.2.24-win32-x64.tar.gz'); expected='14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7'}
)
foreach ($name in @('launch-r7-partial-owner.ps1', 'launch-r7-partial-owner.cmd')) {
    $path = 'test/windows024-native/' + $name
    $checkout = Join-Path $repo $path
    $object = git -C $repo rev-parse ($source + ':' + $path)
    if ($LASTEXITCODE -ne 0) { throw "Cannot resolve committed launcher: $name" }
    $workingObject = git -C $repo hash-object --path=$path $checkout
    if ($LASTEXITCODE -ne 0 -or $workingObject -cne $object) { throw "Launcher differs from the committed input: $name" }
    $staging += @{from=$checkout; to=(Join-Path $root $name); gitBlob=$object}
}
foreach ($item in $staging) {
    $item.bytes = (RequirePlain $item.from $false).Length
    $item.sha256 = Sha $item.from
    if ($item.expected -and $item.sha256 -cne $item.expected) { throw "Input digest differs: $($item.from)" }
}
[void](RequirePlain 'C:\Users' $true)
[void](RequirePlain $publicParent $true)
$available = IsAbsent $root
$user = Get-LocalUser -Name 'ocxr7owner' -ErrorAction SilentlyContinue
$sid = $null
$session = $null
if ($user -and $user.Enabled) {
    $sid = $user.SID.Value
    $sessionRows = @(quser.exe | Where-Object { $_ -match '^\s*>?\s*ocxr7owner\s+\S+\s+(\d+)\s+Active\s+' })
    if ($LASTEXITCODE -ne 0) { throw 'Unable to query interactive sessions.' }
    if ($sessionRows.Count -eq 1 -and $sessionRows[0] -match '^\s*>?\s*ocxr7owner\s+\S+\s+(\d+)\s+Active\s+') {
        $session = [int]$Matches[1]
    }
}
$ready = $false
if ($sid -and $session -gt 0 -and $available) {
    $profileKey = 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList\' + $sid
    $registered = (Get-ItemProperty -LiteralPath $profileKey -Name ProfileImagePath).ProfileImagePath
    $ready = [Environment]::ExpandEnvironmentVariables($registered) -ceq $profile
    if ($ready) { [void](RequirePlain $profile $true) }
}
$receipt = [ordered]@{schema=1; auditedUtc=[DateTime]::UtcNow.ToString('o'); source=$source; input=$staging;
    stagingRoot=$root; activeSession=$session; accountSid=$sid; ready=$ready; staged=$false}
if ($Apply) {
    if (!$ready) { throw 'A dedicated active desktop, exact Profile and absent staging root are required.' }
    $common = ('O:{0}D:P(A;OICI;FA;;;{0})(A;OICI;FA;;;SY)' -f $controller)
    $readonly = $common + ('(A;OICI;0x1200a9;;;{0})' -f $sid)
    $outputAcl = $common + ('(A;OICI;0x1201bf;;;{0})' -f $sid)
    MakeDirectory $root $readonly
    MakeDirectory $inputDirectory $readonly
    MakeDirectory $outputDirectory $outputAcl
    foreach ($item in $staging) { CopyExact $item }
    $manifest = [ordered]@{schema=1; sid=$sid; session=$session; testSha256=$staging[0].sha256; bundleSha256=$staging[1].sha256}
    $stream = [IO.File]::Open((Join-Path $inputDirectory 'manifest.json'), 'CreateNew', 'Write', 'Read')
    try {
        $bytes = $utf8.GetBytes(($manifest | ConvertTo-Json -Compress))
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Flush($true)
    } finally { $stream.Dispose() }
    $receipt.staged = $true
}
$receipt | ConvertTo-Json -Depth 8
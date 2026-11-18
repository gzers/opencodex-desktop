param([Parameter(Mandatory)][ValidateSet('Prepare','Stage')][string]$Action)

$ErrorActionPreference = 'Stop'
$governance = '55dff7d7b6aad0357d310757981641c0f38d5a03'
$sourceBase = '36b70383eac5c44dffc51d34a4c4673cd8a3dbf7'
$sourceRoot = 'C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code'
$sourceRelative = 'test/windows024-native/launch-r7-partial-owner.ps1'
$sourceFile = Join-Path $sourceRoot $sourceRelative
$observerName = 'observe-r7-readback-timing.ps1'
$candidateObserver = Join-Path $PSScriptRoot $observerName
$evidenceRoot = Split-Path -Parent $PSScriptRoot
$previous = Get-Content -Raw -LiteralPath (Join-Path $evidenceRoot 'wl2-r7-team66-timing-stage-20261118-r1/stage-stdout.json') | ConvertFrom-Json
$baseline = Get-Content -Raw -LiteralPath (Join-Path $evidenceRoot 'wl2-r7-probe-staging-20261118-r1/controller-session/staging.json') | ConvertFrom-Json
$changes = Get-Content -Raw -LiteralPath (Join-Path $evidenceRoot 'wl2-r7-range-readback-pins-20261119-r1/pins-ps51.stdout.json') | ConvertFrom-Json
$root = [IO.Path]::GetFullPath('C:/Users/Public/ocx-r7-isolated-owner-range-r1')
$dirs = [ordered]@{root=$root; input=(Join-Path $root 'input'); output=(Join-Path $root 'out01')}
$sections = [Security.AccessControl.AccessControlSections]::Access
$oldLauncher = 'b7207c94ceb391230639fb6230e4fe228faed874e39ece68e75c27f2d3487b4f'
$oldObserver = '5567494d2174270d77bac6f862ab789c0d43eb3aaab7689e4e5a20e771a2f426'
$next = @{
    'launch-r7-partial-owner.ps1' = @{bytes=30158; sha256='6b553204560d59ba7a292f64c322404028eceb1a0899661f5f9dc9d76953e990'}
    $observerName = @{bytes=6394; sha256='f3a0c5ed9f8ba698d0b5d3e37d1b1209b4b88fa5ee4714a5fe06c1ff000aba5f'}
}
$encoding = [Text.UTF8Encoding]::new($false, $true)

function Digest([byte[]]$Bytes) {
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Bytes)).ToLowerInvariant()
}

function ReplaceOnce([string]$Text, [string]$Before, [string]$After) {
    $index = $Text.IndexOf($Before, [StringComparison]::Ordinal)
    if ($index -lt 0 -or $index -ne $Text.LastIndexOf($Before, [StringComparison]::Ordinal)) {
        throw ('Expected one occurrence of ' + $Before)
    }
    $Text.Substring(0, $index) + $After + $Text.Substring($index + $Before.Length)
}

function ReadInventory([bool]$Staged) {
    foreach ($path in @('C:/Users', 'C:/Users/Public') + @($dirs.Values)) {
        $item = Get-Item -LiteralPath $path -Force
        if (!$item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw "Directory differs: $path" }
    }
    foreach ($key in $dirs.Keys) {
        if ((Get-Acl -LiteralPath $dirs[$key]).GetSecurityDescriptorSddlForm($sections) -cne $baseline.acl.$key) { throw "Directory ACL differs: $key" }
    }
    $members = @(Get-ChildItem -LiteralPath @($dirs.Values) -Force | ForEach-Object {
        $_.FullName.Substring($root.Length + 1).Replace([IO.Path]::DirectorySeparatorChar, [char]'/')
    })
    $expectedMembers = @($baseline.members) + @($observerName)
    if ((($members | Sort-Object) -join '|') -cne (($expectedMembers | Sort-Object) -join '|')) { throw 'Fixed placement membership differs.' }
    foreach ($file in @($previous.originalFiles) + @($previous.added)) {
        $item = Get-Item -LiteralPath $file.path -Force
        $expected = $file
        if ($Staged -and $next.ContainsKey($item.Name)) { $expected = $next[$item.Name] }
        $acl = (Get-Acl -LiteralPath $file.path).GetSecurityDescriptorSddlForm($sections)
        $hash = (Get-FileHash -LiteralPath $file.path).Hash.ToLowerInvariant()
        if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or
            $item.Length -ne $expected.bytes -or $hash -cne $expected.sha256 -or $acl -cne $file.acl) {
            throw "Fixed file bytes or ACL differ: $($file.path)"
        }
        [ordered]@{path=$item.FullName; bytes=$item.Length; sha256=$hash; acl=$acl}
    }
}

function ReadHandle($Handle) {
    $Handle.Position = 0
    $buffer = [byte[]]::new([int]$Handle.Length)
    $Handle.ReadExactly($buffer)
    ,$buffer
}

function GitResult([string]$Root, [string[]]$GitArgs) {
    $result = & git -C $Root @GitArgs
    if ($LASTEXITCODE -ne 0 -or @($result).Count -ne 1) { throw ('Git read failed: ' + ($GitArgs -join ' ')) }
    $result
}

if ($root -cne $baseline.publicRoot -or $root -cne $previous.root) { throw 'Fixed root differs.' }
$before = @(ReadInventory $false)
$launcherPath = Join-Path $root 'launch-r7-partial-owner.ps1'
$observerPath = Join-Path $root $observerName
$launcher = $encoding.GetString([IO.File]::ReadAllBytes($launcherPath))
foreach ($change in $changes.launcherReplacements) { $launcher = ReplaceOnce $launcher $change.before $change.after }
$observer = ReplaceOnce ($encoding.GetString([IO.File]::ReadAllBytes($observerPath))) $oldLauncher $next['launch-r7-partial-owner.ps1'].sha256
$payloads = [ordered]@{'launch-r7-partial-owner.ps1'=$encoding.GetBytes($launcher); $observerName=$encoding.GetBytes($observer)}
foreach ($name in $payloads.Keys) {
    $bytes = $payloads[$name]
    if ($bytes.Length -ne $next[$name].bytes -or (Digest $bytes) -cne $next[$name].sha256) { throw "Candidate bytes differ: $name" }
    $tokens = $null
    $errors = $null
    $null = [Management.Automation.Language.Parser]::ParseInput($encoding.GetString($bytes), [ref]$tokens, [ref]$errors)
    if ($errors.Count -ne 0) { throw ($errors | Out-String) }
}
$baseBlob = GitResult $sourceRoot @('rev-parse', ($sourceBase + ':' + $sourceRelative))
if ($baseBlob -cne (GitResult $sourceRoot @('hash-object', '--no-filters', $launcherPath))) { throw 'Original Git and deployed launcher bytes differ.' }

if ($Action -eq 'Prepare') {
    if ((Digest ([IO.File]::ReadAllBytes($sourceFile))) -cne $oldLauncher -or (Test-Path -LiteralPath $candidateObserver)) { throw 'Candidate preparation precondition differs.' }
    $stream = [IO.File]::Open($candidateObserver, 'CreateNew', 'Write', 'None')
    try { $stream.Write($payloads[$observerName]); $stream.Flush($true) }
    finally { $stream.Dispose() }
    [IO.File]::WriteAllBytes($sourceFile, $payloads['launch-r7-partial-owner.ps1'])
    [ordered]@{governance=$governance; action=$Action; gitBaseBlob=$baseBlob; candidate=$next; before=$before} | ConvertTo-Json -Depth 10
    return
}

$sourceHead = GitResult $sourceRoot @('rev-parse', 'HEAD')
if ((GitResult $sourceRoot @('rev-parse', 'HEAD^')) -cne $sourceBase -or
    (GitResult $sourceRoot @('diff', '--name-only', $sourceBase, $sourceHead)) -cne $sourceRelative) { throw 'Source commit range differs.' }
$nextBlob = GitResult $sourceRoot @('rev-parse', ($sourceHead + ':' + $sourceRelative))
if ($nextBlob -cne (GitResult $sourceRoot @('hash-object', '--no-filters', $sourceFile)) -or
    (Digest ([IO.File]::ReadAllBytes($sourceFile))) -cne $next['launch-r7-partial-owner.ps1'].sha256 -or
    (Digest ([IO.File]::ReadAllBytes($candidateObserver))) -cne $next[$observerName].sha256) { throw 'Candidate Git or evidence bytes differ.' }

$launcherHandle = [IO.File]::Open($launcherPath, 'Open', 'ReadWrite', 'None')
try {
    $observerHandle = [IO.File]::Open($observerPath, 'Open', 'ReadWrite', 'None')
    try {
        if ((Digest (ReadHandle $launcherHandle)) -cne $oldLauncher -or (Digest (ReadHandle $observerHandle)) -cne $oldObserver) {
            throw 'Locked original pair differs.'
        }
        $handles = [ordered]@{'launch-r7-partial-owner.ps1'=$launcherHandle; $observerName=$observerHandle}
        foreach ($name in $handles.Keys) {
            $handle = $handles[$name]
            $handle.Position = 0
            $handle.Write($payloads[$name])
            $handle.SetLength($payloads[$name].Length)
            $handle.Flush($true)
        }
        foreach ($name in $handles.Keys) {
            if ((Digest (ReadHandle $handles[$name])) -cne $next[$name].sha256) { throw "Locked readback differs: $name" }
        }
    } finally { $observerHandle.Dispose() }
} finally { $launcherHandle.Dispose() }
$after = @(ReadInventory $true)
if ((GitResult $sourceRoot @('hash-object', '--no-filters', $launcherPath)) -cne $nextBlob) { throw 'Deployed launcher differs from Git bytes.' }
[ordered]@{governance=$governance; action=$Action; source=$sourceHead; gitBaseBlob=$baseBlob; gitLauncherBlob=$nextBlob;
    root=$root; directoryAcl=$baseline.acl; before=$before; after=$after; observedUtc=[DateTime]::UtcNow.ToString('o')} | ConvertTo-Json -Depth 10

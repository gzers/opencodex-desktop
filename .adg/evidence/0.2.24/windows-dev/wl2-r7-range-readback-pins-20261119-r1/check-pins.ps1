param([Parameter(Mandatory)][string]$Fixture)

$ErrorActionPreference = 'Stop'
$utf8 = [Text.UTF8Encoding]::new($false, $true)

function Digest([byte[]]$Bytes) {
    $hasher = [Security.Cryptography.SHA256]::Create()
    try { [BitConverter]::ToString($hasher.ComputeHash($Bytes)).Replace('-', '').ToLowerInvariant() }
    finally { $hasher.Dispose() }
}

function ReplaceOnce([string]$Text, [string]$Before, [string]$After) {
    $index = $Text.IndexOf($Before, [StringComparison]::Ordinal)
    if ($index -lt 0 -or $index -ne $Text.LastIndexOf($Before, [StringComparison]::Ordinal)) {
        throw ('Expected exactly one occurrence of ' + $Before)
    }
    $Text.Substring(0, $index) + $After + $Text.Substring($index + $Before.Length)
}

function ParseScript([string]$Text) {
    $tokens = $null
    $errors = $null
    $ast = [Management.Automation.Language.Parser]::ParseInput($Text, [ref]$tokens, [ref]$errors)
    if ($errors.Count -ne 0) { throw ($errors | Out-String) }
    $ast
}

function GetPinGuard($Ast) {
    $guards = @($Ast.EndBlock.Statements | Where-Object {
        $_ -is [Management.Automation.Language.IfStatementAst] -and
        $_.Extent.Text.Contains('The fixed DiagnosticAfter launcher bytes differ.')
    })
    if ($guards.Count -ne 1) { throw 'Could not identify the exact launcher digest guard.' }
    [scriptblock]::Create($guards[0].Extent.Text)
}

$names = @('launch-r7-partial-owner.cmd', 'launch-r7-partial-owner.ps1', 'observe-r7-readback-timing.ps1')
$expected = @(
    '85f56dd180ee907b70e68ed531d90a4e5a480cb104264bdad4890053e5da60f2',
    'b7207c94ceb391230639fb6230e4fe228faed874e39ece68e75c27f2d3487b4f',
    '5567494d2174270d77bac6f862ab789c0d43eb3aaab7689e4e5a20e771a2f426'
)
$source = @(for ($i = 0; $i -lt $names.Count; $i++) {
    $bytes = [IO.File]::ReadAllBytes((Join-Path $Fixture $names[$i]))
    if ((Digest $bytes) -cne $expected[$i]) { throw ('Fixed input bytes differ: ' + $names[$i]) }
    $utf8.GetString($bytes)
})

$changes = @(
    @{ before = 'function ExportExact([string]$Path, [string]$Name, [int]$Limit) {'; after = 'function ExportExact([string]$Path, [string]$Name, [int]$Limit, [IO.FileShare]$Share = [IO.FileShare]::Read) {' },
    @{ before = '[IO.File]::Open($Path, ''Open'', ''Read'', ''Read'')'; after = '[IO.File]::Open($Path, ''Open'', ''Read'', $Share)' },
    @{ before = 'ExportExact $path ("DiagnosticAfter-" + $name) 65536'; after = 'ExportExact $path ("DiagnosticAfter-" + $name) 65536 ([IO.FileShare]::ReadWrite)' }
)
$nextLauncher = $source[1]
foreach ($change in $changes) { $nextLauncher = ReplaceOnce $nextLauncher $change.before $change.after }
$nextLauncherSha = Digest $utf8.GetBytes($nextLauncher)
$nextObserver = ReplaceOnce $source[2] $expected[1] $nextLauncherSha
$nextObserverSha = Digest $utf8.GetBytes($nextObserver)
$null = ParseScript $source[1]
$null = ParseScript $nextLauncher
$oldGuard = GetPinGuard (ParseScript $source[2])
$nextGuard = GetPinGuard (ParseScript $nextObserver)

$cases = @(
    @{ name = 'old-complete'; guard = $oldGuard; launcher = $expected[1]; cmd = $expected[0]; allowed = $true },
    @{ name = 'new-complete'; guard = $nextGuard; launcher = $nextLauncherSha; cmd = $expected[0]; allowed = $true },
    @{ name = 'old-observer-new-launcher'; guard = $oldGuard; launcher = $nextLauncherSha; cmd = $expected[0]; allowed = $false },
    @{ name = 'new-observer-old-launcher'; guard = $nextGuard; launcher = $expected[1]; cmd = $expected[0]; allowed = $false },
    @{ name = 'new-complete-changed-cmd'; guard = $nextGuard; launcher = $nextLauncherSha; cmd = (Digest $utf8.GetBytes($source[0] + ' ')); allowed = $false }
)
$rows = @(foreach ($case in $cases) {
    $launcherSha = $case.cmd
    $launcherScriptSha = $case.launcher
    $errorText = $null
    try { & $case.guard }
    catch { $errorText = $_.Exception.Message }
    $accepted = $null -eq $errorText
    if ($accepted -ne $case.allowed -or (!$accepted -and $errorText -cne 'The fixed DiagnosticAfter launcher bytes differ.')) {
        throw ('Digest guard differed for ' + $case.name)
    }
    [ordered]@{ name = $case.name; accepted = $accepted; firstError = $errorText }
})
[ordered]@{
    powerShell = $PSVersionTable.PSVersion.ToString()
    fixedSha256 = $expected
    candidateLauncherSha256 = $nextLauncherSha
    candidateLauncherBytes = $utf8.GetByteCount($nextLauncher)
    candidateObserverSha256 = $nextObserverSha
    candidateObserverBytes = $utf8.GetByteCount($nextObserver)
    launcherReplacements = $changes
    guardCases = $rows
    materializedCandidate = $false
} | ConvertTo-Json -Depth 6

$ErrorActionPreference = 'Stop'
$root = 'C:/Users/Public/ocx-r7-isolated-owner-range-r1'
$utf8 = [Text.UTF8Encoding]::new($false, $true)

function ReadScript([string]$Name, [string]$Expected) {
    $bytes = [IO.File]::ReadAllBytes((Join-Path $root $Name))
    $sha = [Security.Cryptography.SHA256]::Create()
    try { $digest = [BitConverter]::ToString($sha.ComputeHash($bytes)).Replace('-', '').ToLowerInvariant() }
    finally { $sha.Dispose() }
    if ($digest -cne $Expected) { throw ("Staged SHA-256 differs: " + $Name) }
    $tokens = $null
    $errors = $null
    $ast = [Management.Automation.Language.Parser]::ParseInput($utf8.GetString($bytes), [ref]$tokens, [ref]$errors)
    if ($errors.Count -ne 0) { throw ($errors | Out-String) }
    $ast
}

$launcherSha = '85f56dd180ee907b70e68ed531d90a4e5a480cb104264bdad4890053e5da60f2'
$launcherScriptSha = '6b553204560d59ba7a292f64c322404028eceb1a0899661f5f9dc9d76953e990'
$observerSha = 'f3a0c5ed9f8ba698d0b5d3e37d1b1209b4b88fa5ee4714a5fe06c1ff000aba5f'
$null = ReadScript 'launch-r7-partial-owner.ps1' $launcherScriptSha
$ast = ReadScript 'observe-r7-readback-timing.ps1' $observerSha
$candidates = @($ast.EndBlock.Statements | Where-Object {
    $_ -is [Management.Automation.Language.IfStatementAst] -and
    $_.Extent.Text.Contains('The fixed DiagnosticAfter launcher bytes differ.')
})
if ($candidates.Count -ne 1) { throw 'Staged digest guard not found.' }
$guard = [scriptblock]::Create($candidates[0].Extent.Text)
& $guard
$launcherScriptSha = 'b7207c94ceb391230639fb6230e4fe228faed874e39ece68e75c27f2d3487b4f'
$mixedError = $null
try { & $guard }
catch { $mixedError = $_.Exception.Message }
if ($mixedError -cne 'The fixed DiagnosticAfter launcher bytes differ.') { throw 'Mixed deployed pair was not refused.' }
[ordered]@{powerShell=$PSVersionTable.PSVersion.ToString(); actualPairAccepted=$true; mixedPairFirstError=$mixedError;
    hostReceiptPresent=(Test-Path -LiteralPath (Join-Path $root 'out01/Host-result.json')); executedDesktopAction=$false} | ConvertTo-Json

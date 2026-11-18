param([Parameter(Mandatory)][string]$Observer)
$ErrorActionPreference = 'Stop'
$tokens = $null
$errors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile($Observer, [ref]$tokens, [ref]$errors)
if ($errors.Count -ne 0) { throw ($errors | Out-String) }
foreach ($def in $ast.EndBlock.Statements) {
    if ($def -is [Management.Automation.Language.FunctionDefinitionAst]) {
        . ([ScriptBlock]::Create($def.Extent.Text))
    }
}
$binding = @{ recoveryActionId = '72ed1549-5bb8-4ce2-9044-d6d196f2bba3' }
$result = @{
    mode = 'DiagnosticAfter'
    passed = $true
    attachment = @('first', 'second')
    firstError = $null
    manager = @{ result = @{ response = @{ facts = @{ snapshot = @{ actionId = $binding.recoveryActionId } } } } }
}
$utf8 = [Text.UTF8Encoding]::new($false)
$rows = foreach ($case in @('fast', 'late', 'launcher-failed', 'wrong-action', 'missing-result')) {
    $root = New-Item -Path (Join-Path $PSScriptRoot $case) -ItemType Directory
    $input = Join-Path $root.FullName 'synthetic.json'
    $launcher = Join-Path $root.FullName 'synthetic.cmd'
    $result.passed = $case -ne 'launcher-failed'
    $result.firstError = if ($case -eq 'launcher-failed') { @{ message = 'synthetic raw31'; hresult = -2147024865; line = 17 } }
    $result.manager.result.response.facts.snapshot.actionId = if ($case -eq 'wrong-action') { 'bad-action' } else { $binding.recoveryActionId }
    [IO.File]::WriteAllText($input, ($result | ConvertTo-Json -Depth 12), $utf8)
    $raw = if ($case -eq 'launcher-failed') { 17 } else { 0 }
    $lines = @('@echo off', 'if not "%~1"=="DiagnosticAfter" exit /b 91', 'echo invoked>>"%~dp0invoked.log"')
    if ($case -eq 'late') { $lines += 'powershell.exe -NoLogo -NoProfile -Command "Start-Sleep -Milliseconds 5100"' }
    if ($case -ne 'missing-result') { $lines += 'copy /b "%~dp0synthetic.json" "%~dp0DiagnosticAfter-result.json" >nul' }
    $lines += "exit /b $raw"
    [IO.File]::WriteAllLines($launcher, $lines, $utf8)
    $report = InvokeTimedReadback $launcher $root.FullName $binding
    $duplicate = $null
    try { [void](InvokeTimedReadback $launcher $root.FullName $binding) }
    catch { $duplicate = $_.Exception.Message }
    $calls = @(Get-Content -LiteralPath (Join-Path $root.FullName 'invoked.log')).Count
    if ($report.accepted -ne ($case -eq 'fast') -or $report.launcherRaw -ne $raw -or
        ($case -eq 'late' -and ($report.strictlyWithinBudget -or $report.elapsedUpperBoundMs -lt 5100)) -or
        ($case -eq 'launcher-failed' -and $report.firstError.message -ne 'synthetic raw31') -or
        ($case -eq 'wrong-action' -and $report.firstError -notlike '*recovery action differs*') -or
        ($case -eq 'missing-result' -and ($report.resultSha256 -or !$report.firstError)) -or
        $duplicate -notlike '*attempt already exists*' -or $calls -ne 1) {
        throw "Synthetic assertions failed for $case. $($report | ConvertTo-Json -Depth 12 -Compress)"
    }
    if ($case -ne 'missing-result' -and (Get-FileHash -LiteralPath $input -Algorithm SHA256).Hash -ine $report.resultSha256) {
        throw "Receipt did not bind the exact $case bytes."
    }
    [ordered]@{ case = $case; invocationCount = $calls; duplicateRefusal = $duplicate; timing = $report }
}
[ordered]@{ version = $PSVersionTable.PSVersion.ToString(); cases = @($rows) } | ConvertTo-Json -Depth 16

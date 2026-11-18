param(
    [Parameter(Mandatory)]
    [ValidatePattern('^[0-9a-f]{64}$')]
    [string]$ExpectedExplicitSha256
)

$ErrorActionPreference = 'Stop'

function WriteNewReceipt([string]$Path, $Receipt) {
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes(($Receipt | ConvertTo-Json -Depth 12))
    $file = [IO.File]::Open($Path, 'CreateNew', 'Write', 'Read')
    try {
        $file.Write($bytes, 0, $bytes.Length)
        $file.Flush($true)
    } finally { $file.Dispose() }
}

function ReadReceipt([string]$Path) {
    $file = [IO.File]::Open($Path, 'Open', 'Read', 'Read')
    try {
        if ($file.Length -gt 1048576) { throw 'Receipt exceeds 1 MiB.' }
        $length = [int]$file.Length
        $reader = [IO.BinaryReader]::new($file, [Text.Encoding]::UTF8, $true)
        try { $bytes = $reader.ReadBytes($length) } finally { $reader.Dispose() }
        if ($bytes.Length -ne $length) { throw 'Receipt read was incomplete.' }
    } finally { $file.Dispose() }
    $hash = [Security.Cryptography.SHA256]::Create()
    try { $digest = [BitConverter]::ToString($hash.ComputeHash($bytes)).Replace('-', '').ToLowerInvariant() }
    finally { $hash.Dispose() }
    @{ sha256 = $digest; data = [Text.UTF8Encoding]::new($false, $true).GetString($bytes) | ConvertFrom-Json }
}

function InvokeTimedReadback([string]$Launcher, [string]$OutputDirectory, $Binding) {
    if (![Diagnostics.Stopwatch]::IsHighResolution) { throw 'A high-resolution monotonic clock is required.' }
    if (@(Get-ChildItem -LiteralPath $OutputDirectory -Filter 'DiagnosticAfter-*' -Force).Count -ne 0) {
        throw 'A DiagnosticAfter attempt already exists. Preserve its receipts.'
    }
    $attempt = Join-Path $OutputDirectory 'DiagnosticAfter-independent-timing-attempt.json'
    $timing = Join-Path $OutputDirectory 'DiagnosticAfter-independent-timing.json'
    $result = Join-Path $OutputDirectory 'DiagnosticAfter-result.json'
    WriteNewReceipt $attempt $Binding

    $frequency = [Diagnostics.Stopwatch]::Frequency
    $launcherError = $null
    $global:LASTEXITCODE = $null
    $started = [Diagnostics.Stopwatch]::GetTimestamp()
    try { & $Launcher DiagnosticAfter | Out-Host }
    catch { $launcherError = $_.Exception.Message }
    finally { $finished = [Diagnostics.Stopwatch]::GetTimestamp() }
    $raw = $global:LASTEXITCODE
    $ticks = $finished - $started
    $within = $ticks -ge 0 -and [decimal]$ticks -lt ([decimal]5 * $frequency)
    $receipt = [ordered]@{
        binding = $Binding
        startedQpc = $started.ToString()
        finishedQpc = $finished.ToString()
        qpcFrequency = $frequency.ToString()
        elapsedUpperBoundMs = [Math]::Ceiling([decimal]1000 * $ticks / $frequency)
        originalBudgetMs = 5000
        strictlyWithinBudget = $within
        launcherRaw = $raw
        launcherError = $launcherError
        resultSha256 = $null
        launcherFirstError = $null
        accepted = $false
        firstError = $null
    }

    try {
        $evidence = ReadReceipt $result
        $receipt.resultSha256 = $evidence.sha256
        $receipt.launcherFirstError = $evidence.data.firstError
        if ($null -eq $raw -or $raw -ne 0 -or $null -ne $launcherError) { throw 'Fixed DiagnosticAfter launcher failed.' }
        $data = $evidence.data
        if ($data.mode -cne 'DiagnosticAfter' -or $data.passed -isnot [bool] -or !$data.passed -or
            $data.manager.result.response.facts.snapshot.actionId -cne $Binding.recoveryActionId -or
            @($data.attachment).Count -ne 2) { throw 'The fixed readback result or its recovery action differs.' }
        if (!$within) { throw 'The upper bound reached the original 5-second deadline. Readback timing is unknown.' }
        $receipt.accepted = $true
    } catch {
        $receipt.firstError = if ($null -ne $receipt.launcherFirstError) { $receipt.launcherFirstError }
            elseif ($null -ne $launcherError) { $launcherError }
            elseif ($null -ne $raw -and $raw -ne 0) { "Fixed DiagnosticAfter launcher returned raw $raw." }
            else { $_.Exception.Message }
    }
    WriteNewReceipt $timing $receipt
    $receipt
}

$public = 'C:\Users\Public\ocx-r7-isolated-owner-range-r1'
$profile = 'C:\Users\marenben'
$out = Join-Path $public 'out01'
$launcher = Join-Path $public 'launch-r7-partial-owner.cmd'
$launcherScript = Join-Path $public 'launch-r7-partial-owner.ps1'
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$session = [Diagnostics.Process]::GetCurrentProcess().SessionId
$admin = [Security.Principal.WindowsPrincipal]::new($identity).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if ($identity.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or $session -ne 7 -or $admin -or $env:USERPROFILE -cne $profile) {
    throw 'Use the existing marenben ordinary session 7.'
}
foreach ($path in @('C:\Users', 'C:\Users\Public', $public, $out, $launcher, $launcherScript, (Join-Path $out 'Explicit-result.json'))) {
    if ((Get-Item -LiteralPath $path -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) {
        throw 'A fixed fixture path is a reparse point.'
    }
}
$launcherSha = (Get-FileHash -LiteralPath $launcher -Algorithm SHA256).Hash.ToLowerInvariant()
$launcherScriptSha = (Get-FileHash -LiteralPath $launcherScript -Algorithm SHA256).Hash.ToLowerInvariant()
if ($launcherSha -cne '85f56dd180ee907b70e68ed531d90a4e5a480cb104264bdad4890053e5da60f2' -or
    $launcherScriptSha -cne 'b7207c94ceb391230639fb6230e4fe228faed874e39ece68e75c27f2d3487b4f') {
    throw 'The fixed DiagnosticAfter launcher bytes differ.'
}
$explicit = ReadReceipt (Join-Path $out 'Explicit-result.json')
if ($explicit.sha256 -cne $ExpectedExplicitSha256 -or $explicit.data.mode -cne 'Explicit' -or
    $explicit.data.passed -isnot [bool] -or !$explicit.data.passed -or
    $explicit.data.recoveryActionId -cnotmatch '^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$') {
    throw 'The independently accepted Explicit result is required first.'
}
$binding = [ordered]@{
    sid = $identity.User.Value
    session = $session
    launcherSha256 = $launcherSha
    launcherScriptSha256 = $launcherScriptSha
    explicitResultSha256 = $explicit.sha256
    recoveryActionId = $explicit.data.recoveryActionId
}
$receipt = InvokeTimedReadback $launcher $out $binding
$receipt | ConvertTo-Json -Depth 12
if ($receipt.accepted) { exit 0 }
exit 1

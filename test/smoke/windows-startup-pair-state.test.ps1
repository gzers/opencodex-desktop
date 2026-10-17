param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference = 'Stop'
if (!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7) { throw 'Windows PowerShell7 required' }
. (Join-Path $PSScriptRoot 'windows-startup-pair-state.ps1')
$output = [IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Path $output -Force | Out-Null
$workspace = Join-Path $output ('state-tests-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $workspace | Out-Null
$checks = [Collections.Generic.List[string]]::new()
function Check([string]$Name, [scriptblock]$Run) { & $Run; $checks.Add($Name) }
function Refuses([scriptblock]$Run, [string]$Reason) {
    try { & $Run | Out-Null } catch {
        if (!$_.Exception.Message.Contains($Reason)) { throw }
        return
    }
    throw 'Expected refusal: ' + $Reason
}
$start = [datetimeoffset]'2026-01-01T00:00:00Z'
$authorization = @{ origin = 'human'; explicitHumanStart = $true; automaticContinuation = $false
    scope = 'startup-pair-cold-warm'; authorizedAtUtc = $start.ToString('o'); expiresAtUtc = $start.AddMinutes(30).ToString('o') }
Check 'original80SamplesAndAlternatingOrder' {
    $schedule = @(Get-StartupPairSchedule)
    if ($schedule.Count -ne 80) { throw 'Sample count changed' }
    foreach ($version in @('baseline019', 'candidate0110')) {
        foreach ($kind in @('cache-cold', 'process-warm')) {
            if (@($schedule | Where-Object { $_.version -ceq $version -and $_.kind -ceq $kind }).Count -ne 20) { throw 'Per-mode count changed' }
        }
    }
    if ($schedule[0].version -cne 'baseline019' -or $schedule[4].version -cne 'candidate0110') { throw 'Order drifted' }
}
Check 'newHumanWindowAndCleanupMargin' {
    $null = Assert-StartupPairWindow -Authorization $authorization -Now $start.AddMinutes(1)
    Refuses { Assert-StartupPairWindow -Authorization $authorization -Now $start.AddMinutes(-1) } 'future'
    Refuses { Assert-StartupPairWindow -Authorization $authorization -Now $start.AddMinutes(30) } 'expired'
    Refuses { Assert-StartupPairWindow -Authorization $authorization -Now $start.AddMinutes(24.5) } 'Insufficient'
    Refuses { Assert-StartupPairWindow -Authorization $authorization -Now $start.AddMinutes(24.5) -Phase prepare } 'Insufficient'
    $automatic = $authorization.Clone(); $automatic.automaticContinuation = $true
    Refuses { Assert-StartupPairWindow -Authorization $automatic -Now $start.AddMinutes(1) } 'human'
    $long = $authorization.Clone(); $long.expiresAtUtc = $start.AddMinutes(31).ToString('o')
    Refuses { Assert-StartupPairWindow -Authorization $long -Now $start.AddMinutes(1) } 'thirty'
}
Check 'unadmittedImagesAndPathAliasesRefusedBeforeReading' {
    Refuses { Assert-StartupPairImages -Baseline @{} -Candidate @{ fullCiRunId = 1; candidateRunId = 1 } } 'Exact admitted'
    Refuses { ConvertTo-StartupPairPath 'C:\fixture\..\outside' } 'dot components'
    Refuses { ConvertTo-StartupPairPath 'C:\fixture:stream' } 'streams'
    Refuses { ConvertTo-StartupPairPath 'C:\' } 'non-drive-root'
}
function Fixtures([string]$Name) {
    $run = Join-Path $workspace ($Name + '-run'); New-Item -ItemType Directory -Path $run | Out-Null
    $bindings = foreach ($version in @('baseline019', 'candidate0110')) {
        $root = Join-Path $workspace ($Name + '-' + $version)
        $folder = Join-Path $root 'manager-state'; New-Item -ItemType Directory -Path $folder -Force | Out-Null
        $file = Join-Path $folder 'preferences.json'
        $value = @{ schema_version = 2
            appearance = @{ theme = 'dark'; visual_effects = 'mid'; glow_render = 'css'; interface_scale = 125 }
            shell = @{ auto_panel = $true; launch_with_codex = $true; keep_proxy_on_close = $true; launch_main = $false }
            updates = @{ app_update_auto_check = $true }; maintenance = @{ startup_cleanup = $true } }
        $bytes = [Text.Encoding]::UTF8.GetBytes(($value | ConvertTo-Json -Depth 8) + "`r`n ")
        [IO.File]::WriteAllBytes($file, $bytes)
        [pscustomobject]@{ version = $version; root = $root; file = $file
            sha256 = (Get-FileHash -LiteralPath $file).Hash.ToLowerInvariant() }
    }
    return @{ run = $run; bindings = @($bindings) }
}
Check 'expiredPrepareChangesNoFixtureBytes' {
    $fixture = Fixtures 'expired'
    Refuses { Save-StartupPairPreferences -RunDirectory $fixture.run -Bindings $fixture.bindings -Authorization $authorization -Now $start.AddHours(1) } 'expired'
    foreach ($binding in $fixture.bindings) {
        if ((Get-FileHash -LiteralPath $binding.file).Hash.ToLowerInvariant() -cne $binding.sha256) { throw 'Refusal mutated fixture' }
    }
    if (Test-Path -LiteralPath (Join-Path $fixture.run 'startup-preference-journal.json')) { throw 'Refusal created journal' }
}
Check 'exactBytesRestoredOnNormalCompletion' {
    $fixture = Fixtures 'normal'
    $journal = Save-StartupPairPreferences -RunDirectory $fixture.run -Bindings $fixture.bindings -Authorization $authorization -Now $start.AddMinutes(1)
    try {
        foreach ($binding in $fixture.bindings) {
            $value = Get-Content -LiteralPath $binding.file -Raw | ConvertFrom-Json
            if ($value.appearance.theme -cne 'light' -or $value.shell.keep_proxy_on_close -or $value.updates.app_update_auto_check) { throw 'Seed differs' }
        }
    } finally { Restore-StartupPairPreferences -Journal $journal }
    foreach ($binding in $fixture.bindings) {
        if ((Get-FileHash -LiteralPath $binding.file).Hash.ToLowerInvariant() -cne $binding.sha256) { throw 'Original bytes not restored' }
    }
    Refuses { Save-StartupPairPreferences -RunDirectory $fixture.run -Bindings $fixture.bindings -Authorization $authorization -Now $start.AddMinutes(1) } 'Never reuse'
}
Check 'callerFailureStillRestoresBothOriginals' {
    $fixture = Fixtures 'failed-caller'
    $journal = Save-StartupPairPreferences -RunDirectory $fixture.run -Bindings $fixture.bindings -Authorization $authorization -Now $start.AddMinutes(1)
    try { throw 'synthetic-readiness-failure' } catch {
        if ($_.Exception.Message -cne 'synthetic-readiness-failure') { throw }
    } finally { Restore-StartupPairPreferences -Journal $journal }
    foreach ($binding in $fixture.bindings) {
        if ((Get-FileHash -LiteralPath $binding.file).Hash.ToLowerInvariant() -cne $binding.sha256) { throw 'Failure did not restore bytes' }
    }
}
Check 'secondFixtureWriteFailureRollsBackFirstFixture' {
    $fixture = Fixtures 'failed-seed'
    $readOnly = $fixture.bindings[1].file
    [IO.File]::SetAttributes($readOnly, [IO.FileAttributes]::ReadOnly)
    try {
        $refused = $false
        try {
            Save-StartupPairPreferences -RunDirectory $fixture.run -Bindings $fixture.bindings -Authorization $authorization -Now $start.AddMinutes(1) | Out-Null
        } catch { $refused = $true }
        if (!$refused) { throw 'Read-only fixture unexpectedly accepted a write' }
        foreach ($binding in $fixture.bindings) {
            if ((Get-FileHash -LiteralPath $binding.file).Hash.ToLowerInvariant() -cne $binding.sha256) { throw 'Partial seed not rolled back' }
        }
        $journal = Get-Content -LiteralPath (Join-Path $fixture.run 'startup-preference-journal.json') -Raw | ConvertFrom-Json
        if ($journal.stage -cne 'restored') { throw 'Failed seed restoration not recorded' }
    } finally { [IO.File]::SetAttributes($readOnly, [IO.FileAttributes]::Normal) }
}
Check 'concurrentWriterRefusesWholeRestoreBeforeAnyWrite' {
    $fixture = Fixtures 'concurrent'
    $journal = Save-StartupPairPreferences -RunDirectory $fixture.run -Bindings $fixture.bindings -Authorization $authorization -Now $start.AddMinutes(1)
    $other = $fixture.bindings[1].file
    [IO.File]::WriteAllText($other, '{"external":"preserve this concurrent write"}')
    $before = @($fixture.bindings | ForEach-Object { (Get-FileHash -LiteralPath $_.file).Hash })
    Refuses { Restore-StartupPairPreferences -Journal $journal } 'Concurrent'
    $after = @($fixture.bindings | ForEach-Object { (Get-FileHash -LiteralPath $_.file).Hash })
    if (@(Compare-Object $before $after).Count) { throw 'Refused restore changed another fixture' }
}
[ordered]@{ result = 'passed'; checks = $checks.ToArray(); workspace = $workspace
    scope = 'temporary preference fixture and pure window/schedule checks only'
    syntheticHumanWindow = $true; realForegroundAuthorityGranted = $false
    productStarted = $false; nativeUiDriven = $false; realPreferencesChanged = $false
    formalStartupGatePassed = $false } | ConvertTo-Json -Depth 5 |
    Tee-Object -FilePath (Join-Path $output 'startup-pair-state-tests.json')

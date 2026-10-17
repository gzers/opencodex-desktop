# State-only helpers for the frozen cache-cold/process-warm startup measurement.
# No product launch, Windows input, timer injection, or acceptance from preparation.

function Assert-StartupPairImages {
    param([Parameter(Mandatory)]$Baseline, [Parameter(Mandatory)]$Candidate)
    $expected = @{
        baseline019 = @{ source = 'de6862d40389d527527b9db8ef52e186a58b625f'
            sha256 = '626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608'; version = '0.1.9' }
        candidate0110 = @{ source = '8e69ff6db69ea13b8cb29feb9f7a1abacd202a7b'
            sha256 = '870cb30c62c0cf563edacf7fe30a35d3b66cbffbb574beeb6b354ed2c7fab15a'; version = '0.1.10' }
    }
    if ($Candidate.fullCiRunId -ne 39032153138 -or $Candidate.candidateRunId -ne 39033532953) {
        throw 'Exact admitted 8e full CI and unique candidate required'
    }
    foreach ($entry in @(@{ key = 'baseline019'; image = $Baseline }, @{ key = 'candidate0110'; image = $Candidate })) {
        $identity = $expected[$entry.key]
        if ($entry.image.source -cne $identity.source -or $entry.image.sha256 -cne $identity.sha256) {
            throw 'Startup source or image identity drifted'
        }
        $file = ConvertTo-StartupPairPath $entry.image.executable
        if ((Get-FileHash -LiteralPath $file).Hash.ToLowerInvariant() -cne $identity.sha256 -or
            (Get-Item -LiteralPath $file).VersionInfo.ProductVersion -cne $identity.version) {
            throw 'Actual startup image bytes or version differ'
        }
    }
    return [ordered]@{ baselineSource = $expected.baseline019.source; candidateSource = $expected.candidate0110.source
        exactImagesVerified = $true; productStarted = $false; startupGatePassed = $false }
}

function Assert-StartupPairNoProduct {
    if (@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count) {
        throw 'Normal product exit is required before changing fixture preferences'
    }
}

function Get-StartupPairSchedule {
    $rows = [Collections.Generic.List[object]]::new()
    for ($round = 1; $round -le 20; $round++) {
        $order = if ($round % 2) { @('baseline019', 'candidate0110') } else { @('candidate0110', 'baseline019') }
        foreach ($version in $order) {
            foreach ($kind in @('cache-cold', 'process-warm')) {
                $rows.Add([pscustomobject]@{ round = $round; version = $version; kind = $kind })
            }
        }
    }
    return $rows.ToArray()
}

function Assert-StartupPairWindow {
    param(
        [Parameter(Mandatory)]$Authorization,
        [Parameter(Mandatory)][datetimeoffset]$Now,
        [ValidateSet('prepare', 'sample')][string]$Phase = 'sample'
    )
    if ($Authorization.origin -cne 'human' -or
        $Authorization.explicitHumanStart -isnot [bool] -or !$Authorization.explicitHumanStart -or
        $Authorization.automaticContinuation -isnot [bool] -or $Authorization.automaticContinuation -or
        $Authorization.scope -cne 'startup-pair-cold-warm') {
        throw 'A current human startup-pair authorization is required'
    }
    try {
        $start = [datetimeoffset]::Parse($Authorization.authorizedAtUtc)
        $end = [datetimeoffset]::Parse($Authorization.expiresAtUtc)
    } catch { throw 'Invalid human authorization timestamps' }
    if ($end -le $start -or ($end - $start).TotalMinutes -gt 30 -or $Now -lt $start -or $Now -ge $end) {
        throw 'Human authorization is future, expired, or longer than thirty minutes'
    }
    # Operational time reservation, not a startup-performance threshold:
    # preserve the existing minute25 close target and five minutes for restore.
    $closeBy = if ($start.AddMinutes(25) -lt $end) { $start.AddMinutes(25) } else { $end }
    if ($Now.AddSeconds(55) -ge $closeBy) {
        # Existing bounds: startup30s + normal exit10s + WebView quiescence15s.
        throw 'Insufficient window for a complete sample and normal exit'
    }
    return [ordered]@{
        authorizedAtUtc = $start.ToUniversalTime().ToString('o')
        expiresAtUtc = $end.ToUniversalTime().ToString('o')
        closeByUtc = $closeBy.ToUniversalTime().ToString('o')
        originalSampleCount = 80
        productLaunchGrantedByThisCheck = $false
    }
}

function ConvertTo-StartupPairPath {
    param([Parameter(Mandatory)][string]$Path)
    if ($Path -notmatch '^[A-Za-z]:\\' -or $Path -match '[\x00-\x1f]' -or
        $Path.Substring(2).Contains(':') -or $Path -match '(^|\\)\.\.?($|\\)') {
        throw 'A local absolute path without streams or dot components is required'
    }
    $full = [IO.Path]::GetFullPath($Path).TrimEnd('\')
    if ($full.Length -le 3 -or ![StringComparer]::OrdinalIgnoreCase.Equals($full, $Path.TrimEnd('\'))) {
        throw 'A canonical non-drive-root directory is required'
    }
    $cursor = $full
    while ($cursor) {
        if (Test-Path -LiteralPath $cursor) {
            $item = Get-Item -LiteralPath $cursor -Force
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'Path alias refused' }
        }
        $cursor = [IO.Path]::GetDirectoryName($cursor)
    }
    return $full
}

function Save-StartupPairPreferences {
    param(
        [Parameter(Mandatory)][string]$RunDirectory,
        [Parameter(Mandatory)][object[]]$Bindings,
        [Parameter(Mandatory)]$Authorization,
        [Parameter(Mandatory)][datetimeoffset]$Now
    )
    $null = Assert-StartupPairWindow -Authorization $Authorization -Now $Now -Phase prepare
    Assert-StartupPairNoProduct
    $run = ConvertTo-StartupPairPath $RunDirectory
    if (!(Test-Path -LiteralPath $run -PathType Container)) { throw 'A pre-created private run directory is required' }
    $journal = Join-Path $run 'startup-preference-journal.json'
    if (Test-Path -LiteralPath $journal) { throw 'Never reuse an earlier preference transaction' }
    if ($Bindings.Count -ne 2 -or
        @($Bindings.version | Sort-Object -Unique).Count -ne 2 -or
        @($Bindings.version | Where-Object { $_ -cnotin @('baseline019', 'candidate0110') }).Count) {
        throw 'Exact baseline and candidate preference bindings are required'
    }
    $entries = @()
    foreach ($binding in $Bindings) {
        $root = ConvertTo-StartupPairPath $binding.root
        if ($run.StartsWith($root + '\', [StringComparison]::OrdinalIgnoreCase) -or
            [StringComparer]::OrdinalIgnoreCase.Equals($root, $run)) {
            throw 'Private evidence must not be inside a mutable fixture root'
        }
        $file = Join-Path $root 'manager-state\preferences.json'
        $null = ConvertTo-StartupPairPath $file
        $bytes = [IO.File]::ReadAllBytes($file)
        $digest = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant()
        if ($binding.sha256 -cnotmatch '^[0-9a-f]{64}$' -or $digest -cne $binding.sha256) {
            throw 'Preference fixture drifted before preservation'
        }
        $value = [Text.Encoding]::UTF8.GetString($bytes).TrimStart([char]0xfeff) | ConvertFrom-Json
        if ($value.schema_version -ne 2) { throw 'Known schema2 preferences required' }
        $backup = Join-Path $run ($binding.version + '-before-preferences.original.json')
        if (Test-Path -LiteralPath $backup) { throw 'Preserved original already exists' }
        $entries += [ordered]@{ version = $binding.version; file = $file; original = $backup
            originalSha256 = $digest; seededSha256 = $null; restore = 'pending' }
    }
    if (@($entries.file | Sort-Object -Unique).Count -ne 2) { throw 'Two distinct fixtures required' }
    # Preserve every original before changing either fixture. Journal stays private.
    foreach ($entry in $entries) {
        [IO.File]::WriteAllBytes($entry.original, [IO.File]::ReadAllBytes($entry.file))
        if ((Get-FileHash -LiteralPath $entry.original).Hash.ToLowerInvariant() -cne $entry.originalSha256) {
            throw 'Fixture changed while preserving originals; no fixture has been written'
        }
    }
    $state = [ordered]@{ schemaVersion = 1; runDirectory = $run; entries = $entries; stage = 'preserved' }
    $state | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $journal -Encoding utf8
    try {
        foreach ($entry in $entries) {
            Assert-StartupPairNoProduct
            if ((Get-FileHash -LiteralPath $entry.file).Hash.ToLowerInvariant() -cne $entry.originalSha256) {
                throw 'Preference changed after preservation'
            }
            $value = Get-Content -LiteralPath $entry.original -Raw -Encoding utf8 | ConvertFrom-Json
            $value.appearance.theme = 'light'
            $value.appearance.visual_effects = 'high'
            $value.appearance.glow_render = 'mesh'
            $value.appearance.interface_scale = 100
            $value.shell.auto_panel = $false
            $value.shell.launch_with_codex = $false
            $value.shell.keep_proxy_on_close = $false
            $value.shell.launch_main = $true
            $value.updates.app_update_auto_check = $false
            $value.maintenance.startup_cleanup = $false
            $seed = [Text.UTF8Encoding]::new($false).GetBytes(($value | ConvertTo-Json -Depth 10))
            # Publish expected seeded bytes before writing, so a failed write can
            # never be mistaken for permission to overwrite a concurrent change.
            $entry.seededSha256 = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($seed)).ToLowerInvariant()
            $state | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $journal -Encoding utf8
            [IO.File]::WriteAllBytes($entry.file, $seed)
        }
        $state.stage = 'seeded'
        $state | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $journal -Encoding utf8
        return $journal
    } catch {
        # Restore only originals or this transaction's exact seeded bytes.
        # A different writer's bytes cause refusal, never a blind overwrite.
        Restore-StartupPairPreferences -Journal $journal
        throw
    }
}

function Restore-StartupPairPreferences {
    param([Parameter(Mandatory)][string]$Journal)
    $state = Get-Content -LiteralPath $Journal -Raw -Encoding utf8 | ConvertFrom-Json
    $run = ConvertTo-StartupPairPath $state.runDirectory
    if ([IO.Path]::GetFullPath($Journal) -cne (Join-Path $run 'startup-preference-journal.json') -or
        $state.schemaVersion -ne 1 -or $state.entries.Count -ne 2) { throw 'Exact private journal required' }
    # Restoration is cleanup and does not require a new foreground window.
    Assert-StartupPairNoProduct
    foreach ($entry in $state.entries) {
        $original = ConvertTo-StartupPairPath $entry.original
        $file = ConvertTo-StartupPairPath $entry.file
        if ($entry.version -cnotin @('baseline019', 'candidate0110') -or
            $original -cne (Join-Path $run ($entry.version + '-before-preferences.original.json')) -or
            !([IO.Path]::GetFileName($file) -ceq 'preferences.json') -or
            !([IO.Path]::GetFileName([IO.Path]::GetDirectoryName($file)) -ceq 'manager-state')) {
            throw 'Preference restoration endpoints changed'
        }
        if ((Get-FileHash -LiteralPath $original).Hash.ToLowerInvariant() -cne $entry.originalSha256) {
            throw 'Preserved preference bytes drifted'
        }
        $current = (Get-FileHash -LiteralPath $file).Hash.ToLowerInvariant()
        if ($current -cne $entry.originalSha256 -and $current -cne $entry.seededSha256) {
            throw 'Concurrent preference change retained; refuse overwrite'
        }
    }
    foreach ($entry in $state.entries) {
        Assert-StartupPairNoProduct
        # An unmodified second fixture may have been the reason setup failed
        # (for example read-only). Do not needlessly rewrite its original bytes.
        if ((Get-FileHash -LiteralPath $entry.file).Hash.ToLowerInvariant() -cne $entry.originalSha256) {
            [IO.File]::WriteAllBytes($entry.file, [IO.File]::ReadAllBytes($entry.original))
        }
        if ((Get-FileHash -LiteralPath $entry.file).Hash.ToLowerInvariant() -cne $entry.originalSha256) {
            throw 'Restored preference readback differs'
        }
        $entry.restore = 'original-bytes-restored'
    }
    $state.stage = 'restored'
    $state | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $Journal -Encoding utf8
}

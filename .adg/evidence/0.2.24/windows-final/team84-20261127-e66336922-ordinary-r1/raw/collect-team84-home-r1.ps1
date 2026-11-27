param([string]$SelfTestRoot)
$ErrorActionPreference = 'Stop'

function Read-PhysicalItem([string]$Path, [bool]$Directory) {
    $item = Get-Item -LiteralPath $Path -Force
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $item.PSIsContainer -ne $Directory) {
        throw "Unexpected path type or reparse point: $Path"
    }
    $item
}

function Read-SmallJson([string]$Path) {
    $item = Read-PhysicalItem $Path $false
    $file = [IO.File]::Open($item.FullName, 'Open', 'Read', 'ReadWrite,Delete')
    try {
        $bytes = New-Object byte[] 65537
        $offset = 0
        while ($offset -lt $bytes.Length) {
            $count = $file.Read($bytes, $offset, $bytes.Length - $offset)
            if ($count -eq 0) { break }
            $offset += $count
        }
        if ($offset -eq $bytes.Length) { throw "Binding file too large: $Path" }
        [Text.UTF8Encoding]::new($false, $true).GetString($bytes, 0, $offset) | ConvertFrom-Json
    } finally { $file.Dispose() }
}

function Read-BoundHome([string]$CaseRoot, [string]$Case) {
    $sandbox = Join-Path $CaseRoot $Case
    $aggregate = Join-Path $sandbox 'data'
    $desktop = Join-Path $aggregate 'desktop'
    $managerState = Join-Path $desktop 'manager-state'
    foreach ($directory in $CaseRoot, $sandbox, $aggregate, $desktop, $managerState) {
        $null = Read-PhysicalItem $directory $true
    }
    $location = Read-SmallJson (Join-Path $desktop 'manager-location.json')
    $recorded = $location.activeRoot
    if ($recorded -is [string] -and $recorded.StartsWith('\\?\', [StringComparison]::Ordinal)) {
        $recorded = $recorded.Substring(4)
    }
    if (($location.schemaVersion -isnot [int] -and $location.schemaVersion -isnot [long]) -or
        $location.schemaVersion -ne 1 -or @($location.PSObject.Properties).Count -ne 2 -or
        $recorded -isnot [string] -or $recorded -ine $aggregate) {
        throw 'Saved aggregate does not match the exact fixture aggregate.'
    }
    $config = Read-SmallJson (Join-Path $managerState 'data-root.json')
    if ($config.structure_version -isnot [string] -or $config.structure_version -cne '2' -or
        $config.opencodex_home_mode -cne 'inside' -or @($config.PSObject.Properties).Count -ne 2) {
        throw 'Saved runtime HOME is not the expected v2 Inside binding.'
    }
    $runtimeParent = Join-Path $aggregate 'opencodex'
    $runtimeHome = Join-Path $runtimeParent 'home'
    $null = Read-PhysicalItem $runtimeParent $true
    $null = Read-PhysicalItem $runtimeHome $true
    $runtimeHome
}

function Read-Case([string]$CaseRoot, [string]$Case) {
    $names = 'ocx.pid', 'runtime-port.json'
    $result = [ordered]@{ case = $Case; home = $null; binding = 'unknown'; bindingError = $null; records = @() }
    try {
        $runtimeHome = Read-BoundHome $CaseRoot $Case
        $entries = @(Get-ChildItem -LiteralPath $runtimeHome -Force)
        $records = @(foreach ($name in $names) {
            $record = [ordered]@{ name = $name; state = 'unknown'; error = $null }
            try {
                $matchingEntries = @($entries | Where-Object { $_.Name -ieq $name })
                if ($matchingEntries.Count -eq 0) {
                    $record.state = 'absent'
                } elseif ($matchingEntries.Count -eq 1 -and $matchingEntries[0].Name -ceq $name) {
                    $item = Read-PhysicalItem (Join-Path $runtimeHome $name) $false
                    $handle = [IO.File]::Open($item.FullName, 'Open', 'Read', 'ReadWrite,Delete')
                    try { $record.state = 'present' } finally { $handle.Dispose() }
                } else {
                    throw 'Runtime record spelling is ambiguous.'
                }
            } catch { $record.error = $_.Exception.Message }
            $record
        })
        if ((Read-BoundHome $CaseRoot $Case) -ine $runtimeHome) { throw 'HOME binding changed during readback.' }
        $result.home = $runtimeHome
        $result.binding = 'saved-v2-inside'
        $result.records = $records
    } catch {
        $result.bindingError = $_.Exception.Message
        $result.records = @(foreach ($name in $names) { [ordered]@{ name = $name; state = 'unknown'; error = $null } })
    }
    $result
}

function Assert-Check([bool]$Condition, [string]$Name) {
    if (!$Condition) { throw "FAIL $Name" }
    Write-Output "PASS $Name"
}

if ($SelfTestRoot) {
    if ([IO.Path]::GetFullPath($SelfTestRoot) -cne $SelfTestRoot -or (Test-Path -LiteralPath $SelfTestRoot)) {
        throw 'Self-test root must be a fresh absolute path.'
    }
    $fixture = Join-Path $SelfTestRoot 'Full'
    $aggregate = Join-Path $fixture 'data'
    $desktop = Join-Path $aggregate 'desktop'
    $state = Join-Path $desktop 'manager-state'
    $runtimeParent = Join-Path $aggregate 'opencodex'
    $runtimeHome = Join-Path $runtimeParent 'home'
    $locationPath = Join-Path $desktop 'manager-location.json'
    $configPath = Join-Path $state 'data-root.json'
    $pidMarker = Join-Path $runtimeHome 'ocx.pid'
    $portMarker = Join-Path $runtimeHome 'runtime-port.json'
    $location = @{ schemaVersion = 1; activeRoot = '\\?\' + $aggregate } | ConvertTo-Json
    try {
        $null = [IO.Directory]::CreateDirectory($state)
        $null = [IO.Directory]::CreateDirectory($runtimeHome)
        [IO.File]::WriteAllText($locationPath, $location)
        [IO.File]::WriteAllText($configPath, '{"structure_version":"2","opencodex_home_mode":"inside"}')
        [IO.File]::WriteAllText($pidMarker, 'synthetic')
        $observed = Read-Case $SelfTestRoot 'Full'
        Assert-Check ($observed.binding -eq 'saved-v2-inside' -and $observed.home -ceq $runtimeHome) 'v2 Inside resolved from verbatim saved root'
        Assert-Check ($observed.records[0].state -eq 'present') 'present record observed at the correct HOME'
        Assert-Check ($observed.records[1].state -eq 'absent') 'absent record observed in an enumerable HOME'
        $exclusive = [IO.File]::Open($pidMarker, 'Open', 'ReadWrite', 'None')
        try { $observed = Read-Case $SelfTestRoot 'Full' } finally { $exclusive.Dispose() }
        Assert-Check ($observed.records[0].state -eq 'unknown' -and $observed.records[0].error) 'unreadable file stays unknown'
        $null = [IO.Directory]::CreateDirectory($portMarker)
        $observed = Read-Case $SelfTestRoot 'Full'
        Assert-Check ($observed.records[1].state -eq 'unknown' -and $observed.records[1].error) 'directory at record path stays unknown'
        [IO.File]::WriteAllText($configPath, '{"structure_version":"2","opencodex_home_mode":"external","opencodex_home_path":"C:\\other"}')
        $observed = Read-Case $SelfTestRoot 'Full'
        Assert-Check ($observed.binding -eq 'unknown' -and @($observed.records | Where-Object state -ne 'unknown').Count -eq 0) 'external HOME rejected before record read'
        [IO.File]::WriteAllText($locationPath, '{"schemaVersion":1,"activeRoot":"C:\\other"}')
        $observed = Read-Case $SelfTestRoot 'Full'
        Assert-Check ($observed.bindingError -match 'Saved aggregate') 'unexpected aggregate rejected before record read'
        $observed = Read-Case $SelfTestRoot 'Missing'
        Assert-Check ($observed.binding -eq 'unknown' -and $observed.records[0].state -eq 'unknown') 'missing fixture never implies missing records'
    } finally {
        foreach ($leaf in $pidMarker, $configPath, $locationPath) {
            if ([IO.File]::Exists($leaf)) { [IO.File]::Delete($leaf) }
        }
        foreach ($directory in $portMarker, $runtimeHome, $runtimeParent, $state, $desktop, $aggregate, $fixture, $SelfTestRoot) {
            if ([IO.Directory]::Exists($directory)) { [IO.Directory]::Delete($directory, $false) }
        }
    }
    Assert-Check (![IO.Directory]::Exists($SelfTestRoot)) 'self-test fixture removed'
    return
}

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$session = (Get-Process -Id $PID).SessionId
if ($identity.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or
    @($identity.Groups | ForEach-Object Value) -contains 'S-1-5-32-544' -or
    $session -ne 3 -or ![Environment]::UserInteractive) {
    throw 'Run in the existing ordinary marenben PowerShell on RDP session 3.'
}
$profileRoot = [Environment]::GetFolderPath('UserProfile')
if ($profileRoot -ine 'C:\Users\marenben' -or $PSScriptRoot -ine 'C:\Users\Public\ocx-team84-e66336922-r2') {
    throw 'Unexpected fixture profile or collector directory.'
}
$null = Read-PhysicalItem $profileRoot $true
$caseRoot = Join-Path $profileRoot 'ocx-team84-e66336922-r2'
$receipts = Join-Path $PSScriptRoot 'receipts'
$null = Read-PhysicalItem $receipts $true
$report = [ordered]@{
    sourceSha = 'e663369226337c9a9cad594ccd3cafd1719c3711'
    sid = $identity.User.Value; session = $session; administratorGroupPresent = $false
    utc = [DateTime]::UtcNow.ToString('o'); collectorSha256 = (Get-FileHash -LiteralPath $PSCommandPath).Hash.ToLowerInvariant()
    cases = @((Read-Case $caseRoot 'Full'), (Read-Case $caseRoot 'Missing'))
    provesIdle = $false
}
$name = 'collect-home-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ') + '.json'
$bytes = [Text.Encoding]::UTF8.GetBytes(($report | ConvertTo-Json -Depth 6) + [Environment]::NewLine)
$file = [IO.File]::Open((Join-Path $receipts $name), 'CreateNew', 'Write', 'Read')
try { $file.Write($bytes, 0, $bytes.Length) } finally { $file.Dispose() }
Write-Output "HOME-READBACK $name"

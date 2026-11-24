param(
    [ValidateSet('custom-before','custom-running','custom-stopped','custom-enabled','custom-removed','default-installed','default-removed')]
    [string]$Phase = 'custom-before'
)
$ErrorActionPreference = 'Stop'
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$sid = $identity.User.Value
$session = [Diagnostics.Process]::GetCurrentProcess().SessionId
$principal = [Security.Principal.WindowsPrincipal]::new($identity)
if ($sid -ne 'S-1-5-21-587408064-856152790-2583328096-1003' -or $session -ne 3 -or
    $env:SESSIONNAME -notlike 'RDP-*' -or -not [Environment]::UserInteractive -or
    $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'Run from normal PowerShell in marenben RDP session 3.'
}
$receipt = Join-Path $PSScriptRoot "receipts-rdp\rdp-r2-$Phase.json"
if ([IO.File]::Exists($receipt)) {
    Write-Host "EXISTING-RECEIPT $Phase"
    exit 0
}
$profileRoot = $env:USERPROFILE
$customRoot = 'D:\Users\marenben\WorkSpace\Test\OpenCodeX Desktop'
$defaultRoot = Join-Path $env:LOCALAPPDATA 'Programs\OpenCodeX Desktop'
$installRoot = if ($Phase -like 'default-*') { $defaultRoot } else { $customRoot }
$aggregate = Join-Path $profileRoot '.codex-desktop'
$desktop = Join-Path $aggregate 'desktop'
$runtimeHome = Join-Path $aggregate 'opencodex\home'
$utf8 = [Text.UTF8Encoding]::new($false)
$clock = [Diagnostics.Stopwatch]::StartNew()
$report = [ordered]@{
    phase=$Phase; utc=[DateTime]::UtcNow.ToString('o'); sid=$sid; session=$session
    sessionName=$env:SESSIONNAME; profile=$profileRoot; installRoot=$installRoot
    desktopRoot=$desktop; expectedRuntimeHome=$runtimeHome; checks=[ordered]@{}; firstError=$null
}
function Read-Fact($name, [scriptblock]$read) {
    try { $report.checks[$name] = [ordered]@{state='observed'; value=(& $read)} }
    catch {
        $message = $_.Exception.Message
        $report.checks[$name] = [ordered]@{state='unknown'; error=$message}
        if ($null -eq $report.firstError) { $report.firstError = $name + ': ' + $message }
    }
}
function Inspect-Path([string]$path, [switch]$NoHash) {
    try { $item = Get-Item -LiteralPath $path -Force }
    catch [Management.Automation.ItemNotFoundException] { return [ordered]@{path=$path; present=$false} }
    if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Reparse point: $path" }
    $fact = [ordered]@{path=$path; present=$true; directory=$item.PSIsContainer; mtimeUtc=$item.LastWriteTimeUtc.ToString('o')}
    if (-not $item.PSIsContainer) {
        $fact.bytes = $item.Length
        if (-not $NoHash) { $fact.sha256 = (Get-FileHash -LiteralPath $path).Hash.ToLowerInvariant() }
    }
    $fact
}
function Read-Json($fact, [long]$maxBytes=65536) {
    if ($fact.directory -or $fact.bytes -gt $maxBytes) { throw "Unexpected metadata size or type: $($fact.path)" }
    Get-Content -Raw -LiteralPath $fact.path | ConvertFrom-Json
}
Read-Fact 'processes' {
    $rows = foreach ($process in Get-CimInstance Win32_Process -Filter "SessionId=$session") {
        if ($process.Name -notin 'opencodex-desktop.exe','ocxd.exe','ocx.exe','node.exe','bun.exe','uninstall.exe') { continue }
        $owner = Invoke-CimMethod -InputObject $process -MethodName GetOwnerSid
        [ordered]@{name=$process.Name;pid=$process.ProcessId;parentPid=$process.ParentProcessId;image=$process.ExecutablePath;sid=$owner.Sid;ownerQueryCode=$owner.ReturnValue;creationUtc=$process.CreationDate.ToUniversalTime().ToString('o')}
    }
    ,@($rows)
}
Read-Fact 'listeners' {
    if ($report.checks.processes.state -ne 'observed') { throw 'Listener ownership cannot be correlated.' }
    $pids = @($report.checks.processes.value | ForEach-Object { $_.pid })
    $rows = @(& netstat.exe -ano)
    if ($LASTEXITCODE -ne 0) { throw "netstat exit=$LASTEXITCODE" }
    $listenerRows = foreach ($line in $rows) {
        if ($line -match '^\s*TCP\s+(\S+)\s+(?:0\.0\.0\.0:0|\[::\]:0)\s+(\S+)\s+(\d+)\s*$') {
            $pidNumber = [int]$Matches[3]
            if ($pidNumber -in $pids) { [ordered]@{local=$Matches[1]; nativeState=$Matches[2]; pid=$pidNumber} }
        }
    }
    ,@($listenerRows)
}
Read-Fact 'roots' {
    $paths = @($customRoot,$defaultRoot,$aggregate,$desktop,$runtimeHome,(Join-Path $runtimeHome 'codex-client'))
    ,@($paths | ForEach-Object { Inspect-Path $_ })
}
Read-Fact 'managerLocation' {
    $fact = Inspect-Path (Join-Path $desktop 'manager-location.json')
    if ($fact.present) {
        $json = Read-Json $fact
        if (@($json.PSObject.Properties.Name).Count -ne 2 -or
            $null -eq $json.schemaVersion -or $json.activeRoot -isnot [string]) {
            throw 'Unexpected manager-location.json field set or types.'
        }
        $fact.activeRoot = $json.activeRoot
        $fact.schemaVersion = $json.schemaVersion
        $rootForComparison = $json.activeRoot
        if ($rootForComparison.StartsWith('\\?\',[StringComparison]::Ordinal) -and
            $rootForComparison.Substring(4) -match '^[A-Za-z]:\\') {
            $rootForComparison = $rootForComparison.Substring(4)
        }
        $fact.comparisonRoot = $rootForComparison
        $fact.matchesExpected = $json.schemaVersion -eq 1 -and $rootForComparison -ieq $aggregate
    }
    $fact
}
Read-Fact 'fixedRuntime' {
    $current = Inspect-Path (Join-Path $installRoot 'runtime\current.json')
    $versionRoot = Join-Path $installRoot 'runtime\versions\2.99.0'
    $image = Inspect-Path (Join-Path $versionRoot '.runtime-image.json')
    $node = Inspect-Path (Join-Path $versionRoot 'dependencies\node\node.exe')
    $entry = Inspect-Path (Join-Path $installRoot 'runtime\bin\ocx-runtime.cmd')
    $selection = if ($current.present) { Read-Json $current }
    $manifest = if ($image.present) { Read-Json $image }
    [ordered]@{
        current=$current; version=$selection.version; platform=$selection.platform; schemaVersion=$selection.schemaVersion
        image=$image; manifestArchiveSha256=$manifest.officialArchiveSha256; manifestTreeSha256=$manifest.treeSha256
        node=$node; entry=$entry; selectedImageDigestMatches=($image.present -and $image.sha256 -eq $selection.imageSha256)
        nodeDigestMatches=($node.present -and $node.sha256 -eq $manifest.nodeSha256)
        officialArchiveMatches=($manifest.officialArchiveSha256 -eq 'b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c')
    }
}
Read-Fact 'runtimePort' {
    $fact = Inspect-Path (Join-Path $runtimeHome 'runtime-port.json') -NoHash
    if ($fact.present) {
        $json = Read-Json $fact 4096
        $fact.pid = [int]$json.pid
        $fact.port = [int]$json.port
    }
    $fact
}
Read-Fact 'runtimePid' { Inspect-Path (Join-Path $runtimeHome 'ocx.pid') }
Read-Fact 'realClientConfig' { Inspect-Path (Join-Path $profileRoot '.codex\config.toml') }
Read-Fact 'managerLog' { Inspect-Path (Join-Path $desktop 'logs\app.log') -NoHash }
Read-Fact 'terminalFiles' {
    ,@((Inspect-Path (Join-Path $desktop 'manager-state\terminal-cli.json')),
       (Inspect-Path (Join-Path $desktop 'manager-state\terminal-cli.pending.json')),
       (Inspect-Path (Join-Path $installRoot 'ocx.exe')))
}
Read-Fact 'terminalResolution' {
    ,@(Get-Command -Name 'ocx.exe' -CommandType Application -All -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source)
}
Read-Fact 'userPath' {
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment')
    try {
        $present = $null -ne $key -and $key.GetValueNames() -contains 'Path'
        $fact = [ordered]@{present=$present}
        if ($present) {
            $value = [string]$key.GetValue('Path',$null,[Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
            $hasher = [Security.Cryptography.SHA256]::Create()
            try { $digest = $hasher.ComputeHash($utf8.GetBytes($value)) } finally { $hasher.Dispose() }
            $fact.kind = $key.GetValueKind('Path').ToString()
            $fact.rawUtf8Sha256 = [BitConverter]::ToString($digest).Replace('-','').ToLowerInvariant()
            $fact.chars = $value.Length
            $fact.installEntryCount = @($value.Split(';') | Where-Object { $_.Trim().Trim('"').TrimEnd('\') -ieq $installRoot }).Count
        }
        $fact
    } finally { if ($null -ne $key) { $key.Dispose() } }
}
Read-Fact 'registrations' {
    $rows = foreach ($view in 'Registry64','Registry32') {
        $base = [Microsoft.Win32.RegistryKey]::OpenBaseKey('CurrentUser',$view)
        try {
            $key = $base.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Uninstall')
            if ($null -eq $key) { continue }
            try {
                foreach ($name in $key.GetSubKeyNames()) {
                    $entry = $key.OpenSubKey($name)
                    if ($null -eq $entry) { continue }
                    try {
                        if ($entry.GetValue('DisplayName') -eq 'OpenCodeX Desktop') {
                            [ordered]@{view=$view; key=$name; version=$entry.GetValue('DisplayVersion'); location=$entry.GetValue('InstallLocation'); uninstall=$entry.GetValue('UninstallString')}
                        }
                    } finally { $entry.Dispose() }
                }
            } finally { $key.Dispose() }
        } finally { $base.Dispose() }
    }
    ,@($rows)
}
if ($Phase -eq 'custom-before') {
    $checks = $report.checks
    $locations = $checks.managerLocation.value
    $selection = $checks.fixedRuntime.value
    $processRows = @($checks.processes.value)
    $gui = @($processRows | Where-Object {
        $_.pid -eq 29800 -and $_.sid -eq $sid -and $_.ownerQueryCode -eq 0 -and
        $_.image -ieq (Join-Path $customRoot 'opencodex-desktop.exe') -and
        [DateTimeOffset]::Parse($_.creationUtc) -eq [DateTimeOffset]::Parse('2026-11-24T11:39:21.6264620Z')
    })
    $report.readyForStart = (
        $null -eq $report.firstError -and $locations.matchesExpected -and
        $locations.sha256 -eq '7609f15b0ec176bc321fff268778584a871da22499373424ff534be458845e94' -and
        $gui.Count -eq 1 -and $processRows.Count -eq 1 -and
        @($checks.listeners.value).Count -eq 0 -and
        $checks.runtimePort.value.present -eq $false -and $checks.runtimePid.value.present -eq $false -and
        @($checks.roots.value | Where-Object { $_.path -ieq (Join-Path $runtimeHome 'codex-client') -and $_.present -eq $false }).Count -eq 1 -and
        $selection.version -eq '2.99.0' -and $selection.platform -eq 'win32-x64' -and
        $selection.selectedImageDigestMatches -and $selection.nodeDigestMatches -and $selection.officialArchiveMatches -and
        $checks.realClientConfig.value.sha256 -eq 'd19c0773b53b6f016fbf35f317cae551c540b70e67fd5309a8a4c1d847bc7700' -and
        [DateTimeOffset]::Parse([string]$checks.realClientConfig.value.mtimeUtc) -eq [DateTimeOffset]::Parse('2026-11-17T05:09:09.0914453Z')
    )
}
if ($Phase -in 'custom-running','custom-stopped') {
    Read-Fact 'lifecycleEvents' {
        $baselinePath = Join-Path $PSScriptRoot 'receipts-rdp\rdp-r2-custom-before.json'
        $baseline = Read-Json (Inspect-Path $baselinePath) 65536
        if (-not $baseline.readyForStart -or $baseline.sid -ne $sid -or $baseline.session -ne $session) {
            throw 'No admitted r2 baseline for the current SID and session.'
        }
        $log = $report.checks.managerLog.value
        if (-not $log.present -or $log.directory -or $log.bytes -gt 5MB) {
            throw 'Manager log missing or outside expected size.'
        }
        $stream = [IO.File]::Open($log.path,'Open','Read','ReadWrite')
        try {
            $reader = [IO.StreamReader]::new($stream,$utf8)
            try {
                $rows = while ($null -ne ($line = $reader.ReadLine())) {
                    if ($line -notmatch '^(?<utc>\S+) \[manager\] (?<action>start|stop|restart): (?<result>.+)$') {continue}
                    $eventUtc = $Matches.utc
                    $action = $Matches.action
                    $result = $Matches.result
                    if ([DateTimeOffset]::Parse($eventUtc) -lt [DateTimeOffset]::Parse($baseline.finishedUtc)) {continue}
                    $known = $result -in 'start confirmed','stop completed','restart confirmed'
                    $ms = if ($result -match 'duration_ms=(\d+)') {[long]$Matches[1]}
                    [ordered]@{utc=$eventUtc;action=$action;outcome=$(if ($known) {$result} else {'failed or unclassified'});productDurationMs=$ms}
                }
            } finally {$reader.Dispose()}
        } finally {$stream.Dispose()}
        ,@($rows)
    }
}
$report.finishedUtc = [DateTime]::UtcNow.ToString('o')
$report.elapsedMs = $clock.ElapsedMilliseconds
$bytes = $utf8.GetBytes(($report | ConvertTo-Json -Depth 10))
$file = [IO.File]::Open($receipt,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::None)
try { $file.Write($bytes,0,$bytes.Length) } finally { $file.Dispose() }
Write-Host "RECEIPT $Phase"
if ($report.firstError) { Write-Host $report.firstError }

if ($Phase -eq 'custom-before') {
    if ($report.readyForStart) {Write-Host 'READY-FOR-START'}
    else {Write-Host 'HOLD: preflight did not match. Do not click Start.'; exit 1}
}
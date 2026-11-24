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
$receipt = Join-Path $PSScriptRoot "receipts-rdp\rdp-$Phase.json"
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
        $fact.activeRoot = $json.active_root
        $fact.schemaVersion = $json.schema_version
        $fact.matchesExpected = $json.active_root -ieq $aggregate
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
$report.finishedUtc = [DateTime]::UtcNow.ToString('o')
$report.elapsedMs = $clock.ElapsedMilliseconds
$bytes = $utf8.GetBytes(($report | ConvertTo-Json -Depth 10))
$file = [IO.File]::Open($receipt,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::None)
try { $file.Write($bytes,0,$bytes.Length) } finally { $file.Dispose() }
Write-Host "RECEIPT $Phase"
if ($report.firstError) { Write-Host $report.firstError }

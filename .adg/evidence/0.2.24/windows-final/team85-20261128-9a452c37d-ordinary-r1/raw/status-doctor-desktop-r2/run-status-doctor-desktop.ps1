$ErrorActionPreference = 'Stop'
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$session = (Get-Process -Id $PID).SessionId
$profileDirectory = [Environment]::GetFolderPath('UserProfile')
if ($identity.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or
    @($identity.Groups | ForEach-Object Value) -contains 'S-1-5-32-544' -or
    $session -eq 0 -or -not [Environment]::UserInteractive -or
    $profileDirectory -ine 'C:/Users/marenben'.Replace('/',[IO.Path]::DirectorySeparatorChar)) {
    throw 'Run from the existing marenben desktop in ordinary PowerShell.'
}
$fixture = Split-Path -Parent $PSScriptRoot
$receipts = Join-Path $fixture 'receipts'
$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ')
$name = 'status-doctor-desktop-r2-' + $stamp
$sandbox = Join-Path $profileDirectory ('ocx-team85-' + $name)
$program = Join-Path $sandbox 'program'
$resultPath = Join-Path $receipts ($name + '.json')
$report = [ordered]@{
    source = '9a452c37d3005119042d81ee1ae8f62c4920c4d0'; artifactId = 15224497695; accepted = $false
    token = $identity.Name; sid = $identity.User.Value; session = $session; sandbox = $sandbox
    launchRequestedUtc = $null; managerPid = $null; observerExitCode = $null; firstError = $null
}
function Save-Receipt([string]$Path, $Data) {
    $bytes = [Text.Encoding]::UTF8.GetBytes(($Data | ConvertTo-Json -Depth 16) + [Environment]::NewLine)
    $stream = [IO.File]::Open($Path, 'CreateNew', 'Write', 'Read')
    try { $stream.Write($bytes, 0, $bytes.Length) } finally { $stream.Dispose() }
}
function Assert-Digest([string]$Path, [string]$Hash) {
    if ((Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash -ine $Hash) { throw ('Digest mismatch: ' + $Path) }
}
function Read-Log([string]$Path) {
    if (!(Test-Path -LiteralPath $Path -PathType Leaf)) { return $null }
    $share = [IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete
    $stream = [IO.File]::Open($Path, 'Open', 'Read', $share)
    try {
        if ($stream.Length -gt 1048576) { throw 'Log exceeds 1 MiB' }
        $reader = [IO.StreamReader]::new($stream)
        try { $reader.ReadToEnd() } finally { $reader.Dispose() }
    } finally { $stream.Dispose() }
}
try {
    if (Test-Path -LiteralPath $sandbox) { throw 'The new sandbox must be absent.' }
    $manifest = Join-Path $fixture 'payload.json'
    Assert-Digest $manifest '94C9A9937A4254DFFB8791ABD6A8215B00615CD011F7580C32D6F7E5390E173F'
    $rows = Get-Content -LiteralPath $manifest -Raw | ConvertFrom-Json
    if (@($rows).Count -lt 8) { throw 'Incomplete payload manifest' }
    [IO.Directory]::CreateDirectory($program) | Out-Null
    foreach ($row in $rows) {
        $member = [string]$row.path
        if ($member -notmatch '^[A-Za-z0-9_.-]+(/[A-Za-z0-9_.-]+)*$' -or $member -match '(?:^|/)\.\.?(/|$)') { throw 'Unsafe payload member' }
        $target = Join-Path $program $member
        [IO.Directory]::CreateDirectory((Split-Path -Parent $target)) | Out-Null
        Copy-Item -LiteralPath (Join-Path (Join-Path $fixture 'program') $member) -Destination $target
        if ((Get-Item -LiteralPath $target).Length -ne $row.bytes) { throw ('Payload size mismatch: ' + $member) }
        Assert-Digest $target $row.sha256
    }
    $node = Join-Path $fixture 'observer-node/runtime/node/node.exe'
    Assert-Digest $node '5CBA0EA928508C65DDADEFC0681D2FB6D95F1F3BEEA4428F04397631140EE8E5'
    $observerScript = Join-Path $PSScriptRoot 'observe-status-doctor.mjs'
    $report.observerSha256 = (Get-FileHash -LiteralPath $observerScript -Algorithm SHA256).Hash
    $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
    $listener.Start()
    try { $port = $listener.LocalEndpoint.Port } finally { $listener.Stop() }
    $report.cdpPort = $port
    foreach ($key in 'HOME','CODEX_HOME','OPENCODEX_HOME','OPENCODEX_SANDBOX_BOUNDARY','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','NODE_OPTIONS') {
        [Environment]::SetEnvironmentVariable($key, $null, 'Process')
    }
    $env:OPENCODEX_SANDBOX = '1'
    $env:OPENCODEX_SANDBOX_ROOT = $sandbox
    $env:OPENCODEX_WINDOWS_SMOKE_CDP_PORT = [string]$port
    $env:WEBVIEW2_USER_DATA_FOLDER = Join-Path $sandbox 'EBWebView'
    $tracePath = Join-Path $sandbox 'ipc-trace.json'
    $arguments = '"' + $observerScript + '" http://127.0.0.1:' + $port + ' "' + $sandbox + '" "' + $tracePath + '"'
    $observer = Start-Process -FilePath $node -ArgumentList $arguments -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $sandbox 'observer-stdout.log') -RedirectStandardError (Join-Path $sandbox 'observer-stderr.log')
    $report.observerPid = $observer.Id
    try {
        $report.launchRequestedUtc = [DateTime]::UtcNow.ToString('o')
        $manager = Start-Process -FilePath (Join-Path $program 'opencodex-desktop.exe') -WorkingDirectory $program -PassThru -RedirectStandardOutput (Join-Path $sandbox 'manager-stdout.log') -RedirectStandardError (Join-Path $sandbox 'manager-stderr.log')
        try { $report.managerPid = $manager.Id } finally { $manager.Dispose() }
        Write-Host ('OBSERVING PID=' + $report.managerPid + ' ' + $name)
        if (!$observer.WaitForExit(460000)) { throw 'Desktop observation deadline expired. Preserve the window and its logs.' }
        $report.observerExitCode = $observer.ExitCode
    } finally { $observer.Dispose() }
    $trace = Read-Log $tracePath
    if (!$trace) { throw 'Observer trace is absent.' }
    $report.traceSha256 = (Get-FileHash -LiteralPath $tracePath -Algorithm SHA256).Hash
    [IO.File]::Copy($tracePath, (Join-Path $receipts ($name + '-ipc.json')), $false)
    $report.trace = $trace | ConvertFrom-Json
    if ($report.trace.firstError) { throw $report.trace.firstError }
    if ($report.observerExitCode -ne 0) { throw 'Observer exited unsuccessfully.' }
} catch {
    $report.firstError = $_.Exception.Message
} finally {
    foreach ($entry in @('data/desktop/logs/app.log','observer-stderr.log','manager-stderr.log')) {
        try { $report[$entry] = Read-Log (Join-Path $sandbox $entry) }
        catch { $report[$entry] = $_.Exception.Message }
    }
    Save-Receipt $resultPath $report
}
if ($report.firstError) {
    Write-Host ('DESKTOP-READBACK-FAILED ' + (Split-Path -Leaf $resultPath))
    exit 1
}
Write-Host ('DESKTOP-READBACK-OK ' + (Split-Path -Leaf $resultPath))
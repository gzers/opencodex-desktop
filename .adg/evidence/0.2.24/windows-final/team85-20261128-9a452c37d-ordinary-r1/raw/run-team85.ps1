param([ValidateSet('Full','Missing','Collect')][string]$Mode='Full')
$ErrorActionPreference='Stop'
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
$session=(Get-Process -Id $PID).SessionId
$groups=@($identity.Groups | ForEach-Object Value)
if ($identity.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or
    $groups -contains 'S-1-5-32-544' -or $session -eq 0 -or -not [Environment]::UserInteractive) {
    throw 'Run in the existing marenben desktop using ordinary PowerShell.'
}
$profileDirectory=[Environment]::GetFolderPath('UserProfile')
if ($profileDirectory -ine 'C:\Users\marenben') { throw 'Unexpected profile directory' }
$caseRoot=Join-Path $profileDirectory 'ocx-team85-9a452c37d-r1'
$receipts=Join-Path $PSScriptRoot 'receipts'
$stagedProgram=Join-Path $PSScriptRoot 'program'
$archive='runtime-bundle/runtime-bundle-0.2.24-win32-x64.tar.gz'
$sourceSha='9a452c37d3005119042d81ee1ae8f62c4920c4d0'
function Save-Receipt([string]$Name,$Data) {
    $bytes=[Text.Encoding]::UTF8.GetBytes(($Data | ConvertTo-Json -Depth 9)+[Environment]::NewLine)
    $stream=[IO.File]::Open((Join-Path $receipts $Name),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
    try { $stream.Write($bytes,0,$bytes.Length) } finally { $stream.Dispose() }
}
function Read-Log([string]$Path) {
    if (!(Test-Path -LiteralPath $Path -PathType Leaf)) { return $null }
    $stream=[IO.File]::Open($Path,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
    try {
        if ($stream.Length -gt 1048576) { throw 'Log exceeds 1 MiB' }
        $reader=[IO.StreamReader]::new($stream)
        try { $reader.ReadToEnd() } finally { $reader.Dispose() }
    } finally { $stream.Dispose() }
}
function Get-CaseFacts([string]$Name) {
    $root=Join-Path $caseRoot $Name
    $runtimeHome=Join-Path $root 'data/opencodex/home'
    $programPath=Join-Path $root 'program'
    $image=Join-Path $programPath 'opencodex-desktop.exe'
    [ordered]@{
        case=$Name; sandbox=$root; runtimeHome=$runtimeHome
        archivePresent=(Test-Path -LiteralPath (Join-Path $programPath $archive) -PathType Leaf)
        current=Read-Log (Join-Path $programPath 'runtime/current.json')
        appLog=Read-Log (Join-Path $root 'data/desktop/logs/app.log')
        pidRecord=Read-Log (Join-Path $runtimeHome 'ocx.pid')
        portRecord=Read-Log (Join-Path $runtimeHome 'runtime-port.json')
        managers=@(Get-Process -Name 'opencodex-desktop' -ErrorAction SilentlyContinue | Where-Object {$_.Path -ieq $image} | Select-Object Id,SessionId,Path)
    }
}
$report=[ordered]@{sourceSha=$sourceSha; artifactId=15224497695; account=$identity.Name; sid=$identity.User.Value; session=$session; utc=[DateTime]::UtcNow.ToString('o'); administratorGroupPresent=$false}
if ($Mode -eq 'Collect') {
    $report.cases=@(Get-CaseFacts 'Full'; Get-CaseFacts 'Missing')
    $name='collect-'+[DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ')+'.json'
    Save-Receipt $name $report
    Write-Host ('COLLECT-OK '+$name)
    return
}
$report.case=$Mode
$sandbox=Join-Path $caseRoot $Mode
$report.sandbox=$sandbox
$program=Join-Path $sandbox 'program'
try {
    if (Test-Path -LiteralPath $sandbox) { throw 'Case root already exists. Preserve it and use Collect.' }
    if ((Test-Path -LiteralPath $caseRoot) -and ((Get-Item -LiteralPath $caseRoot -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Unexpected root reparse point' }
    $manifest=Join-Path $PSScriptRoot 'payload.json'
    if ((Get-FileHash -LiteralPath $manifest).Hash -cne '94C9A9937A4254DFFB8791ABD6A8215B00615CD011F7580C32D6F7E5390E173F') { throw 'Payload manifest hash mismatch' }
    $rows=Get-Content -LiteralPath $manifest -Raw | ConvertFrom-Json
    if (@($rows).Count -lt 8) { throw 'Incomplete payload manifest' }
    [IO.Directory]::CreateDirectory($program) | Out-Null
    foreach ($row in $rows) {
        if ($Mode -eq 'Missing' -and $row.path -ceq $archive) { continue }
        if ($row.path -notmatch '^[A-Za-z0-9_.-]+(/[A-Za-z0-9_.-]+)*$' -or $row.path -match '(?:^|/)\.\.?(/|$)') { throw 'Unsafe payload member' }
        $target=Join-Path $program ([string]$row.path)
        [IO.Directory]::CreateDirectory((Split-Path -Parent $target)) | Out-Null
        Copy-Item -LiteralPath (Join-Path $stagedProgram ([string]$row.path)) -Destination $target
        if ((Get-Item -LiteralPath $target).Length -ne $row.bytes -or (Get-FileHash -LiteralPath $target).Hash -ine $row.sha256) { throw ('Payload mismatch: '+$row.path) }
    }
    $node=Join-Path $PSScriptRoot 'observer-node/runtime/node/node.exe'
    if ((Get-FileHash -LiteralPath $node).Hash -cne '5CBA0EA928508C65DDADEFC0681D2FB6D95F1F3BEEA4428F04397631140EE8E5') { throw 'Observer Node hash mismatch' }
    [Environment]::SetEnvironmentVariable('NODE_OPTIONS',$null,'Process')
    $socket=& $node -p 'typeof WebSocket'
    if ($LASTEXITCODE -ne 0 -or $socket -cne 'function') { throw 'Node WebSocket is unavailable' }
    $report.observerNodeVersion=(& $node --version)
    $listener=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0)
    $listener.Start()
    try { $port=$listener.LocalEndpoint.Port } finally { $listener.Stop() }
    $report.cdpPort=$port
    foreach ($name in 'HOME','CODEX_HOME','OPENCODEX_HOME','OPENCODEX_SANDBOX_BOUNDARY','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','NODE_OPTIONS') { [Environment]::SetEnvironmentVariable($name,$null,'Process') }
    $env:OPENCODEX_SANDBOX='1'
    $env:OPENCODEX_SANDBOX_ROOT=$sandbox
    $env:OPENCODEX_WINDOWS_SMOKE_CDP_PORT=[string]$port
    $env:WEBVIEW2_USER_DATA_FOLDER=Join-Path $sandbox 'EBWebView'
    $observerOut=Join-Path $receipts ($Mode+'-observer')
    New-Item -ItemType Directory -Path $observerOut | Out-Null
    $report.image=Join-Path $program 'opencodex-desktop.exe'
    $report.imageSha256=(Get-FileHash -LiteralPath $report.image).Hash.ToLowerInvariant()
    $script=Join-Path $PSScriptRoot 'observer/test/smoke/windows-webview.mjs'
    $arguments='"'+$script+'" http://127.0.0.1:'+$port+' "'+$observerOut+'" "'+$sandbox+'" observe-preparation'
    $observer=Start-Process -FilePath $node -ArgumentList $arguments -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $observerOut 'stdout.log') -RedirectStandardError (Join-Path $observerOut 'stderr.log')
    $report.observerPid=$observer.Id
    $ipcScript=Join-Path $PSScriptRoot 'team85-ipc.mjs'
    $ipcArguments='"'+$ipcScript+'" http://127.0.0.1:'+$port+' "'+(Join-Path $observerOut 'ipc-observation.jsonl')+'"'
    $ipc=Start-Process -FilePath $node -ArgumentList $ipcArguments -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $observerOut 'ipc-stdout.log') -RedirectStandardError (Join-Path $observerOut 'ipc-stderr.log')
    $report.ipcObserverPid=$ipc.Id
    $report.launchRequestedUtc=[DateTime]::UtcNow.ToString('o')
    $manager=Start-Process -FilePath $report.image -WorkingDirectory $program -PassThru -RedirectStandardOutput (Join-Path $sandbox 'manager-stdout.log') -RedirectStandardError (Join-Path $sandbox 'manager-stderr.log')
    $report.managerPid=$manager.Id
    $report.launchReturnedUtc=[DateTime]::UtcNow.ToString('o')
    Write-Host ('LAUNCH-OK '+$Mode+' PID='+$manager.Id)
} catch {
    $report.firstError=$_.Exception.Message
    throw
} finally {
    Save-Receipt ($Mode+'-launch.json') $report
}

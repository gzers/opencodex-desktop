$ErrorActionPreference = 'Stop'
$evidenceRoot = Split-Path -Parent $PSScriptRoot
$supervisor = 'C:\Users\15119\.codex\tmp\ocx-0110-single-session.ps1'
$candidate = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\software\candidateafff\opencodex-desktop.exe'
$pwsh = 'C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\native\powershell\pwsh.exe'
$expectedSource = 'afff490a430a81f8c233d44928aa3681a23e91e0'
$expectedExeSha = 'bf8a39e977ce5c554923f5f826a02b8f71cc37ed00ac08c258e436f104d9a7a1'
$expectedSupervisorSha = 'f13b4335ce9ef1eed94d4cafb2fd832f2af5f2da8ad4fef07d4e96acd4457d1e'
if ((Get-FileHash -LiteralPath $candidate -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expectedExeSha) {
    throw 'Candidate identity mismatch'
}
if ((Get-FileHash -LiteralPath $supervisor -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expectedSupervisorSha) {
    throw 'Supervisor identity mismatch'
}
$existing = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'")
if ($existing.Count) { throw 'Existing application still present; refuse elevated replacement' }
$preferencesFile = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\windows0110\data\manager-state\preferences.json'
$preferences = Get-Content -LiteralPath $preferencesFile -Raw | ConvertFrom-Json
if ($preferences.shell.auto_panel -ne $false -or
    $preferences.shell.launch_with_codex -ne $false -or
    $preferences.updates.app_update_auto_check -ne $false) {
    throw 'Startup panel, Codex launch and automatic checks must remain disabled'
}
$arguments = @(
    '-NoProfile', '-NonInteractive', '-File', $supervisor,
    '-Executable', $candidate, '-ExpectedSha256', $expectedExeSha,
    '-ExpectedCommit', $expectedSource, '-Mode', 'administrator'
)
# Every argument is a frozen whitespace-free path or value, not user shell text.
if (@($arguments | Where-Object { $_ -match '\s' }).Count) { throw 'Unexpected argument whitespace' }
$request = [ordered]@{
    requestedAtUtc = [DateTime]::UtcNow.ToString('o')
    sourceCommit = $expectedSource
    executableSha256 = $expectedExeSha
    supervisorSha256 = $expectedSupervisorSha
    requestedMode = 'administrator'
    requesterPid = $PID
    applicationCountBeforeRequest = $existing.Count
    status = 'requesting-standard-windows-runas'
    filePath = $pwsh
    arguments = $arguments
    windowStyle = 'Hidden'
    securitySettingsChanged = $false
    authenticationAutomated = $false
    packageLifecycleScriptsExecuted = $false
    installerExecuted = $false
    administratorApplicationTokenProved = $false
}
function Save-Request {
    $request | ConvertTo-Json -Depth 6 |
        Set-Content -LiteralPath (Join-Path $evidenceRoot 'elevation-request.json') -Encoding utf8
}
Save-Request
try {
    # User completes any Windows authorization prompt. No credentials are read,
    # supplied, stored or entered by this launcher.
    $process = Start-Process -FilePath $pwsh -ArgumentList $arguments -Verb RunAs -WindowStyle Hidden -PassThru
    $request.status = 'elevated-supervisor-started-token-not-yet-verified'
    $request.supervisorPid = $process.Id
    $request.returnedAtUtc = [DateTime]::UtcNow.ToString('o')
    Save-Request
    $process.Dispose()
} catch {
    $request.status = 'standard-runas-not-completed'
    $request.returnedAtUtc = [DateTime]::UtcNow.ToString('o')
    $request.errorType = $_.Exception.GetType().FullName
    $request.hresult = $_.Exception.HResult
    Save-Request
    throw
}
$request | ConvertTo-Json -Depth 6

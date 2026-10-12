$ErrorActionPreference = 'Stop'
$evidenceRoot = Split-Path -Parent $PSScriptRoot
$codeRoot = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$control = Join-Path $codeRoot 'test/runtime/windows0110/control/current-session.json'
$session = Get-Content -LiteralPath $control -Raw | ConvertFrom-Json
if ($session.applicationPid -ne 42372 -or $session.supervisorPid -ne 44052 -or
    $session.sourceCommit -ne 'afff490a430a81f8c233d44928aa3681a23e91e0') {
    throw 'Administrator session identity changed'
}
. (Join-Path $codeRoot 'test/smoke/windows-process-token.ps1')
$readerToken = Get-WindowsHarnessToken
Assert-WindowsProcessMode -Observation $readerToken -Mode administrator
$retainedApp = [Diagnostics.Process]::GetProcessById(42372)
try {
    $applicationToken = Get-WindowsProcessToken -Process $retainedApp -ExpectedExecutable $session.executable
    Assert-WindowsProcessMode -Observation $applicationToken -Mode administrator
} finally {
    $retainedApp.Dispose()
}
# This helper only reads the already selected native HWND and its owner.
& 'C:\Users\15119\.codex\tmp\ocx-0110-native-metrics.ps1' -WindowHandle 381618848 |
    Set-Content -LiteralPath (Join-Path $evidenceRoot 'administrator-native-metrics-elevated-reader.json') -Encoding utf8
if ($LASTEXITCODE -and $LASTEXITCODE -ne 0) { throw 'Native metrics reader failed' }
$vmResult = try {
    $machines = @(Get-VM -ErrorAction Stop)
    $machines | Select-Object Name, Id, State, Generation |
        ConvertTo-Json -Depth 4 |
        Set-Content -LiteralPath 'C:\Users\15119\.codex\tmp\ocx-0110-private-vm-inventory.json' -Encoding utf8
    @{ status = 'query_succeeded'; count = $machines.Count; privateNamesIncludedInPublicEvidence = $false; existingVmModified = $false }
} catch {
    @{ status = 'query_failed'; errorType = $_.Exception.GetType().FullName; hresult = $_.Exception.HResult; existingVmModified = $false }
}
$application = Get-CimInstance Win32_Process -Filter 'ProcessId=42372'
[ordered]@{
    collectedAtUtc = [DateTime]::UtcNow.ToString('o')
    applicationToken = $applicationToken
    readerToken = $readerToken
    sourceCommit = $session.sourceCommit
    applicationSessionId = $application.SessionId
    applicationParentPid = $application.ParentProcessId
    retainedApplicationCount = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'").Count
    vmInventory = $vmResult
    uiInputPerformed = $false
    securitySettingsChanged = $false
    scope = 'independent elevated read-only helper; same retained application, not a second application instance'
} | ConvertTo-Json -Depth 7 |
    Set-Content -LiteralPath (Join-Path $evidenceRoot 'independent-administrator-token.json') -Encoding utf8

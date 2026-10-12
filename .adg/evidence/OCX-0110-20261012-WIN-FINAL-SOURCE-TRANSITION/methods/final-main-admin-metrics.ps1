$ErrorActionPreference = 'Stop'
$code = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$evidence = 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-FINAL-SOURCE-TRANSITION'
$session = Get-Content -LiteralPath (Join-Path $code 'test/runtime/windows0110/control/current-session.json') -Raw |
    ConvertFrom-Json -DateKind String
if ($session.sourceCommit -cne 'eae85322ebca3b755c034db97cbebf2ea5bf71fc' -or
    $session.applicationPid -ne 40132 -or $session.supervisorPid -ne 36156 -or $session.status -ne 'running') {
    throw 'Final administrator lifecycle changed'
}
. (Join-Path $code 'test/smoke/windows-process-token.ps1')
$reader = Get-WindowsHarnessToken
Assert-WindowsProcessMode -Observation $reader -Mode administrator
$application = [Diagnostics.Process]::GetProcessById(40132)
try {
    $token = Get-WindowsProcessToken -Process $application -ExpectedExecutable $session.executable
    Assert-WindowsProcessMode -Observation $token -Mode administrator
    if ($token.creationTimeUtc -cne $session.applicationToken.creationTimeUtc) { throw 'Application identity changed' }
} finally {
    $application.Dispose()
}
& 'C:\Users\15119\.codex\tmp\ocx-0110-native-metrics.ps1' -WindowHandle 985452 |
    Set-Content -LiteralPath (Join-Path $evidence 'new-administrator-native-window.json') -Encoding utf8
@{
    collectedAtUtc = [DateTime]::UtcNow.ToString('o')
    applicationToken = $token
    readerToken = $reader
    sourceCommit = $session.sourceCommit
    applicationCount = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'").Count
    uiInputPerformed = $false
    scope = 'independent elevated read-only token and explicitly selected main HWND owner/DPI/DWM read'
} | ConvertTo-Json -Depth 7 |
    Set-Content -LiteralPath (Join-Path $evidence 'new-administrator-independent-token.json') -Encoding utf8

$ErrorActionPreference = 'Stop'
$taskTarget = Join-Path $PSScriptRoot 'checkpoint.original.json'
if (Test-Path -LiteralPath $taskTarget) { throw 'Do not replace a historical checkpoint' }
$taskPrior = Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-FG-493-CONTINUE'
$taskLaunch = Get-Content -LiteralPath (Join-Path $taskPrior 'ordinary-launch.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskAdmin = Get-Content -LiteralPath (Join-Path $taskPrior 'administrator-launch.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskSource = (& git -C $taskCode rev-parse HEAD).Trim()
if ($taskSource -cne $taskAdmin.source -or @(& git -C $taskCode status --porcelain).Count) {
    throw 'Code identity or cleanliness changed'
}
$taskFixtureDelta = @(& git -C $taskCode diff --name-only bd2f6eb10d58115235e35d575507bb69f44666da HEAD -- apps/desktop/tauri/src/infrastructure/windows_query_fixture.rs)
if ($taskFixtureDelta.Count) { throw 'Historical fixture source changed' }
$taskNative = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'" |
    Select-Object ProcessId, CreationDate, ExecutablePath)
if ($taskNative.Count -gt 1 -or ($taskNative.Count -eq 1 -and $taskNative[0].ProcessId -ne $taskAdmin.pid)) {
    throw 'Unexpected native instance'
}
$taskDaily = (Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLowerInvariant()
if ($taskDaily -cne $taskLaunch.dailyExeSha256) { throw 'Daily image changed' }
$taskCandidate = (Get-FileHash -LiteralPath $taskAdmin.executable).Hash.ToLowerInvariant()
if ($taskCandidate -cne $taskAdmin.sha256) { throw 'Candidate image changed' }
$taskProtected = [ordered]@{}
foreach ($taskProperty in $taskLaunch.protectedSandboxHashes.PSObject.Properties) {
    $taskHash = (Get-FileHash -LiteralPath (Join-Path $taskCode ('test/runtime/windows0110/data/manager-state/' + $taskProperty.Name))).Hash.ToLowerInvariant()
    if ($taskHash -cne $taskProperty.Value) { throw 'Protected sandbox changed' }
    $taskProtected[$taskProperty.Name] = $taskHash
}
$taskTaskHash = (Get-FileHash -LiteralPath 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\work\objects\task\TASK-OPENCODEX-DESKTOP-182.json').Hash.ToLowerInvariant()
if ($taskTaskHash -cne 'bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93') { throw 'Original TASK changed' }
[ordered]@{
    atUtc = [datetimeoffset]::UtcNow.ToString('o')
    source = $taskSource
    codeWorktreeClean = $true
    originalBd2FixtureSourceUnchanged = $true
    nativeCount = $taskNative.Count
    nativeProcesses = $taskNative
    administratorExitCode = $null
    dailyExeSha256 = $taskDaily
    candidateExeSha256 = $taskCandidate
    protectedSandboxHashes = $taskProtected
    originalTaskSha256 = $taskTaskHash
    productLaunched = $false
    nodeLaunched = $false
    etwSessionStarted = $false
    uiInputInjected = $false
    agentForcedTermination = $false
    macNativeHandoffPerformed = $false
    fullVersionGatePassed = $false
    taskClosed = $false
    releaseRequested = $false
} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskTarget -Encoding utf8

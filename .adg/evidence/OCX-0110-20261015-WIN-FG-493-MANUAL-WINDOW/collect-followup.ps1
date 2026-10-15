$ErrorActionPreference = 'Stop'
$taskCheckpoint = Join-Path $PSScriptRoot 'followup-checkpoint.original.json'
if (Test-Path -LiteralPath $taskCheckpoint) { throw 'Do not overwrite the followup checkpoint' }
$taskOriginal = Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-FG-493-CONTINUE'
$taskOrdinary = Get-Content -LiteralPath (Join-Path $taskOriginal 'ordinary-launch.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskAdmin = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-launch.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
if ((& git -C $taskCode rev-parse HEAD).Trim() -cne $taskAdmin.source -or
    @(& git -C $taskCode status --porcelain).Count) { throw 'Candidate source changed or worktree not clean' }
$taskCandidate = (Get-FileHash -LiteralPath $taskAdmin.executable).Hash.ToLowerInvariant()
if ($taskCandidate -cne $taskAdmin.sha256) { throw 'Candidate image changed' }
$taskDaily = (Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLowerInvariant()
if ($taskDaily -cne $taskOrdinary.dailyExeSha256) { throw 'Daily image changed' }
$taskProtected = [ordered]@{}
foreach ($taskProperty in $taskOrdinary.protectedSandboxHashes.PSObject.Properties) {
    $taskHash = (Get-FileHash -LiteralPath (Join-Path $taskCode ('test/runtime/windows0110/data/manager-state/' + $taskProperty.Name))).Hash.ToLowerInvariant()
    if ($taskHash -cne $taskProperty.Value) { throw 'Protected sandbox changed' }
    $taskProtected[$taskProperty.Name] = $taskHash
}
$taskTaskHash = (Get-FileHash -LiteralPath 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\work\objects\task\TASK-OPENCODEX-DESKTOP-182.json').Hash.ToLowerInvariant()
if ($taskTaskHash -cne 'bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93') { throw 'Original TASK changed' }
$taskNative = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'" |
    Select-Object ProcessId, CreationDate, ExecutablePath)
if ($taskNative.Count -gt 1 -or ($taskNative.Count -eq 1 -and $taskNative[0].ProcessId -ne $taskAdmin.pid)) {
    throw 'Unexpected native process'
}
$taskExitPath = Join-Path $PSScriptRoot 'followup-exit-observer-result.json'
$taskExit = $null
if (Test-Path -LiteralPath $taskExitPath) {
    $taskExit = Get-Content -LiteralPath $taskExitPath -Raw -Encoding utf8 | ConvertFrom-Json
    if ($taskExit.pid -ne $taskAdmin.pid) { throw 'Exit observer identity mismatch' }
}
[IO.File]::WriteAllBytes((Join-Path $PSScriptRoot 'app-log-followup.original.txt'),
    [IO.File]::ReadAllBytes((Join-Path $taskAdmin.dataRoot 'logs/app.log')))
[ordered]@{
    atUtc = [datetimeoffset]::UtcNow.ToString('o')
    source = $taskAdmin.source
    candidateExeSha256 = $taskCandidate
    codeWorktreeClean = $true
    nativeCount = $taskNative.Count
    nativeProcesses = $taskNative
    administratorExitObserverResult = $taskExit
    dailyExeUnchanged = $true
    dailyExeSha256 = $taskDaily
    protectedSandboxUnchanged = $true
    protectedSandboxHashes = $taskProtected
    originalTaskSha256 = $taskTaskHash
    uiInputInjected = $false
    productLaunched = $false
    agentForcedTermination = $false
    installerExecuted = $false
    macNativeHandoffPerformed = $false
    fullVersionGatePassed = $false
    taskClosed = $false
    releaseRequested = $false
} | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $taskCheckpoint -Encoding utf8

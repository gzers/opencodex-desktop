$ErrorActionPreference = 'Stop'
$taskDestination = Join-Path $PSScriptRoot 'checkpoint.original.json'
if (Test-Path -LiteralPath $taskDestination) { throw 'Do not replace the historical checkpoint' }
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskPrior = Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-FG-493-CONTINUE'
$taskLaunch = Get-Content -LiteralPath (Join-Path $taskPrior 'ordinary-launch.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskAdmin = Get-Content -LiteralPath (Join-Path $taskPrior 'administrator-launch.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskHead = (& git -C $taskCode rev-parse HEAD).Trim()
if ($taskHead -cne $taskAdmin.source -or @(& git -C $taskCode status --porcelain).Count) {
    throw 'Source identity or worktree cleanliness changed'
}
$taskMethods = @(
    'apps/desktop/ui/qa/tree-table.geometry.mjs',
    'apps/desktop/ui/qa/tree-table.geometry.html',
    'apps/desktop/ui/qa/tree-table.fixture.ts',
    'apps/desktop/ui/qa/TreeTableGeometry.vue',
    'apps/desktop/ui/src/components/ui/UiTreeTable.vue',
    'apps/desktop/ui/src/components/ui/treeTable.ts',
    'apps/desktop/ui/src/styles/base.css',
    'apps/desktop/ui/src/styles/tokens.css',
    'apps/desktop/ui/package-lock.json'
)
$taskSources = @($taskMethods | ForEach-Object {
    $taskFile = Join-Path $taskCode $_
    [ordered]@{path=$_;bytes=(Get-Item -LiteralPath $taskFile).Length;sha256=(Get-FileHash -LiteralPath $taskFile).Hash.ToLowerInvariant()}
})
$taskDaily = (Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLowerInvariant()
if ($taskDaily -cne $taskLaunch.dailyExeSha256) { throw 'Daily image changed' }
$taskProtected = [ordered]@{}
foreach ($taskProperty in $taskLaunch.protectedSandboxHashes.PSObject.Properties) {
    $taskHash = (Get-FileHash -LiteralPath (Join-Path $taskCode ('test/runtime/windows0110/data/manager-state/' + $taskProperty.Name))).Hash.ToLowerInvariant()
    if ($taskHash -cne $taskProperty.Value) { throw 'Protected sandbox changed' }
    $taskProtected[$taskProperty.Name] = $taskHash
}
$taskTaskHash = (Get-FileHash -LiteralPath 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\work\objects\task\TASK-OPENCODEX-DESKTOP-182.json').Hash.ToLowerInvariant()
if ($taskTaskHash -cne 'bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93') { throw 'Original TASK changed' }
$taskNative = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'" |
    Select-Object ProcessId, CreationDate, ExecutablePath)
if ($taskNative.Count -ne 1 -or $taskNative[0].ProcessId -ne $taskAdmin.pid) { throw 'Unexpected native process state; record a new phase' }
$taskAccounts = @(Get-CimInstance Win32_UserAccount -Filter "LocalAccount=True AND Name='ocx0110_accept'")
[ordered]@{
    atUtc = [datetimeoffset]::UtcNow.ToString('o')
    source = $taskHead
    codeWorktreeClean = $true
    sourceFiles = $taskSources
    nativeCount = $taskNative.Count
    nativeProcesses = $taskNative
    administratorExitCode = $null
    currentOrdinaryCimCannotReadElevatedImage = $true
    dailyExeSha256 = $taskDaily
    protectedSandboxHashes = $taskProtected
    originalTaskSha256 = $taskTaskHash
    isolatedAcceptanceAccountExists = ($taskAccounts.Count -gt 0)
    windowsSandboxExecutablePresent = (Test-Path -LiteralPath 'C:\Windows\System32\WindowsSandbox.exe')
    headlessBrowserOnly = $true
    productLaunched = $false
    nativeUiInputInjected = $false
    administratorWindowOperated = $false
    installerExecuted = $false
    newAccountOrProfileCreated = $false
    forcedTermination = $false
    macNativeHandoffPerformed = $false
    physicalDpiOrNativeWebviewAccepted = $false
    fullVersionGatePassed = $false
    taskClosed = $false
    releaseRequested = $false
} | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $taskDestination -Encoding utf8

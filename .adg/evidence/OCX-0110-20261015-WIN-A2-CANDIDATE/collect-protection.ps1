param(
    [ValidateSet('final-protection.original.json','after-ci-protection.original.json')]
    [string]$OutputName = 'final-protection.original.json'
)
$ErrorActionPreference = 'Stop'
$taskDestination = Join-Path $PSScriptRoot $OutputName
if (Test-Path -LiteralPath $taskDestination) { throw 'Do not overwrite this checkpoint' }
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskExpected = 'a2e4ea1786723ceab0a1ed0d322503ac79cd64e0'
if ((& git -C $taskCode rev-parse HEAD).Trim() -cne $taskExpected -or
    @(& git -C $taskCode status --porcelain).Count) { throw 'Source changed or worktree is not clean' }
$taskPrior = Get-Content -LiteralPath (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-FG-493-CONTINUE/ordinary-launch.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskDaily = (Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLowerInvariant()
if ($taskDaily -cne $taskPrior.dailyExeSha256) { throw 'Daily image changed' }
$taskProtected = [ordered]@{}
foreach ($taskProperty in $taskPrior.protectedSandboxHashes.PSObject.Properties) {
    $taskHash = (Get-FileHash -LiteralPath (Join-Path $taskCode ('test/runtime/windows0110/data/manager-state/' + $taskProperty.Name))).Hash.ToLowerInvariant()
    if ($taskHash -cne $taskProperty.Value) { throw 'Protected sandbox changed' }
    $taskProtected[$taskProperty.Name] = $taskHash
}
$taskTaskPath = 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\work\objects\task\TASK-OPENCODEX-DESKTOP-182.json'
$taskTaskHash = (Get-FileHash -LiteralPath $taskTaskPath).Hash.ToLowerInvariant()
if ($taskTaskHash -cne 'bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93') { throw 'Original TASK changed' }
$taskNative = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'" |
    Select-Object ProcessId, CreationDate, ExecutablePath)
if ($taskNative.Count) { throw 'Native product exists; cannot claim zero' }
$taskAccounts = @(Get-CimInstance Win32_UserAccount -Filter "LocalAccount=True AND Name='ocx0110_accept'")
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskExpected;codeWorktreeClean=$true
    applicationTree=(& git -C $taskCode rev-parse HEAD:apps/desktop).Trim()
    dailyExeSha256=$taskDaily;protectedSandboxHashes=$taskProtected;originalTaskSha256=$taskTaskHash
    originalTaskBytes=(Get-Item -LiteralPath $taskTaskPath).Length
    nativeCount=$taskNative.Count;nativeProcesses=$taskNative
    isolatedAcceptanceAccountExists=($taskAccounts.Count -gt 0)
    windowsSandboxExecutablePresent=(Test-Path -LiteralPath 'C:\Windows\System32\WindowsSandbox.exe')
    localProductOrOriginalLibraryOrFixedNodeStarted=$false;localEtwSessionStarted=$false
    localUiInputInjected=$false;hostInstallerExecuted=$false;newAccountOrProfileCreated=$false
    localStartupChoiceRegistryWritten=$false;permissionsOrSystemConfigurationChanged=$false
    macNativeHandoffPerformed=$false;fullVersionGatePassed=$false;taskClosed=$false;releaseRequested=$false
} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskDestination -Encoding utf8

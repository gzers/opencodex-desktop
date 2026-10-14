param(
    [Parameter(Mandatory=$true)][string]$Executable,
    [Parameter(Mandatory=$true)][string]$ExpectedSha256
)
$ErrorActionPreference = 'Stop'
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskSource = 'e6a4b32f8b60e362911cc02bc67fc0a2f82e7963'
$taskExe = (Resolve-Path -LiteralPath $Executable).Path
$taskSoftware = (Resolve-Path -LiteralPath (Join-Path $taskCode 'test/software')).Path.TrimEnd('\')
if (!$taskExe.StartsWith($taskSoftware + '\', [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Only admitted task-owned candidate executable may start'
}
if ((Get-FileHash -LiteralPath $taskExe).Hash.ToLowerInvariant() -cne $ExpectedSha256) {
    throw 'Candidate executable bytes differ from the verified package receipt'
}
if ((& git -C $taskCode rev-parse HEAD).Trim() -cne $taskSource) { throw 'Source changed' }
if ([DateTimeOffset]::Now -ge [DateTimeOffset]::Parse('2026-10-15T00:09:28+08:00')) {
    throw 'The approved 30-minute foreground window has ended'
}
$taskRuntime = Join-Path $taskCode 'test/runtime/foreground-e6a4b32-20261014'
if (Test-Path -LiteralPath $taskRuntime) { throw 'This foreground run already has output; inspect without repeating' }
if (@(Get-CimInstance Win32_Process | Where-Object {$_.Name -in @('opencodex-desktop.exe','ocxd.exe')}).Count) {
    throw 'A native instance is already present'
}
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
$taskHarness = Get-WindowsHarnessToken
Assert-WindowsProcessMode -Observation $taskHarness -Mode ordinary
New-Item -ItemType Directory -Path (Join-Path $taskRuntime 'data/manager-state') -Force | Out-Null
$taskPreferences = Get-Content -LiteralPath (Join-Path $taskCode 'apps/desktop/tauri/config/preferences.defaults.json') -Raw |
    ConvertFrom-Json
$taskPreferences.auto_panel = $false
$taskPreferences.launch_with_codex = $false
$taskPreferences.app_update_auto_check = $false
$taskPreferences.startup_cleanup = $false
# Fresh, explicit source-default fixture. No clock/cache/schedule fabrication.
$taskPreferences | ConvertTo-Json -Depth 10 |
    Set-Content -LiteralPath (Join-Path $taskRuntime 'data/manager-state/preferences.json') -Encoding utf8
$taskProtected = [ordered]@{}
foreach ($taskName in @('preferences.json','data-root.json','backup-cleanup-policy.json','update-schedule.json')) {
    $taskProtected[$taskName] = (Get-FileHash -LiteralPath (Join-Path $taskCode "test/runtime/windows0110/data/manager-state/$taskName")).Hash.ToLowerInvariant()
}
$taskDailyHash = (Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLowerInvariant()
$taskSocket = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0)
$taskSocket.Start(); $taskPort = $taskSocket.LocalEndpoint.Port; $taskSocket.Stop()
$taskInfo = [Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName = $taskExe
$taskInfo.WorkingDirectory = Split-Path -Parent $taskExe
$taskInfo.UseShellExecute = $false
$taskInfo.Environment['OPENCODEX_SANDBOX'] = '1'
$taskInfo.Environment['OPENCODEX_SANDBOX_ROOT'] = Join-Path $taskRuntime 'data'
$taskInfo.Environment['OPENCODEX_SANDBOX_BOUNDARY'] = $taskRuntime
$taskInfo.Environment['OPENCODEX_WINDOWS_SMOKE_CDP_PORT'] = [string]$taskPort
foreach ($taskName in @('OPENCODEX_WINDOWS_SMOKE_SCALE','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','WEBVIEW2_USER_DATA_FOLDER')) {
    [void]$taskInfo.Environment.Remove($taskName)
}
$taskProcess = [Diagnostics.Process]::Start($taskInfo)
$taskToken = Get-WindowsProcessToken -Process $taskProcess -ExpectedExecutable $taskExe
Assert-WindowsProcessMode -Observation $taskToken -Mode ordinary
$taskRecord = [ordered]@{
    atUtc = [DateTime]::UtcNow.ToString('o')
    authorization = '可以开始 Windows 前台验证，我现在有 30 分钟。'
    subsequentDpiDirection = '不用管dpi的问题'
    foregroundDeadline = '2026-10-15T00:09:28+08:00'
    source = $taskSource
    executable = $taskExe
    sha256 = $ExpectedSha256
    runtime = $taskRuntime
    pid = $taskProcess.Id
    creationTimeUtc = $taskProcess.StartTime.ToUniversalTime().ToString('o')
    applicationToken = $taskToken
    harnessToken = $taskHarness
    port = $taskPort
    dataRoot = Join-Path $taskRuntime 'data'
    dailyExeBefore = $taskDailyHash
    protectedSandboxBefore = $taskProtected
    physicalDpiAccepted = $false
    hostInstallerExecuted = $false
    forcedScaleUsed = $false
    globalTransparencySettingsChanged = $false
}
$taskRecord | ConvertTo-Json -Depth 10 |
    Set-Content -LiteralPath (Join-Path $PSScriptRoot 'ordinary-launch.json') -Encoding utf8
$taskRecord | ConvertTo-Json -Depth 10

param([ValidateSet('ordinary','administrator')][string]$Mode = 'ordinary')
$ErrorActionPreference = 'Stop'
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskExe = Join-Path $taskCode 'test/software/candidate-e6a4b32-38591604133/msi-readonly/opencodex-desktop.exe'
$taskSha = '2c9cad960fcee27b6eb1ed6ff9301b0d89101d29d24c16f0aa8a6b06ae01f6b3'
$taskSource = 'e6a4b32f8b60e362911cc02bc67fc0a2f82e7963'
$taskRuntime = Join-Path $taskCode 'test/runtime/foreground-tray-admin-e6-20261015'
$taskResultPath = Join-Path $PSScriptRoot "$Mode-launch.json"
if (Test-Path -LiteralPath $taskResultPath) { throw 'This permission-mode instance has already started; inspect instead of repeating' }
if ((Get-FileHash -LiteralPath $taskExe).Hash.ToLowerInvariant() -cne $taskSha) { throw 'Candidate bytes changed' }
if ((& git -C $taskCode rev-parse HEAD).Trim() -cne $taskSource) { throw 'Source changed' }
if (@(Get-CimInstance Win32_Process | Where-Object {$_.Name -in @('opencodex-desktop.exe','ocxd.exe')}).Count) {
    throw 'Native instance already present'
}
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
$taskHarness = Get-WindowsHarnessToken
Assert-WindowsProcessMode -Observation $taskHarness -Mode $Mode
if ($Mode -eq 'ordinary') {
    if (Test-Path -LiteralPath $taskRuntime) { throw 'Do not reuse an unreviewed runtime' }
    New-Item -ItemType Directory -Path (Join-Path $taskRuntime 'data/manager-state') -Force | Out-Null
    $taskPrefs = Get-Content -LiteralPath (Join-Path $taskCode 'apps/desktop/tauri/config/preferences.defaults.json') -Raw |
        ConvertFrom-Json
    $taskPrefs.auto_panel = $false
    $taskPrefs.launch_with_codex = $false
    $taskPrefs.app_update_auto_check = $false
    $taskPrefs.startup_cleanup = $false
    $taskPrefs.keep_proxy_on_close = $true
    $taskPrefs | ConvertTo-Json -Depth 10 |
        Set-Content -LiteralPath (Join-Path $taskRuntime 'data/manager-state/preferences.json') -Encoding utf8
} else {
    if (!(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'ordinary-exit-postcheck.json'))) {
        throw 'Ordinary mode must be exited and reviewed before administrator mode'
    }
    $taskExit = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'ordinary-exit-postcheck.json') -Raw | ConvertFrom-Json
    if ($taskExit.nativeCount -ne 0 -or !$taskExit.normalExitObserved) { throw 'Ordinary-mode exit not verified' }
}
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
Assert-WindowsProcessMode -Observation $taskToken -Mode $Mode
$taskRecord = [ordered]@{
    atUtc=[DateTime]::UtcNow.ToString('o'); authorization='继续前台验证'; dpiDirection='不用管dpi的问题'
    source=$taskSource; sha256=$taskSha; mode=$Mode; executable=$taskExe; runtime=$taskRuntime
    dataRoot=(Join-Path $taskRuntime 'data'); pid=$taskProcess.Id
    creationTimeUtc=$taskToken.creationTimeUtc; applicationToken=$taskToken; harnessToken=$taskHarness
    port=$taskPort; hostInstallerExecuted=$false; forcedScaleUsed=$false
}
$taskRecord | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $taskResultPath -Encoding utf8
$taskRecord | ConvertTo-Json -Depth 10

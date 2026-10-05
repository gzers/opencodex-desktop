param(
    [Parameter(Mandatory=$true)][string]$Executable,
    [Parameter(Mandatory=$true)][string]$Node,
    [Parameter(Mandatory=$true)][string]$OutputDirectory,
    [int]$ObserveSeconds = 30,
    [string]$ExpectedCommit = ''
)
$ErrorActionPreference = 'Stop'
$taskExe = (Resolve-Path -LiteralPath $Executable).Path
$taskNode = (Resolve-Path -LiteralPath $Node).Path
$taskOutput = [System.IO.Path]::GetFullPath($OutputDirectory)
if (!$ExpectedCommit) {
    $taskHead = & git rev-parse HEAD 2>$null
    if ($LASTEXITCODE -eq 0 -and $taskHead -match '^[0-9a-f]{40}$') { $ExpectedCommit = $taskHead }
}
New-Item -ItemType Directory -Path $taskOutput -Force | Out-Null
$taskSandbox = Join-Path $taskOutput ('sandbox-' + [guid]::NewGuid().ToString('N'))
$taskListener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback, 0)
$taskListener.Start()
$taskPort = $taskListener.LocalEndpoint.Port
$taskListener.Stop()
$taskInfo = [System.Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName = $taskExe
$taskInfo.WorkingDirectory = Split-Path -LiteralPath $taskExe
$taskInfo.UseShellExecute = $false
$taskInfo.CreateNoWindow = $true
$taskInfo.WindowStyle = 'Hidden'
$taskInfo.RedirectStandardError = $true
$taskInfo.RedirectStandardOutput = $true
[void]$taskInfo.Environment.Remove('HOME')
$taskInfo.Environment['OPENCODEX_SANDBOX'] = '1'
$taskInfo.Environment['OPENCODEX_SANDBOX_ROOT'] = $taskSandbox
$taskInfo.Environment['WEBVIEW2_USER_DATA_FOLDER'] = Join-Path $taskSandbox 'EBWebView'
$taskInfo.Environment['WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS'] = "--remote-debugging-port=$taskPort"
$taskProcess = [System.Diagnostics.Process]::Start($taskInfo)
$taskErrorRead = $taskProcess.StandardError.ReadToEndAsync()
$taskOutputRead = $taskProcess.StandardOutput.ReadToEndAsync()
try {
    $taskDeadline = (Get-Date).AddSeconds($ObserveSeconds)
    while ((Get-Date) -lt $taskDeadline) {
        if ($taskProcess.HasExited) { throw "Application exited before smoke completed: $($taskProcess.ExitCode)" }
        Start-Sleep -Milliseconds 250
    }
    $taskProcess.Refresh()
    if ($taskProcess.MainWindowHandle -eq 0) { throw 'Main window handle missing' }
    $taskLog = Join-Path $taskSandbox 'logs/app.log'
    if (!(Test-Path -LiteralPath $taskLog)) { throw 'Sandbox startup log missing' }
    if ($ExpectedCommit -and !(Get-Content -LiteralPath $taskLog -Raw).Contains("commit=$ExpectedCommit")) {
        throw "EXE build provenance does not match source commit $ExpectedCommit"
    }
    if (!(Test-Path -LiteralPath (Join-Path $taskSandbox 'manager-state/data-root.json'))) { throw 'Sandbox data root not initialized' }
    & $taskNode (Join-Path $PSScriptRoot 'windows-webview.mjs') "http://127.0.0.1:$taskPort" $taskOutput
    if ($LASTEXITCODE -ne 0) { throw "WebView smoke failed: $LASTEXITCODE" }
    @{
        result = 'pass'; executable = $taskExe; sha256 = (Get-FileHash -LiteralPath $taskExe -Algorithm SHA256).Hash.ToLower()
        homeRemoved = $true; sandbox = $taskSandbox; observedSeconds = $ObserveSeconds
        mainWindowPresent = $true; responding = $taskProcess.Responding; pid = $taskProcess.Id
        expectedCommit = $ExpectedCommit; sourceCommitMatched = [bool]$ExpectedCommit
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskOutput 'startup.json') -Encoding utf8
} finally {
    # 只结束本脚本启动的隔离 EXE；保留证据，不删除用户或测试数据目录。
    if (!$taskProcess.HasExited) { $taskProcess.Kill(); [void]$taskProcess.WaitForExit(10000) }
    if ($taskErrorRead.IsCompleted) { $taskErrorRead.Result | Set-Content -LiteralPath (Join-Path $taskOutput 'startup-stderr.log') -Encoding utf8 }
    if ($taskOutputRead.IsCompleted) { $taskOutputRead.Result | Set-Content -LiteralPath (Join-Path $taskOutput 'startup-stdout.log') -Encoding utf8 }
    $taskProcess.Dispose()
}

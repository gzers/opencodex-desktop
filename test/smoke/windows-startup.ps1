param(
    [Parameter(Mandatory=$true)][string]$Executable,
    [Parameter(Mandatory=$true)][string]$Node,
    [Parameter(Mandatory=$true)][string]$OutputDirectory,
    [ValidateRange(1,300)][int]$ObserveSeconds = 30,
    [string]$ExpectedCommit = ''
)
$ErrorActionPreference = 'Stop'
if (!$IsWindows) { throw 'This smoke requires Windows and PowerShell 7' }
$taskExe = (Resolve-Path -LiteralPath $Executable).Path
$taskNode = (Resolve-Path -LiteralPath $Node).Path
$taskOutput = [System.IO.Path]::GetFullPath($OutputDirectory)
if (!$ExpectedCommit) {
    $taskHead = & git rev-parse HEAD 2>$null
    if ($LASTEXITCODE -eq 0 -and $taskHead -match '^[0-9a-f]{40}$') { $ExpectedCommit = $taskHead }
}
if ($ExpectedCommit -notmatch '^[0-9a-f]{40}$') { throw 'A full source commit is required for EXE provenance verification' }

function Get-SmokeDiagnostics([int]$RootProcessId, [int]$Port, [datetime]$StartedAt) {
    $diagnostics = [ordered]@{ collectedAtUtc = [datetime]::UtcNow.ToString('o'); rootPid = $RootProcessId; cdpPort = $Port }
    try {
        if (!('OpenCodexSmokeToken' -as [type])) {
            Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
public static class OpenCodexSmokeToken {
    [DllImport("advapi32.dll", SetLastError = true)]
    private static extern bool GetTokenInformation(IntPtr token, int kind, out int value, int size, out int returned);
    public static int Read(IntPtr token, int kind) {
        if (!GetTokenInformation(token, kind, out int value, sizeof(int), out int returned))
            throw new Win32Exception(Marshal.GetLastWin32Error());
        return value;
    }
}
'@
        }
        $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
        try {
            $principal = [System.Security.Principal.WindowsPrincipal]::new($identity)
            $diagnostics.effectiveAdministrator = $principal.IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)
            $diagnostics.tokenElevated = [bool][OpenCodexSmokeToken]::Read($identity.Token, 20)
            $diagnostics.tokenElevationType = [OpenCodexSmokeToken]::Read($identity.Token, 18)
        } finally { $identity.Dispose() }
    } catch { $diagnostics.tokenError = $_.Exception.Message }
    try {
        # 只保存本脚本启动的进程及其后代；不导出其它应用的命令行。
        $processes = @(Get-CimInstance -ClassName Win32_Process -ErrorAction Stop)
        $owned = [System.Collections.Generic.HashSet[int]]::new()
        if ($RootProcessId -gt 0) { [void]$owned.Add($RootProcessId) }
        do {
            $changed = $false
            foreach ($candidate in $processes) {
                if ($owned.Contains([int]$candidate.ParentProcessId) -and $candidate.CreationDate -ge $StartedAt -and $owned.Add([int]$candidate.ProcessId)) { $changed = $true }
            }
        } while ($changed)
        $diagnostics.processes = @($processes | Where-Object { $owned.Contains([int]$_.ProcessId) } | Select-Object ProcessId, ParentProcessId, Name, ExecutablePath, CommandLine)
        $diagnostics.listeners = @(Get-NetTCPConnection -State Listen -ErrorAction Stop | Where-Object { $_.LocalPort -eq $Port } | Select-Object LocalAddress, LocalPort, OwningProcess)
    } catch { $diagnostics.processOrListenerError = $_.Exception.Message }
    return $diagnostics
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
$taskInfo.Environment['OPENCODEX_WINDOWS_SMOKE_CDP_PORT'] = [string]$taskPort
# 本轮明确验证应用传参；不让继承的 WebView2 环境覆盖掩盖结果。
[void]$taskInfo.Environment.Remove('WEBVIEW2_USER_DATA_FOLDER')
[void]$taskInfo.Environment.Remove('WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS')
$taskProcess = $null
$taskErrorRead = $null
$taskOutputRead = $null
$taskFailure = $null
$taskStartedAt = Get-Date
$taskResult = [ordered]@{
    result = 'fail'; executable = $taskExe; sha256 = (Get-FileHash -LiteralPath $taskExe -Algorithm SHA256).Hash.ToLower()
    homeRemoved = $true; sandbox = $taskSandbox; observedSeconds = $ObserveSeconds; cdpPort = $taskPort
    expectedCommit = $ExpectedCommit; sourceCommitMatched = $false; mainWindowPresent = $false
}
try {
    $taskProcess = [System.Diagnostics.Process]::Start($taskInfo)
    $taskResult.pid = $taskProcess.Id
    $taskErrorRead = $taskProcess.StandardError.ReadToEndAsync()
    $taskOutputRead = $taskProcess.StandardOutput.ReadToEndAsync()
    $taskDeadline = (Get-Date).AddSeconds($ObserveSeconds)
    while ((Get-Date) -lt $taskDeadline) {
        if ($taskProcess.HasExited) { throw "Application exited before smoke completed: $($taskProcess.ExitCode)" }
        Start-Sleep -Milliseconds 250
    }
    $taskProcess.Refresh()
    if ($taskProcess.MainWindowHandle -eq 0) { throw 'Main window handle missing' }
    $taskResult.mainWindowPresent = $true
    $taskResult.responding = $taskProcess.Responding
    $taskLog = Join-Path $taskSandbox 'logs/app.log'
    if (!(Test-Path -LiteralPath $taskLog)) { throw 'Sandbox startup log missing' }
    if (!(Get-Content -LiteralPath $taskLog -Raw).Contains("commit=$ExpectedCommit")) {
        throw "EXE build provenance does not match source commit $ExpectedCommit"
    }
    $taskResult.sourceCommitMatched = $true
    if (!(Test-Path -LiteralPath (Join-Path $taskSandbox 'manager-state/data-root.json'))) { throw 'Sandbox data root not initialized' }
    & $taskNode (Join-Path $PSScriptRoot 'windows-webview.mjs') "http://127.0.0.1:$taskPort" $taskOutput
    if ($LASTEXITCODE -ne 0) { throw "WebView smoke failed: $LASTEXITCODE" }
    $taskResult.result = 'pass'
} catch {
    $taskFailure = $_
    $taskResult.error = $_.Exception.Message
} finally {
    try {
        $taskRootId = if ($taskProcess) { $taskProcess.Id } else { 0 }
        Get-SmokeDiagnostics -RootProcessId $taskRootId -Port $taskPort -StartedAt $taskStartedAt | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskOutput 'startup-diagnostics.json') -Encoding utf8
    } catch { $taskResult.diagnosticsError = $_.Exception.Message }
    # 只结束本脚本启动的隔离进程树；失败也保留证据，不删除任何数据根。
    try {
        if ($taskProcess -and !$taskProcess.HasExited) { $taskProcess.Kill($true); [void]$taskProcess.WaitForExit(10000) }
        if ($taskErrorRead -and $taskErrorRead.Wait(5000)) { $taskErrorRead.Result | Set-Content -LiteralPath (Join-Path $taskOutput 'startup-stderr.log') -Encoding utf8 }
        if ($taskOutputRead -and $taskOutputRead.Wait(5000)) { $taskOutputRead.Result | Set-Content -LiteralPath (Join-Path $taskOutput 'startup-stdout.log') -Encoding utf8 }
    } catch {
        $taskResult.cleanupError = $_.Exception.Message
        $taskResult.result = 'fail'
        if (!$taskFailure) { $taskFailure = $_ }
    } finally { if ($taskProcess) { $taskProcess.Dispose() } }
    $taskResult | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $taskOutput 'startup.json') -Encoding utf8
}
if ($taskFailure) { throw $taskFailure }

# Embedded by windows_restart.rs. Inputs are UTF-16 hex literals, never evaluated
# as PowerShell source. No profile, registry, data-root or security-policy writes.
$ErrorActionPreference = 'Stop'
function Read-RestartString([string]$Hex) {
    $taskBytes = New-Object byte[] ($Hex.Length / 2)
    for ($taskIndex = 0; $taskIndex -lt $taskBytes.Length; $taskIndex++) {
        $taskBytes[$taskIndex] = [Convert]::ToByte($Hex.Substring($taskIndex * 2, 2), 16)
    }
    $taskEncoding = New-Object Text.UnicodeEncoding($false, $false, $true)
    return $taskEncoding.GetString($taskBytes)
}
function Restart-ImageIdentity([string]$Path) {
    if ($Path.StartsWith('\\?\') -and $Path.Length -gt 7 -and $Path[5] -eq ':') {
        return $Path.Substring(4)
    }
    return $Path
}
$taskParent = $null
try {
    # Pipe data is separate from executable PowerShell source. Long or unusual
    # application arguments therefore do not become shell code or bloat its CLI.
    $taskParentId = [int][Console]::In.ReadLine()
    $taskParentCreated = [long][Console]::In.ReadLine()
    $taskParentImageHex = [Console]::In.ReadLine()
    $taskTargetHex = [Console]::In.ReadLine()
    $taskDirectoryHex = [Console]::In.ReadLine()
    $taskArgumentsHex = [Console]::In.ReadLine()
    $taskExitTimeoutMs = [int][Console]::In.ReadLine()
    $taskInfo = New-Object Diagnostics.ProcessStartInfo
    $taskInfo.FileName = Read-RestartString $taskTargetHex
    $taskInfo.WorkingDirectory = Read-RestartString $taskDirectoryHex
    $taskInfo.Arguments = Read-RestartString $taskArgumentsHex
    $taskInfo.UseShellExecute = $false
    if (![IO.File]::Exists($taskInfo.FileName) -or
        ![IO.Directory]::Exists($taskInfo.WorkingDirectory) -or
        $taskExitTimeoutMs -lt 1 -or $taskExitTimeoutMs -gt 30000) {
        exit 46
    }
    $taskParent = [Diagnostics.Process]::GetProcessById($taskParentId)
    # Acquire the actual generation's handle before announcing readiness. Keep
    # it across the wait; reopening the numeric PID after exit is forbidden.
    [void]$taskParent.Handle
    $taskExpectedImage = Read-RestartString $taskParentImageHex
    if ($taskParent.StartTime.ToFileTimeUtc() -ne $taskParentCreated -or
        ![StringComparer]::OrdinalIgnoreCase.Equals(
            (Restart-ImageIdentity $taskParent.MainModule.FileName),
            (Restart-ImageIdentity $taskExpectedImage))) {
        exit 41
    }
    [Console]::Out.WriteLine('OCX_RESTART_READY_V1')
    [Console]::Out.Flush()
    # EOF/abort cannot turn an uncommitted preparation into a later launch.
    if ([Console]::In.ReadLine() -cne 'OCX_RESTART_COMMIT_V1') { exit 42 }
    if (!$taskParent.WaitForExit($taskExitTimeoutMs)) { exit 43 }
    $taskReplacement = [Diagnostics.Process]::Start($taskInfo)
    if ($null -eq $taskReplacement) { exit 44 }
    $taskReplacement.Dispose()
} catch {
    # Do not publish exception text, environment, paths or arguments.
    exit 45
} finally {
    if ($null -ne $taskParent) { $taskParent.Dispose() }
}

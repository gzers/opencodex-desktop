param([Parameter(Mandatory=$true)][string]$OutputDirectory)
$ErrorActionPreference = 'Stop'
if (!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7) { throw 'Windows PowerShell 7 required' }
. (Join-Path $PSScriptRoot 'windows-process-token.ps1')
$taskOutput = [IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Path $taskOutput -Force | Out-Null
$taskSummary = [ordered]@{
    schemaVersion = 1; result = 'fail'; checks = @(); harnessToken = $null; childToken = $null
    sourceCommit = (& git rev-parse HEAD); powershell = $PSVersionTable.PSVersion.ToString()
    scope = 'read-only token helper; does not accept native UI or dual-permission application gates'
}
$taskChild = $null
function Test-TokenCheck([string]$Name, [scriptblock]$Action) {
    try {
        $detail = & $Action
        $taskSummary.checks += [ordered]@{ name = $Name; result = 'pass'; detail = $detail }
    } catch {
        $taskSummary.checks += [ordered]@{ name = $Name; result = 'fail'; error = $_.Exception.Message }
    }
}
function Assert-TokenThrows([scriptblock]$Action, [string]$Message) {
    $caught = $false
    try { & $Action | Out-Null }
    catch {
        if (!$_.Exception.Message.Contains($Message)) { throw }
        $caught = $true
    }
    if (!$caught) { throw 'Invalid target or permission was accepted' }
}
try {
    Test-TokenCheck 'parse-all-windows-smoke-scripts' {
        $files = @(Get-ChildItem -LiteralPath $PSScriptRoot -Filter '*.ps1')
        foreach ($file in $files) {
            $tokens = $null; $parseErrors = $null
            [void][Management.Automation.Language.Parser]::ParseFile($file.FullName, [ref]$tokens, [ref]$parseErrors)
            if ($parseErrors.Count) { throw ($parseErrors | Out-String) }
        }
        @{ files = $files.Name }
    }
    $taskSummary.harnessToken = Get-WindowsHarnessToken
    Test-TokenCheck 'harness-process-identity' {
        Assert-WindowsProcessMode $taskSummary.harnessToken
        if ($taskSummary.harnessToken.processId -ne $PID) { throw 'Harness PID mismatch' }
        @{ processId = $taskSummary.harnessToken.processId }
    }
    # cmd waits on our open stdin pipe. No GUI, UAC prompt, installation or network.
    $taskInfo = [Diagnostics.ProcessStartInfo]::new()
    $taskInfo.FileName = $env:ComSpec
    $taskInfo.UseShellExecute = $false; $taskInfo.CreateNoWindow = $true
    $taskInfo.RedirectStandardInput = $true
    $taskInfo.ArgumentList.Add('/d'); $taskInfo.ArgumentList.Add('/c')
    $taskInfo.ArgumentList.Add('set /p OCX_TOKEN_TEST=')
    $taskChild = [Diagnostics.Process]::Start($taskInfo)
    $taskSummary.childToken = Get-WindowsProcessToken -Process $taskChild -ExpectedExecutable $taskInfo.FileName
    Test-TokenCheck 'actual-child-identity-and-inherited-token' {
        $observed = $taskSummary.childToken
        Assert-WindowsProcessMode $observed
        if ($observed.processId -ne $taskChild.Id -or $observed.processId -eq $PID) { throw 'Child PID replaced by harness PID' }
        if ([datetime]::Parse($observed.creationTimeUtc).ToUniversalTime() -ne $taskChild.StartTime.ToUniversalTime()) { throw 'Process creation time mismatch' }
        if ($observed.elevated -ne $taskSummary.harnessToken.elevated) { throw 'Unexpected inherited elevation' }
        $mode = if ($observed.elevated) { 'administrator' } else { 'ordinary' }
        Assert-WindowsProcessMode -Observation $observed -Mode $mode
        @{ processId = $observed.processId; mode = $mode; elevationType = $observed.elevationType }
    }
    Test-TokenCheck 'reject-wrong-executable' {
        Assert-TokenThrows { Get-WindowsProcessToken -Process $taskChild -ExpectedExecutable $taskSummary.harnessToken.executable } 'executable does not match'
    }
    Test-TokenCheck 'reject-opposite-permission-mode' {
        $opposite = if ($taskSummary.childToken.elevated) { 'ordinary' } else { 'administrator' }
        Assert-TokenThrows { Assert-WindowsProcessMode -Observation $taskSummary.childToken -Mode $opposite } 'does not match requested mode'
    }
    Test-TokenCheck 'reject-missing-token-evidence' {
        Assert-TokenThrows { Assert-WindowsProcessMode -Observation @{status='unavailable'} } 'missing or invalid'
    }
    Test-TokenCheck 'reject-string-elevation-value' {
        $invalid = [ordered]@{schemaVersion=1;status='observed';processId=$taskChild.Id;elevated='false';elevationType=1}
        Assert-TokenThrows { Assert-WindowsProcessMode -Observation $invalid } 'missing or invalid'
    }
    $taskChild.Kill($true); [void]$taskChild.WaitForExit(5000)
    Test-TokenCheck 'reject-exited-process' {
        Assert-TokenThrows { Get-WindowsProcessToken -Process $taskChild -ExpectedExecutable $taskInfo.FileName } 'Target process has exited'
    }
    $taskChild.Dispose()
    Test-TokenCheck 'reject-disposed-process' {
        # Process.SafeHandle must fail rather than reopening this PID.
        $caught = $false
        try { Get-WindowsProcessToken -Process $taskChild -ExpectedExecutable $taskInfo.FileName | Out-Null }
        catch { $caught = $true }
        if (!$caught) { throw 'Disposed process was accepted' }
    }
    $taskChild = $null
    if (@($taskSummary.checks | Where-Object {$_.result -ne 'pass'}).Count) { throw 'Process-token self-check failed' }
    $taskSummary.result = 'pass'
} catch { $taskSummary.error = $_.Exception.Message }
finally {
    if ($taskChild) {
        try { if (!$taskChild.HasExited) { $taskChild.Kill($true); [void]$taskChild.WaitForExit(5000) } }
        finally { $taskChild.Dispose() }
    }
    $taskSummary.collectedAtUtc = [datetime]::UtcNow.ToString('o')
    $taskSummary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskOutput 'process-token.json') -Encoding utf8
}
if ($taskSummary.result -ne 'pass') { throw 'Windows process-token evidence checks failed; see process-token.json' }

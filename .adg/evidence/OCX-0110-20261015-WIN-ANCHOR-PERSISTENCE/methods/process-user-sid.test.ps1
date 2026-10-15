$ErrorActionPreference = 'Stop'
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskDestination = Join-Path (Split-Path -Parent $PSScriptRoot) 'sid-method-test.original.json'
if (Test-Path -LiteralPath $taskDestination) { throw 'Preserve this original method test' }
$taskBefore = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'")
if ($taskBefore.Count) { throw 'Keep method-only validation separate from native acceptance' }
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
. (Join-Path $PSScriptRoot 'process-user-sid.ps1')
$taskExpected = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData(
    [Text.Encoding]::UTF8.GetBytes([Security.Principal.WindowsIdentity]::GetCurrent().User.Value)
)).ToLowerInvariant()
$taskChecks = [Collections.Generic.List[string]]::new()
$taskSelf = [Diagnostics.Process]::GetCurrentProcess()
$taskChild = $null
try {
    $taskExecutable = $taskSelf.MainModule.FileName
    $taskSelfObservation = Get-AcceptanceProcessSidEvidence -Process $taskSelf -ExpectedExecutable $taskExecutable
    if ($taskSelfObservation.sidSha256 -cne $taskExpected -or $taskSelfObservation.processId -ne $PID) {
        throw 'Retained current-process SID does not match the independently queried current identity'
    }
    $taskChecks.Add('retained-target-current-identity-matches-independent-identity')
    $taskInfo = [Diagnostics.ProcessStartInfo]::new($taskExecutable)
    $taskInfo.UseShellExecute=$false
    $taskInfo.CreateNoWindow=$true
    $taskInfo.WindowStyle='Hidden'
    $taskInfo.RedirectStandardInput=$true
    foreach ($taskArgument in @('-NoLogo','-NoProfile','-NonInteractive','-Command','[Console]::ReadLine() | Out-Null')) {
        $taskInfo.ArgumentList.Add($taskArgument)
    }
    $taskChild = [Diagnostics.Process]::Start($taskInfo)
    $taskChildObservation = Get-AcceptanceProcessSidEvidence -Process $taskChild -ExpectedExecutable $taskExecutable
    if ($taskChildObservation.sidSha256 -cne $taskExpected -or
        $taskChildObservation.processId -ne $taskChild.Id -or $taskChildObservation.processId -eq $PID) {
        throw 'Retained child identity mismatch'
    }
    $taskChecks.Add('independent-child-token-sid-matches-with-distinct-process-generation')
    $taskRejected = $false
    try {
        Get-AcceptanceProcessSidEvidence -Process $taskChild -ExpectedExecutable (Join-Path $PSScriptRoot 'not-this-process.exe') | Out-Null
    } catch { $taskRejected = $true }
    if (!$taskRejected) { throw 'A wrong expected executable was accepted' }
    $taskChecks.Add('wrong-executable-rejected-before-publishing-user-evidence')
    $taskChild.StandardInput.WriteLine('finish')
    if (!$taskChild.WaitForExit(5000) -or $taskChild.ExitCode -ne 0) { throw 'Owned child did not exit normally' }
    $taskRejected = $false
    try {
        Get-AcceptanceProcessSidEvidence -Process $taskChild -ExpectedExecutable $taskExecutable | Out-Null
    } catch { $taskRejected = $true }
    if (!$taskRejected) { throw 'Exited target was accepted as a current identity' }
    $taskChecks.Add('exited-target-rejected-without-reopening-a-pid')
    $taskAfter = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'")
    if ($taskAfter.Count) { throw 'Unexpected native instance appeared during method-only validation' }
    $taskResult = [ordered]@{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');result='passed';scope='Read-only SID method with owned PowerShell child only'
        source=(& git -C $taskCode rev-parse HEAD).Trim()
        checks=$taskChecks.ToArray();checksPassed=$taskChecks.Count
        harness=$taskSelfObservation;ownedChild=$taskChildObservation
        childExitedNormally=$true;nativeBeforeCount=$taskBefore.Count;nativeAfterCount=$taskAfter.Count
        targetApplicationObserved=$false;dualPermissionOrSameRootAcceptanceProved=$false
        registryOrAccountOrAclChanged=$false;foregroundInputInjected=$false;productStarted=$false
    }
    $taskJson = $taskResult | ConvertTo-Json -Depth 8
    if ($taskJson.Contains([Security.Principal.WindowsIdentity]::GetCurrent().User.Value)) {
        throw 'Raw SID leaked into method receipt'
    }
    $taskJson | Set-Content -LiteralPath $taskDestination -Encoding utf8
    [ordered]@{result='passed';checksPassed=$taskChecks.Count;productStarted=$false;registryOrAccountOrAclChanged=$false} | ConvertTo-Json
} finally {
    if ($taskChild) {
        try {
            if (!$taskChild.HasExited) {
                $taskChild.StandardInput.WriteLine('finish')
                if (!$taskChild.WaitForExit(5000)) { $taskChild.Kill(); [void]$taskChild.WaitForExit(5000) }
            }
        } finally { $taskChild.Dispose() }
    }
    $taskSelf.Dispose()
}

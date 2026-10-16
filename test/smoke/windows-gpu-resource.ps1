# Read-only companion for a separately bound native launch. Use outside a
# formal idle counter span; this does not start, activate, or stop a process.
function Get-WindowsWebViewRole {
    param([AllowNull()][string]$CommandLine)
    if ([string]::IsNullOrWhiteSpace($CommandLine)) { return 'unavailable' }
    if (!('OpenCodexOwnedArgv' -as [type])) {
        Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
public static class OpenCodexOwnedArgv {
    [DllImport("shell32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    static extern IntPtr CommandLineToArgvW(string value, out int count);
    [DllImport("kernel32.dll")] static extern IntPtr LocalFree(IntPtr value);
    public static string[] Split(string value) {
        if (value.Length > 32767) throw new ArgumentException("Command line too long.");
        int count;
        IntPtr pointer = CommandLineToArgvW(value, out count);
        if (pointer == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error());
        try {
            var result = new string[count];
            for (int i=0; i<count; i++)
                result[i] = Marshal.PtrToStringUni(Marshal.ReadIntPtr(pointer, i*IntPtr.Size));
            return result;
        } finally { LocalFree(pointer); }
    }
}
'@
    }
    $taskArguments = @([OpenCodexOwnedArgv]::Split($CommandLine) | Select-Object -Skip 1)
    $taskTypes = @($taskArguments | Where-Object {$_.StartsWith('--type=', [StringComparison]::Ordinal)})
    if ($taskTypes.Count -eq 0) {
        if ($taskArguments -ccontains '--type') { return 'unrecognized' }
        return 'browser-no-type-flag'
    }
    if ($taskTypes.Count -ne 1) { return 'ambiguous' }
    $taskValue = $taskTypes[0].Substring(7)
    switch -CaseSensitive ($taskValue) {
        'gpu-process' { return 'gpu' }
        'renderer' { return 'renderer' }
        'utility' { return 'utility' }
        'crashpad-handler' { return 'crash-handler' }
        default { return 'unrecognized' }
    }
}

function Select-WindowsOwnedGpuRows {
    param([AllowEmptyCollection()][object[]]$Rows, [Parameter(Mandatory)][int[]]$OwnedPids)
    $taskOwned = [Collections.Generic.HashSet[int]]::new()
    foreach ($taskId in $OwnedPids) {
        if ($taskId -le 0) { throw 'Positive bound process identities required' }
        [void]$taskOwned.Add($taskId)
    }
    $taskSeen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($taskRow in $Rows) {
        $taskMatch = [regex]::Match([string]$taskRow.Name, '^pid_([0-9]+)_')
        $taskId = 0
        if (!$taskMatch.Success -or
            ![int]::TryParse($taskMatch.Groups[1].Value, [ref]$taskId) -or
            !$taskOwned.Contains($taskId)) { continue }
        if (!$taskSeen.Add([string]$taskRow.Name)) { throw 'Duplicate GPU counter instance' }
        $taskResult = [ordered]@{pid=$taskId;instance=[string]$taskRow.Name}
        foreach ($taskField in @('DedicatedUsage','SharedUsage','TotalCommitted','LocalUsage',
                'NonLocalUsage','Timestamp_Sys100NS','Frequency_Sys100NS')) {
            $taskValue = $null
            if ($null -ne $taskRow.$taskField) {
                $taskNumber = [uint64]0
                if (![uint64]::TryParse([string]$taskRow.$taskField, [ref]$taskNumber)) {
                    throw 'Invalid unsigned GPU counter value'
                }
                # Preserve UInt64 precision and missing values across JSON readers.
                $taskValue = $taskNumber.ToString([Globalization.CultureInfo]::InvariantCulture)
            }
            $taskResult[$taskField] = $taskValue
        }
        [pscustomobject]$taskResult
    }
}

# Shared read-only binding reader. Controlled probes require native0; the
# desktop entry below retains the stricter image, mode and native1 boundary.
function Get-WindowsBoundGpuResourceSnapshot {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Launch,
        [Parameter(Mandatory)][ValidateSet('any','ordinary','administrator')][string]$Mode,
        [switch]$RequireDesktopInstance
    )
    $ErrorActionPreference = 'Stop'
    if (!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7) { throw 'Windows PowerShell 7 required' }
    if ($Launch.exeSha256 -notmatch '^[0-9a-f]{64}$' -or
        [string]::IsNullOrWhiteSpace([string]$Launch.executable)) {
        throw 'An explicit bound launch image and digest are required'
    }
    if ((Get-FileHash -LiteralPath $Launch.executable).Hash.ToLowerInvariant() -cne $Launch.exeSha256) {
        throw 'Bound launch image digest changed'
    }
    . (Join-Path $PSScriptRoot 'windows-process-token.ps1')
    $taskStarted = [datetimeoffset]::UtcNow
    $taskHandles = [Collections.Generic.List[Diagnostics.Process]]::new()
    try {
        $taskRoot = [Diagnostics.Process]::GetProcessById([int]$Launch.pid)
        [void]$taskRoot.SafeHandle
        $taskHandles.Add($taskRoot)
        if ($taskRoot.StartTime.ToUniversalTime().Ticks -ne
            ([datetimeoffset]$Launch.creationTimeUtc).UtcTicks) { throw 'Bound root generation changed' }
        $taskToken = Get-WindowsProcessToken -Process $taskRoot -ExpectedExecutable $Launch.executable
        Assert-WindowsProcessMode -Observation $taskToken -Mode $Mode
        $taskRows = @(Get-CimInstance -Query 'SELECT ProcessId,ParentProcessId,CreationDate,Name FROM Win32_Process')
        $taskNative = @($taskRows | Where-Object Name -in @('opencodex-desktop.exe','ocxd.exe'))
        if ($RequireDesktopInstance) {
            if ($taskNative.Count -ne 1 -or $taskNative[0].ProcessId -ne $taskRoot.Id) {
                throw 'A single bound native instance is required'
            }
        } elseif ($taskNative.Count) {
            throw 'Controlled probes require no desktop product instance'
        }
        $taskFamily = @{}
        $taskRootRow = @($taskRows | Where-Object ProcessId -eq $taskRoot.Id)
        if ($taskRootRow.Count -ne 1) { throw 'Root missing from process snapshot' }
        $taskFamily[$taskRoot.Id] = $taskRootRow[0]
        do {
            $taskAdded = $false
            foreach ($taskRow in $taskRows) {
                $taskId = [int]$taskRow.ProcessId
                $taskParent = $taskFamily[[int]$taskRow.ParentProcessId]
                if ($taskFamily.ContainsKey($taskId) -or $null -eq $taskParent) { continue }
                if ([datetime]$taskRow.CreationDate -lt [datetime]$taskParent.CreationDate) { continue }
                $taskFamily[$taskId] = $taskRow
                $taskAdded = $true
            }
        } while ($taskAdded)
        $taskMembers = [Collections.Generic.List[object]]::new()
        foreach ($taskRow in ($taskFamily.Values | Sort-Object CreationDate,ProcessId)) {
            $taskId = [int]$taskRow.ProcessId
            $taskProcess = if ($taskId -eq $taskRoot.Id) {$taskRoot} else {
                $taskChild = [Diagnostics.Process]::GetProcessById($taskId)
                [void]$taskChild.SafeHandle
                $taskHandles.Add($taskChild)
                $taskChild
            }
            $taskCreated = $taskProcess.StartTime.ToUniversalTime()
            # WMI CreationDate has microsecond precision; retain the actual
            # process handle and validate that same generation after all reads.
            if ([math]::Abs($taskCreated.Ticks -
                ([datetime]$taskRow.CreationDate).ToUniversalTime().Ticks) -gt 9) {
                throw 'Descendant generation changed'
            }
            $taskRole = 'other-owned-process'
            if ($taskId -eq $taskRoot.Id) {
                $taskRole = if ($RequireDesktopInstance) {'native-root'} else {'controlled-root'}
            }
            elseif ($taskRow.Name -ceq 'msedgewebview2.exe') {
                if ([IO.Path]::GetFileName($taskProcess.MainModule.FileName) -ine 'msedgewebview2.exe') {
                    throw 'Owned WebView image changed'
                }
                $taskCommand = Get-CimInstance -Query "SELECT CommandLine FROM Win32_Process WHERE ProcessId=$taskId"
                $taskRole = Get-WindowsWebViewRole -CommandLine $taskCommand.CommandLine
            }
            $taskProcess.Refresh()
            $taskMembers.Add([pscustomobject]@{pid=$taskId;creationUtc=$taskCreated.ToString('o')
                role=$taskRole;workingSetBytes=$taskProcess.WorkingSet64})
        }
        $taskCounterStatus = 'available'
        $taskCounters = $null
        try {
            $taskCounters = @(Select-WindowsOwnedGpuRows -OwnedPids @($taskFamily.Keys) -Rows @(
                Get-CimInstance -ClassName Win32_PerfFormattedData_GPUPerformanceCounters_GPUProcessMemory))
        } catch [Microsoft.Management.Infrastructure.CimException] {
            if ([string]$_.Exception.StatusCode -cne 'InvalidClass') { throw }
            # GPU-less CI hosts may not expose this performance provider.
            # Missing provider data remains null, never a zero-memory sample.
            $taskCounterStatus = 'unavailable-class'
        }
        foreach ($taskProcess in $taskHandles) {
            if ($taskProcess.HasExited) { throw 'Owned process exited during the snapshot' }
        }
        [pscustomobject][ordered]@{
            startedAtUtc=$taskStarted.ToString('o');finishedAtUtc=[datetimeoffset]::UtcNow.ToString('o')
            rootPid=$taskRoot.Id;rootCreationUtc=$Launch.creationTimeUtc;exeSha256=$Launch.exeSha256
            actualRootToken=$taskToken;members=$taskMembers.ToArray();gpuCounterInstances=$taskCounters
            gpuCounterClass='Win32_PerfFormattedData_GPUPerformanceCounters_GPUProcessMemory'
            gpuCounterReadStatus=$taskCounterStatus;desktopRootVerified=[bool]$RequireDesktopInstance
            controlledSnapshotOnly=(!$RequireDesktopInstance)
            missingCounterInstancesAreNotZero=$true;rawCommandLinesSaved=$false
            unrelatedProcessRowsSerialized=$false;retainedHandlesUsed=$true
            shortLivedChildrenFullyCovered=$false;individualGlBuffersAttributed=$false
            snapshotOnly=$true;formalIdleSpanMeasured=$false;uiInputInjected=$false
            processesStartedOrStopped=$false;fullPerformanceGatePassed=$false
        }
    } finally {
        foreach ($taskProcess in $taskHandles) { $taskProcess.Dispose() }
    }
}

function Get-WindowsGpuResourceSnapshot {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Launch,
        [Parameter(Mandatory)][ValidateSet('ordinary','administrator')][string]$Mode
    )
    if ($Launch.exeSha256 -notmatch '^[0-9a-f]{64}$' -or
        [IO.Path]::GetFileName([string]$Launch.executable) -cne 'opencodex-desktop.exe') {
        throw 'An explicit desktop launch image and digest are required'
    }
    Get-WindowsBoundGpuResourceSnapshot -Launch $Launch -Mode $Mode -RequireDesktopInstance
}

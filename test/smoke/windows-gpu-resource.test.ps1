param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'windows-gpu-resource.ps1')
$taskChecks = 0
function Require-GpuCheck([bool]$Value) {
    if (!$Value) { throw 'GPU companion evidence regression' }
    $script:taskChecks++
}
foreach ($taskCase in @(
    @('webview.exe --type=gpu-process --user-data-dir=PRIVATE','gpu'),
    @('webview.exe "--type=renderer"','renderer'),
    @('webview.exe --type="utility"','utility'),
    @('webview.exe --type=crashpad-handler','crash-handler'),
    @('webview.exe --type=gpu-process --type=renderer','ambiguous'),
    @('webview.exe --type=PRIVATE_SECRET','unrecognized'),
    @('webview.exe --not-type=gpu-process','browser-no-type-flag'),
    @('webview.exe --user-data-dir="C:\folder --type=gpu-process more"','browser-no-type-flag'),
    @('webview.exe --type','unrecognized'),
    @('','unavailable')
)) {
    Require-GpuCheck ((Get-WindowsWebViewRole -CommandLine $taskCase[0]) -ceq $taskCase[1])
}
$taskRows = @(
    [pscustomobject]@{Name='pid_11_luid_a_phys_0';DedicatedUsage=[uint64]18446744073709551615;SharedUsage=0;Secret='PRIVATE'},
    [pscustomobject]@{Name='pid_111_luid_b_phys_0';DedicatedUsage=999;Secret='UNRELATED'},
    [pscustomobject]@{Name='pid_11x_luid_c_phys_0';DedicatedUsage=999},
    [pscustomobject]@{Name='_Total';DedicatedUsage=999}
)
$taskSelected = @(Select-WindowsOwnedGpuRows -Rows $taskRows -OwnedPids @(11))
Require-GpuCheck ($taskSelected.Count -eq 1)
Require-GpuCheck ($taskSelected[0].DedicatedUsage -ceq '18446744073709551615')
Require-GpuCheck ($taskSelected[0].SharedUsage -ceq '0')
Require-GpuCheck ($null -eq $taskSelected[0].TotalCommitted)
Require-GpuCheck (!(($taskSelected | ConvertTo-Json) -match 'PRIVATE|UNRELATED|Secret|pid_111'))
Require-GpuCheck (@(Select-WindowsOwnedGpuRows -Rows @() -OwnedPids @(11)).Count -eq 0)
foreach ($taskInvalid in @(
    @([pscustomobject]@{Name='pid_11_luid_a_phys_0';DedicatedUsage=-1}),
    @($taskRows[0],$taskRows[0])
)) {
    $taskRejected = $false
    try { Select-WindowsOwnedGpuRows -Rows $taskInvalid -OwnedPids @(11) | Out-Null }
    catch { $taskRejected = $true }
    Require-GpuCheck $taskRejected
}
$taskRejected = $false
try {
    Get-WindowsGpuResourceSnapshot -Launch ([pscustomobject]@{
        executable=(Get-Process -Id $PID).Path;exeSha256=('0'*64)
    }) -Mode ordinary | Out-Null
} catch {
    Require-GpuCheck ($_.Exception.Message -eq 'An explicit desktop launch image and digest are required')
    $taskRejected = $true
}
Require-GpuCheck $taskRejected
$taskTokens = $null; $taskErrors = $null
[void][Management.Automation.Language.Parser]::ParseFile(
    (Join-Path $PSScriptRoot 'windows-gpu-resource.ps1'),[ref]$taskTokens,[ref]$taskErrors)
Require-GpuCheck ($taskErrors.Count -eq 0)
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Use a fresh output directory' }
New-Item -ItemType Directory -Path $OutputDirectory | Out-Null
$taskChild = $null
$taskNormalExit = $false
try {
    $taskInfo = [Diagnostics.ProcessStartInfo]::new()
    $taskInfo.FileName = (Get-Process -Id $PID).Path
    $taskInfo.WorkingDirectory = [IO.Path]::GetFullPath($OutputDirectory)
    $taskInfo.UseShellExecute = $false
    $taskInfo.CreateNoWindow = $true
    $taskInfo.RedirectStandardInput = $true
    $taskInfo.RedirectStandardOutput = $true
    $taskInfo.RedirectStandardError = $true
    foreach ($taskArgument in @('-NoLogo','-NoProfile','-NonInteractive','-Command',
            '[Console]::WriteLine("ready"); $null = [Console]::In.ReadLine()')) {
        $taskInfo.ArgumentList.Add($taskArgument)
    }
    $taskChild = [Diagnostics.Process]::Start($taskInfo)
    [void]$taskChild.SafeHandle
    $taskReady = $taskChild.StandardOutput.ReadLineAsync()
    if (!$taskReady.Wait(10000)) { throw 'Controlled process readiness timed out' }
    Require-GpuCheck ($taskReady.Result -ceq 'ready')
    $taskLaunch = [pscustomobject]@{
        pid=$taskChild.Id;executable=$taskInfo.FileName
        creationTimeUtc=$taskChild.StartTime.ToUniversalTime().ToString('o')
        exeSha256=(Get-FileHash -LiteralPath $taskInfo.FileName).Hash.ToLowerInvariant()
    }
    $taskLaunch | ConvertTo-Json | Set-Content -LiteralPath (
        Join-Path $OutputDirectory 'controlled-launch.original.json') -Encoding utf8
    foreach ($taskGuard in @('generation','digest')) {
        $taskBad = $taskLaunch.PSObject.Copy()
        $taskExpected = if ($taskGuard -eq 'generation') {
            $taskBad.creationTimeUtc = ([datetimeoffset]$taskLaunch.creationTimeUtc).AddTicks(1).ToString('o')
            'Bound root generation changed'
        } else {
            $taskBad.exeSha256 = '0'*64
            'Bound launch image digest changed'
        }
        $taskRejected = $false
        try { Get-WindowsBoundGpuResourceSnapshot -Launch $taskBad -Mode any | Out-Null }
        catch {
            Require-GpuCheck ($_.Exception.Message -eq $taskExpected)
            $taskRejected = $true
        }
        Require-GpuCheck $taskRejected
    }
    $taskSnapshot = Get-WindowsBoundGpuResourceSnapshot -Launch $taskLaunch -Mode any
    $taskSnapshot | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (
        Join-Path $OutputDirectory 'controlled-snapshot.original.json') -Encoding utf8
    Require-GpuCheck ($taskSnapshot.rootPid -eq $taskChild.Id)
    Require-GpuCheck ($taskSnapshot.actualRootToken.processId -eq $taskChild.Id)
    Require-GpuCheck ($taskSnapshot.controlledSnapshotOnly -and !$taskSnapshot.desktopRootVerified)
    Require-GpuCheck ($taskSnapshot.members.Count -ge 1)
    Require-GpuCheck (@($taskSnapshot.members | Where-Object {
        $_.pid -eq $taskChild.Id -and $_.role -ceq 'controlled-root'
    }).Count -eq 1)
    Require-GpuCheck (!$taskSnapshot.rawCommandLinesSaved -and !$taskSnapshot.unrelatedProcessRowsSerialized)
    Require-GpuCheck (!$taskSnapshot.uiInputInjected -and !$taskSnapshot.processesStartedOrStopped)
    Require-GpuCheck ($taskSnapshot.gpuCounterReadStatus -cin @('available','unavailable-class'))
    $taskMemberIds = @($taskSnapshot.members.pid)
    Require-GpuCheck (@($taskSnapshot.gpuCounterInstances | Where-Object {
        $null -ne $_ -and $_.pid -notin $taskMemberIds
    }).Count -eq 0)
    Require-GpuCheck (!$taskSnapshot.individualGlBuffersAttributed -and !$taskSnapshot.fullPerformanceGatePassed)
} finally {
    if ($taskChild) {
        if (!$taskChild.HasExited) {
            $taskChild.StandardInput.WriteLine('finish')
            $taskChild.StandardInput.Close()
        }
        if (!$taskChild.WaitForExit(10000)) { throw 'Controlled process normal exit timed out; no forced termination' }
        $taskNormalExit = $taskChild.ExitCode -eq 0
        [ordered]@{pid=$taskChild.Id;exitObserved=$true;exitCode=$taskChild.ExitCode
            forcedTermination=$false;controlledProcessOnly=$true} | ConvertTo-Json |
            Set-Content -LiteralPath (Join-Path $OutputDirectory 'controlled-exit.original.json') -Encoding utf8
        $taskChild.Dispose()
    }
}
Require-GpuCheck $taskNormalExit
$taskResult = [ordered]@{status='passed';checks=$taskChecks;productStarted=$false
    controlledSnapshotExecutionTested=$true;controlledNormalExit=$taskNormalExit
    nativeSnapshotExecutionTested=$false;gpuBuffersAttributed=$false;fullPerformanceGatePassed=$false}
$taskResult | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $OutputDirectory 'result.json') -Encoding utf8
$taskResult | ConvertTo-Json

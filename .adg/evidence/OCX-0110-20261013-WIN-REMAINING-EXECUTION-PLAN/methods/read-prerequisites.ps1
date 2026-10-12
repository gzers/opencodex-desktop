$ErrorActionPreference = 'Stop'
$taskBase = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\windows0110'
$taskSession = Get-Content -LiteralPath ($taskBase + '\control\current-session.json') -Raw | ConvertFrom-Json -DateKind String
$taskMonitor = Get-Content -LiteralPath ($taskBase + '\cycles\eae-stable-panel-20261012\monitor-state.json') -Raw | ConvertFrom-Json -DateKind String
if ($taskSession.sourceCommit -cne 'eae85322ebca3b755c034db97cbebf2ea5bf71fc' -or $taskSession.applicationPid -ne 37276 -or
    $taskSession.status -cne 'running' -or $taskMonitor.monitorPid -ne 3920) { throw 'Original final-source lifecycle differs' }
$taskProcesses = @()
foreach ($taskIdentity in @(
    @{ Id = 37276; Start = '2026-10-12T14:53:12.0088010Z' },
    @{ Id = 24300; Start = '2026-10-12T14:53:10.4430287Z' },
    @{ Id = 3920; Start = '2026-10-12T14:55:52.1278297Z' }
)) {
    $taskProcess = Get-Process -Id $taskIdentity.Id -ErrorAction Stop
    try {
        $taskHandle = $taskProcess.SafeHandle
        if ($taskHandle.IsInvalid -or $taskProcess.HasExited -or
            $taskProcess.StartTime.ToUniversalTime() -ne [datetime]::Parse($taskIdentity.Start).ToUniversalTime()) {
            throw 'Original process handle or creation identity differs'
        }
        $taskProcesses += [ordered]@{
            processId = $taskProcess.Id; actualLiveHandle = $true
            creationTimeUtc = $taskProcess.StartTime.ToUniversalTime().ToString('o')
            executable = $taskProcess.Path
        }
    } finally { $taskProcess.Dispose() }
}
$taskCount = @(Get-CimInstance Win32_Process -Filter "Name = 'opencodex-desktop.exe'").Count
if ($taskCount -ne 1) { throw 'One-desktop constraint differs' }
$taskDisk = @(Get-CimInstance Win32_LogicalDisk -Filter 'DriveType = 3' |
    Select-Object DeviceID, FileSystem, Size, FreeSpace)
$taskVmms = Get-Service vmms -ErrorAction SilentlyContinue
$taskCandidateUser = Get-LocalUser -Name 'ocx0110_accept' -ErrorAction SilentlyContinue
$taskDaily = Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{7069DA97-F262-4585-98CA-AE416E6833A5}' -ErrorAction Stop
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskCandidateMsi = $taskCode + '\test\software\downloads\11750970498\apps\desktop\tauri\target\release\bundle\msi\OpenCodeX Desktop_0.1.10_x64_en-US.msi'
$taskMsiHash = (Get-FileHash -LiteralPath $taskCandidateMsi -Algorithm SHA256).Hash.ToLowerInvariant()
if ($taskMsiHash -cne '17f70d357f163a881a41b740614b7506a654207041bc2ac3e7ad57717b6c73f6') { throw 'Final candidate installer bytes differ' }
[ordered]@{
    collectedAtUtc = [datetime]::UtcNow.ToString('o')
    sourceCommit = $taskSession.sourceCommit; applicationPid = 37276
    retainedProcesses = $taskProcesses; desktopApplicationCount = $taskCount
    applicationToken = $taskSession.applicationToken
    managerDeadlineEpoch = $taskMonitor.managerDeadlineEpoch
    panelDeadlineEpoch = $taskMonitor.panelDeadlineEpoch
    cycleStatus = $taskMonitor.status; cycleExecSession = 46484; cycleAccepted = $taskMonitor.fullCycleAccepted
    currentFixedDisks = $taskDisk
    vmms = if ($taskVmms) { @{ status = $taskVmms.Status.ToString(); startType = $taskVmms.StartType.ToString() } } else { $null }
    currentVmInventoryQueried = $false
    vmServiceAvailabilityProvesUsableVm = $false
    candidatePerformanceUser = 'ocx0110_accept'
    candidatePerformanceUserExists = [bool]$taskCandidateUser
    proposedPerformanceRoot = 'D:\OpenCodex-Acceptance\0.1.10-20261013'
    proposedPerformanceRootExists = Test-Path -LiteralPath 'D:\OpenCodex-Acceptance\0.1.10-20261013'
    finalCandidateMsiSha256 = $taskMsiHash
    dailyRegistration = @{
        productCode = '{7069DA97-F262-4585-98CA-AE416E6833A5}'
        displayName = $taskDaily.DisplayName; displayVersion = $taskDaily.DisplayVersion
        installLocation = $taskDaily.InstallLocation; windowsInstaller = $taskDaily.WindowsInstaller
    }
    dailyExecutableSha256 = (Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe' -Algorithm SHA256).Hash.ToLowerInvariant()
    task182LocalExists = Test-Path -LiteralPath '.adg\work\objects\task\TASK-OPENCODEX-DESKTOP-182.json'
    accountOrProfileCreated = $false; virtualMachineCreated = $false; installerExecuted = $false
    permissionsChanged = $false; passwordOrCredentialCollected = $false
    sourceChannelOrClockChanged = $false; productOrCycleRestarted = $false
} | ConvertTo-Json -Depth 9

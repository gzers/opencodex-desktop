$ErrorActionPreference = 'Stop'
$taskDestination = Join-Path $PSScriptRoot 'environment-readiness.original.json'
if (Test-Path -LiteralPath $taskDestination) { throw 'Do not overwrite environment observation' }
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskSource = (& git -C $taskCode rev-parse HEAD).Trim()
if ($taskSource -cne '43142bdbcc7a0b9f1325103febae3f44dce407ab' -or
    @(& git -C $taskCode status --porcelain).Count) { throw 'Source changed or dirty' }
$taskNative = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'" |
    Select-Object ProcessId, CreationDate, ExecutablePath)
if ($taskNative.Count) { throw 'Unexpected native product instance' }
$taskVm = [ordered]@{querySucceeded=$false;virtualMachineCount=$null;states=$null;errorType=$null}
try {
    $taskVms = @(Get-VM -ErrorAction Stop)
    $taskVm.querySucceeded = $true
    $taskVm.virtualMachineCount = $taskVms.Count
    $taskVm.states = @($taskVms | Group-Object State | ForEach-Object {
        [ordered]@{state=[string]$_.Name;count=$_.Count}
    })
} catch { $taskVm.errorType = $_.Exception.GetType().Name }
$taskSystem = Get-CimInstance Win32_ComputerSystem
$taskOs = Get-CimInstance Win32_OperatingSystem
$taskAccount = @(Get-CimInstance Win32_UserAccount -Filter "LocalAccount=True AND Name='ocx0110_accept'")
$taskPrior = Get-Content -LiteralPath (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-FG-493-CONTINUE/ordinary-launch.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskDaily = (Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLowerInvariant()
if ($taskDaily -cne $taskPrior.dailyExeSha256) { throw 'Daily image changed' }
$taskProtected = [ordered]@{}
foreach ($taskProperty in $taskPrior.protectedSandboxHashes.PSObject.Properties) {
    $taskHash = (Get-FileHash -LiteralPath (Join-Path $taskCode ('test/runtime/windows0110/data/manager-state/' + $taskProperty.Name))).Hash.ToLowerInvariant()
    if ($taskHash -cne $taskProperty.Value) { throw 'Protected sandbox changed' }
    $taskProtected[$taskProperty.Name] = $taskHash
}
$taskTaskPath = 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\work\objects\task\TASK-OPENCODEX-DESKTOP-182.json'
$taskTaskHash = (Get-FileHash -LiteralPath $taskTaskPath).Hash.ToLowerInvariant()
if ($taskTaskHash -cne 'bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93') { throw 'Original TASK changed' }
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskSource;codeWorktreeClean=$true
    nativeCount=$taskNative.Count;nativeProcesses=$taskNative
    logicalProcessors=$taskSystem.NumberOfLogicalProcessors
    totalPhysicalMemoryBytes=$taskSystem.TotalPhysicalMemory
    freePhysicalMemoryMiB=[math]::Round($taskOs.FreePhysicalMemory/1024,1)
    freeDiskGiB=@(Get-PSDrive -Name C,D | ForEach-Object {
        [ordered]@{drive=$_.Name;freeGiB=[math]::Round($_.Free/1GB,1)}
    })
    hypervisorPresent=$taskSystem.HypervisorPresent
    hyperVModuleAvailable=(@(Get-Module -ListAvailable -Name Hyper-V).Count -gt 0)
    vmmsStatus=[string](Get-Service -Name vmms -ErrorAction SilentlyContinue).Status
    vmInventory=$taskVm
    vmNamesOrUserDataSaved=$false;vmUsabilityProven=$false
    windowsSandboxExecutablePresent=(Test-Path -LiteralPath 'C:\Windows\System32\WindowsSandbox.exe')
    isolatedAcceptanceAccountExists=($taskAccount.Count -gt 0)
    dailyExeSha256=$taskDaily;protectedSandboxHashes=$taskProtected;originalTaskSha256=$taskTaskHash
    accountOrProfileCreated=$false;vmCreatedOrStarted=$false;systemConfigurationOrAclChanged=$false
    productStarted=$false;foregroundInputIssued=$false;hostInstallerExecuted=$false
    fullVersionGatePassed=$false;releaseRequested=$false
} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskDestination -Encoding utf8

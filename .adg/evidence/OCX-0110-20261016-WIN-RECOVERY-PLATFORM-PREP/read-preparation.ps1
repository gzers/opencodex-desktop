param([Parameter(Mandatory)][string]$PreparationDirectory)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'platform-binding.ps1')
. (Join-Path $PSScriptRoot 'directory-identity.ps1')
$taskReady=Assert-RecoveryPlatformRootsAbsent -PreparationDirectory $PreparationDirectory
$taskPlan=Get-Content -LiteralPath (Join-Path $PreparationDirectory 'profile-isolation-plan.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskSaved=@($taskPlan.physicalProfiles | ForEach-Object {Get-RecoveryDirectoryIdentity -Path $_.destination})
$taskPointer=(Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop').StartupDataRootV1
$taskPointerSha=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($taskPointer))).ToLowerInvariant()
if($taskPointerSha -cne $taskPlan.startupPointerSha256){throw 'Preparation changed the startup pointer'}
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskDiagnostic='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop'
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
$taskToken=Get-WindowsHarnessToken
Assert-WindowsProcessMode -Observation $taskToken -Mode ordinary
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');ready=$taskReady;savedNativeDirectories=$taskSaved
    actualObservationHarnessToken=$taskToken;scope='ordinary calling Codex host context; not another launcher or administrator product proof'
    pointerUnchangedExact=$true;pointerSha256=$taskPointerSha
    dailyExecutableSha256=(Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLowerInvariant()
    taskSha256=(Get-FileHash -LiteralPath 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\work\objects\task\TASK-OPENCODEX-DESKTOP-182.json').Hash.ToLowerInvariant()
    productSource=(& git -C $taskCode rev-parse HEAD).Trim()
    diagnosticSource=(& git -C $taskDiagnostic rev-parse HEAD).Trim()
    productTrackedChanges=@(& git -C $taskCode status --porcelain --untracked-files=no).Count
    diagnosticTrackedChanges=@(& git -C $taskDiagnostic status --porcelain --untracked-files=no).Count
    productStarted=$false;installerExecuted=$false;foregroundWindowGranted=$false
    dpiSpecialMatrixCancelledByUser=$true;fullRecoveryGatePassed=$false} |
    ConvertTo-Json -Depth 9 | Set-Content -LiteralPath (Join-Path $PreparationDirectory 'final-preparation-readback.json') -Encoding utf8

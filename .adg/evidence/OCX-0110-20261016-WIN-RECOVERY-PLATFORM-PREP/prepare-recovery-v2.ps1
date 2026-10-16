param([Parameter(Mandatory)][string]$RunDirectory,[Parameter(Mandatory)][string]$PreparationDirectory)
$ErrorActionPreference='Stop'
$taskFrozen='D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261016-WIN-823-FRONT-PREP'
. (Join-Path $taskFrozen 'run-binding.ps1')
$taskRun=Assert-PreparedIdleRun -RunDirectory $RunDirectory -RequireCurrentHumanWindow
. (Join-Path $PSScriptRoot 'platform-binding.ps1')
Assert-RecoveryPlatformRootsAbsent -PreparationDirectory $PreparationDirectory |
    ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $taskRun 'platform-preflight-before-prepare.json') -Encoding utf8
& (Join-Path $taskFrozen 'prepare-recovery.ps1') -RunDirectory $taskRun

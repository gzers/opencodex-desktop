$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'foreground-contract.ps1')
$taskNow=[datetimeoffset]'2026-10-15T01:35:00Z'
$taskBase=[ordered]@{
    origin='human';foregroundStartGranted=$true;userText='synthetic in-memory valid-input guard case, not actual user permission'
    source='ca26e771b0808b8b1b58c7ffadf5d1bad46189ff';scope='isolated-backup-tree'
    atUtc='2026-10-15T01:34:00Z';windowMinutes=30
}
$taskPositive=Assert-BackupForegroundAuthorization -Authorization ([pscustomobject]$taskBase) -AtUtc $taskNow
if ($taskPositive.remainingSeconds -ne 1740) { throw 'Expected the exact remaining window.' }
$taskRejected=0
foreach ($taskCase in @(
    @{origin='goal-continuation'},@{foregroundStartGranted=$false},@{userText=''},
    @{source=('0'*40)},@{scope='release'},@{atUtc='2026-10-14T00:00:00Z'},
    @{atUtc='2026-10-15T01:36:00Z'},@{windowMinutes=0},@{windowMinutes=31},@{windowMinutes=1.5}
)) {
    $taskInput=$taskBase.Clone()
    foreach ($taskKey in $taskCase.Keys) {$taskInput[$taskKey]=$taskCase[$taskKey]}
    $taskFailed=$false
    try {Assert-BackupForegroundAuthorization -Authorization ([pscustomobject]$taskInput) -AtUtc $taskNow | Out-Null}
    catch {$taskFailed=$true}
    if (!$taskFailed) {throw 'Invalid authorization input was admitted.'}
    $taskRejected++
}
[ordered]@{
    atUtc=[DateTime]::UtcNow.ToString('o');validInputChecks=1;negativeGuardChecks=$taskRejected
    purelyInMemory=$true;actualForegroundPermissionGranted=$false
    applicationStarted=$false;runtimeCreated=$false;preferencesWritten=$false
    inputInjected=$false;syntheticAuthorizationFileWritten=$false
} | ConvertTo-Json

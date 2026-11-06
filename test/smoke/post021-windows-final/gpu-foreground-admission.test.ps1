$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'gpu-foreground-admission.ps1')
$taskNow=[datetimeoffset]'2026-11-05T04:50:00Z'
$taskIdentity=[pscustomobject]@{pid=75988;createdAtUtcTicks='639294472023734550'}
$taskGood=@{pid=75988;createdAtUtcTicks='639294472023734550';foreground=$true;at='2026-11-05T04:50:00Z';foregroundSince='2026-11-05T04:49:57Z'}
if(!(Test-GpuForegroundHeartbeat ([pscustomobject]$taskGood) $taskIdentity $taskNow)){throw 'Fresh stable native foreground must be admitted'}
$taskCases=@(
    @{foreground=$false},@{pid=42},@{createdAtUtcTicks='639294472023734551'},
    @{at='2026-11-05T04:49:58Z'},@{at='2026-11-05T04:50:01Z'},
    @{foregroundSince='2026-11-05T04:49:58Z'},@{at='invalid'},@{foregroundSince=$null}
)
foreach($taskCase in $taskCases){
    $taskValue=$taskGood.Clone()
    foreach($taskKey in $taskCase.Keys){$taskValue[$taskKey]=$taskCase[$taskKey]}
    if(Test-GpuForegroundHeartbeat ([pscustomobject]$taskValue) $taskIdentity $taskNow){throw 'Unsafe foreground heartbeat admitted'}
}
Write-Output 'Foreground admission: stable positive and8 stale/changed/background/malformed negatives passed'

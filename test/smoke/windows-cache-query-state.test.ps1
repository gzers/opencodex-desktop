param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'windows-cache-query-state.ps1')
$taskRepo=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
if(!$taskOutput.StartsWith((Join-Path $taskRepo 'test')+'\',[StringComparison]::OrdinalIgnoreCase) -or
    (Test-Path -LiteralPath $taskOutput)){throw 'Fresh test-owned output directory required'}
New-Item -ItemType Directory -Path (Join-Path $taskOutput 'fixture\manager-state') -Force|Out-Null
New-Item -ItemType Directory -Path (Join-Path $taskOutput 'fixture\cache\updates') -Force|Out-Null
$taskRoot=Join-Path $taskOutput 'fixture'
$taskNow=[datetimeoffset]::UtcNow
$taskStart=$taskNow.ToUnixTimeSeconds()-60
$taskPrefs=[ordered]@{
    schema_version=2
    appearance=@{theme='system';visual_effects='high';glow_render='mesh';interface_scale=100}
    shell=@{auto_panel=$false;launch_with_codex=$false;keep_proxy_on_close=$true;launch_main=$true}
    updates=@{app_update_auto_check=$false;app_update_channel='stable';app_update_check_interval_seconds=86400}
    maintenance=@{startup_cleanup=$false}
}
$taskState=@{schema=1;entries=@{
    'manager-stable'=@{next_due=$taskStart+86400;failures=0;last_success=$taskStart}
    'panel-latest'=@{next_due=$taskStart+86400;failures=0;last_success=$taskStart}
}}
$taskManager=@{channel='stable';current_version='0.1.10';error=$null
    last_checked_at=[datetimeoffset]::FromUnixTimeSeconds($taskStart).ToString('o')}
$taskPanel=@{tag='latest';version='fixture-not-a-real-query';integrity='sha512-'+[Convert]::ToBase64String([byte[]]::new(64))}
$taskFiles=@{
    'manager-state\preferences.json'=$taskPrefs
    'manager-state\update-schedule.json'=$taskState
    'cache\updates\manager-stable.json'=$taskManager
    'cache\updates\panel-latest.json'=$taskPanel
}
foreach($taskFile in $taskFiles.Keys){
    $taskFiles[$taskFile]|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $taskRoot $taskFile) -Encoding utf8
}
$taskChecks=0
$taskBefore=Get-CacheQueryFixtureReadiness -Root $taskRoot -Now $taskNow -Through $taskNow.AddMinutes(30)
if(!$taskBefore.cacheValidThroughHorizon -or $taskBefore.preferencesReady){throw 'Unprepared source fixture misclassified'}
$taskChecks++
function Assert-ReadinessRejected([string]$Message){
    $rejected=$false
    try{Get-CacheQueryFixtureReadiness -Root $taskRoot -Now $taskNow -Through $taskNow.AddMinutes(30) -RequirePreparedPreferences|Out-Null}
    catch{if(!$_.Exception.Message.Contains($Message)){throw};$rejected=$true}
    if(!$rejected){throw "Expected readiness refusal: $Message"}
}
Assert-ReadinessRejected 'automatic checks enabled'
$taskChecks++
$taskPrefFile=Join-Path $taskRoot 'manager-state\preferences.json'
$taskOriginal=[IO.File]::ReadAllBytes($taskPrefFile)
$taskSeed=New-CacheQueryPreferenceBytes -Original $taskOriginal
[IO.File]::WriteAllBytes($taskPrefFile,$taskSeed)
$taskPrepared=Get-CacheQueryFixtureReadiness -Root $taskRoot -Now $taskNow -Through $taskNow.AddMinutes(30) -RequirePreparedPreferences
if(!$taskPrepared.preferencesReady){throw 'Prepared preferences not admitted'}
$taskChecks++
foreach($taskFile in $taskBefore.cacheFiles|Where-Object relative -ne 'manager-state\preferences.json'){
    if((Get-FileHash -LiteralPath (Join-Path $taskRoot $taskFile.relative)).Hash.ToLowerInvariant() -cne $taskFile.sha256){
        throw 'Preference setup changed a cache response or deadline'
    }
}
$taskChecks++
$taskState.entries.'panel-latest'.failures=$false
$taskState|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $taskRoot 'manager-state\update-schedule.json') -Encoding utf8
Assert-ReadinessRejected 'Real successful cache deadline'
$taskChecks++
$taskState.entries.'panel-latest'.failures=0
$taskState|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $taskRoot 'manager-state\update-schedule.json') -Encoding utf8
$taskBadPrefs=[Text.Encoding]::UTF8.GetString($taskSeed)|ConvertFrom-Json
$taskBadPrefs.shell.launch_main=$false
$taskBadPrefs|ConvertTo-Json -Depth 12|Set-Content -LiteralPath $taskPrefFile -Encoding utf8
Assert-ReadinessRejected 'automatic checks enabled'
$taskChecks++
[IO.File]::WriteAllBytes($taskPrefFile,$taskSeed)
$taskState.entries.'panel-latest'.failures=1
$taskState|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $taskRoot 'manager-state\update-schedule.json') -Encoding utf8
Assert-ReadinessRejected 'Real successful cache deadline'
$taskChecks++
$taskState.entries.'panel-latest'.failures=0
$taskState.entries.'panel-latest'.last_success=$taskNow.ToUnixTimeSeconds()-86400+60
$taskState.entries.'panel-latest'.next_due=$taskNow.ToUnixTimeSeconds()+60
$taskState|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $taskRoot 'manager-state\update-schedule.json') -Encoding utf8
Assert-ReadinessRejected 'Real successful cache deadline'
$taskChecks++
$taskState.entries.'panel-latest'.last_success=$taskStart
$taskState.entries.'panel-latest'.next_due=$taskStart+86400
$taskState|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $taskRoot 'manager-state\update-schedule.json') -Encoding utf8
$taskManager.current_version='0.1.9'
$taskManager|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $taskRoot 'cache\updates\manager-stable.json') -Encoding utf8
Assert-ReadinessRejected 'Current stable0.1.10'
$taskChecks++
$taskManager.current_version='0.1.10'
$taskManager|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $taskRoot 'cache\updates\manager-stable.json') -Encoding utf8
$taskManager.last_checked_at=$taskNow.ToString('o')
$taskManager|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $taskRoot 'cache\updates\manager-stable.json') -Encoding utf8
Assert-ReadinessRejected 'success anchor differ'
$taskChecks++
$taskManager.last_checked_at=[datetimeoffset]::FromUnixTimeSeconds($taskStart).ToString('o')
$taskManager|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $taskRoot 'cache\updates\manager-stable.json') -Encoding utf8
$taskPanel.integrity='sha512-AA=='
$taskPanel|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $taskRoot 'cache\updates\panel-latest.json') -Encoding utf8
Assert-ReadinessRejected 'full SHA512 digest'
$taskChecks++
[ordered]@{status='passed';checks=$taskChecks;onlyOwnedSyntheticFixtureUsed=$true
    realCacheResponsesWritten=$false;productStarted=$false;foregroundAuthorized=$false
    zeroAutomaticQueryGatePassed=$false}|ConvertTo-Json

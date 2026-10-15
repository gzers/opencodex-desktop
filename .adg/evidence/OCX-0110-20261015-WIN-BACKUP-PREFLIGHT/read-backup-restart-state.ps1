param([Parameter(Mandatory=$true)][ValidatePattern('^[a-z][a-z0-9-]{0,63}$')][string]$Phase)
$ErrorActionPreference='Stop'
$taskLaunch=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'ordinary-restart.json') -Raw | ConvertFrom-Json
$taskProcess=Get-Process -Id $taskLaunch.pid
if ($taskProcess.StartTime.ToUniversalTime().Ticks -ne ([datetimeoffset]$taskLaunch.creationTimeUtc).UtcDateTime.Ticks) {
    throw 'Bound product identity changed.'
}
$taskData=[IO.Path]::GetFullPath($taskLaunch.dataRoot).TrimEnd('\')
$taskOutput=Join-Path $PSScriptRoot ('backup-state-'+$Phase)
if (Test-Path -LiteralPath $taskOutput) { throw 'Historical backup snapshot already exists.' }
New-Item -ItemType Directory -Path $taskOutput | Out-Null
$taskSources=[Collections.Generic.List[object]]::new()
foreach ($taskName in @('preferences.json','backup-cleanup-policy.json','notifications.json')) {
    $taskPath=Join-Path $taskData ('manager-state/'+$taskName)
    if (Test-Path -LiteralPath $taskPath) {$taskSources.Add((Get-Item -LiteralPath $taskPath))}
}
$taskBackups=Join-Path $taskData 'backups'
if (Test-Path -LiteralPath $taskBackups) {
    $taskChildren=@(Get-ChildItem -LiteralPath $taskBackups -Force -Recurse)
    if (@($taskChildren | Where-Object {$_.Attributes -band [IO.FileAttributes]::ReparsePoint}).Count) {
        throw 'Do not follow linked backup snapshot entries.'
    }
    foreach ($taskFile in $taskChildren | Where-Object {!$_.PSIsContainer}) {$taskSources.Add($taskFile)}
}
if ($taskSources.Count -gt 512) { throw 'Bounded snapshot file count exceeded.' }
$taskEntries=@();$taskTotal=0L;$taskManifests=@()
foreach ($taskFile in $taskSources) {
    if (!$taskFile.FullName.StartsWith($taskData+'\',[StringComparison]::OrdinalIgnoreCase)) { throw 'Snapshot escaped its owned data root.' }
    $taskTotal+=$taskFile.Length
    if ($taskFile.Length -gt 4MB -or $taskTotal -gt 16MB) { throw 'Bounded snapshot bytes exceeded.' }
    $taskRelative=$taskFile.FullName.Substring($taskData.Length+1)
    $taskTarget=Join-Path $taskOutput $taskRelative
    New-Item -ItemType Directory -Path (Split-Path -Parent $taskTarget) -Force | Out-Null
    Copy-Item -LiteralPath $taskFile.FullName -Destination $taskTarget
    $taskEntries+=[ordered]@{
        path=($taskRelative -replace '\\','/');bytes=$taskFile.Length
        sha256=(Get-FileHash -LiteralPath $taskTarget).Hash.ToLowerInvariant()
    }
    if ($taskFile.Name -eq 'backup-manifest.json') {
        $taskManifest=Get-Content -LiteralPath $taskTarget -Raw | ConvertFrom-Json
        $taskManifests+=$taskManifest
    }
}
$taskPreferences=Get-Content -LiteralPath (Join-Path $taskOutput 'manager-state/preferences.json') -Raw | ConvertFrom-Json
$taskRecord=[ordered]@{
    atUtc=[DateTime]::UtcNow.ToString('o');phase=$Phase;source=$taskLaunch.source
    pid=$taskLaunch.pid;creationTimeUtc=$taskLaunch.creationTimeUtc;dataRoot=$taskData
    preferencesTheme=$taskPreferences.appearance.theme;preferencesSchema=$taskPreferences.schema_version
    manifests=$taskManifests;files=$taskEntries;bytes=$taskTotal
    inputInjected=$false;sourceDataChangedByReader=$false;backupDatesChangedByReader=$false
}
$taskRecord | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath (Join-Path $taskOutput 'snapshot.json') -Encoding utf8
$taskRecord | ConvertTo-Json -Depth 20

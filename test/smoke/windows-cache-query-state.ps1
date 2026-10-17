# Read-only cache admission and an in-memory preference seed. This never grants
# a foreground window, moves a profile, changes a pointer, or launches a process.
. (Join-Path $PSScriptRoot 'windows-startup-pair-state.ps1')

function New-CacheQueryPreferenceBytes {
    param([Parameter(Mandatory)][byte[]]$Original)
    $value=[Text.Encoding]::UTF8.GetString($Original).TrimStart([char]0xfeff)|ConvertFrom-Json
    if($value.schema_version -ne 2){throw 'Known schema2 preferences required'}
    $value.appearance.theme='light'
    $value.appearance.visual_effects='high'
    $value.appearance.glow_render='mesh'
    $value.appearance.interface_scale=100
    $value.shell.auto_panel=$false
    $value.shell.launch_with_codex=$false
    $value.shell.keep_proxy_on_close=$false
    $value.shell.launch_main=$true
    $value.updates.app_update_auto_check=$true
    $value.updates.app_update_channel='stable'
    $value.maintenance.startup_cleanup=$false
    # Return one byte-array object, not a pipeline that enumerates its bytes.
    ,[Text.UTF8Encoding]::new($false).GetBytes(($value|ConvertTo-Json -Depth 16))
}

function Get-CacheQueryFixtureReadiness {
    param(
        [Parameter(Mandatory)][string]$Root,
        [Parameter(Mandatory)][datetimeoffset]$Now,
        [Parameter(Mandatory)][datetimeoffset]$Through,
        [switch]$RequirePreparedPreferences
    )
    $taskRoot=ConvertTo-StartupPairPath $Root
    if($Through -le $Now){throw 'An explicit future observation horizon is required'}
    $taskRelative=@(
        'manager-state\preferences.json','manager-state\update-schedule.json',
        'cache\updates\manager-stable.json','cache\updates\panel-latest.json'
    )
    $taskFiles=@(foreach($taskName in $taskRelative){
        $taskFile=ConvertTo-StartupPairPath (Join-Path $taskRoot $taskName)
        $taskItem=Get-Item -LiteralPath $taskFile -Force
        if($taskItem.PSIsContainer -or $taskItem.Length -gt 512KB){throw 'Bounded ordinary fixture file required'}
        [pscustomobject]@{relative=$taskName;bytes=$taskItem.Length
            sha256=(Get-FileHash -LiteralPath $taskFile).Hash.ToLowerInvariant()}
    })
    $taskPrefs=Get-Content -LiteralPath (Join-Path $taskRoot $taskRelative[0]) -Raw -Encoding utf8|ConvertFrom-Json
    $taskState=Get-Content -LiteralPath (Join-Path $taskRoot $taskRelative[1]) -Raw -Encoding utf8|ConvertFrom-Json
    $taskManager=Get-Content -LiteralPath (Join-Path $taskRoot $taskRelative[2]) -Raw -Encoding utf8|ConvertFrom-Json
    $taskPanel=Get-Content -LiteralPath (Join-Path $taskRoot $taskRelative[3]) -Raw -Encoding utf8|ConvertFrom-Json
    if($taskPrefs.schema_version -ne 2 -or $taskPrefs.updates.app_update_channel -cne 'stable'){
        throw 'Current schema2 stable fixture required'
    }
    if($taskState.schema -ne 1 -or @($taskState.entries.PSObject.Properties).Count -gt 3){
        throw 'Known production schedule schema required'
    }
    $taskDeadlines=@(foreach($taskTarget in @('manager-stable','panel-latest')){
        $taskEntry=$taskState.entries.$taskTarget
        if(!$taskEntry -or
            ($taskEntry.failures -isnot [long] -and $taskEntry.failures -isnot [int]) -or
            ($taskEntry.last_success -isnot [long] -and $taskEntry.last_success -isnot [int]) -or
            ($taskEntry.next_due -isnot [long] -and $taskEntry.next_due -isnot [int]) -or
            $taskEntry.failures -ne 0 -or $null -eq $taskEntry.last_success -or
            $taskEntry.last_success -lt 0 -or $taskEntry.last_success -gt $Now.ToUnixTimeSeconds() -or
            $taskEntry.next_due -le $Through.ToUnixTimeSeconds() -or
            $taskEntry.next_due-$taskEntry.last_success -ne 86400){
            throw 'Real successful cache deadline must remain valid throughout the horizon'
        }
        [pscustomobject]@{target=$taskTarget;lastSuccess=$taskEntry.last_success;nextDue=$taskEntry.next_due
            nextDueUtc=[datetimeoffset]::FromUnixTimeSeconds($taskEntry.next_due).ToString('o')}
    })
    if($taskManager.channel -cne 'stable' -or $taskManager.current_version -cne '0.1.10' -or
        $null -ne $taskManager.error -or !$taskManager.last_checked_at){
        throw 'Current stable0.1.10 successful manager cache required'
    }
    $taskChecked=[datetimeoffset]::Parse($taskManager.last_checked_at)
    if($taskChecked.ToUnixTimeSeconds() -ne $taskState.entries.'manager-stable'.last_success){
        throw 'Manager cache and persisted success anchor differ'
    }
    if($taskPanel.tag -cne 'latest' -or !$taskPanel.version -or $taskPanel.integrity -cnotmatch '^sha512-[A-Za-z0-9+/]+={0,2}$'){
        throw 'Known panel latest cache with integrity required'
    }
    if([Convert]::FromBase64String($taskPanel.integrity.Substring(7)).Length -ne 64){
        throw 'Panel integrity must contain a full SHA512 digest'
    }
    $taskPrepared=(
        $taskPrefs.updates.app_update_auto_check -is [bool] -and $taskPrefs.updates.app_update_auto_check -and
        $taskPrefs.shell.keep_proxy_on_close -is [bool] -and !$taskPrefs.shell.keep_proxy_on_close -and
        $taskPrefs.shell.auto_panel -is [bool] -and !$taskPrefs.shell.auto_panel -and
        $taskPrefs.shell.launch_with_codex -is [bool] -and !$taskPrefs.shell.launch_with_codex -and
        $taskPrefs.shell.launch_main -is [bool] -and $taskPrefs.shell.launch_main -and
        $taskPrefs.maintenance.startup_cleanup -is [bool] -and !$taskPrefs.maintenance.startup_cleanup -and
        $taskPrefs.appearance.theme -ceq 'light' -and $taskPrefs.appearance.visual_effects -ceq 'high' -and
        $taskPrefs.appearance.glow_render -ceq 'mesh' -and $taskPrefs.appearance.interface_scale -eq 100)
    if($RequirePreparedPreferences -and !$taskPrepared){
        throw 'Cache experiment requires automatic checks enabled and normal-close fixture preferences'
    }
    [ordered]@{
        observedAtUtc=$Now.ToString('o');throughUtc=$Through.ToString('o');cacheRoot=$taskRoot
        cacheFiles=$taskFiles;deadlines=$taskDeadlines
        managerCacheVersion=$taskManager.current_version;panelCacheVersion=$taskPanel.version
        cacheValidThroughHorizon=$true;preferencesReady=$taskPrepared
        sourceAuthenticityProvedByThisReader=$false
        deadlineOrCacheResponseChanged=$false;productStarted=$false;foregroundAuthorized=$false
        zeroAutomaticQueryGatePassed=$false
    }
}

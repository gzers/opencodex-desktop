function Assert-RecoveryPlatformRootsAbsent {
    param([Parameter(Mandatory)][string]$PreparationDirectory)
    $taskAllowed='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop\test\runtime'
    $taskPreparation=[IO.Path]::GetFullPath($PreparationDirectory)
    if(!$taskPreparation.StartsWith($taskAllowed+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned preparation directory required'}
    $taskPlan=Get-Content -LiteralPath (Join-Path $taskPreparation 'profile-isolation-plan.json') -Raw -Encoding utf8 | ConvertFrom-Json
    $taskResult=Get-Content -LiteralPath (Join-Path $taskPreparation 'profile-isolation-result.json') -Raw -Encoding utf8 | ConvertFrom-Json
    $taskExpected=@(
        'C:\Users\15119\AppData\Roaming\com.gzers.opencodex.desktop',
        'C:\Users\15119\AppData\Local\com.gzers.opencodex.desktop',
        'C:\Users\15119\AppData\Local\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Roaming\com.gzers.opencodex.desktop',
        'C:\Users\15119\AppData\Local\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Local\com.gzers.opencodex.desktop'
    )
    if($taskPlan.profiles.Count -ne 4 -or !$taskResult.allFourOriginalDirectoriesAbsent -or
       !$taskResult.startupPointerUnchangedExact -or $taskResult.productStarted -or $taskResult.dataDeleted){
        throw 'Completed scoped isolation evidence required'
    }
    $taskPaths=@($taskPlan.profiles.path | Sort-Object -Unique)
    if($taskPaths.Count -ne 4 -or @(Compare-Object ($taskExpected | Sort-Object) $taskPaths).Count){throw 'Platform root plan changed'}
    foreach($taskProfile in $taskPlan.profiles){
        if(Test-Path -LiteralPath $taskProfile.path -ErrorAction Stop){
            throw 'Platform app-data root exists, including an empty root; do not launch recovery'
        }
        $taskSaved=[IO.Path]::GetFullPath((Join-Path $taskPreparation ('saved-'+$taskProfile.name)))
        if($taskSaved -cne $taskProfile.destination -or !$taskSaved.StartsWith($taskPreparation+'\',[StringComparison]::OrdinalIgnoreCase)){
            throw 'Saved profile scope differs'
        }
        $taskItem=Get-Item -LiteralPath $taskSaved -Force
        if(!$taskItem.PSIsContainer -or $taskItem.Attributes -band [IO.FileAttributes]::ReparsePoint -or
           [string]$taskItem.CreationTimeUtc.ToFileTimeUtc() -cne $taskProfile.creationFileTime){
            throw 'Saved profile directory changed'
        }
    }
    if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Require native0'}
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');fourPlatformRootsAbsent=$true
        allSavedDirectoryCreationIdentitiesMatch=$true;nativeCount=0
        preparationPlanSha256=(Get-FileHash -LiteralPath (Join-Path $taskPreparation 'profile-isolation-plan.json')).Hash.ToLowerInvariant()
        preparationResultSha256=(Get-FileHash -LiteralPath (Join-Path $taskPreparation 'profile-isolation-result.json')).Hash.ToLowerInvariant()
        foregroundAuthorizationGranted=$false;productStarted=$false}
}

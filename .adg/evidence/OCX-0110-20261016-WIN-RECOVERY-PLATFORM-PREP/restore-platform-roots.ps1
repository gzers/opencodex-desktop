# Restore each physical directory once; preserve state newly created at any alias.
param([Parameter(Mandatory)][string]$RunDirectory)
$ErrorActionPreference='Stop'
$taskAllowed='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop\test\runtime'
$taskRun=[IO.Path]::GetFullPath($RunDirectory)
if(!$taskRun.StartsWith($taskAllowed+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned runtime required'}
$taskPlan=Get-Content -LiteralPath (Join-Path $taskRun 'profile-isolation-plan.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskExpected=@{
    'ordinary-roaming'='C:\Users\15119\AppData\Roaming\com.gzers.opencodex.desktop'
    'ordinary-local'='C:\Users\15119\AppData\Local\com.gzers.opencodex.desktop'
    'package-roaming'='C:\Users\15119\AppData\Local\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Roaming\com.gzers.opencodex.desktop'
    'package-local'='C:\Users\15119\AppData\Local\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Local\com.gzers.opencodex.desktop'
}
if($taskPlan.aliases.Count -ne 4 -or
   @(Compare-Object ($taskExpected.Values | Sort-Object) ($taskPlan.aliases.path | Sort-Object)).Count){
    throw 'Exact four logical profile paths required'
}
if(@(Compare-Object ($taskExpected.Values | Sort-Object) ($taskPlan.physicalProfiles.logicalPaths | Sort-Object)).Count -or
   @($taskPlan.physicalProfiles.logicalPaths).Count -ne 4){
    throw 'Physical groups must cover each logical path once'
}
. (Join-Path $PSScriptRoot 'directory-identity.ps1')
foreach($taskAlias in $taskPlan.aliases){
    if(Test-Path -LiteralPath $taskAlias.path){throw 'Preserve newly created state; never overwrite it'}
}
foreach($taskProfile in $taskPlan.physicalProfiles){
    $taskSource=$taskExpected[$taskProfile.name]
    $taskSaved=[IO.Path]::GetFullPath((Join-Path $taskRun ('saved-'+$taskProfile.name)))
    if(!$taskSource -or $taskSource -cne $taskProfile.source -or $taskSaved -cne $taskProfile.destination -or
       !$taskSaved.StartsWith($taskRun+'\',[StringComparison]::OrdinalIgnoreCase)){
        throw 'Restoration scope differs from exact approved directories'
    }
    if(!(Test-Path -LiteralPath $taskSaved -PathType Container)){throw 'Saved directory missing'}
    foreach($taskPath in @($taskSource,$taskSaved)){
        $taskCursor=$taskPath
        while($taskCursor){
            if(Test-Path -LiteralPath $taskCursor){
                if((Get-Item -LiteralPath $taskCursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint){
                    throw 'Restoration path alias refused'
                }
            }
            $taskCursor=[IO.Path]::GetDirectoryName($taskCursor)
        }
    }
    $taskSavedIdentity=Get-RecoveryDirectoryIdentity -Path $taskSaved
    if($taskSavedIdentity.fileId -cne $taskProfile.fileId -or
       $taskSavedIdentity.creationFileTime -cne $taskProfile.creationFileTime){
        throw 'Saved native directory identity differs'
    }
}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Require native0'}
foreach($taskProfile in $taskPlan.physicalProfiles){
    if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native instance appeared; stop restoration'}
    # Both resolved absolute paths above are exact approved app/saved directories.
    Move-Item -LiteralPath $taskProfile.destination -Destination $taskProfile.source
    if(Test-Path -LiteralPath $taskProfile.destination){throw 'Saved path still exists after move'}
    foreach($taskAlias in $taskProfile.logicalPaths){
        $taskIdentity=Get-RecoveryDirectoryIdentity -Path $taskAlias
        if($taskIdentity.fileId -cne $taskProfile.fileId -or
           $taskIdentity.creationFileTime -cne $taskProfile.creationFileTime){
            throw 'Restored directory native identity differs'
        }
    }
}
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');fourLogicalPathsRestored=$true
    physicalDirectoryMoves=$taskPlan.physicalProfiles.Count;nativeDirectoryIdsPreserved=$true
    dataDeleted=$false;registryOrAclChanged=$false;productStarted=$false;uiInputInjected=$false} |
    ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRun 'profile-restoration-result.json') -Encoding utf8

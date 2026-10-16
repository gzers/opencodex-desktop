# Restore the exact moved directories only while their original locations are absent.
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
if($taskPlan.profiles.Count -ne 4){throw 'Exact four-profile plan required'}
foreach($taskProfile in $taskPlan.profiles){
    $taskSource=$taskExpected[$taskProfile.name]
    $taskSaved=[IO.Path]::GetFullPath((Join-Path $taskRun ('saved-'+$taskProfile.name)))
    if(!$taskSource -or $taskSource -cne $taskProfile.path -or $taskSaved -cne $taskProfile.destination -or
       !$taskSaved.StartsWith($taskRun+'\',[StringComparison]::OrdinalIgnoreCase)){
        throw 'Restoration scope differs from exact approved directories'
    }
    if(Test-Path -LiteralPath $taskSource){throw 'Preserve newly created state; never overwrite it'}
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
    if([string](Get-Item -LiteralPath $taskSaved -Force).CreationTimeUtc.ToFileTimeUtc() -cne $taskProfile.creationFileTime){
        throw 'Saved directory creation identity differs'
    }
}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Require native0'}
foreach($taskProfile in $taskPlan.profiles){
    if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native instance appeared; stop restoration'}
    # Both resolved absolute paths above are exact approved app/saved directories.
    Move-Item -LiteralPath $taskProfile.destination -Destination $taskProfile.path
    if((Test-Path -LiteralPath $taskProfile.destination) -or
       [string](Get-Item -LiteralPath $taskProfile.path -Force).CreationTimeUtc.ToFileTimeUtc() -cne $taskProfile.creationFileTime){
        throw 'Restored directory readback differs'
    }
}
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');fourDirectoriesRestored=$true
    dataDeleted=$false;registryOrAclChanged=$false;productStarted=$false;uiInputInjected=$false} |
    ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRun 'profile-restoration-result.json') -Encoding utf8

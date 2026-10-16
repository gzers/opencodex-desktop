# Group logical paths by real directory handle identity before moving any profile.
param([Parameter(Mandatory)][string]$RunDirectory)
$ErrorActionPreference='Stop'
$taskAllowed='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop\test\runtime'
$taskRun=[IO.Path]::GetFullPath($RunDirectory)
if(!$taskRun.StartsWith($taskAllowed+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned runtime directory required'}
if(Test-Path -LiteralPath $taskRun){throw 'Never overwrite a previous preparation'}
$taskProfiles=@(
    @{name='ordinary-roaming';path='C:\Users\15119\AppData\Roaming\com.gzers.opencodex.desktop'},
    @{name='ordinary-local';path='C:\Users\15119\AppData\Local\com.gzers.opencodex.desktop'},
    @{name='package-roaming';path='C:\Users\15119\AppData\Local\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Roaming\com.gzers.opencodex.desktop'},
    @{name='package-local';path='C:\Users\15119\AppData\Local\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Local\com.gzers.opencodex.desktop'}
)
function Assert-TaskPath([string]$Path,[bool]$MustExist){
    $taskFull=[IO.Path]::GetFullPath($Path)
    if($taskFull -cne $Path){throw 'Canonical absolute path required'}
    $taskCursor=$taskFull
    while($taskCursor){
        if(Test-Path -LiteralPath $taskCursor){
            $taskItem=Get-Item -LiteralPath $taskCursor -Force
            if($taskItem.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Path alias refused'}
            if($taskCursor -ceq $taskFull -and !$taskItem.PSIsContainer){throw 'Directory required'}
        }elseif($taskCursor -ceq $taskFull -and $MustExist){throw 'Expected existing profile missing'}
        $taskCursor=[IO.Path]::GetDirectoryName($taskCursor)
    }
}
function Assert-NativeZero {
    if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){
        throw 'No desktop product may run during preparation'
    }
}
. (Join-Path $PSScriptRoot 'directory-identity.ps1')
Assert-NativeZero
Assert-TaskPath -Path $taskRun -MustExist $false
foreach($taskProfile in $taskProfiles){
    if([IO.Path]::GetFileName($taskProfile.path) -cne 'com.gzers.opencodex.desktop'){throw 'Unexpected application directory'}
    Assert-TaskPath -Path $taskProfile.path -MustExist $true
    $taskDestination=[IO.Path]::GetFullPath((Join-Path $taskRun ('saved-'+$taskProfile.name)))
    if(!$taskDestination.StartsWith($taskRun+'\',[StringComparison]::OrdinalIgnoreCase) -or
       [IO.Path]::GetPathRoot($taskDestination) -cne [IO.Path]::GetPathRoot($taskProfile.path)){
        throw 'Exact same-volume task destination required'
    }
    if(Test-Path -LiteralPath $taskDestination){throw 'Destination exists'}
    Assert-TaskPath -Path $taskDestination -MustExist $false
    $taskProfile.identity=Get-RecoveryDirectoryIdentity -Path $taskProfile.path
}
$taskPhysical=@(foreach($taskGroup in ($taskProfiles | Group-Object {$_.identity.fileId})){
    $taskRepresentative=$taskGroup.Group[0]
    if(@($taskGroup.Group.identity.finalPath | Sort-Object -Unique).Count -ne 1){
        throw 'Same directory ID has inconsistent final paths'
    }
    [ordered]@{name=$taskRepresentative.name;source=$taskRepresentative.path
        destination=(Join-Path $taskRun ('saved-'+$taskRepresentative.name))
        fileId=$taskRepresentative.identity.fileId;creationFileTime=$taskRepresentative.identity.creationFileTime
        originalFinalPath=$taskRepresentative.identity.finalPath;logicalPaths=@($taskGroup.Group.path)}
})
if($taskPhysical.Count -lt 1 -or $taskPhysical.Count -gt 4){throw 'Invalid physical profile grouping'}
$null=New-Item -ItemType Directory -Path $taskRun
. 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261016-WIN-823-FRONT-PREP\run-binding.ps1'
$null=Assert-PreparedIdleRun -RunDirectory $taskRun
$taskPointer=(Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop' -Name StartupDataRootV1).StartupDataRootV1
$taskPointerBytes=[Text.Encoding]::UTF8.GetBytes($taskPointer)
$taskPointerSha=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($taskPointerBytes)).ToLowerInvariant()
$taskPlan=[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o')
    authorization='2026-10-16 user allows this-machine OpenCodex/Desktop uninstall and configuration cleanup'
    scope='four verified logical app paths grouped by native directory ID; preserve each physical directory once'
    aliases=$taskProfiles;physicalProfiles=$taskPhysical;startupPointerSha256=$taskPointerSha
    source='823a8a0615153068594daef530c2e80f1d95fc29'
    foregroundAuthorizationGranted=$false;productLaunchAllowed=$false
    rawSidOrSddlPublished=$false
}
$taskPlan | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $taskRun 'profile-isolation-plan.json') -Encoding utf8
$taskCompleted=[Collections.Generic.List[object]]::new()
try {
    foreach($taskProfile in $taskPhysical){
        Assert-NativeZero
        $taskBefore=Get-RecoveryDirectoryIdentity -Path $taskProfile.source
        if($taskBefore.fileId -cne $taskProfile.fileId -or $taskBefore.creationFileTime -cne $taskProfile.creationFileTime){
            throw 'Physical profile changed before move'
        }
        # Each resolved absolute source is an exact approved application directory.
        # Each destination was checked above to remain below this owned runtime.
        Move-Item -LiteralPath $taskProfile.source -Destination $taskProfile.destination
        if(@($taskProfile.logicalPaths | Where-Object {Test-Path -LiteralPath $_ -ErrorAction Stop}).Count -or
           !(Test-Path -LiteralPath $taskProfile.destination -PathType Container)){
            throw 'Profile move readback failed'
        }
        $taskAfter=Get-RecoveryDirectoryIdentity -Path $taskProfile.destination
        if($taskAfter.fileId -cne $taskProfile.fileId -or $taskAfter.creationFileTime -cne $taskProfile.creationFileTime){
            throw 'Moved directory native identity changed'
        }
        $taskCompleted.Add([ordered]@{name=$taskProfile.name;source=$taskProfile.source;destination=$taskProfile.destination
            logicalPaths=$taskProfile.logicalPaths;before=$taskBefore;after=$taskAfter
            allLogicalPathsAbsent=$true;contentsMovedWithoutDeletion=$true})
    }
    Assert-NativeZero
    $taskPointerAfter=(Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop' -Name StartupDataRootV1).StartupDataRootV1
    if($taskPointerAfter -cne $taskPointer){throw 'Startup pointer changed concurrently'}
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');profiles=$taskCompleted.ToArray()
        allFourOriginalDirectoriesAbsent=$true;startupPointerUnchangedExact=$true;startupPointerSha256=$taskPointerSha
        logicalPathCount=4;physicalDirectoryMoves=$taskCompleted.Count;nativeDirectoryIdsPreserved=$true
        nativeCount=0;productStarted=$false;uiInputInjected=$false;aclChanged=$false;dataDeleted=$false
        profileContentsPublished=$false;foregroundAuthorizationGranted=$false;fullRecoveryGatePassed=$false
        nextStep='new human window; revalidate all absent roots, then frozen prepare/launch recovery once'} |
        ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskRun 'profile-isolation-result.json') -Encoding utf8
}catch{
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');exceptionType=$_.Exception.GetType().FullName
        message=$_.Exception.Message;completedMoves=$taskCompleted.ToArray();productStarted=$false
        automaticRetry=$false;dataDeleted=$false} | ConvertTo-Json -Depth 8 |
        Set-Content -LiteralPath (Join-Path $taskRun 'profile-isolation-failure.json') -Encoding utf8
    throw
}

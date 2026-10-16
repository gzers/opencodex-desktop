$ErrorActionPreference='Stop'
$taskDocs='D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop'
$taskRun='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop\test\runtime\foreground-b4-pair-20261016T204338'
$taskPrior=Join-Path $taskDocs '.adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES'
$taskPrep=Join-Path $taskDocs '.adg/evidence/OCX-0110-20261016-WIN-B4-FRONT-PREP'
$taskOutput=Join-Path $PSScriptRoot 'restoration-readback.original.json'
$taskMarkerCopy=Join-Path $PSScriptRoot 'fixture-restored.original.json'
if((Test-Path -LiteralPath $taskOutput) -or (Test-Path -LiteralPath $taskMarkerCopy)){throw 'Preserve existing recovery evidence'}
$taskContract=Get-Content -LiteralPath (Join-Path $taskPrep 'contract.json') -Raw | ConvertFrom-Json
$taskReady=Get-Content -LiteralPath (Join-Path $taskPrior 'original/fixture-ready.json') -Raw | ConvertFrom-Json
$taskPrepared=Get-Content -LiteralPath (Join-Path $taskPrep 'preparation-readback.json') -Raw | ConvertFrom-Json
if((Get-FileHash -LiteralPath $taskContract.directoryIdentityMethod).Hash.ToLowerInvariant() -cne $taskContract.directoryIdentityMethodSha256){throw 'Frozen directory identity method changed'}
. $taskContract.directoryIdentityMethod
$taskNativeBefore=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'")
$taskOriginal=Get-RecoveryDirectoryIdentity -Path $taskContract.baselineFixtureRoot
$taskMeasured=Get-RecoveryDirectoryIdentity -Path (Join-Path $taskRun 'baseline-measured')
$taskPointer=[string](Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop').StartupDataRootV1
$taskPointerHash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($taskPointer))).ToLowerInvariant()
$taskPreviousExists=Test-Path -LiteralPath (Join-Path $taskRun 'baseline-previous')
$taskOriginalBefore=@($taskPrepared.fixtureReadback | Where-Object path -CEQ $taskContract.baselineFixtureRoot)
if($taskOriginalBefore.Count -ne 1){throw 'One frozen ordinary profile identity required'}
$taskPreferences=Join-Path $taskContract.baselineFixtureRoot 'manager-state/preferences.json'
$taskPreferencesHash=(Get-FileHash -LiteralPath $taskPreferences).Hash.ToLowerInvariant()
$taskGenerations=@(
    foreach($taskVersion in @('baseline019','candidate0110')){
        $taskRoles=Get-Content -LiteralPath (Join-Path $taskPrior "original/$taskVersion-settled-roles.original.json") -Raw | ConvertFrom-Json
        foreach($taskMember in $taskRoles.members){
            $taskProcess=$null
            $taskSame=$false
            try{
                $taskProcess=[Diagnostics.Process]::GetProcessById([int]$taskMember.pid)
                [void]$taskProcess.SafeHandle
                $taskSame=(!$taskProcess.HasExited -and
                    $taskProcess.StartTime.ToUniversalTime().Ticks -eq ([datetimeoffset]$taskMember.creationUtc).UtcTicks)
            }catch [ArgumentException]{
                $taskSame=$false
            }finally{if($taskProcess){$taskProcess.Dispose()}}
            [ordered]@{version=$taskVersion;capturedPid=$taskMember.pid
                capturedCreationUtc=$taskMember.creationUtc;role=$taskMember.role
                sameGenerationStillRunning=$taskSame;unrelatedProcessDetailsSaved=$false}
        }
    }
)
$taskNativeAfter=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'")
$taskPass=($taskNativeBefore.Count -eq 0 -and $taskNativeAfter.Count -eq 0 -and
    !@($taskGenerations | Where-Object sameGenerationStillRunning).Count -and !$taskPreviousExists -and
    $taskOriginal.fileId -ceq $taskReady.originalDirectory.fileId -and
    $taskOriginal.creationFileTime -ceq $taskReady.originalDirectory.creationFileTime -and
    $taskOriginal.finalPath -ceq $taskReady.originalDirectory.finalPath -and
    $taskMeasured.fileId -ceq $taskReady.cloneDirectory.fileId -and
    $taskMeasured.creationFileTime -ceq $taskReady.cloneDirectory.creationFileTime -and
    $taskPointerHash -ceq $taskPrepared.startupPointerSha256 -and
    $taskPreferencesHash -ceq $taskOriginalBefore[0].preferencesSha256)
$taskMarker=Join-Path $taskRun 'fixture-restored.json'
[IO.File]::Copy($taskMarker,$taskMarkerCopy,$false)
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');passed=$taskPass
    nativeCountBefore=$taskNativeBefore.Count;nativeCountAfter=$taskNativeAfter.Count
    originalDirectory=$taskOriginal;measuredCloneDirectory=$taskMeasured
    baselinePreviousExists=$taskPreviousExists
    restoredPreferencesSha256=$taskPreferencesHash
    startupPointerSha256=$taskPointerHash
    taskFileSha256=(Get-FileHash -LiteralPath (Join-Path $taskDocs '.adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json')).Hash.ToLowerInvariant()
    ownedGenerations=$taskGenerations
    markerRuntimePath=$taskMarker
    markerSha256=(Get-FileHash -LiteralPath $taskMarker).Hash.ToLowerInvariant()
    directoryIdentityMethodSha256=$taskContract.directoryIdentityMethodSha256
    humanReportedExit=$true;humanMessage='已退出'
    exactExitTimeProven=$false;candidateExitCode=$null
    originalExitTimeoutsPreserved=$true;formalPairLifecycleAdmitted=$false
    productStarted=$false;uiInputInjected=$false;forceTerminated=$false
    dataDeleted=$false;releaseAccepted=$false
} | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $taskOutput -Encoding utf8
if(!$taskPass){throw 'Recovery readback failed; preserve the result and inspect'}
Get-Content -LiteralPath $taskOutput

param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
if(!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7){throw 'Windows PowerShell 7 required'}
$taskRepo=(Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../..')).Path
$taskBoundary=[IO.Path]::GetFullPath((Join-Path $taskRepo 'test/runtime'))
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
if(!$taskOutput.StartsWith($taskBoundary+'\',[StringComparison]::OrdinalIgnoreCase) -or
   (Test-Path -LiteralPath $taskOutput)){throw 'Use a fresh task-owned runtime output directory'}
New-Item -ItemType Directory -Path $taskOutput | Out-Null
. (Join-Path $PSScriptRoot 'windows-idle-family.ps1')
$script:taskCheckCount=0
$script:taskFailureRecords=[Collections.Generic.List[object]]::new()
$script:taskFailurePath=Join-Path $taskOutput 'failure-records.original.jsonl'
$taskOnFailure={
    param($Record)
    $script:taskFailureRecords.Add($Record)
    [IO.File]::AppendAllText($script:taskFailurePath,($Record | ConvertTo-Json -Depth 6 -Compress)+"`n",
        [Text.UTF8Encoding]::new($false))
}
function Assert-IdleTest([bool]$Value,[string]$Name){
    if(!$Value){throw "Idle family regression: $Name"}
    $script:taskCheckCount++
}
function Assert-IdleRejected([scriptblock]$Body,[string]$Expected){
    $taskRejected=$false
    try {& $Body | Out-Null} catch {
        if(!$_.Exception.Message.Contains($Expected)){throw}
        $taskRejected=$true
    }
    Assert-IdleTest $taskRejected $Expected
}
$taskSelf=Get-Process -Id $PID
$taskStates=[Collections.Generic.List[object]]::new()
function New-TestIdleState {
    $taskState=New-WindowsIdleFamily -RootProcess $taskSelf -OnFailure $taskOnFailure
    $taskStates.Add($taskState)
    return $taskState
}
$taskChild=$null
$taskRelease=Join-Path $taskOutput 'release-owned-fixture'
try {
    $taskState=New-TestIdleState
    Assert-WindowsIdleFrame -State $taskState -Foreground $true -Visible $true -Minimized $false -NativeCount 1
    $taskSample=Get-WindowsIdleFamilySample -State $taskState -Rows @()
    Assert-IdleTest ($taskSample.members.Count -eq 1) 'root alone is measured'
    Assert-IdleTest ($taskSample.members[0].pid -eq $PID) 'root generation is bound'
    Assert-IdleTest ($taskSample.workingSetBytes -gt 0) 'root working set is real'
    Assert-IdleTest ($taskSample.cpuSeconds -ge 0) 'root CPU is real'
    Assert-IdleTest (!$taskState.Failed) 'valid sample retains validity'
    $taskUnrelatedRow=[pscustomobject]@{ProcessId=0;ParentProcessId=0;CreationDate=$null;Name='System Idle Process'}
    $taskUnrelatedSample=Get-WindowsIdleFamilySample -State $taskState -Rows @($taskUnrelatedRow)
    Assert-IdleTest ($taskUnrelatedSample.members.Count -eq 1 -and !$taskState.Failed) 'unrelated system rows do not require private or unavailable creation times'
    $taskBadGeneration=New-TestIdleState
    $taskBadRow=[pscustomobject]@{ProcessId=2147483646;ParentProcessId=$PID;CreationDate=$null;Name='missing-generation-fixture.exe'}
    Assert-IdleRejected {Get-WindowsIdleFamilySample -State $taskBadGeneration -Rows @($taskBadRow)} 'child-snapshot-generation'
    Assert-IdleTest ($taskBadGeneration.Failures[0].processId -eq 2147483646 -and
        $null -eq $taskBadGeneration.Failures[0].expectedCreationUtc) 'owned unreadable creation is diagnosed rather than skipped'

    $taskMissing=New-TestIdleState
    $taskMissingRow=[pscustomobject]@{
        ProcessId=2147483647;ParentProcessId=$PID
        CreationDate=$taskSelf.StartTime.ToUniversalTime().AddMilliseconds(1);Name='missing-fixture.exe'
    }
    Assert-IdleRejected {Get-WindowsIdleFamilySample -State $taskMissing -Rows @($taskMissingRow)} 'child-process-open'
    $taskFailure=$taskMissing.Failures[0]
    Assert-IdleTest ($taskFailure.processId -eq 2147483647 -and $taskFailure.parentProcessId -eq $PID) 'failed PID and parent retained'
    Assert-IdleTest ($taskFailure.expectedCreationUtc -eq $taskMissingRow.CreationDate.ToString('o')) 'failed CIM generation retained'
    Assert-IdleTest (!!$taskFailure.exceptionType -and $null -ne $taskFailure.hresult) 'exception type and HRESULT retained'
    Assert-IdleTest (!$taskFailure.rawExceptionMessageSaved -and !$taskFailure.commandLineSaved) 'raw exception and command lines excluded'
    Assert-IdleTest ($taskMissing.Members.Count -eq 1) 'failed child is never a successful counter'
    Assert-IdleRejected {Get-WindowsIdleFamilySample -State $taskMissing -Rows @()} 'already invalid'
    Assert-IdleTest ($taskMissing.Failures.Count -eq 1) 'invalid state cannot retry or duplicate failure'

    $taskMismatch=New-TestIdleState
    $taskMismatchRow=[pscustomobject]@{
        ProcessId=$PID;ParentProcessId=$PID
        CreationDate=$taskSelf.StartTime.ToUniversalTime().AddMilliseconds(100);Name='generation-fixture.exe'
    }
    Assert-IdleRejected {Get-WindowsIdleFamilySample -State $taskMismatch -Rows @($taskMismatchRow)} 'child-generation-match'
    Assert-IdleTest ($taskMismatch.Failures[0].actualCreationUtc -eq $taskSelf.StartTime.ToUniversalTime().ToString('o')) 'actual conflicting generation retained'

    $taskFixture=Join-Path $taskOutput 'owned-fixture.ps1'
    @'
param([string]$Ready,[string]$Release)
$ErrorActionPreference='Stop'
[IO.File]::WriteAllText($Ready,'ready')
$fixtureTimer=[Diagnostics.Stopwatch]::StartNew()
while(!(Test-Path -LiteralPath $Release) -and $fixtureTimer.Elapsed.TotalSeconds -lt 10){
    Start-Sleep -Milliseconds 10
}
if(!(Test-Path -LiteralPath $Release)){exit 7}
$fixtureWork=[Diagnostics.Stopwatch]::StartNew()
while($fixtureWork.ElapsedMilliseconds -lt 150){[void][Math]::Sqrt(1234567)}
exit 0
'@ | Set-Content -LiteralPath $taskFixture -Encoding utf8
    $taskReady=Join-Path $taskOutput 'owned-fixture-ready'
    $taskChild=Start-Process -FilePath ([Diagnostics.Process]::GetCurrentProcess().MainModule.FileName) `
        -ArgumentList @('-NoLogo','-NoProfile','-NonInteractive','-File',$taskFixture,'-Ready',$taskReady,'-Release',$taskRelease) `
        -WindowStyle Hidden -PassThru
    $taskReadyTimer=[Diagnostics.Stopwatch]::StartNew()
    while(!(Test-Path -LiteralPath $taskReady) -and $taskReadyTimer.Elapsed.TotalSeconds -lt 5){
        Start-Sleep -Milliseconds 10
    }
    Assert-IdleTest (Test-Path -LiteralPath $taskReady) 'owned child starts cooperatively'
    $taskChildRow=Get-CimInstance Win32_Process -Filter ('ProcessId='+$taskChild.Id)
    Assert-IdleTest ($taskChildRow.ParentProcessId -eq $PID) 'real child parent matches the test root'
    $taskFamily=New-TestIdleState
    $taskCensus=@(Get-CimInstance -Query 'SELECT ProcessId,ParentProcessId,CreationDate,Name FROM Win32_Process')
    $taskBefore=Get-WindowsIdleFamilySample -State $taskFamily -Rows $taskCensus
    $taskBefore | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $taskOutput 'before-members.original.json') -Encoding utf8
    Assert-IdleTest (@($taskBefore.members | Where-Object pid -eq $PID).Count -eq 1 -and
        @($taskBefore.members | Where-Object pid -eq $taskChild.Id).Count -eq 1) 'root and live child generations are each retained once'
    $taskExpectedMembers=$taskBefore.members.Count
    $taskAgain=Get-WindowsIdleFamilySample -State $taskFamily -Rows @($taskChildRow)
    Assert-IdleTest ($taskAgain.members.Count -eq $taskExpectedMembers) 'same generation is not double counted'
    [IO.File]::WriteAllText($taskRelease,'release')
    Assert-IdleTest ($taskChild.WaitForExit(5000)) 'fixture exits without force termination'
    Assert-IdleTest ($taskChild.ExitCode -eq 0) 'fixture exits zero'
    $taskAfter=Get-WindowsIdleFamilySample -State $taskFamily -Rows @()
    $taskRetained=@($taskAfter.members | Where-Object pid -eq $taskChild.Id)
    Assert-IdleTest ($taskRetained.Count -eq 1 -and $taskRetained[0].exited) 'exited generation remains retained'
    Assert-IdleTest ($taskRetained[0].workingSetBytes -eq 0) 'exited process contributes no live RSS'
    Assert-IdleTest ($taskRetained[0].cpuSeconds -gt 0 -and $taskAfter.cpuSeconds -ge $taskBefore.cpuSeconds) 'exited CPU is not dropped'
    $taskStaleParent=[pscustomobject]@{
        ProcessId=2147483646;ParentProcessId=$taskChild.Id
        CreationDate=$taskChild.ExitTime.ToUniversalTime().AddSeconds(1);Name='unrelated-reused-parent.exe'
    }
    $taskWithoutUnrelated=Get-WindowsIdleFamilySample -State $taskFamily -Rows @($taskStaleParent)
    Assert-IdleTest ($taskWithoutUnrelated.members.Count -eq $taskExpectedMembers -and !$taskFamily.Failed) 'new generation cannot inherit an exited numeric parent PID'

    foreach($taskCase in @(
        @{Stage='foreground-lost';Foreground=$false;Visible=$true;Minimized=$false;NativeCount=1},
        @{Stage='window-hidden';Foreground=$true;Visible=$false;Minimized=$false;NativeCount=1},
        @{Stage='window-minimized';Foreground=$true;Visible=$true;Minimized=$true;NativeCount=1},
        @{Stage='native-instance-count';Foreground=$true;Visible=$true;Minimized=$false;NativeCount=2}
    )){
        $taskFrameState=New-TestIdleState
        $taskArgs=$taskCase.Clone()
        $taskStage=$taskArgs.Stage
        $taskArgs.Remove('Stage')
        Assert-IdleRejected {Assert-WindowsIdleFrame -State $taskFrameState @taskArgs} $taskStage
        Assert-IdleTest ($taskFrameState.Failed -and $taskFrameState.Failures[0].lastWindowFrame.nativeCount -eq $taskCase.NativeCount) 'invalid frame is retained immediately'
    }
    $taskWriterFailure=New-WindowsIdleFamily -RootProcess $taskSelf -OnFailure {throw 'fixture-evidence-writer-failed'}
    $taskStates.Add($taskWriterFailure)
    Assert-IdleRejected {Assert-WindowsIdleFrame -State $taskWriterFailure -Foreground $false -Visible $true -Minimized $false -NativeCount 1} 'fixture-evidence-writer-failed'
    Assert-IdleTest $taskWriterFailure.Failed 'evidence writer failure cannot create a valid sample'
    # Synthetic elapsed values test terminal qualification only. No product
    # timing or 600-second measurement is represented by these fixtures.
    $taskStatusState=New-TestIdleState
    $taskStatus=Write-WindowsIdleTerminal -State $taskStatusState -OutputDirectory $taskOutput `
        -SampleCount 2 -FirstElapsed 1.0 -LastElapsed 600.9 -Completed $true -CollectorCpuSeconds 0
    Assert-IdleTest (!$taskStatus.measurementCompleted -and $taskStatus.observationInvalid) '599.9 seconds cannot qualify'
    $taskStatus=Write-WindowsIdleTerminal -State $taskStatusState -OutputDirectory $taskOutput `
        -SampleCount 2 -FirstElapsed 1.0 -LastElapsed 601.0 -Completed $true -CollectorCpuSeconds 0
    Assert-IdleTest ($taskStatus.measurementCompleted -and !$taskStatus.observationInvalid) '600 seconds uses direct completed counters'
    Assert-IdleTest (!$taskStatus.performanceBudgetClaimedPassed) 'complete counters do not claim a performance budget'
    $taskStatus=Write-WindowsIdleTerminal -State $taskMissing -OutputDirectory $taskOutput `
        -SampleCount 2 -FirstElapsed 1.0 -LastElapsed 601.0 -Completed $true -CollectorCpuSeconds 0
    Assert-IdleTest (!$taskStatus.measurementCompleted -and $taskStatus.readOrFrameFailures -eq 1) 'a family failure overrides nominal completion'
    Assert-IdleTest ((Get-FileHash -LiteralPath (Join-Path $taskOutput 'terminal.json')).Hash -eq
        (Get-FileHash -LiteralPath (Join-Path $taskOutput 'progress.json')).Hash) 'final progress is identical to terminal'
    $taskStatus=Write-WindowsIdleTerminal -State $taskStatusState -OutputDirectory $taskOutput `
        -SampleCount 0 -FirstElapsed $null -LastElapsed $null -Completed $false -CollectorCpuSeconds 0
    Assert-IdleTest ($taskStatus.phase -eq 'invalid' -and $taskStatus.counterSpanSeconds -eq 0) 'early error immediately ends measuring status'
    Assert-IdleTest (!$taskStatus.automaticRetryRequested) 'terminal never schedules a retry'
    $taskObserver=Join-Path $PSScriptRoot 'windows-idle-observe.ps1'
    $taskNeverOutput=Join-Path $taskOutput 'must-not-start-observation'
    $taskLaunchInput=Join-Path $taskOutput 'guard-launch.json'
    $taskReadinessInput=Join-Path $taskOutput 'guard-readiness.json'
    [IO.File]::WriteAllText($taskReadinessInput,'{}')
    Assert-IdleRejected {& $taskObserver -LaunchPath $taskLaunchInput -ReadinessPath $taskReadinessInput `
        -OutputDirectory $taskNeverOutput} 'existing plain observation input'
    Assert-IdleTest (!(Test-Path -LiteralPath $taskNeverOutput)) 'missing identity input creates no measurement'
    $taskLaunchFixture=@{
        source='7704a17902c98ceed047cc3dec2a361937b5e1f7'
        executable=$taskSelf.MainModule.FileName;exeSha256=('0'*64)
        pid=$PID;creationTimeUtc=$taskSelf.StartTime.ToUniversalTime().ToString('o')
        sandboxUsed=$false;cdpUsed=$false
    }
    $taskLaunchFixture | ConvertTo-Json | Set-Content -LiteralPath $taskLaunchInput -Encoding utf8
    Assert-IdleRejected {& $taskObserver -LaunchPath $taskLaunchInput -ReadinessPath $taskReadinessInput `
        -OutputDirectory $taskNeverOutput} 'exact executable digest required'
    Assert-IdleTest (!(Test-Path -LiteralPath $taskNeverOutput)) 'wrong image digest creates no measurement'
    $taskLaunchFixture.exeSha256=(Get-FileHash -LiteralPath $taskSelf.MainModule.FileName).Hash.ToLowerInvariant()
    $taskLaunchFixture.sandboxUsed=$true
    $taskLaunchFixture | ConvertTo-Json | Set-Content -LiteralPath $taskLaunchInput -Encoding utf8
    Assert-IdleRejected {& $taskObserver -LaunchPath $taskLaunchInput -ReadinessPath $taskReadinessInput `
        -OutputDirectory $taskNeverOutput} 'excludes sandbox/CDP'
    Assert-IdleTest (!(Test-Path -LiteralPath $taskNeverOutput)) 'instrumented launch creates no measurement'
    $taskLaunchFixture.sandboxUsed=$false
    $taskLaunchFixture | ConvertTo-Json | Set-Content -LiteralPath $taskLaunchInput -Encoding utf8
    Assert-IdleRejected {& $taskObserver -LaunchPath $taskLaunchInput -ReadinessPath $taskReadinessInput `
        -OutputDirectory $taskNeverOutput} 'Observed preparation scene'
    Assert-IdleTest (!(Test-Path -LiteralPath $taskNeverOutput)) 'missing live readiness creates no measurement'
    Close-WindowsIdleFamily -State $taskState
    Close-WindowsIdleFamily -State $taskState
    Assert-IdleRejected {Get-WindowsIdleFamilySample -State $taskState -Rows @()} 'already invalid or closed'
    Assert-IdleTest (!(Get-Process -Id $PID).HasExited) 'closing observation handles does not terminate the root'
    $taskResult=[ordered]@{
        status='passed';checks=$script:taskCheckCount;failureRecords=$script:taskFailureRecords.Count
        controlledPowershellChildrenStarted=1;productStarted=$false;etwStarted=$false
        foregroundActivated=$false;longMeasurementRun=$false;fixtureForceTerminated=$false
        injectedMissingPidAndGenerationSnapshotsUsed=$true
        originalPerformanceBudgetChanged=$false;fullPerformanceGatePassed=$false
    }
    $taskResult | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskOutput 'result.json') -Encoding utf8
    $taskResult | ConvertTo-Json
} finally {
    if($taskChild -and !$taskChild.HasExited){
        [IO.File]::WriteAllText($taskRelease,'release')
        [void]$taskChild.WaitForExit(5000)
    }
    foreach($taskStateItem in $taskStates){Close-WindowsIdleFamily -State $taskStateItem}
    if($taskChild){$taskChild.Dispose()}
    $taskSelf.Dispose()
}

# Read-only process-family counters. This module does not start/close apps,
# activate windows, retry a measurement, or decide a performance budget.
function Stop-WindowsIdleObservation {
    param(
        [Parameter(Mandatory)][hashtable]$State,
        [Parameter(Mandatory)][string]$Stage,
        [object]$Row,
        [Nullable[datetime]]$ActualCreationUtc,
        [System.Exception]$Failure
    )
    $State.Failed=$true
    $taskExpectedCreation=$null
    if($Row -and $Row.CreationDate -is [datetime]){
        $taskExpectedCreation=$Row.CreationDate.ToUniversalTime().ToString('o')
    }
    $taskRecord=[ordered]@{
        schemaVersion=1;atUtc=[datetimeoffset]::UtcNow.ToString('o')
        elapsedSeconds=$State.Timer.Elapsed.TotalSeconds;stage=$Stage
        rootPid=$State.RootPid;rootCreationUtc=$State.RootCreationUtc.ToString('o')
        processId=if($Row){[int]$Row.ProcessId}else{$null}
        parentProcessId=if($Row){[int]$Row.ParentProcessId}else{$null}
        imageName=if($Row){[IO.Path]::GetFileName([string]$Row.Name)}else{$null}
        expectedCreationUtc=$taskExpectedCreation
        actualCreationUtc=if($null -ne $ActualCreationUtc){$ActualCreationUtc.ToString('o')}else{$null}
        exceptionType=if($Failure){$Failure.GetType().FullName}else{$null}
        hresult=if($Failure){$Failure.HResult}else{$null}
        baseExceptionType=if($Failure){$Failure.GetBaseException().GetType().FullName}else{$null}
        baseHresult=if($Failure){$Failure.GetBaseException().HResult}else{$null}
        lastWindowFrame=if($State.ContainsKey('LastFrame')){$State.LastFrame}else{$null}
        rawExceptionMessageSaved=$false;commandLineSaved=$false
        observationInvalid=$true;measurementRetryRequested=$false
    }
    $State.Failures.Add([pscustomobject]$taskRecord)
    # A failed evidence writer must also fail the caller, never turn a missing
    # diagnostic into a successful sample.
    & $State.OnFailure ([pscustomobject]$taskRecord) | Out-Null
    throw [InvalidOperationException]::new("Idle observation invalid: $Stage")
}

function New-WindowsIdleFamily {
    param(
        [Parameter(Mandatory)][Diagnostics.Process]$RootProcess,
        [Parameter(Mandatory)][scriptblock]$OnFailure
    )
    $taskRoot=[Diagnostics.Process]::GetProcessById($RootProcess.Id)
    try {
        [void]$taskRoot.SafeHandle
        if($taskRoot.StartTime.ToUniversalTime().Ticks -ne $RootProcess.StartTime.ToUniversalTime().Ticks){
            throw 'Root process generation changed'
        }
        $taskCreated=$taskRoot.StartTime.ToUniversalTime()
        $taskRow=[pscustomobject]@{ProcessId=$taskRoot.Id;ParentProcessId=0;CreationDate=$taskCreated;Name=$taskRoot.ProcessName}
        $taskState=@{
            RootPid=$taskRoot.Id;RootCreationUtc=$taskCreated;OnFailure=$OnFailure
            Members=[Collections.Generic.List[object]]::new()
            Failures=[Collections.Generic.List[object]]::new()
            Timer=[Diagnostics.Stopwatch]::StartNew();Failed=$false;Closed=$false
        }
        $taskState.Members.Add(@{
            Process=$taskRoot;CreationUtc=$taskCreated;ExitUtc=$null;Row=$taskRow
            CimCreationTicks=$taskCreated.Ticks
        })
        return $taskState
    } catch {
        $taskRoot.Dispose()
        throw
    }
}

function Assert-WindowsIdleFrame {
    param(
        [Parameter(Mandatory)][hashtable]$State,
        [Parameter(Mandatory)][bool]$Foreground,
        [Parameter(Mandatory)][bool]$Visible,
        [Parameter(Mandatory)][bool]$Minimized,
        [Parameter(Mandatory)][int]$NativeCount
    )
    if($State.Failed -or $State.Closed){throw 'Idle observation already invalid or closed'}
    $State.LastFrame=[pscustomobject]@{
        foreground=$Foreground;visible=$Visible;minimized=$Minimized;nativeCount=$NativeCount
    }
    if(!$Foreground){Stop-WindowsIdleObservation -State $State -Stage 'foreground-lost'}
    if(!$Visible){Stop-WindowsIdleObservation -State $State -Stage 'window-hidden'}
    if($Minimized){Stop-WindowsIdleObservation -State $State -Stage 'window-minimized'}
    if($NativeCount -ne 1){Stop-WindowsIdleObservation -State $State -Stage 'native-instance-count'}
}

function Get-WindowsIdleFamilySample {
    param(
        [Parameter(Mandatory)][hashtable]$State,
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Rows
    )
    if($State.Failed -or $State.Closed){throw 'Idle observation already invalid or closed'}
    foreach($taskMember in $State.Members){
        try {
            $taskMember.Process.Refresh()
            if($taskMember.Process.HasExited){
                $taskMember.ExitUtc=$taskMember.Process.ExitTime.ToUniversalTime()
            }
        } catch {
            Stop-WindowsIdleObservation -State $State -Stage 'retained-lifecycle-read' -Row $taskMember.Row -Failure $_.Exception
        }
    }
    do {
        $taskChanged=$false
        foreach($taskRow in $Rows){
            $taskCandidateParents=@($State.Members | Where-Object {
                $_.Row.ProcessId -eq $taskRow.ParentProcessId
            })
            if(!$taskCandidateParents.Count){continue}
            try {
                $taskCreated=$taskRow.CreationDate.ToUniversalTime()
            } catch {
                Stop-WindowsIdleObservation -State $State -Stage 'child-snapshot-generation' -Row $taskRow -Failure $_.Exception
            }
            $taskKnown=@($State.Members | Where-Object {
                $_.Row.ProcessId -eq $taskRow.ProcessId -and
                [Math]::Abs($_.CreationUtc.Ticks-$taskCreated.Ticks) -le 10
            })
            if($taskKnown.Count){continue}
            $taskParents=@($taskCandidateParents | Where-Object {
                $_.CreationUtc -le $taskCreated -and
                ($null -eq $_.ExitUtc -or $taskCreated -le $_.ExitUtc)
            })
            if(!$taskParents.Count){continue}
            if($taskParents.Count -ne 1){
                Stop-WindowsIdleObservation -State $State -Stage 'ambiguous-parent-generation' -Row $taskRow
            }
            $taskChild=$null
            $taskStage='child-process-open'
            try {
                $taskChild=[Diagnostics.Process]::GetProcessById([int]$taskRow.ProcessId)
                [void]$taskChild.SafeHandle
                $taskActual=$taskChild.StartTime.ToUniversalTime()
                $taskStage='child-generation-match'
                if([Math]::Abs($taskActual.Ticks-$taskCreated.Ticks) -gt 10){
                    $taskChild.Dispose()
                    $taskChild=$null
                    Stop-WindowsIdleObservation -State $State -Stage $taskStage -Row $taskRow -ActualCreationUtc $taskActual
                }
                $taskStateMember=@{
                    Process=$taskChild;CreationUtc=$taskActual;ExitUtc=$null
                    CimCreationTicks=$taskCreated.Ticks;Row=$taskRow
                }
                $State.Members.Add($taskStateMember)
                $taskChild=$null
                $taskChanged=$true
            } catch {
                if($null -ne $taskChild){$taskChild.Dispose()}
                if($State.Failed){throw}
                Stop-WindowsIdleObservation -State $State -Stage $taskStage -Row $taskRow -Failure $_.Exception
            }
        }
    } while($taskChanged)
    $taskCpu=0.0
    $taskRss=[long]0
    $taskCounters=[Collections.Generic.List[object]]::new()
    foreach($taskMember in $State.Members){
        try {
            $taskProcess=$taskMember.Process
            $taskProcess.Refresh()
            $taskExited=$taskProcess.HasExited
            $taskSeconds=$taskProcess.TotalProcessorTime.TotalSeconds
            $taskBytes=if($taskExited){[long]0}else{$taskProcess.WorkingSet64}
            $taskCpu+=$taskSeconds
            $taskRss+=$taskBytes
            $taskCounters.Add([pscustomobject]@{
                pid=$taskProcess.Id;creationUtc=$taskMember.CreationUtc.ToString('o')
                name=$taskMember.Row.Name;cpuSeconds=$taskSeconds
                workingSetBytes=$taskBytes;exited=$taskExited
            })
        } catch {
            Stop-WindowsIdleObservation -State $State -Stage 'retained-counter-read' -Row $taskMember.Row -Failure $_.Exception
        }
    }
    return [pscustomobject]@{
        cpuSeconds=$taskCpu;workingSetBytes=$taskRss;members=$taskCounters.ToArray()
        processHandlesRetained=$true;shortLivedChildrenFullyCovered=$false
    }
}

function Close-WindowsIdleFamily {
    param([Parameter(Mandatory)][hashtable]$State)
    if($State.Closed){return}
    foreach($taskMember in $State.Members){$taskMember.Process.Dispose()}
    $State.Closed=$true
    $State.Timer.Stop()
}

function Write-WindowsIdleTerminal {
    param(
        [Parameter(Mandatory)][hashtable]$State,
        [Parameter(Mandatory)][string]$OutputDirectory,
        [Parameter(Mandatory)][int]$SampleCount,
        [AllowNull()][Nullable[double]]$FirstElapsed,
        [AllowNull()][Nullable[double]]$LastElapsed,
        [Parameter(Mandatory)][bool]$Completed,
        [Parameter(Mandatory)][double]$CollectorCpuSeconds
    )
    $taskSpan=if($null -ne $FirstElapsed -and $null -ne $LastElapsed){$LastElapsed-$FirstElapsed}else{0}
    $taskValid=$Completed -and !$State.Failed -and $SampleCount -gt 1 -and $taskSpan -ge 600
    $taskTerminal=[ordered]@{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');phase=if($taskValid){'completed'}else{'invalid'}
        measurementCompleted=$taskValid;observationInvalid=(!$taskValid)
        samples=$SampleCount;counterSpanSeconds=$taskSpan;readOrFrameFailures=$State.Failures.Count
        collectorCpuSeconds=$CollectorCpuSeconds
        productLeftAliveForBoundNormalClose=$true;uiInputInjected=$false
        performanceBudgetClaimedPassed=$false;automaticRetryRequested=$false
    }
    $taskJson=$taskTerminal | ConvertTo-Json
    $taskJson | Set-Content -LiteralPath (Join-Path $OutputDirectory 'terminal.json') -Encoding utf8
    $taskJson | Set-Content -LiteralPath (Join-Path $OutputDirectory 'progress.json') -Encoding utf8
    return [pscustomobject]$taskTerminal
}

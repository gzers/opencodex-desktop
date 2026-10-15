$ErrorActionPreference='Stop'
$taskReadyPath=Join-Path $PSScriptRoot 'administrator-window-sampler-ready.json'
$taskDataPath=Join-Path $PSScriptRoot 'administrator-window-samples.original.jsonl'
$taskResultPath=Join-Path $PSScriptRoot 'administrator-window-sampler-result.json'
foreach($taskPath in @($taskReadyPath,$taskDataPath,$taskResultPath)){
    if(Test-Path -LiteralPath $taskPath){throw 'Do not overwrite sampler evidence'}
}
try {
    . (Join-Path $PSScriptRoot 'foreground-contract-live.ps1')
    $taskAuth=Assert-BackupForegroundAuthorization -Authorization (
        Get-Content -LiteralPath (Join-Path $PSScriptRoot 'foreground-authorization.json') -Raw | ConvertFrom-Json)
    # Read-only initialization binds the actual administrator HWND, token and
    # creation time. All P/Invokes in this reader query state; none inject input.
    . (Join-Path $PSScriptRoot 'read-state.ps1') -Phase first-screen -Mode administrator | Out-Null
    $taskIdentity=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-launch.json') -Raw | ConvertFrom-Json
    $taskActual=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-first-screen.json') -Raw | ConvertFrom-Json
    if(!$taskActual.applicationToken.elevated -or $taskActual.applicationToken.elevationType -ne 2){throw 'Actual administrator token required'}
    $taskBoundProcess=Get-Process -Id $taskIdentity.pid
    $taskHeldHandle=$taskBoundProcess.Handle
    if($taskBoundProcess.StartTime.ToUniversalTime().Ticks -ne ([datetimeoffset]$taskIdentity.creationTimeUtc).UtcDateTime.Ticks){
        throw 'PID identity changed'
    }
    $taskWindow=[IntPtr]::new([long]$taskActual.hwnd)
    $taskDuration=[Math]::Max(1,[Math]::Min(240,[Math]::Floor($taskAuth.remainingSeconds)))
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskIdentity.pid
        creationTimeUtc=$taskIdentity.creationTimeUtc;hwnd=$taskActual.hwnd
        actualToken=$taskActual.applicationToken;handleHeld=$true;maxSeconds=$taskDuration
        intervalMilliseconds=1000;inputInjected=$false;sourceDataChanged=$false
        systemSettingsChanged=$false;unrelatedWindowIdsOrTitlesSaved=$false} |
        ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskReadyPath -Encoding utf8
    $taskTimer=[Diagnostics.Stopwatch]::StartNew()
    $taskStream=[IO.StreamWriter]::new([IO.File]::Open($taskDataPath,[IO.FileMode]::CreateNew),[Text.UTF8Encoding]::new($false))
    $taskCount=0;$taskExitSeen=$false
    try {
        while($taskTimer.Elapsed.TotalSeconds -lt $taskDuration) {
            $taskBoundProcess.Refresh()
            if($taskBoundProcess.HasExited){$taskExitSeen=$true;break}
            $taskWindowPid=[uint32]0
            [void][ForegroundGateReadOnly]::GetWindowThreadProcessId($taskWindow,[ref]$taskWindowPid)
            if($taskWindowPid -ne $taskIdentity.pid){throw 'Original main HWND no longer belongs to the admitted process'}
            $taskRect=[ForegroundGateReadOnly+Rect]::new()
            $taskRectValid=[ForegroundGateReadOnly]::GetWindowRect($taskWindow,[ref]$taskRect)
            $taskBackdrop=0
            $taskDwm=[ForegroundGateReadOnly]::DwmGetWindowAttribute($taskWindow,38,[ref]$taskBackdrop,4)
            $taskRow=[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o')
                elapsedMilliseconds=$taskTimer.Elapsed.TotalMilliseconds;pid=$taskIdentity.pid
                creationTimeUtc=$taskIdentity.creationTimeUtc;hwnd=$taskActual.hwnd
                visible=[ForegroundGateReadOnly]::IsWindowVisible($taskWindow)
                maximized=[ForegroundGateReadOnly]::IsZoomed($taskWindow)
                minimized=[ForegroundGateReadOnly]::IsIconic($taskWindow)
                foreground=([ForegroundGateReadOnly]::GetForegroundWindow() -eq $taskWindow)
                rectValid=$taskRectValid;outerRect=$taskRect
                backdropHresult=$taskDwm;backdropValue=$taskBackdrop}
            $taskStream.WriteLine(($taskRow|ConvertTo-Json -Depth 5 -Compress))
            $taskStream.Flush();$taskCount++
            Start-Sleep -Milliseconds 1000
        }
    } finally {$taskStream.Dispose()}
    $taskResult=[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskIdentity.pid
        creationTimeUtc=$taskIdentity.creationTimeUtc;elapsedSeconds=$taskTimer.Elapsed.TotalSeconds
        samples=$taskCount;exitObserved=$taskExitSeen
        exitCode=$(if($taskExitSeen){$taskBoundProcess.ExitCode}else{$null})
        observationLimitReached=(-not $taskExitSeen);inputInjected=$false;agentForcedTermination=$false
        macNativeHandoffPerformed=$false;fullVersionGatePassed=$false}
    $taskResult | ConvertTo-Json | Set-Content -LiteralPath $taskResultPath -Encoding utf8
    $taskBoundProcess.Dispose()
} catch {
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');errorType=$_.Exception.GetType().Name
        message=$_.Exception.Message;inputInjected=$false;automaticRetryAllowed=$false} |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-window-sampler-error.json') -Encoding utf8
    throw
}

$ErrorActionPreference='Stop'
$taskShell=[Diagnostics.Process]::GetCurrentProcess().MainModule.FileName
$taskStarted=[datetimeoffset]::UtcNow
$taskLimit=$taskStarted.AddMinutes(34)
$taskFocused=$false
try{
    while([datetimeoffset]::UtcNow -lt $taskLimit){
        $taskProgressPath=Join-Path $PSScriptRoot 'logpoint-progress.json'
        $taskResultPath=Join-Path $PSScriptRoot 'logpoint-result.json'
        if(Test-Path -LiteralPath $taskResultPath){break}
        if(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'logpoint-failure.original.json')){throw 'Observer failed; inspect without fabricating a completed floor'}
        $taskProgress=$null
        try{$taskProgress=Get-Content -LiteralPath $taskProgressPath -Raw | ConvertFrom-Json}catch{}
        if($taskProgress -and !$taskFocused -and [datetimeoffset]::UtcNow -ge ([datetimeoffset]$taskProgress.dueAtUtcEstimate).AddSeconds(-8)){
            & $taskShell -NoLogo -NoProfile -NonInteractive -ExecutionPolicy RemoteSigned -File (Join-Path $PSScriptRoot 'native.ps1') -Launch state-floor-retry -Phase state-floor-before-real-deadline *> (Join-Path $PSScriptRoot 'before-real-deadline.log')
            if($LASTEXITCODE -ne 0){throw 'Could not bring the same real timing instance foreground before its deadline'}
            $taskFocused=$true
        }
        Start-Sleep -Seconds 1
    }
    if(!(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'logpoint-result.json'))){throw 'No result before finish-controller bound'}
    # Wait for the observer's final debugger cleanup; do not reset the product.
    Start-Sleep -Seconds 2
    $taskResult=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'logpoint-result.json') -Raw | ConvertFrom-Json
    & $taskShell -NoLogo -NoProfile -NonInteractive -ExecutionPolicy RemoteSigned -File (Join-Path $PSScriptRoot 'native.ps1') -Launch state-floor-retry -Phase state-floor-after-real-deadline *> (Join-Path $PSScriptRoot 'after-real-deadline.log')
    if($LASTEXITCODE -ne 0){throw 'Final same-instance evidence failed'}
    & $taskShell -NoLogo -NoProfile -NonInteractive -ExecutionPolicy RemoteSigned -File (Join-Path $PSScriptRoot 'native.ps1') -Launch state-floor-retry -Phase state-floor-normal-close -Action close *> (Join-Path $PSScriptRoot 'normal-close.log')
    if($LASTEXITCODE -ne 0){throw 'Final normal-close action failed'}
    Start-Sleep -Seconds 1
    $taskExit=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'state-floor-retry-exit.json') -Raw | ConvertFrom-Json
    [ordered]@{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');finishControllerStartedAt=$taskStarted.ToString('o')
        foregroundBeforeDuePerformed=$taskFocused
        realMonotonicRetryGapMs=$taskResult.realMonotonicRetryGapMs
        noEarlyPlannerCalls=$taskResult.noEarlyPlannerCalls
        realRetryFulfilled=$taskResult.realRetryFulfilled;debuggerPauses=$taskResult.pauses
        exitObserved=$taskExit.exitObserved;exitCode=$taskExit.exitCode
        nativeCount=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count
        forcedTermination=$false;fullSchedulerGatePassed=$false;fullVersionGatePassed=$false
    } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'finish-result.json') -Encoding utf8
}catch{
    $_ | Out-String | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'finish-failure.original.txt') -Encoding utf8
    throw
}

param(
    [Parameter(Mandatory)][string]$OutputDirectory,
    [Parameter(Mandatory)][string]$NodeExecutable,
    [switch]$MechanismOnly,
    [string]$HumanWindow,
    [string]$DecoderDirectory,
    [switch]$RecordApproved
)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'windows-process-token.ps1')
. (Join-Path $PSScriptRoot 'windows-startup-pair-state.ps1')
if($MechanismOnly -and $RecordApproved){throw 'Choose mechanism validation or approved recording once'}
if(!$MechanismOnly -and !$RecordApproved){throw 'Prepared-only: actual recording needs a fresh human window'}
$taskAuth=$null
if($RecordApproved){
    if(!$HumanWindow -or !$DecoderDirectory){throw 'Fresh human window and pinned decoder required'}
    $taskAuth=Get-Content -LiteralPath (ConvertTo-StartupPairPath $HumanWindow) -Raw -Encoding utf8|ConvertFrom-Json
    $taskStart=[datetimeoffset]::Parse($taskAuth.authorizedAtUtc)
    $taskEnd=[datetimeoffset]::Parse($taskAuth.expiresAtUtc)
    $taskNow=[datetimeoffset]::UtcNow
    $taskClose=$taskStart.AddMinutes(25)
    if($taskEnd -lt $taskClose){$taskClose=$taskEnd}
    if($taskAuth.origin -cne 'human' -or $taskAuth.explicitHumanStart -isnot [bool] -or
        !$taskAuth.explicitHumanStart -or $taskAuth.automaticContinuation -isnot [bool] -or
        $taskAuth.automaticContinuation -or $taskEnd -le $taskStart -or
        ($taskEnd-$taskStart).TotalMinutes -gt 30 -or $taskNow -lt $taskStart -or
        $taskNow -ge $taskEnd -or $taskNow.AddSeconds(75) -ge $taskClose){
        throw 'Fresh human window and calibration cleanup reservation required'
    }
}
Assert-WindowsProcessMode -Observation (Get-WindowsHarnessToken) -Mode ordinary
Assert-StartupPairNoProduct
$taskRepo=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$taskRoot=ConvertTo-StartupPairPath ([IO.Path]::GetFullPath($OutputDirectory))
if(!$taskRoot.StartsWith((Join-Path $taskRepo 'test/runtime')+'\',[StringComparison]::OrdinalIgnoreCase) -or
    (Test-Path -LiteralPath $taskRoot)){throw 'Fresh owned diagnostic root required'}
$taskNode=ConvertTo-StartupPairPath $NodeExecutable
New-Item -ItemType Directory -Path $taskRoot|Out-Null
$taskCopy=Join-Path $taskRoot 'owned-calibration-node.exe'
Copy-Item -LiteralPath $taskNode -Destination $taskCopy
$taskSourceSha=(Get-FileHash -LiteralPath $taskNode).Hash.ToLowerInvariant()
if((Get-FileHash -LiteralPath $taskCopy).Hash.ToLowerInvariant() -cne $taskSourceSha){
    throw 'Installed Node copy differs'
}
$taskResult=Join-Path $taskRoot 'probe-result.json'
$taskInfo=[Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName=$taskCopy
$taskInfo.WorkingDirectory=$taskRoot
$taskInfo.UseShellExecute=$false
$taskInfo.CreateNoWindow=$true
$taskInfo.RedirectStandardInput=$true
$taskInfo.RedirectStandardOutput=$true
$taskInfo.RedirectStandardError=$true
[void]$taskInfo.Environment.Remove('NODE_OPTIONS')
foreach($taskArg in @((Join-Path $PSScriptRoot 'windows-query-calibration-target.mjs'),'--probe',$taskResult)){
    $taskInfo.ArgumentList.Add($taskArg)
}
$taskProcess=$null
$taskError=$null
$taskToken=$null
$taskData=$null
$taskStdout=$null
$taskStderr=$null
$taskAdministrator=$null
$taskStarted=$false
$taskNative=$null
$taskNativeProof=$null
$taskEtwReady=$false
$taskMethods=$null
try{
    if($RecordApproved){
        $taskMethods=@(foreach($taskName in @('windows-owned-etw.cs','windows-query-counters.cs',
            'windows-query-counters.ps1','windows-query-calibration-admin.ps1')){
            [ordered]@{name=$taskName;sha256=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $taskName)).Hash.ToLowerInvariant()}
        })
        $taskPlanPath=Join-Path $taskRoot 'etw-plan.json'
        [ordered]@{authorization=$taskAuth;runDirectory=$taskRoot;probeExecutable=$taskCopy
            nodeSha256=$taskSourceSha;decoderDirectory=(ConvertTo-StartupPairPath $DecoderDirectory)
            methods=$taskMethods}|ConvertTo-Json -Depth 8|Set-Content -LiteralPath $taskPlanPath -Encoding utf8
        $taskPowerShell=Join-Path $PSHOME 'pwsh.exe'
        $taskAdminScript=Join-Path $PSScriptRoot 'windows-query-calibration-admin.ps1'
        $taskAdministrator=Start-Process -FilePath $taskPowerShell -Verb RunAs -WindowStyle Hidden -PassThru -ArgumentList @(
            '-NoProfile','-NonInteractive','-File',('"'+$taskAdminScript+'"'),
            '-Plan',('"'+$taskPlanPath+'"'),'-RecordApproved')
        $taskReady=Join-Path $taskRoot 'native-calibration\ready.json'
        $taskWait=[Diagnostics.Stopwatch]::StartNew()
        while(!(Test-Path -LiteralPath $taskReady)){
            if($taskAdministrator.HasExited){throw 'Elevated observer exited before ready'}
            if($taskWait.Elapsed.TotalSeconds -ge 25){throw 'Elevated observer readiness deadline'}
            Start-Sleep -Milliseconds 100
        }
        $taskEtwReady=$true
    }
    $taskProcess=[Diagnostics.Process]::Start($taskInfo)
    $taskStarted=$true
    $null=$taskProcess.SafeHandle
    $taskStdout=$taskProcess.StandardOutput.ReadToEndAsync()
    $taskStderr=$taskProcess.StandardError.ReadToEndAsync()
    $taskToken=Get-WindowsProcessToken -Process $taskProcess -ExpectedExecutable $taskCopy
    Assert-WindowsProcessMode -Observation $taskToken -Mode ordinary
    $taskWait=[Diagnostics.Stopwatch]::StartNew()
    while(!(Test-Path -LiteralPath $taskResult)){
        if($taskProcess.HasExited){throw 'Diagnostic root exited before its result'}
        if($taskWait.Elapsed.TotalSeconds -ge 30){throw 'Diagnostic workload deadline'}
        Start-Sleep -Milliseconds 50
    }
    $taskData=Get-Content -LiteralPath $taskResult -Raw|ConvertFrom-Json
    if($taskData.pid -ne $taskProcess.Id -or $taskData.children.Count -ne 20 -or
        @($taskData.children|Where-Object exitCode -ne 0).Count -or
        $taskData.loopbackHttpRequests -ne 4 -or $taskData.loopbackBodyBytes -ne 1024 -or
        $taskData.externalNetworkUsed -or $taskData.productStarted){
        throw 'Owned diagnostic workload result differs'
    }
    if($RecordApproved){
        $taskWait=[Diagnostics.Stopwatch]::StartNew()
        $taskRootSeen=$false
        $taskEvents=Join-Path $taskRoot 'native-calibration\owned-events.original.jsonl'
        while(!$taskRootSeen){
            if(Test-Path -LiteralPath $taskEvents){
                $taskFile=[IO.FileStream]::new($taskEvents,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::ReadWrite)
                $taskReader=[IO.StreamReader]::new($taskFile)
                try{
                    while(!$taskReader.EndOfStream){
                        $taskLine=$taskReader.ReadLine()
                        try{$taskEvent=$taskLine|ConvertFrom-Json}catch{continue}
                        if($taskEvent.kind -ceq 'process-start' -and $taskEvent.pid -eq $taskProcess.Id -and
                            $taskEvent.exactLiveImageVerified){$taskRootSeen=$true;break}
                    }
                }finally{$taskReader.Dispose();$taskFile.Dispose()}
            }
            if($taskRootSeen){break}
            if($taskAdministrator.HasExited -or $taskWait.Elapsed.TotalSeconds -ge 10){
                throw 'Actual ETW root start not observed while the retained diagnostic root is alive'
            }
            Start-Sleep -Milliseconds 100
        }
    }
}catch{
    $taskError=$_.Exception.ToString()
    throw
}finally{
    $taskExit=$null
    $taskExited=$false
    if($taskProcess){
        if(!$taskProcess.HasExited){
            $taskProcess.StandardInput.WriteLine('exit')
            $taskProcess.StandardInput.Flush()
            $taskProcess.StandardInput.Close()
        }
        $taskExited=$taskProcess.WaitForExit(10000)
        if($taskExited){$taskExit=$taskProcess.ExitCode}
        if($taskStdout -and $taskStdout.Wait(1000)){
            [IO.File]::WriteAllText((Join-Path $taskRoot 'stdout.original.log'),$taskStdout.Result)
        }
        if($taskStderr -and $taskStderr.Wait(1000)){
            [IO.File]::WriteAllText((Join-Path $taskRoot 'stderr.original.log'),$taskStderr.Result)
        }
        $taskProcess.Dispose()
    }
    if($taskAdministrator){
        [IO.File]::WriteAllText((Join-Path $taskRoot 'stop-etw.flag'),'stop')
        if(!$taskAdministrator.WaitForExit(20000)){
            $taskError='Elevated calibration observer did not exit within its cleanup bound'
        }elseif($taskAdministrator.ExitCode -ne 0){
            $taskError='Elevated calibration observer exited nonzero; terminal/raw events retained'
        }else{
            try{
                $taskNative=Get-Content -LiteralPath (Join-Path $taskRoot 'native-calibration\terminal.json') -Raw|ConvertFrom-Json
                if(!$taskNative.complete -or $taskNative.rootStarts -ne 1 -or $taskNative.maximumConcurrentRoots -ne 1){
                    throw 'Native calibration capture completeness differs'
                }
                $taskAll=@(Get-Content -LiteralPath (Join-Path $taskRoot 'native-calibration\owned-events.original.jsonl')|
                    ForEach-Object {$_|ConvertFrom-Json})
                $taskChildStarts=@($taskAll|Where-Object {$_.kind -ceq 'process-start' -and
                    $_.imageBasename -ieq 'cmd.exe' -and $_.rootPid -eq $taskToken.processId})
                if($taskChildStarts.Count -ne 20 -or
                    @(Compare-Object (@($taskData.children.pid)|Sort-Object) (@($taskChildStarts.pid)|Sort-Object)).Count){
                    throw 'Original ETW child starts differ from the ordinary target receipt'
                }
                $taskDurations=@(foreach($taskChild in $taskChildStarts){
                    $taskStops=@($taskAll|Where-Object {$_.kind -ceq 'process-stop' -and
                        $_.pid -eq $taskChild.pid -and $_.generation -eq $taskChild.generation})
                    if($taskStops.Count -ne 1 -or $taskStops[0].exitStatus -ne 0){throw 'Exact child generation did not close normally'}
                    ($taskStops[0].original.qpc-$taskChild.original.qpc)*1000.0/[Diagnostics.Stopwatch]::Frequency
                })
                if(!@($taskDurations|Where-Object {$_ -gt 0 -and $_ -lt 1000}).Count){
                    throw 'No actual subsecond child lifetime observed'
                }
                $taskSend=@($taskAll|Where-Object {$_.kind -ceq 'network' -and $_.original.eventName -like 'TcpIp/*Send*'})
                $taskReceive=@($taskAll|Where-Object {$_.kind -ceq 'network' -and $_.original.eventName -like 'TcpIp/*Recv*'})
                if(!$taskSend.Count -or !$taskReceive.Count -or $taskNative.networkBytes -lt 1024){
                    throw 'Actual owned TCP send/receive positive control not established'
                }
                $taskNativeProof=[ordered]@{passed=$true;rootPid=$taskToken.processId
                    exactChildLifetimes=20;childDurationMs=$taskDurations;qpcFrequency=[Diagnostics.Stopwatch]::Frequency
                    tcpSendEvents=$taskSend.Count;tcpReceiveEvents=$taskReceive.Count
                    kernelTransportBytes=$taskNative.networkBytes;subsecondLifecycleObserved=$true
                    udpOrIpv6CalibrationClaimed=$false}
            }catch{$taskError=$_.Exception.ToString()}
        }
        $taskAdministrator.Dispose()
    }
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');nodeSource=$taskNode;nodeSha256=$taskSourceSha
        ordinaryRoot=$taskToken;workload=$taskData;normalExitObserved=$taskExited;exitCode=$taskExit
        error=$taskError;forcedTermination=$false
        etwSessionStarted=$(if($MechanismOnly){$false}elseif($taskEtwReady){$true}else{$null})
        nativePositiveControl=$taskNativeProof
        observerMethods=$taskMethods
        rawKernelPositiveControlPassed=($null -ne $taskNativeProof -and !$taskError)
        productStarted=$false;productForegroundUsed=$false;systemElevationRequested=[bool]$RecordApproved
        foregroundUsed=$(if($MechanismOnly){$false}else{$null})
        externalNetworkUsed=$false}|ConvertTo-Json -Depth 8|
        Set-Content -LiteralPath (Join-Path $taskRoot 'mechanism-result.json') -Encoding utf8
    if($taskStarted -and (!$taskExited -or $taskExit -ne 0)){throw 'Owned diagnostic normal exit not confirmed'}
    if($taskError){throw $taskError}
}
if($RecordApproved){Write-Output 'Owned ETW positive control completed; no product started.'}
else{Write-Output 'Owned hidden workload completed: 20 normal children and four loopback requests; ETW still uncalibrated.'}

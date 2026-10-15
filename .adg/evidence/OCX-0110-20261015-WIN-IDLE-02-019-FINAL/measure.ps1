$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskKit=Join-Path $taskCode 'test/software/acceptance-kit-02a0d4-20261015'
$taskShell=[Diagnostics.Process]::GetCurrentProcess().MainModule.FileName
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
. (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/methods/process-user-sid.ps1')
Add-Type -TypeDefinition @'
using System; using System.Runtime.InteropServices;
public static class IdleNative {
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr w);
 [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr w);
}
'@
$taskContract=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'contract.json') -Raw | ConvertFrom-Json
$taskCompleted=@()
try{
    foreach($taskVersion in $taskContract.order){
        $taskResume=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'foreground-resume.json') -Raw | ConvertFrom-Json
        $taskNativeCountBefore=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count
        if($taskNativeCountBefore -ne $(if($taskVersion -eq 'baseline019'){1}else{0})){throw 'Unexpected instance count before paired version'}
        $taskExe=if($taskVersion -eq 'candidate0110'){
            Join-Path $taskCode 'test/runtime/foreground-02-current-20261015/install-b/opencodex-desktop.exe'
        }else{Join-Path $taskKit 'bin/baseline019/opencodex-desktop.exe'}
        $taskExpected=if($taskVersion -eq 'candidate0110'){$taskContract.candidateExeSha256}else{$taskContract.baselineExeSha256}
        $taskSource=if($taskVersion -eq 'candidate0110'){$taskContract.candidateSource}else{$taskContract.baselineSource}
        if((Get-FileHash -LiteralPath $taskExe).Hash.ToLowerInvariant() -cne $taskExpected){throw 'Paired executable changed'}
        $taskInfo=[Diagnostics.ProcessStartInfo]::new()
        $taskInfo.FileName=$taskExe;$taskInfo.WorkingDirectory=Split-Path -Parent $taskExe;$taskInfo.UseShellExecute=$false
        foreach($taskName in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY','OPENCODEX_WINDOWS_SMOKE_CDP_PORT','OPENCODEX_WINDOWS_SMOKE_SCALE','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','WEBVIEW2_USER_DATA_FOLDER')){
            [void]$taskInfo.Environment.Remove($taskName)
        }
        $taskProcess=if($taskVersion -eq 'baseline019'){
            $taskExisting=Get-Process -Id $taskResume.existingProductPid
            if($taskExisting.StartTime.ToUniversalTime().Ticks -ne ([datetimeoffset]$taskResume.existingProductCreationUtc).UtcTicks){throw 'Resumed candidate generation changed'}
            $taskExisting
        }else{[Diagnostics.Process]::Start($taskInfo)}
        $taskToken=Get-AcceptanceProcessSidEvidence -Process $taskProcess -ExpectedExecutable $taskExe
        Assert-WindowsProcessMode -Observation $taskToken -Mode ordinary
        if($taskToken.sidSha256 -cne '373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310'){throw 'Paired user identity changed'}
        $taskPhase='idle-'+$taskVersion
        [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');phase=$taskPhase;mode='ordinary';source=$taskSource
            executable=$taskExe;exeSha256=$taskExpected;pid=$taskProcess.Id;creationTimeUtc=$taskToken.creationTimeUtc
            applicationToken=$taskToken;applicationUser=$taskToken;sandboxUsed=$false;cdpUsed=$false
        } | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $PSScriptRoot ($taskPhase+'-launch.json')) -Encoding utf8
        Start-Sleep -Seconds 4
        & $taskShell -NoLogo -NoProfile -NonInteractive -ExecutionPolicy RemoteSigned -File (Join-Path $PSScriptRoot 'native.ps1') -Launch $taskPhase -Phase ($taskPhase+'-ready') *> (Join-Path $PSScriptRoot ($taskPhase+'-ready.log'))
        if($LASTEXITCODE -ne 0){throw 'Paired overview readiness capture failed'}
        $taskView=Get-Content -LiteralPath (Join-Path $PSScriptRoot ($taskPhase+'-ready.json')) -Raw | ConvertFrom-Json
        if(!@($taskView.ax | Where-Object {$_.name -eq '尚未接入 OpenCodex'}).Count){throw 'Paired overview preparation scenario did not match'}
        $taskWindow=[IntPtr]$taskView.hwnd
        [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');version=$taskVersion;phase='settling';seconds=60;pid=$taskProcess.Id} |
            ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'progress.json') -Encoding utf8
        # The controller never blocks the interactive tool for this interval.
        for($taskSettle=0;$taskSettle -lt 60;$taskSettle++){Start-Sleep -Seconds 1}
        $taskHandles=@{}
        $taskHandles[[int]$taskProcess.Id]=@{process=$taskProcess;creation=$taskProcess.StartTime.ToUniversalTime();name='opencodex-desktop';root=$true}
        $taskStream=[IO.File]::Open((Join-Path $PSScriptRoot ($taskVersion+'-samples.original.jsonl')),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
        $taskWriter=[IO.StreamWriter]::new($taskStream,[Text.UTF8Encoding]::new($false))
        $taskTimer=[Diagnostics.Stopwatch]::StartNew()
        $taskController=[Diagnostics.Process]::GetCurrentProcess()
        $taskControllerCpuBefore=$taskController.TotalProcessorTime.TotalSeconds
        $taskSamples=0;$taskLostForeground=0;$taskInstanceViolations=0;$taskMissingChildren=0
        $taskFirstCounterElapsed=$null
        try{
            do{
                $taskIteration=[Diagnostics.Stopwatch]::StartNew()
                $taskRows=@(Get-CimInstance -Query 'SELECT ProcessId,ParentProcessId,CreationDate,Name FROM Win32_Process')
                $taskChanged=$true
                while($taskChanged){
                    $taskChanged=$false
                    foreach($taskRow in $taskRows){
                        $taskId=[int]$taskRow.ProcessId
                        if(!$taskHandles.ContainsKey($taskId) -and $taskHandles.ContainsKey([int]$taskRow.ParentProcessId)){
                            $taskParent=$taskHandles[[int]$taskRow.ParentProcessId]
                            if($taskRow.CreationDate.ToUniversalTime() -lt $taskParent.creation){continue}
                            try{
                                $taskChild=[Diagnostics.Process]::GetProcessById($taskId)
                                if([Math]::Abs($taskChild.StartTime.ToUniversalTime().Ticks-$taskRow.CreationDate.ToUniversalTime().Ticks) -gt 10){$taskChild.Dispose();throw 'CIM process generation differs beyond its microsecond precision'}
                                # Acquire a retained handle while this generation is live.
                                [void]$taskChild.SafeHandle
                                $taskHandles[$taskId]=@{process=$taskChild;creation=$taskChild.StartTime.ToUniversalTime();name=$taskRow.Name;root=$false}
                                $taskChanged=$true
                            }catch{$taskMissingChildren++}
                        }
                    }
                }
                $taskCpu=0.0;$taskRss=[long]0;$taskMembers=@()
                foreach($taskItem in $taskHandles.Values){
                    $taskMember=$taskItem.process
                    try{
                        $taskMember.Refresh()
                        $taskExited=$taskMember.HasExited
                        $taskMemberCpu=$taskMember.TotalProcessorTime.TotalSeconds
                        $taskMemberRss=if($taskExited){[long]0}else{$taskMember.WorkingSet64}
                        $taskCpu+=$taskMemberCpu;$taskRss+=$taskMemberRss
                        $taskMembers+=@{pid=$taskMember.Id;creationUtc=$taskItem.creation.ToString('o');name=$taskItem.name;cpuSeconds=$taskMemberCpu;workingSetBytes=$taskMemberRss;exited=$taskExited}
                    }catch{throw 'Retained process metric unavailable; do not silently drop its CPU'}
                }
                $taskForeground=[IdleNative]::GetForegroundWindow() -eq $taskWindow
                if(!$taskForeground){$taskLostForeground++}
                $taskNativeCount=@($taskRows | Where-Object {$_.Name -in @('opencodex-desktop.exe','ocxd.exe')}).Count
                if($taskNativeCount -ne 1){$taskInstanceViolations++}
                $taskElapsed=$taskTimer.Elapsed.TotalSeconds
                if($null -eq $taskFirstCounterElapsed){$taskFirstCounterElapsed=$taskElapsed}
                $taskWriter.WriteLine(([ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');elapsedSeconds=$taskElapsed
                    cpuSeconds=$taskCpu;workingSetBytes=$taskRss;foreground=$taskForeground;nativeCount=$taskNativeCount
                    visible=[IdleNative]::IsWindowVisible($taskWindow);minimized=[IdleNative]::IsIconic($taskWindow);members=$taskMembers
                } | ConvertTo-Json -Depth 7 -Compress))
                $taskWriter.Flush();$taskSamples++
                if($taskSamples % 10 -eq 0){
                    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');version=$taskVersion;phase='measuring';elapsedSeconds=$taskElapsed;samples=$taskSamples;lostForegroundSamples=$taskLostForeground;nativeInstanceViolations=$taskInstanceViolations} |
                        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'progress.json') -Encoding utf8
                }
                if($taskProcess.HasExited){throw 'Product exited during the paired measurement'}
                if(($taskElapsed-$taskFirstCounterElapsed) -ge 600){break}
                Start-Sleep -Milliseconds ([Math]::Max(1,1000-[int]$taskIteration.ElapsedMilliseconds))
            }while($true)
        }finally{$taskWriter.Dispose();$taskStream.Dispose()}
        $taskController.Refresh()
        [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');version=$taskVersion;source=$taskSource;exeSha256=$taskExpected
            samples=$taskSamples;durationSeconds=$taskTimer.Elapsed.TotalSeconds;observedCounterSpanSeconds=$taskElapsed-$taskFirstCounterElapsed;lostForegroundSamples=$taskLostForeground
            nativeInstanceViolations=$taskInstanceViolations;childDiscoveryReadFailures=$taskMissingChildren
            collectorCpuSeconds=$taskController.TotalProcessorTime.TotalSeconds-$taskControllerCpuBefore
            processHandlesRetained=$true;shortLivedChildrenFullyCovered=$false;fullPerformanceGatePassed=$false
        } | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $PSScriptRoot ($taskVersion+'-measurement.json')) -Encoding utf8
        & $taskShell -NoLogo -NoProfile -NonInteractive -ExecutionPolicy RemoteSigned -File (Join-Path $PSScriptRoot 'native.ps1') -Launch $taskPhase -Phase ($taskPhase+'-close') -Action close *> (Join-Path $PSScriptRoot ($taskPhase+'-close.log'))
        if(!$taskProcess.WaitForExit(10000)){throw 'Paired version did not exit normally; do not start the other version'}
        [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskProcess.Id;exitCode=$taskProcess.ExitCode;exitObserved=$true;forcedTermination=$false} |
            ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot ($taskVersion+'-exit.json')) -Encoding utf8
        foreach($taskItem in $taskHandles.Values){$taskItem.process.Dispose()}
        $taskController.Dispose()
        $taskCompleted+=$taskVersion
        Start-Sleep -Seconds 2
    }
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');completed=$taskCompleted;measurementCompleted=$true;fullPerformanceGatePassed=$false} |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'terminal.json') -Encoding utf8
}catch{
    $_ | Out-String | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'failure.original.txt') -Encoding utf8
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');completed=$taskCompleted;measurementCompleted=$false;fullPerformanceGatePassed=$false} |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'terminal.json') -Encoding utf8
    throw
}

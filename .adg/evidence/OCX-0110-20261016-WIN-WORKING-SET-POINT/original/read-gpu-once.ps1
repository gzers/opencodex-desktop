$ErrorActionPreference='Stop'
$taskRun=$PSScriptRoot
$taskRepo='C:\Users\15119\.codex\worktrees\windows-working-set\opencodex-desktop'
$taskDocs='D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop'
$taskAdmission=Get-Content -LiteralPath (Join-Path $taskRun 'method-admission.json') -Raw | ConvertFrom-Json
$taskMethod=Join-Path $taskRepo 'test/smoke/windows-working-set.ps1'
if($taskAdmission.source -cne '3a2ff08a6b480d237e93502626861090aa14423d' -or
   !$taskAdmission.windowsMethodJobSucceeded -or $taskAdmission.controlledChecks -ne 34 -or
   (git -C $taskRepo rev-parse HEAD).Trim() -cne $taskAdmission.source -or
   (Get-FileHash -LiteralPath $taskMethod).Hash.ToLowerInvariant() -cne $taskAdmission.localMethodSha256){throw 'Committed, controlled Windows method admission required'}
$taskEvidence=Join-Path $taskDocs '.adg/evidence/OCX-0110-20261016-WIN-B4-IDLE-ROLES'
& python (Join-Path $taskEvidence 'verify.py') *> (Join-Path $taskRun 'counter-binding-verification.original.log')
if($LASTEXITCODE -ne 0){throw 'Frozen counter/role source changed'}
$taskRoot=Get-Content -LiteralPath (Join-Path $taskEvidence 'original/idle-candidate0110-launch.json') -Raw | ConvertFrom-Json
$taskRoles=Get-Content -LiteralPath (Join-Path $taskEvidence 'original/candidate0110-settled-roles.original.json') -Raw | ConvertFrom-Json
$taskGpu=@($taskRoles.members | Where-Object role -CEQ 'gpu')
if($taskGpu.Count -ne 1 -or $taskRoles.rootPid -ne $taskRoot.pid -or
   $taskRoot.exeSha256 -cne 'f3fab9d6840c105e426a7dc90efe94fe839e283e47af3929d251d82a6e8675db'){throw 'Actual frozen GPU generation required'}
$taskAttempt=Join-Path $taskRun 'gpu-read-attempt.json'
if(Test-Path -LiteralPath $taskAttempt){throw 'One product metadata attempt only; inspect rather than retry'}
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');toolSource=$taskAdmission.source
    methodSha256=$taskAdmission.localMethodSha256;rootPid=$taskRoot.pid
    gpuPid=$taskGpu[0].pid;gpuCreationUtc=$taskGpu[0].creationUtc
    maximumMetadataReadMilliseconds=20000;automaticRetry=$false
    scope='late read-only diagnostic metadata outside completed performance spans'
    foregroundPermissionGranted=$false;newProductStartAllowed=$false
    fullPerformanceGateAccepted=$false} | ConvertTo-Json -Depth 6 |
    Set-Content -LiteralPath $taskAttempt -Encoding utf8
$taskGpuProcess=$null
try {
    $taskGpuProcess=[Diagnostics.Process]::GetProcessById([int]$taskGpu[0].pid)
    [void]$taskGpuProcess.SafeHandle
    if($taskGpuProcess.StartTime.ToUniversalTime().Ticks -ne ([datetimeoffset]$taskGpu[0].creationUtc).UtcTicks){throw 'Original GPU generation has ended'}
    $taskTarget=[pscustomobject]@{
        pid=$taskGpu[0].pid;creationTimeUtc=$taskGpu[0].creationUtc
        executable=$taskGpuProcess.MainModule.FileName
        exeSha256=(Get-FileHash -LiteralPath $taskGpuProcess.MainModule.FileName).Hash.ToLowerInvariant()
    }
    [ordered]@{toolSource=$taskAdmission.source;rootLaunch=$taskRoot;gpuLaunch=$taskTarget
        currentGpuProductVersion=(Get-Item -LiteralPath $taskTarget.executable).VersionInfo.ProductVersion
        currentSceneNotReobserved=$true;formalCounterSpanEndedAtUtc='2026-10-16T13:13:10.9767863+00:00'
        pointSnapshotCannotBackfillCounterSpanClassification=$true} |
        ConvertTo-Json -Depth 9 | Set-Content -LiteralPath (Join-Path $taskRun 'gpu-read-binding.json') -Encoding utf8
    . $taskMethod
    Get-WindowsBoundWorkingSetSnapshot -Launch $taskTarget -RootLaunch $taskRoot |
        ConvertTo-Json -Depth 15 | Set-Content -LiteralPath (Join-Path $taskRun 'gpu-working-set.original.json') -Encoding utf8
}catch{
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');exceptionType=$_.Exception.GetType().FullName
        message=$_.Exception.Message;hresult=$_.Exception.HResult;automaticRetry=$false
        newProductStarted=$false;uiInputInjected=$false;formalPerformanceGateAccepted=$false} |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRun 'gpu-working-set.failure.json') -Encoding utf8
    throw
}finally{if($taskGpuProcess){$taskGpuProcess.Dispose()}}
Get-Content -LiteralPath (Join-Path $taskRun 'gpu-working-set.original.json')

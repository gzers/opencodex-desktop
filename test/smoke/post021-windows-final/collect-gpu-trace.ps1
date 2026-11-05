param(
    [Parameter(Mandatory)][string]$TargetIdentityFile,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [string]$ForegroundReadyFile,
    [ValidateRange(5,45)][int]$DurationSeconds=30,
    [switch]$PlanOnly
)
$ErrorActionPreference='Stop'
$taskRepo=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../..'))
$taskWorkspace=Split-Path -Parent $taskRepo
$taskTools=Join-Path $taskWorkspace '.toolchain'
$taskEvidence=Join-Path $taskTools 'post021-msvc-followup'
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
$taskIdentityPath=[IO.Path]::GetFullPath($TargetIdentityFile)
function Assert-ChildPath([string]$Path,[string]$Boundary){
    if(!$Path.StartsWith($Boundary+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)){
        throw "Path is outside the explicit test boundary: $Path"
    }
    $taskCurrent=$Path
    while($taskCurrent){
        if(Test-Path -LiteralPath $taskCurrent){
            if((Get-Item -LiteralPath $taskCurrent -Force).Attributes -band [IO.FileAttributes]::ReparsePoint){
                throw "Reparse-point boundary is not admitted: $taskCurrent"
            }
        }
        $taskNext=Split-Path -Parent $taskCurrent
        if($taskNext -eq $taskCurrent){break}
        $taskCurrent=$taskNext
    }
}
Assert-ChildPath $taskOutput $taskEvidence
Assert-ChildPath $taskIdentityPath $taskEvidence
if($ForegroundReadyFile){
    $taskForegroundPath=[IO.Path]::GetFullPath($ForegroundReadyFile)
    Assert-ChildPath $taskForegroundPath $taskEvidence
    if((Split-Path -Parent $taskForegroundPath) -ine (Split-Path -Parent $taskIdentityPath)){throw 'Foreground heartbeat must share the exact target handoff directory'}
}
if(Test-Path -LiteralPath $taskOutput){throw 'Fresh evidence directory required; previous traces are immutable'}
$taskIdentity=Get-Content -LiteralPath $taskIdentityPath -Raw -Encoding utf8|ConvertFrom-Json
$taskExe=[IO.Path]::GetFullPath([string]$taskIdentity.exe)
Assert-ChildPath $taskExe $taskTools
$taskAllowedRoots=@(
    (Join-Path $taskTools 'post021-windows-final-msvc-41913003716'),
    (Join-Path $taskTools 'post021-windows-final-msvc-retry-41944607861')
)
if(!@($taskAllowedRoots|Where-Object {$taskExe.StartsWith($_+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)}).Count){
    throw 'Only the explicitly verified MSVC test roots are admitted'
}
if([IO.Path]::GetFileName($taskExe) -cne 'opencodex-desktop.exe'){throw 'Unexpected target executable'}
if([string]$taskIdentity.sha256 -notin @(
    'dd6f37cf2f6094cf9254ead08217cf8456e190cb213d3198c1b528172f1a0184',
    'fa9df83a55153f764673781e85167093d20602fd2e34742c69f6b2f5b37f9c68'
)){throw 'Exact reviewed MSVC executable hash required'}
if((Get-FileHash -LiteralPath $taskExe -Algorithm SHA256).Hash.ToLowerInvariant() -cne [string]$taskIdentity.sha256){
    throw 'Target binary hash changed'
}
function Read-OwnedTarget {
    $taskExpectedPid=[uint32]$taskIdentity.pid
    $taskP=Get-CimInstance Win32_Process -Filter ('ProcessId = '+$taskExpectedPid)
    if(!$taskP){throw 'The admitted test process is no longer running'}
    if($taskP.ExecutablePath -ine $taskExe -or
       [string]$taskP.CreationDate.ToUniversalTime().Ticks -cne [string]$taskIdentity.createdAtUtcTicks -or
       $taskP.SessionId -ne [uint32]$taskIdentity.sessionId){throw 'Target PID/path/creation/session changed'}
    $taskOwner=Invoke-CimMethod -InputObject $taskP -MethodName GetOwner
    if($taskOwner.ReturnValue -ne 0 -or $taskOwner.User -ine [string]$taskIdentity.ownerUser -or
       $taskOwner.Domain -ine [string]$taskIdentity.ownerDomain){throw 'Target process owner changed or unknown'}
    return [ordered]@{pid=$taskP.ProcessId;exe=$taskP.ExecutablePath;createdAtUtcTicks=[string]$taskP.CreationDate.ToUniversalTime().Ticks;sessionId=$taskP.SessionId;ownerDomain=$taskOwner.Domain;ownerUser=$taskOwner.User}
}
$taskBefore=Read-OwnedTarget
$taskProfile=Join-Path $PSScriptRoot 'gpu-dwm-scheduling.wprp'
if(!(Test-Path -LiteralPath $taskProfile)){throw 'Reviewed GPU scheduling profile is missing'}
$taskProfileHash=(Get-FileHash -LiteralPath $taskProfile -Algorithm SHA256).Hash.ToLowerInvariant()
. (Join-Path $PSScriptRoot 'gpu-foreground-admission.ps1')
$taskForeground=$null
if($ForegroundReadyFile -and !$PlanOnly){
    Write-Output 'Waiting up to120s for isolated OpenCodeX foreground. Switch now; recording starts only after3s stable foreground.'
    $taskForegroundDeadline=[datetimeoffset]::UtcNow.AddSeconds(120)
    while([datetimeoffset]::UtcNow -lt $taskForegroundDeadline){
        $taskCandidate=$null
        try{$taskCandidate=Get-Content -LiteralPath $taskForegroundPath -Raw -Encoding utf8|ConvertFrom-Json}catch{}
        if(Test-GpuForegroundHeartbeat $taskCandidate $taskIdentity ([datetimeoffset]::UtcNow)){
            $taskForeground=$taskCandidate;break
        }
        Start-Sleep -Milliseconds 200
    }
    if(!$taskForeground){throw 'No fresh stable native foreground heartbeat; no recording started'}
    $taskBefore=Read-OwnedTarget
}
$taskInstance='ocx-post021-gpu-'+[guid]::NewGuid().ToString('N')
$taskWpr=Join-Path ([Environment]::GetFolderPath('System')) 'wpr.exe'
if(!(Test-Path -LiteralPath $taskWpr)){throw 'Windows Performance Recorder is unavailable'}
New-Item -ItemType Directory -Path $taskOutput|Out-Null
function Write-Report([string]$Name,$Value){
    # PS5 Set-Content -Encoding utf8 emits a BOM. Use the same UTF-8 wire
    # format in PS5 and PS7 so the ordinary Node sampler can read it.
    [IO.File]::WriteAllText((Join-Path $taskOutput $Name),($Value|ConvertTo-Json -Depth 8),[Text.UTF8Encoding]::new($false))
}
function Invoke-Recorder([string[]]$Arguments,[string]$Name,[int]$WaitMilliseconds=20000){
    $taskStdout=Join-Path $taskOutput ($Name+'.stdout.log')
    $taskStderr=Join-Path $taskOutput ($Name+'.stderr.log')
    # File redirection survives an observation timeout; disposing an unread
    # pipe would lose the eventual merge result. Never kill a timed-out WPR.
    $taskP=Start-Process -FilePath $taskWpr -ArgumentList (($Arguments|ForEach-Object {'"'+$_+'"'}) -join ' ') -WindowStyle Hidden -PassThru -RedirectStandardOutput $taskStdout -RedirectStandardError $taskStderr
    # Keep the native handle open before a short-lived status command exits.
    # Windows PowerShell 5.1 can otherwise return a null ExitCode.
    $taskHandle=$taskP.Handle
    try{
        Write-Report ($Name+'.process.json') ([ordered]@{pid=$taskP.Id;exe=$taskWpr;createdAtUtcTicks=[string]$taskP.StartTime.ToUniversalTime().Ticks;arguments=$Arguments;observedAtUtc=[datetimeoffset]::UtcNow.ToString('o')})
        if(!$taskP.WaitForExit($WaitMilliseconds)){throw ('Recorder command still running after '+$WaitMilliseconds+'ms; inspect '+$Name+'.process.json and PID '+$taskP.Id+' before recovery. No automatic cancel or process kill.')}
        $taskP.WaitForExit()
        $taskOutText=[IO.File]::ReadAllText($taskStdout);$taskErrText=[IO.File]::ReadAllText($taskStderr)
        return [ordered]@{arguments=$Arguments;exitCode=$taskP.ExitCode;stdout=$taskOutText;stderr=$taskErrText}
    }finally{$taskP.Dispose()}
}
$taskReport=[ordered]@{
    result='failed';acceptance=$false;planOnly=[bool]$PlanOnly;targetBefore=$taskBefore
    targetExeSha256=$taskIdentity.sha256;durationSeconds=$DurationSeconds
    profiles=@('OcxGpuScheduling');profilePath=$taskProfile;profileSha256=$taskProfileHash
    traceScope='Five explicit event providers; no system collector or stacks. Machine-wide scheduling metadata stays local.'
    instanceName=$taskInstance
    rawTracePolicy='ETL stays local for review; do not publish machine-wide trace automatically'
    appProcessAction='none';dailyConfigAction='none';recorderStarted=$false;recorderStopped=$false
    foregroundAdmission=$taskForeground
}
$taskStarted=$false
try{
    $taskReport.statusBefore=Invoke-Recorder @('-status') 'status-before'
    $taskStatus=([string]$taskReport.statusBefore.stdout)+' '+([string]$taskReport.statusBefore.stderr)
    # Keep this file ASCII so Windows PowerShell 5.1 can parse UTF-8 without
    # a BOM. Regex Unicode escapes preserve localized status admission.
    if($taskReport.statusBefore.exitCode -ne 0 -or $taskStatus -notmatch '(?i)WPR is not recording|WPR \u672a\u5728\u8bb0\u5f55|WPR \u672a\u5f55\u5236'){
        throw 'An existing recorder or unknown recorder state is present; no start/stop/cancel issued'
    }
    # Instance name MUST be the last argument on every mutating command. A
    # concurrent administrator's default or other named recording is never ours.
    $taskReport.proposedStart=@('-start',($taskProfile+'!OcxGpuScheduling'),'-filemode','-recordtempto',$taskOutput,'-instancename',$taskInstance)
    $taskReport.recoveryStop=@($taskWpr,'-stop',(Join-Path $taskOutput 'gpu-dwm.etl'),'-instancename',$taskInstance)
    Write-Report 'collector-start-intent.json' $taskReport
    if($PlanOnly){$taskReport.result='plan-validated-no-recording';return}
    $taskPrincipal=[Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
    if(!$taskPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){
        throw 'Run the reviewed collector manually from an administrator shell; no elevation or policy change is performed by this script'
    }
    # Revalidate immediately before recorder mutation. The collector never
    # starts, stops, restarts, or kills the application or any other process.
    $taskReport.targetAtStart=Read-OwnedTarget
    $taskReport.start=Invoke-Recorder $taskReport.proposedStart 'start'
    if($taskReport.start.exitCode -ne 0){throw 'GPU/DWM profile admission failed; preserve logs; no policy adjustment attempted'}
    $taskStarted=$true;$taskReport.recorderStarted=$true
    $taskReport.startedAtUtc=[datetimeoffset]::UtcNow.ToString('o')
    Write-Report 'collector-started.json' $taskReport
    Write-Output ('GPU/DWM trace active for '+$DurationSeconds+'s. Keep the isolated test window foreground; run the planned scene in its existing session.')
    Start-Sleep -Seconds $DurationSeconds
    $taskReport.targetAtEnd=Read-OwnedTarget
    $taskReport.result='captured-diagnostic-only'
}catch{
    $taskReport.error=$_.Exception.Message
    throw
}finally{
    if($taskStarted){
        $taskTrace=Join-Path $taskOutput 'gpu-dwm.etl'
        try{
            $taskReport.stop=Invoke-Recorder @('-stop',$taskTrace,'-instancename',$taskInstance) 'stop' 60000
            $taskReport.recorderStopped=($taskReport.stop.exitCode -eq 0)
            if(!$taskReport.recorderStopped){$taskReport.result='failed';$taskReport.stopError='Owned recording did not confirm stop; manual inspection required, no automatic cancel'}
            if(Test-Path -LiteralPath $taskTrace){
                $taskReport.etlBytes=(Get-Item -LiteralPath $taskTrace).Length
                $taskReport.etlSha256=(Get-FileHash -LiteralPath $taskTrace -Algorithm SHA256).Hash.ToLowerInvariant()
            }
        }catch{$taskReport.result='failed';$taskReport.stopError=$_.Exception.Message}
    }
    $taskReport.finishedAtUtc=[datetimeoffset]::UtcNow.ToString('o')
    Write-Report 'collector-result.json' $taskReport
}
if($taskReport.result -eq 'failed'){
    throw 'Collector did not confirm a complete owned recording; inspect collector-result.json and its named recovery command'
}

param([switch]$RecordApproved)
$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop'
$taskSource='7704a17902c98ceed047cc3dec2a361937b5e1f7'
$taskPrepRoot=Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261016-WIN-QUERY-THREAD-PREP'
$taskPrep=Get-Content -LiteralPath (Join-Path $taskPrepRoot 'preparation.original.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskRuntime=Join-Path $taskCode 'test/runtime/bd2-named-query-once-20261016'
$taskPlan=[ordered]@{
    collectorSource=$taskSource;testSource='bd2f6eb10d58115235e35d575507bb69f44666da'
    originalFailedCiRun=38537166560;testExeSha256=$taskPrep.testSha256;nodeSha256=$taskPrep.nodeSha256
    originalLibraryMaximumRuns=1;defaultConcurrency=$true;prewarm=$false
    originalDeadlinesSeconds=@(2,4,5,8);outerSeconds=120;maximumSerializedMiB=64
    maximumQueueRecords=32768;maximumTrackedIrps=32768;queryNameHistoryMaximum=512
    hotScope='owned descendants and seven exact named root query test threads'
    userAuthorization='批准修正版后续都可以管理员后台 ETW 录制'
    productStarted=$false;failureOrRootCausePreclaimed=$false
    outputRoot=$taskRuntime;automaticRecordingRetryAllowed=$false
}
if(!$RecordApproved){$taskPlan | ConvertTo-Json -Depth 5;return}
$taskIdentity=[Security.Principal.WindowsIdentity]::GetCurrent()
$taskPrincipal=[Security.Principal.WindowsPrincipal]::new($taskIdentity)
if(!$taskPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){
    throw 'Recording needs a real administrator token; this entry does not elevate itself'
}
if((& git -C $taskCode rev-parse HEAD).Trim() -cne $taskSource){throw 'Collector source changed'}
& git -C $taskCode diff --quiet HEAD -- test/smoke/windows-owned-etw.cs test/smoke/windows-owned-etw.ps1 test/smoke/windows-owned-etw.test.ps1
if($LASTEXITCODE -ne 0){throw 'Prepared collector changed'}
$taskAdmissionPath=Join-Path $PSScriptRoot 'ci-admission.json'
if(!(Test-Path -LiteralPath $taskAdmissionPath)){throw 'Exact-source complete CI admission not yet recorded'}
$taskAdmission=Get-Content -LiteralPath $taskAdmissionPath -Raw -Encoding utf8 | ConvertFrom-Json
$taskTree=(& git -C $taskCode rev-parse 'HEAD^{tree}').Trim()
if($taskAdmission.source -cne $taskSource -or $taskAdmission.tree -cne $taskTree -or
   $taskAdmission.runId -ne 38826901050 -or $taskAdmission.runAttempt -ne 1 -or
   !$taskAdmission.wholeCiSucceeded -or !$taskAdmission.originalCheckoutAndJobsVerified){
    throw 'Recording CI admission does not match the collector source'
}
$taskChecks=Get-Content -LiteralPath (Join-Path $taskPrepRoot 'pure-checks-final.original.log') -Raw -Encoding utf8 | ConvertFrom-Json
if($taskChecks.namedQueryThreadScopeChecks -ne 39 -or $taskChecks.etwSessionStarted -or $taskChecks.testOrNodeStarted){
    throw 'Prepared scope checks differ'
}
if((Test-Path -LiteralPath $taskRuntime) -or (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'recording-attempt.json'))){
    throw 'This one-shot recording already has output; inspect it without a new run'
}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){
    throw 'Native application exists; do not interfere'
}
$taskDaily='D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe'
$taskTask='D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\work\objects\task\TASK-OPENCODEX-DESKTOP-182.json'
$taskState=[ordered]@{
    startedAtUtc=[datetimeoffset]::UtcNow.ToString('o');controllerPid=$PID
    controllerCreatedUtc=(Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o')
    controllerIsAdministrator=$true;authorization=$taskPlan.userAuthorization;plan=$taskPlan
    ciAdmissionSha256=(Get-FileHash -LiteralPath $taskAdmissionPath).Hash.ToLowerInvariant()
    dailyBefore=(Get-FileHash -LiteralPath $taskDaily).Hash.ToLowerInvariant()
    taskBefore=(Get-FileHash -LiteralPath $taskTask).Hash.ToLowerInvariant()
    recordEntryReturned=$false;historicalFailureCleared=$false
}
$taskState | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'recording-attempt.json') -Encoding utf8
try{
    $taskResult=& (Join-Path $taskCode 'test/smoke/windows-owned-etw.ps1') `
        -DecoderDirectory $taskPrep.decoderDirectory -TestExecutable $taskPrep.testExecutable `
        -ExpectedTestSha256 $taskPrep.testSha256 -Node $taskPrep.node `
        -ExpectedNodeSha256 $taskPrep.nodeSha256 -OutputRoot $taskRuntime -RecordApproved
    $taskState.result=$taskResult | ConvertFrom-Json
    $taskState.recordEntryReturned=$true
}catch{
    $taskState.errorType=$_.Exception.GetType().FullName
    $_ | Out-String | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'recording-error.original.txt') -Encoding utf8
    throw
}finally{
    $taskState.finishedAtUtc=[datetimeoffset]::UtcNow.ToString('o')
    $taskState.dailyAfter=(Get-FileHash -LiteralPath $taskDaily).Hash.ToLowerInvariant()
    $taskState.taskAfter=(Get-FileHash -LiteralPath $taskTask).Hash.ToLowerInvariant()
    $taskState | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'recording-result.json') -Encoding utf8
    $taskIdentity.Dispose()
}
$taskState | ConvertTo-Json -Depth 10

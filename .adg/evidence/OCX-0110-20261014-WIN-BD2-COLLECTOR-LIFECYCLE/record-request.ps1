param([switch]$ApprovedByUser)
$ErrorActionPreference = 'Stop'
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskCollectorSource = 'abc3a9140fc6f1b5d2aa6e739299af260f0f5d94'
$taskPlan = [ordered]@{
    mode='review-only-unless-explicitly-approved';testSource='bd2f6eb10d58115235e35d575507bb69f44666da'
    collectorSource=$taskCollectorSource;originalFailedCiRun=38537166560
    originalFailedPrCheckout='31e7748d9e82c79bef97fc1b73233f08bc1f1b3d'
    testBinarySha256='497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3'
    nodeVersion='24.21.0';nodeSha256='ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32'
    originalLibraryRuns=1;testDeadlineSeconds=120;maximumSerializedMiB=64
    originalQuerySeconds=2;originalCleanupSeconds=4;originalObserverSeconds=5;originalWatchdogSeconds=8
    defaultConcurrency=$true;originalCapture=$true;prewarm=$false;desktopStarted=$false
    requiresElevatedController=$true;selfElevationPerformed=$false
    kernelEventsTransientlyDecodedInMemory=$true;wholeMachineEtlWritten=$false
    onlyOwnedFamilyRecordsPersisted=$true;counterpartySchedulingIdsPersisted=$false
    commandLinesOrSidsPersisted=$false;networkOrRegistryProviderEnabled=$false
    unrelatedSystemSessionRestarted=$false;defenseOrGlobalPathChanged=$false
    kernelEnabledBeforeSourceGetter=$true;traceEventGenericStopInvoked=$false
    nativeStopOnlyUniqueNamedSession=$true;globalCpuSampleIntervalWritten=$false
    preparationScopeChecks=38;preparationLifecycleChecks=7;preparationNativeStopChecks=8
    evidenceUploadOrExternalMessageSent=$false;failureOrRootCausePreclaimed=$false
    outputRoot=Join-Path $taskCode 'test/runtime/bd2-etw-approved-run-20261014'
}
if (!$ApprovedByUser) {
    $taskPlan | ConvertTo-Json -Depth 5
    return
}
$taskPrincipal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
if (!$taskPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'Explicit approval also requires a manually opened elevated PowerShell 7; this script never self-elevates'
}
$taskActualSource = (& git -C $taskCode rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $taskActualSource -cne $taskCollectorSource) {
    throw 'Collector checkout changed; re-review the command rather than switching it automatically'
}
& git -C $taskCode diff --quiet HEAD -- test/smoke/windows-owned-etw.cs test/smoke/windows-owned-etw.ps1 test/smoke/windows-owned-etw.test.ps1
if ($LASTEXITCODE -ne 0) { throw 'Prepared collector has local changes; re-review before elevated execution' }
$taskPackage = Get-Content -LiteralPath (Join-Path $PSScriptRoot '../OCX-0110-20261014-WIN-BD2-CAPTURE-PREPARATION/traceevent-package-provenance.json') -Raw | ConvertFrom-Json
foreach ($taskFile in $taskPackage.decoderFiles) {
    if ((Get-FileHash -LiteralPath (Join-Path $taskPackage.runtime $taskFile.file)).Hash.ToLower() -cne $taskFile.sha256) {
        throw 'Prepared decoder bytes changed'
    }
}
if (Test-Path -LiteralPath $taskPlan.outputRoot) { throw 'This one-shot recording already has output; inspect it before any new action' }
if (@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'").Count) {
    throw 'Do not interfere with a native desktop/performance instance'
}
$taskProtectedBefore = @{}
foreach ($taskName in @('preferences.json','data-root.json','backup-cleanup-policy.json','update-schedule.json')) {
    $taskProtectedBefore[$taskName] = (Get-FileHash -LiteralPath (Join-Path $taskCode "test/runtime/windows0110/data/manager-state/$taskName")).Hash.ToLower()
}
$taskDailyBefore = (Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLower()
$taskRecord = [ordered]@{
    authorization='explicit -ApprovedByUser after reviewing the plan'
    startedAtUtc=[DateTime]::UtcNow.ToString('o');plan=$taskPlan
    protectedBefore=$taskProtectedBefore;dailyExeBefore=$taskDailyBefore
}
try {
    $taskOutput = & (Join-Path $taskCode 'test/smoke/windows-owned-etw.ps1') `
        -DecoderDirectory $taskPackage.runtime `
        -TestExecutable (Join-Path $taskCode 'test/software/diagnostic-bd2-pr-failure-38537166560/opencodex_desktop_lib-4ef9ce48b814b037.exe') `
        -ExpectedTestSha256 $taskPlan.testBinarySha256 `
        -Node (Join-Path $taskCode 'test/software/node-v24.21.0-official/node.exe') `
        -ExpectedNodeSha256 $taskPlan.nodeSha256 `
        -OutputRoot $taskPlan.outputRoot -RecordApproved
    $taskRecord.result = $taskOutput | ConvertFrom-Json
} catch {
    $taskRecord.error = $_.Exception.Message
    throw
} finally {
    $taskAfter = @{}
    foreach ($taskName in $taskProtectedBefore.Keys) {
        $taskAfter[$taskName] = (Get-FileHash -LiteralPath (Join-Path $taskCode "test/runtime/windows0110/data/manager-state/$taskName")).Hash.ToLower()
    }
    $taskRecord.protectedAfter = $taskAfter
    $taskRecord.dailyExeAfter = (Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLower()
    $taskRecord.finishedAtUtc = [DateTime]::UtcNow.ToString('o')
    if (Test-Path -LiteralPath $taskPlan.outputRoot) {
        # Results remain runtime output. Frozen preparation evidence is never overwritten.
        $taskRecord | ConvertTo-Json -Depth 12 |
            Set-Content -LiteralPath (Join-Path $taskPlan.outputRoot 'invocation-result.json') -Encoding utf8
    }
}
$taskRecord | ConvertTo-Json -Depth 10

$ErrorActionPreference = 'Stop'
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskEvidence = $PSScriptRoot
$taskAdmission = Get-Content -LiteralPath (Join-Path $taskEvidence 'test-artifact-admission.json') -Raw | ConvertFrom-Json
$taskNodeAdmission = Get-Content -LiteralPath (Join-Path $taskEvidence 'official-node-admission.json') -Raw | ConvertFrom-Json
$taskBinary = $taskAdmission.testExecutable
$taskNode = $taskNodeAdmission.path
if ((Get-FileHash -LiteralPath $taskBinary).Hash.ToLower() -cne $taskAdmission.testExecutableSha256) { throw 'Test binary hash changed' }
if ((Get-FileHash -LiteralPath $taskNode).Hash.ToLower() -cne $taskNodeAdmission.exeSha256) { throw 'Node binary hash changed' }
if (@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'").Count) { throw 'Native application exists; do not interfere with its sampling' }
$taskRuntime = Join-Path $taskCode 'test/runtime/bd2-owned-trace-20261014'
if (Test-Path -LiteralPath $taskRuntime) { throw 'This is a one-shot diagnostic, not a retry-to-green loop' }
New-Item -ItemType Directory -Path $taskRuntime,(Join-Path $taskRuntime 'temp'),(Join-Path $taskRuntime 'home') | Out-Null
$taskBefore = @{}
foreach ($taskName in @('preferences.json','data-root.json','backup-cleanup-policy.json','update-schedule.json')) {
    $taskBefore[$taskName] = (Get-FileHash -LiteralPath (Join-Path $taskCode "test/runtime/windows0110/data/manager-state/$taskName")).Hash.ToLower()
}
$taskDailyHash = (Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLower()
# Compile the observer before the tested processes exist; this does not launch
# or warm Node. No changes are made to the original test binary or fixture.
Add-Type -Path (Join-Path $taskEvidence 'OwnedProcessObserver.cs')
$taskPsi = [Diagnostics.ProcessStartInfo]::new()
$taskPsi.FileName = $taskBinary
$taskPsi.WorkingDirectory = Join-Path $taskCode 'apps/desktop/tauri'
$taskPsi.UseShellExecute = $false
$taskPsi.CreateNoWindow = $true
$taskPsi.RedirectStandardOutput = $true
$taskPsi.RedirectStandardError = $true
$taskPsi.Environment['OPENCODEX_TEST_NODE'] = $taskNode
$taskPsi.Environment['TEMP'] = Join-Path $taskRuntime 'temp'
$taskPsi.Environment['TMP'] = Join-Path $taskRuntime 'temp'
$taskPsi.Environment['HOME'] = Join-Path $taskRuntime 'home'
$taskPsi.Environment['USERPROFILE'] = Join-Path $taskRuntime 'home'
foreach ($taskRemove in @('RUST_TEST_THREADS','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY','OPENCODEX_WINDOWS_SMOKE_CDP_PORT')) {
    [void]$taskPsi.Environment.Remove($taskRemove)
}
# Default Rust libtest concurrency and capture are preserved. No filter,
# --test-threads, nocapture, timeout extension, Node prewarm, or production change.
$taskClock = [Diagnostics.Stopwatch]::StartNew()
$taskProcess = $null
$taskObserver = $null
$taskForced = $false
$taskRecord = [ordered]@{
    source=$taskAdmission.source;testBinary=$taskBinary;testBinarySha256=$taskAdmission.testExecutableSha256
    node=$taskNode;nodeSha256=$taskNodeAdmission.exeSha256;nodeVersion=$taskNodeAdmission.version
    atUtc=[DateTime]::UtcNow.ToString('o');arguments=@();defaultConcurrencyPreserved=$true
    testCapturePreserved=$true;nodePrewarmed=$false;localNativeBefore=0;maximumOuterDiagnosticSeconds=120
    sampleIntervalAfterEachReadMs=50;threadSnapshotsNotSchedulingEtw=$true
    protectedSandboxBefore=$taskBefore;dailyExeBefore=$taskDailyHash;status='preparing'
}
try {
    $taskProcess = [Diagnostics.Process]::Start($taskPsi)
    $taskRecord.testPid = $taskProcess.Id
    $taskRecord.testCreationUtc = $taskProcess.StartTime.ToUniversalTime().ToString('o')
    $taskRecord.rootHandleRetained = $true
    $taskStdout = $taskProcess.StandardOutput.ReadToEndAsync()
    $taskStderr = $taskProcess.StandardError.ReadToEndAsync()
    $taskObserver = [OwnedProcessObserver]::new(
        $taskProcess.Id, $taskBinary, $taskNode, (Join-Path $taskRuntime 'temp'),
        (Join-Path $taskEvidence 'owned-process-thread-io.raw.jsonl'))
    $taskRecord.observerInitializedAtMs = $taskClock.Elapsed.TotalMilliseconds
    $taskRecord.status = 'observing'
    while (!$taskProcess.HasExited -and $taskClock.Elapsed.TotalSeconds -lt 120) {
        $taskObserver.Sample()
        Start-Sleep -Milliseconds 50
    }
    if (!$taskProcess.HasExited) {
        $taskForced = $true
        $taskProcess.Kill($true)
        [void]$taskProcess.WaitForExit(10000)
    }
    $taskObserver.Sample()
    $taskRecord.testExitCode = $taskProcess.ExitCode
    $taskRecord.elapsedSeconds = $taskClock.Elapsed.TotalSeconds
    $taskRecord.samples = $taskObserver.Samples
    $taskRecord.processesObserved = $taskObserver.ProcessesObserved
    $taskRecord.maximumSampleMs = $taskObserver.MaximumSampleMs
    $taskRecord.fileWatcherBufferOverflow = $taskObserver.FileBufferOverflow
    if ($taskStdout.Wait(5000)) { [IO.File]::WriteAllText((Join-Path $taskEvidence 'test-stdout.original.log'),$taskStdout.Result,[Text.UTF8Encoding]::new($false)) }
    if ($taskStderr.Wait(5000)) { [IO.File]::WriteAllText((Join-Path $taskEvidence 'test-stderr.original.log'),$taskStderr.Result,[Text.UTF8Encoding]::new($false)) }
    $taskRecord.status = 'single-original-run-ended'
} catch {
    $taskRecord.status = 'diagnostic-preparation-or-observation-error'
    $taskRecord.error = $_.Exception.Message
    throw
} finally {
    if ($taskObserver) { $taskObserver.Dispose() }
    if ($taskProcess) {
        if (!$taskProcess.HasExited) { $taskForced=$true;$taskProcess.Kill($true);[void]$taskProcess.WaitForExit(10000) }
        $taskProcess.Dispose()
    }
    $taskRecord.forcedOuterCleanup = $taskForced
    $taskRecord.finishedAtUtc = [DateTime]::UtcNow.ToString('o')
    $taskRecord.desktopStarted = $false
    $taskRecord.wprOrSchedulingEtwRecorded = $false
    $taskRecord.finalNativeCount = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'").Count
    $taskAfter = @{}
    foreach ($taskName in $taskBefore.Keys) {
        $taskAfter[$taskName] = (Get-FileHash -LiteralPath (Join-Path $taskCode "test/runtime/windows0110/data/manager-state/$taskName")).Hash.ToLower()
    }
    $taskRecord.protectedSandboxAfter = $taskAfter
    $taskRecord.dailyExeAfter = (Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLower()
    $taskRecord | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $taskEvidence 'single-run-result.json') -Encoding utf8
}
$taskRecord | ConvertTo-Json -Depth 5 -Compress

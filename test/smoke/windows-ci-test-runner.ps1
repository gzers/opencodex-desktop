param(
    [Parameter(Mandatory=$true,Position=0)][string]$Executable,
    [Parameter(Position=1,ValueFromRemainingArguments=$true)][string[]]$TestArguments
)
$ErrorActionPreference = 'Stop'
if (!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7 -or
    $env:GITHUB_ACTIONS -cne 'true' -or $env:GITHUB_REPOSITORY -cne 'gzers/opencodex-desktop' -or
    $env:GITHUB_EVENT_NAME -cnotin @('push','pull_request') -or $env:GITHUB_RUN_ATTEMPT -cne '1') {
    throw 'Cargo diagnostic runner requires one ordinary ephemeral CI attempt'
}
$taskRepo = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../..')).Path
$taskDeps = [IO.Path]::GetFullPath((Join-Path $taskRepo 'apps/desktop/tauri/target/debug/deps'))
$taskExe = (Resolve-Path -LiteralPath $Executable).Path
if (!(Split-Path -Parent $taskExe).Equals($taskDeps,[StringComparison]::OrdinalIgnoreCase) -or
    [IO.Path]::GetExtension($taskExe) -cne '.exe') { throw 'Cargo target is outside the admitted test directory' }
foreach ($taskPath in @((Get-Item -LiteralPath $taskExe),(Get-Item -LiteralPath $taskDeps))) {
    if ($taskPath.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Test target aliases are refused' }
}
if ((Split-Path -Leaf $taskExe) -cnotmatch '^opencodex_desktop_lib-[0-9a-f]+\.exe$') {
    # Other workspace suites run once through Cargo with their unchanged args.
    & $taskExe @TestArguments
    exit $LASTEXITCODE
}
if ($TestArguments.Count -ne 1 -or $TestArguments[0] -cne '--show-output') {
    throw 'Keep original libtest output capture, default concurrency and unfiltered arguments'
}
$taskAdmission = Get-Content -LiteralPath (Join-Path $taskRepo 'test/out/windows-query-etw/admission.json') -Raw -Encoding utf8 | ConvertFrom-Json
if (!$taskAdmission.firstOrdinaryCiRegression -or
    $taskAdmission.collectorSource -cne (& git -C $taskRepo rev-parse HEAD).Trim() -or
    $taskAdmission.applicationTree -cne (& git -C $taskRepo rev-parse HEAD:apps/desktop).Trim()) {
    throw 'First-regression source admission changed'
}
$taskAttempt = Join-Path $taskRepo 'test/out/windows-query-etw/first-regression-attempt.json'
if (Test-Path -LiteralPath $taskAttempt) { throw 'Never execute the library twice to obtain a passing or failing capture' }
$taskHash = (Get-FileHash -LiteralPath $taskExe).Hash.ToLowerInvariant()
$taskNodeSignature = Get-AuthenticodeSignature -LiteralPath $taskAdmission.node
if ($taskNodeSignature.Status -ne 'Valid' -or $taskNodeSignature.SignerCertificate.Subject -notlike '*OpenJS*') {
    throw 'Pinned official Node signature is invalid'
}
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskAdmission.collectorSource
    executable=$taskExe;sha256=$taskHash;bytes=(Get-Item -LiteralPath $taskExe).Length
    originalFailureArtifactSha256=$taskAdmission.testSha256
    executableIsOriginalFailedArtifact=($taskHash -ceq $taskAdmission.testSha256)
    provenance='Current ordinary CI build; not relabelled as historical failed binary'
    arguments=$TestArguments;libraryExecutionsAllowed=1;nodeSha256=$taskAdmission.nodeSha256
    captureOverheadIsNotPerformanceAcceptance=$true
} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $taskAttempt -Encoding utf8
$taskOutput = & (Join-Path $PSScriptRoot 'windows-owned-etw.ps1') `
    -DecoderDirectory $taskAdmission.decoder -TestExecutable $taskExe -ExpectedTestSha256 $taskHash `
    -Node $taskAdmission.node -ExpectedNodeSha256 $taskAdmission.nodeSha256 `
    -OutputRoot $taskAdmission.outputRoot -RecordApproved -RunnerRegressionEnvironment -FirstCargoRegression |
    ConvertFrom-Json
$taskCapture = Join-Path $taskAdmission.outputRoot 'capture'
[Console]::Out.Write([IO.File]::ReadAllText((Join-Path $taskCapture 'test-stdout.original.log')))
[Console]::Error.Write([IO.File]::ReadAllText((Join-Path $taskCapture 'test-stderr.original.log')))
if (!$taskOutput.originalLibraryCompleted -or !$taskOutput.ownedRootExited -or
    !$taskOutput.rootProcessStartSeen -or !$taskOutput.processTraceJoined -or !$taskOutput.writerJoined -or
    !$taskOutput.ownedSessionStopSucceeded -or !$taskOutput.serializedBytesWithinLimit -or
    $taskOutput.droppedRecords -ne 0 -or $taskOutput.kernelEventsLost -ne 0 -or
    $taskOutput.decoderCallbackErrors -ne 0 -or $taskOutput.knownImageCoverageGap -or
    $taskOutput.nativeOwnedSessionStop.Status -ne 0 -or
    $taskOutput.nativeOwnedSessionStop.FinalEventsLost -ne 0 -or
    $taskOutput.nativeOwnedSessionStop.FinalRealTimeBuffersLost -ne 0 -or
    $taskOutput.captureError -or $taskOutput.runError -or $taskOutput.writerError -or
    $taskOutput.cleanupErrors.Count -ne 0) {
    throw 'Original result preserved; first-regression capture incomplete, no automatic retry'
}
exit [int]$taskOutput.testExitCode

param(
    [Parameter(Mandatory=$true)][string]$DecoderDirectory,
    [Parameter(Mandatory=$true)][string]$TestExecutable,
    [Parameter(Mandatory=$true)][string]$ExpectedTestSha256,
    [Parameter(Mandatory=$true)][string]$Node,
    [Parameter(Mandatory=$true)][string]$ExpectedNodeSha256,
    [Parameter(Mandatory=$true)][string]$OutputRoot
)
$ErrorActionPreference = 'Stop'
$taskScript = Join-Path $PSScriptRoot 'windows-owned-etw.ps1'
$taskArguments = @{
    DecoderDirectory=$DecoderDirectory;TestExecutable=$TestExecutable
    ExpectedTestSha256=$ExpectedTestSha256;Node=$Node
    ExpectedNodeSha256=$ExpectedNodeSha256;OutputRoot=$OutputRoot
}
$taskResult = & $taskScript @taskArguments | ConvertFrom-Json
if ($taskResult.status -ne 'prepared-only' -or $taskResult.pureScopeChecks -ne 38 -or
    $taskResult.pureLifecycleChecks -ne 7 -or
    $taskResult.pureNativeStopChecks -ne 8 -or
    $taskResult.controllerProcessImageChecks -ne 5 -or
    $taskResult.descendantHotScopeChecks -ne 10 -or $taskResult.utf8LineBudgetChecks -ne 10 -or
    $taskResult.executableAdmissionChecks -ne 13 -or
    $taskResult.runnerEnvironmentChecks -ne 10 -or
    $taskResult.regressionEventChecks -ne 7 -or
    $taskResult.etwSessionStarted -or $taskResult.testOrNodeStarted -or $taskResult.recordingApproved) {
    throw 'Default invocation must prepare and exercise pure scope contracts only'
}
function Assert-GuardRejected([hashtable]$Arguments,[string]$Message) {
    $taskRejected = $false
    try { & $taskScript @Arguments | Out-Null } catch {
        if (!$_.Exception.Message.Contains($Message)) { throw }
        $taskRejected = $true
    }
    if (!$taskRejected) { throw "Expected guard: $Message" }
}
$taskWrongHash = $taskArguments.Clone()
$taskWrongHash.ExpectedTestSha256 = '0' * 64
Assert-GuardRejected $taskWrongHash 'Test EXE digest mismatch'
$taskWrongScope = $taskArguments.Clone()
$taskWrongScope.OutputRoot = Join-Path ([IO.Path]::GetTempPath()) 'opencodex-outside-workspace'
Assert-GuardRejected $taskWrongScope 'All recording output must remain inside this task runtime'
$taskFirstRegressionArguments = $taskArguments.Clone()
$taskFirstRegressionArguments.FirstCargoRegression = $true
Assert-GuardRejected $taskFirstRegressionArguments 'First cargo regression capture is ordinary CI-only'
$taskRunnerGuard = 'not-applicable-on-dispatched-runner'
if ($env:GITHUB_EVENT_NAME -cne 'workflow_dispatch' -or $env:GITHUB_ACTIONS -cne 'true' -or
    $env:GITHUB_REPOSITORY -cne 'gzers/opencodex-desktop' -or $env:GITHUB_RUN_ATTEMPT -cne '1') {
    $taskRunnerArguments = $taskArguments.Clone()
    $taskRunnerArguments.RunnerRegressionEnvironment = $true
    Assert-GuardRejected $taskRunnerArguments 'Runner environment capture is CI-dispatch-only'
    $taskRunnerGuard = 'passed'
}
$taskOrdinaryGuard = 'not-applicable-on-elevated-controller'
if (!$taskResult.controllerElevated) {
    $taskRejected = $false
    try {
        [OpenCodex.Diagnostics.OwnedEtwRecorder]::RunOnce(
            $TestExecutable,$Node,(Split-Path -Parent $TestExecutable),
            (Join-Path $OutputRoot 'capture'),(Join-Path $OutputRoot 'temp'),
            (Join-Path $OutputRoot 'home'),('OpenCodex-BD2-Guard-' + [guid]::NewGuid().ToString('N'))) |
            Out-Null
    } catch {
        if (!$_.Exception.ToString().Contains('Kernel ETW capture requires an approved elevated controller')) { throw }
        $taskRejected = $true
    }
    if (!$taskRejected) { throw 'Ordinary token must fail before session/process creation' }
    $taskOrdinaryGuard = 'passed'
}
if (Test-Path -LiteralPath $OutputRoot) { throw 'Preparation or negative guards created a recording root' }
[ordered]@{
    status='passed';pureScopeChecks=38;pureLifecycleChecks=7;pureNativeStopChecks=8
    controllerProcessImageChecks=5
    descendantHotScopeChecks=10;utf8LineBudgetChecks=10
    executableAdmissionChecks=13
    runnerEnvironmentChecks=10
    regressionEventChecks=7
    digestGuard='passed';outputScopeGuard='passed'
    runnerEnvironmentHostAndEventGuard=$taskRunnerGuard
    ordinaryPrivilegeGuard=$taskOrdinaryGuard
    etwSessionStarted=$false;testOrNodeStarted=$false;recordingRootCreated=$false
    diagnosticRecordExecutionTested=$false
} | ConvertTo-Json -Depth 4

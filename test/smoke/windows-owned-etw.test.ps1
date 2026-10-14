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
    digestGuard='passed';outputScopeGuard='passed'
    ordinaryPrivilegeGuard=$taskOrdinaryGuard
    etwSessionStarted=$false;testOrNodeStarted=$false;recordingRootCreated=$false
    diagnosticRecordExecutionTested=$false
} | ConvertTo-Json -Depth 4

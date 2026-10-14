param(
    [Parameter(Mandatory=$true)][string]$DecoderDirectory,
    [Parameter(Mandatory=$true)][string]$TestExecutable,
    [Parameter(Mandatory=$true)][string]$ExpectedTestSha256,
    [Parameter(Mandatory=$true)][string]$Node,
    [Parameter(Mandatory=$true)][string]$ExpectedNodeSha256,
    [Parameter(Mandatory=$true)][string]$OutputRoot,
    [switch]$RecordApproved
)
$ErrorActionPreference = 'Stop'
if (!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7) { throw 'Windows PowerShell 7 required' }
if ([IntPtr]::Size -ne 8) { throw 'Admitted Windows x64 binaries require a 64-bit controller' }
$taskRepo = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../..')).Path
$taskSoftware = (Resolve-Path -LiteralPath (Join-Path $taskRepo 'test/software')).Path
$taskRuntime = (Resolve-Path -LiteralPath (Join-Path $taskRepo 'test/runtime')).Path
$taskExe = (Resolve-Path -LiteralPath $TestExecutable).Path
$taskNode = (Resolve-Path -LiteralPath $Node).Path
$taskOutput = [IO.Path]::GetFullPath($OutputRoot)
foreach ($taskPath in @($taskExe,$taskNode)) {
    if (!$taskPath.StartsWith($taskSoftware + '\',[StringComparison]::OrdinalIgnoreCase)) {
        throw 'Test EXE and Node must be explicit, task-owned, admitted binaries'
    }
}
if (!$taskOutput.StartsWith($taskRuntime + '\',[StringComparison]::OrdinalIgnoreCase)) {
    throw 'All recording output must remain inside this task runtime'
}
if ($ExpectedTestSha256 -notmatch '^[0-9a-f]{64}$' -or
    (Get-FileHash -LiteralPath $taskExe).Hash.ToLower() -cne $ExpectedTestSha256) { throw 'Test EXE digest mismatch' }
if ($ExpectedNodeSha256 -notmatch '^[0-9a-f]{64}$' -or
    (Get-FileHash -LiteralPath $taskNode).Hash.ToLower() -cne $ExpectedNodeSha256) { throw 'Node digest mismatch' }
if (@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'").Count) {
    throw 'Native application exists; do not interfere with its performance sampling'
}
$taskDecoder = (Resolve-Path -LiteralPath $DecoderDirectory).Path
if (!$taskDecoder.StartsWith($taskRuntime + '\',[StringComparison]::OrdinalIgnoreCase)) {
    throw 'Decoder must remain in task-local prepared runtime'
}
$taskDecoderFiles = @(
    (Join-Path $taskDecoder 'Microsoft.Diagnostics.FastSerialization.dll'),
    (Join-Path $taskDecoder 'Microsoft.Diagnostics.Tracing.TraceEvent.dll')
)
foreach ($taskFile in $taskDecoderFiles) { [void][Reflection.Assembly]::LoadFrom($taskFile) }
$taskReferences = @((Get-ChildItem -LiteralPath (Join-Path $PSHOME 'ref') -Filter '*.dll').FullName) + $taskDecoderFiles
Add-Type -Path (Join-Path $PSScriptRoot 'windows-owned-etw.cs') -ReferencedAssemblies $taskReferences
$taskScopeChecks = [OpenCodex.Diagnostics.OwnedEtwScope]::SelfTest()
$taskLifecycleChecks = [OpenCodex.Diagnostics.OwnedEtwLifecycle]::SelfTest()
$taskNativeStopChecks = [OpenCodex.Diagnostics.OwnedEtwNativeStop]::SelfTest()
$taskProcessImageChecks = [OpenCodex.Diagnostics.OwnedEtwRecorder]::SelfTestProcessImage()
if (!$RecordApproved) {
    [ordered]@{
        status='prepared-only';pureScopeChecks=$taskScopeChecks
        pureLifecycleChecks=$taskLifecycleChecks
        pureNativeStopChecks=$taskNativeStopChecks
        controllerProcessImageChecks=$taskProcessImageChecks
        testSha256=$ExpectedTestSha256;nodeSha256=$ExpectedNodeSha256
        compiler='PowerShell Add-Type';controllerElevated=[Microsoft.Diagnostics.Tracing.Session.TraceEventSession]::IsElevated()
        etwSessionStarted=$false;testOrNodeStarted=$false;recordingApproved=$false
        nextAction='Only after explicit approval, invoke this same command with -RecordApproved from an elevated PowerShell 7.'
    } | ConvertTo-Json -Depth 4
    return
}
if ([Microsoft.Diagnostics.Tracing.Session.TraceEventSession]::IsElevated() -ne $true) {
    throw 'Recording requires an explicitly approved elevated controller; no self-elevation is performed'
}
if (Test-Path -LiteralPath $taskOutput) { throw 'Recording root already exists; no repeat-to-green or overwrite' }
$taskSession = 'OpenCodex-BD2-' + [guid]::NewGuid().ToString('N')
$taskCapture = Join-Path $taskOutput 'capture'
$taskTemp = Join-Path $taskOutput 'temp'
$taskHome = Join-Path $taskOutput 'home'
New-Item -ItemType Directory -Path $taskOutput | Out-Null
# No execution policy, defense, service, global PATH, registry, or task scheduler
# changes. Controller elevation is confined to the temporary ETW session.
$taskRun = [OpenCodex.Diagnostics.OwnedEtwRecorder]::RunOnce(
    $taskExe,$taskNode,(Join-Path $taskRepo 'apps/desktop/tauri'),
    $taskCapture,$taskTemp,$taskHome,$taskSession)
Write-Output $taskRun

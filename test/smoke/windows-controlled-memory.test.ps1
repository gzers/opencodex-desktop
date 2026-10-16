param(
    [Parameter(Mandatory)][string]$DecoderDirectory,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [switch]$RecordControlledSelf
)
$ErrorActionPreference='Stop'
$taskRepo=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
if(!@(@((Join-Path $taskRepo 'test/runtime'),(Join-Path $taskRepo 'test/out')) |
    Where-Object {$taskOutput.StartsWith($_+'\',[StringComparison]::OrdinalIgnoreCase)}).Count){throw 'Test-owned output required'}
if(Test-Path -LiteralPath $taskOutput){throw 'New test output required'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Require native0'}
$taskAdmin=([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if($RecordControlledSelf -and !$taskAdmin){throw 'Requested positive recording needs an actual elevated token'}
New-Item -ItemType Directory -Path $taskOutput | Out-Null
$taskPwsh=(Get-Process -Id $PID).Path
$taskRunner=Join-Path $PSScriptRoot 'windows-controlled-memory.ps1'
$taskChecks=0
function Assert-MemoryTest($Condition,[string]$Message){
    if(!$Condition){throw $Message}
    $script:taskChecks++
}
function Invoke-MemoryChild([string]$Name,[string]$Decoder,[string]$Target,[switch]$Record){
    $taskArguments=@('-NoLogo','-NoProfile','-NonInteractive','-File',$taskRunner,
        '-DecoderDirectory',$Decoder,'-OutputDirectory',$Target)
    if($Record){$taskArguments+='-RecordControlledSelf'}
    $taskLog=Join-Path $taskOutput ($Name+'.decoded.log')
    $taskLines=@(& $taskPwsh @taskArguments 2>&1)
    $taskExit=$LASTEXITCODE
    $taskText=$taskLines -join "`n"
    [IO.File]::WriteAllText($taskLog,$taskText+"`n",[Text.UTF8Encoding]::new($false))
    return [pscustomobject]@{ExitCode=$taskExit;Text=$taskText}
}
$taskGuardTarget=Join-Path $taskOutput 'guard-mode-no-recording'
$taskGuard=Invoke-MemoryChild 'guards' $DecoderDirectory $taskGuardTarget
Assert-MemoryTest ($taskGuard.ExitCode -eq 0) 'Real compiler and method guards failed'
$taskGuardResult=$taskGuard.Text | ConvertFrom-Json
Assert-MemoryTest ($taskGuardResult.checks -eq 20 -and $taskGuardResult.ownProbeReleased) 'Expected controlled ABI/lifecycle/stop tests required'
Assert-MemoryTest (!(Test-Path -LiteralPath $taskGuardTarget) -and !$taskGuardResult.traceStarted) 'Guard mode created recording output or a session'
$taskExisting=Join-Path $taskOutput 'existing'
New-Item -ItemType Directory -Path $taskExisting | Out-Null
$taskSentinel=Join-Path $taskExisting 'preserve.txt'
[IO.File]::WriteAllText($taskSentinel,'controlled sentinel',[Text.UTF8Encoding]::new($false))
$taskExistingResult=Invoke-MemoryChild 'existing-refusal' $DecoderDirectory $taskExisting
Assert-MemoryTest ($taskExistingResult.ExitCode -ne 0 -and $taskExistingResult.Text.Contains('Existing output refused')) 'Existing output must be rejected'
Assert-MemoryTest ([IO.File]::ReadAllText($taskSentinel) -ceq 'controlled sentinel') 'Refusal changed existing data'
$taskEscape=Join-Path $taskRepo 'not-a-test-memory-output'
$taskEscapeResult=Invoke-MemoryChild 'boundary-refusal' $DecoderDirectory $taskEscape
Assert-MemoryTest ($taskEscapeResult.ExitCode -ne 0 -and $taskEscapeResult.Text.Contains('Task-owned test output required')) 'Non-test output must be rejected'
$taskBadDecoder=Join-Path $taskOutput 'bad-decoder'
New-Item -ItemType Directory -Path $taskBadDecoder | Out-Null
Copy-Item -LiteralPath (Join-Path $DecoderDirectory 'Microsoft.Diagnostics.FastSerialization.dll') -Destination $taskBadDecoder
[IO.File]::WriteAllText((Join-Path $taskBadDecoder 'Microsoft.Diagnostics.Tracing.TraceEvent.dll'),'not an assembly',[Text.UTF8Encoding]::new($false))
$taskBadResult=Invoke-MemoryChild 'decoder-refusal' $taskBadDecoder (Join-Path $taskOutput 'bad-recording')
Assert-MemoryTest ($taskBadResult.ExitCode -ne 0 -and $taskBadResult.Text.Contains('Pinned decoder bytes required')) 'Wrong decoder must fail before loading it'
$taskOrdinaryRefusal=$false
if(!$taskAdmin){
    $taskNoAdmin=Join-Path $taskOutput 'ordinary-recording-refused'
    $taskOrdinary=Invoke-MemoryChild 'ordinary-token-refusal' $DecoderDirectory $taskNoAdmin -Record
    Assert-MemoryTest ($taskOrdinary.ExitCode -ne 0 -and $taskOrdinary.Text.Contains('requires an elevated token')) 'Ordinary token must not start recording'
    Assert-MemoryTest (!(Test-Path -LiteralPath $taskNoAdmin)) 'Ordinary refusal created recording output'
    $taskOrdinaryRefusal=$true
}
$taskCapture=$null
if($RecordControlledSelf){
    $taskTarget=Join-Path $taskOutput 'capture'
    $taskActual=Invoke-MemoryChild 'capture' $DecoderDirectory $taskTarget -Record
    Assert-MemoryTest ($taskActual.ExitCode -eq 0) 'Controlled recording failed; preserve its first result'
    $taskCapture=Get-Content -LiteralPath (Join-Path $taskTarget 'result.json') -Raw | ConvertFrom-Json
    Assert-MemoryTest ($taskCapture.passed -and $taskCapture.matchingProbeAllocationEvents -gt 0 -and $taskCapture.matchingAllocationStacks -gt 0) 'Actual probe allocation and stack required'
    Assert-MemoryTest ($taskCapture.nativeStop.Status -eq 0 -and $taskCapture.nativeStop.FinalEventsLost -eq 0 -and $taskCapture.nativeStop.FinalRealTimeBuffersLost -eq 0) 'Owned stop and zero lost events required'
    Assert-MemoryTest ($taskCapture.ownedSessionAbsent -and $taskCapture.readerJoined -and !$taskCapture.stopProcessingUsed) 'Owned session must stop and fully drain'
    Assert-MemoryTest ($taskCapture.probeFreed -and $taskCapture.retainedSelfGenerationUnchanged -and $taskCapture.retainedSelfProcessHandle) 'Controlled allocation or process generation differs'
    Assert-MemoryTest (!$taskCapture.genericSessionStopInvoked -and !$taskCapture.globalSampleIntervalExplicitlyWritten -and !$taskCapture.globalSampleIntervalBeforeAfterMeasured) 'Do not invoke a global reset or claim an unmeasured system state'
    Assert-MemoryTest (!$taskCapture.systemWideEtlWritten -and !$taskCapture.unrelatedEventPayloadsSerialized -and !$taskCapture.originalTestLibraryExecuted -and !$taskCapture.officialQueryNodeExecuted) 'Controlled capture scope changed'
}
Assert-MemoryTest (@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count -eq 0) 'Native state changed'
$taskResult=[ordered]@{passed=$true;checks=$taskChecks;compiledGuardChecks=$taskGuardResult.checks
    ordinaryTokenRefusalTested=$taskOrdinaryRefusal;administratorToken=$taskAdmin
    controlledRecordingRequested=[bool]$RecordControlledSelf
    controlledRecordingPassed=($null -ne $taskCapture -and $taskCapture.passed)
    nativeDesktopStarted=$false;originalTestLibraryExecuted=$false;officialQueryNodeExecuted=$false
    productAllocationCauseAttributed=$false;fullProductGatePassed=$false
    sourceSha256=@{recorder=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'windows-controlled-memory.cs')).Hash.ToLowerInvariant()
        wrapper=(Get-FileHash -LiteralPath $taskRunner).Hash.ToLowerInvariant()
        sharedNativeStop=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'windows-owned-etw.cs')).Hash.ToLowerInvariant()}}
$taskResult | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $taskOutput 'result.json') -Encoding utf8
$taskResult | ConvertTo-Json -Depth 6

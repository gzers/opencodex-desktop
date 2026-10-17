param(
    [Parameter(Mandatory)][string]$DecoderDirectory,
    [string]$Plan,
    [switch]$RecordApproved
)
$ErrorActionPreference='Stop'
if(!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7 -or [IntPtr]::Size -ne 8){
    throw 'Windows PowerShell 7 x64 required'
}
$taskRepo=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$taskDecoder=[IO.Path]::GetFullPath($DecoderDirectory)
$taskHashes=@{
    'Microsoft.Diagnostics.Tracing.TraceEvent.dll'='401e948dac99d0f9756ef96d5a8b8e4736e19e9f82ce8cdd744495951e3a94bb'
    'Microsoft.Diagnostics.FastSerialization.dll'='83e89baf0bbecca8cfa194cff13365f716849e22efc374b669206f26998a019f'
}
$taskLibraries=@(foreach($taskName in $taskHashes.Keys){
    $taskFile=Join-Path $taskDecoder $taskName
    if((Get-FileHash -LiteralPath $taskFile).Hash.ToLowerInvariant() -cne $taskHashes[$taskName]){
        throw 'Pinned TraceEvent3.1.21 decoder mismatch'
    }
    [void][Reflection.Assembly]::LoadFrom($taskFile)
    $taskFile
})
$taskReferences=@((Get-ChildItem -LiteralPath (Join-Path $PSHOME 'ref') -Filter '*.dll').FullName)+$taskLibraries
if(!('OpenCodex.Diagnostics.ProductQueryCounters' -as [type])){
    Add-Type -Path @((Join-Path $PSScriptRoot 'windows-owned-etw.cs'),
        (Join-Path $PSScriptRoot 'windows-query-counters.cs')) -ReferencedAssemblies $taskReferences
}
$taskRules=[OpenCodex.Diagnostics.QueryCounterRules]::SelfTest()
$taskScope=[OpenCodex.Diagnostics.OwnedEtwScope]::SelfTest()
$taskStop=[OpenCodex.Diagnostics.OwnedEtwNativeStop]::SelfTest()
if(!$RecordApproved){
    [ordered]@{status='prepared-only';counterRules=$taskRules;existingScopeRules=$taskScope
        existingNativeStopRules=$taskStop;etwSessionStarted=$false;productStarted=$false
        foregroundUsed=$false;nativeUiDriven=$false;fullPerformanceGatePassed=$false}|ConvertTo-Json
    return
}
if(!$Plan){throw 'Current exact observation plan required'}
. (Join-Path $PSScriptRoot 'windows-startup-pair-state.ps1')
$taskPlanPath=ConvertTo-StartupPairPath $Plan
$taskPlan=Get-Content -LiteralPath $taskPlanPath -Raw -Encoding utf8|ConvertFrom-Json
$taskAuth=$taskPlan.authorization
if($taskAuth.origin -cne 'human' -or $taskAuth.explicitHumanStart -isnot [bool] -or
    !$taskAuth.explicitHumanStart -or $taskAuth.automaticContinuation -isnot [bool] -or
    $taskAuth.automaticContinuation){throw 'Current explicit human foreground window required'}
$taskStart=[datetimeoffset]::Parse($taskAuth.authorizedAtUtc)
$taskEnd=[datetimeoffset]::Parse($taskAuth.expiresAtUtc)
$taskNow=[datetimeoffset]::UtcNow
if($taskEnd -le $taskStart -or ($taskEnd-$taskStart).TotalMinutes -gt 30 -or
    $taskNow -lt $taskStart -or $taskNow -ge $taskEnd -or $taskNow -ge $taskStart.AddMinutes(25)){
    throw 'Fresh foreground window/normal cleanup reservation required'
}
$taskDeadline=$taskStart.AddMinutes(25)
if($taskEnd -lt $taskDeadline){$taskDeadline=$taskEnd}
$taskRun=ConvertTo-StartupPairPath $taskPlan.runDirectory
$taskRuntime=[IO.Path]::GetFullPath((Join-Path $taskRepo 'test/runtime'))
if(!$taskRun.StartsWith($taskRuntime+'\',[StringComparison]::OrdinalIgnoreCase)){
    throw 'Owned private run required'
}
$taskExecutable=ConvertTo-StartupPairPath $taskPlan.executable
$taskImages=@{
    '8e69ff6db69ea13b8cb29feb9f7a1abacd202a7b'='870cb30c62c0cf563edacf7fe30a35d3b66cbffbb574beeb6b354ed2c7fab15a'
    'de6862d40389d527527b9db8ef52e186a58b625f'='626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608'
}
if(!$taskImages.ContainsKey($taskPlan.source) -or $taskImages[$taskPlan.source] -cne $taskPlan.exeSha256 -or
    (Get-FileHash -LiteralPath $taskExecutable).Hash.ToLowerInvariant() -cne $taskPlan.exeSha256){
    throw 'Exact admitted product image required'
}
$taskStopFile=ConvertTo-StartupPairPath $taskPlan.stopFile
if(!$taskStopFile.StartsWith($taskRun+'\',[StringComparison]::OrdinalIgnoreCase) -or
    (Test-Path -LiteralPath $taskStopFile)){throw 'Fresh owned stop endpoint required'}
if($taskPlan.debugPort -isnot [long] -and $taskPlan.debugPort -isnot [int]){
    throw 'Integer debugger port required'
}
if($taskPlan.debugPort -lt 1 -or $taskPlan.debugPort -gt 65535){throw 'Explicit debugger port required'}
Assert-StartupPairNoProduct
# The native launch controller starts the ordinary-token product only after
# this elevated observer publishes ready.json. This recorder never starts it.
$taskTerminal=[OpenCodex.Diagnostics.ProductQueryCounters]::Observe(
    $taskExecutable,(Join-Path $taskRun 'native-counters'),$taskStopFile,
    $taskDeadline.UtcDateTime,[int]$taskPlan.debugPort)
$taskResult=Get-Content -LiteralPath $taskTerminal -Raw|ConvertFrom-Json
if(!$taskResult.complete){throw 'Capture incomplete; terminal and raw events retained'}
Write-Output $taskTerminal

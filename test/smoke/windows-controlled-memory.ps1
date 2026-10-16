param(
    [Parameter(Mandatory)][string]$DecoderDirectory,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [switch]$RecordControlledSelf
)
$ErrorActionPreference='Stop'
if(!$IsWindows -or [IntPtr]::Size -ne 8 -or $PSVersionTable.PSVersion.Major -ne 7){throw 'Windows x64 PowerShell 7 required'}
$taskAdministrator=([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if($RecordControlledSelf -and !$taskAdministrator){throw 'Controlled recording requires an elevated token; no self-elevation or trace start performed'}
$taskRepo=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
$taskRoots=@((Join-Path $taskRepo 'test/runtime'),(Join-Path $taskRepo 'test/out'))
if(!@($taskRoots | Where-Object {$taskOutput.StartsWith($_+'\',[StringComparison]::OrdinalIgnoreCase)}).Count){throw 'Task-owned test output required'}
if(Test-Path -LiteralPath $taskOutput){throw 'Existing output refused; no overwrite or automatic retry'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Require native0'}
foreach($taskPath in @($taskOutput,$DecoderDirectory)){
    $taskCursor=[IO.Path]::GetFullPath($taskPath)
    while($taskCursor){
        if((Test-Path -LiteralPath $taskCursor) -and ((Get-Item -LiteralPath $taskCursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Test path aliases refused'}
        $taskCursor=[IO.Path]::GetDirectoryName($taskCursor)
    }
}
if('OpenCodex.Diagnostics.ControlledMemoryRecorder' -as [type]){throw 'Use a fresh controller process'}
$taskAssemblies=@()
foreach($taskEntry in @(
    @{Name='Microsoft.Diagnostics.FastSerialization.dll';Sha='83e89baf0bbecca8cfa194cff13365f716849e22efc374b669206f26998a019f'}
    @{Name='Microsoft.Diagnostics.Tracing.TraceEvent.dll';Sha='401e948dac99d0f9756ef96d5a8b8e4736e19e9f82ce8cdd744495951e3a94bb'}
)){
    $taskPath=Join-Path $DecoderDirectory $taskEntry.Name
    if((Get-FileHash -LiteralPath $taskPath).Hash.ToLowerInvariant() -cne $taskEntry.Sha){throw 'Pinned decoder bytes required'}
    [void][Reflection.Assembly]::LoadFrom($taskPath)
    $taskAssemblies+=$taskPath
}
$taskReferences=@((Get-ChildItem -LiteralPath (Join-Path $PSHOME 'ref') -Filter '*.dll').FullName)+$taskAssemblies
Add-Type -Path @((Join-Path $PSScriptRoot 'windows-owned-etw.cs'),(Join-Path $PSScriptRoot 'windows-controlled-memory.cs')) -ReferencedAssemblies $taskReferences
$taskChecks=[OpenCodex.Diagnostics.OwnedEtwNativeStop]::SelfTest()+
    [OpenCodex.Diagnostics.OwnedEtwLifecycle]::SelfTest()+
    [OpenCodex.Diagnostics.ControlledMemoryRecorder]::SelfTest()
if(!$RecordControlledSelf){
    [ordered]@{status='controlled-method-guards-only';checks=$taskChecks;administratorToken=$taskAdministrator
        ownProbeBytes=8388608;ownProbeReleased=$true;traceStarted=$false
        originalTestLibraryExecuted=$false;officialQueryNodeExecuted=$false
        nativeProductStarted=$false;fullProductGatePassed=$false} | ConvertTo-Json
    return
}
$taskResult=[OpenCodex.Diagnostics.ControlledMemoryRecorder]::RunOnce($taskOutput)
$taskResult
if(!(ConvertFrom-Json $taskResult).passed){throw 'Controlled memory capture did not meet its original assertions; preserve evidence and inspect'}

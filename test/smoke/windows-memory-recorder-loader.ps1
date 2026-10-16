function Initialize-WindowsMemoryRecorder {
    param([Parameter(Mandatory)][string]$DecoderDirectory)
    $ErrorActionPreference='Stop'
    if('OpenCodex.Diagnostics.ControlledMemoryRecorder' -as [type]){throw 'Fresh memory recorder process required'}
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
}

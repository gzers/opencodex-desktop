. (Join-Path $PSScriptRoot 'windows-process-token.ps1')
. (Join-Path $PSScriptRoot 'windows-startup-pair-state.ps1')

function Save-WindowsObservationBinding {
    param(
        [Parameter(Mandatory)][Diagnostics.Process]$Process,
        [Parameter(Mandatory)][string]$ExpectedExecutable,
        [Parameter(Mandatory)][ValidateRange(1,65535)][int]$DebugPort,
        [Parameter(Mandatory)][string]$OutputPath
    )
    $ErrorActionPreference='Stop'
    $taskImage=ConvertTo-StartupPairPath $ExpectedExecutable
    $taskOutput=ConvertTo-StartupPairPath $OutputPath
    $taskRuntime=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../runtime'))
    if(!$taskOutput.StartsWith($taskRuntime+'\',[StringComparison]::OrdinalIgnoreCase) -or
        (Test-Path -LiteralPath $taskOutput)){throw 'Fresh owned binding receipt required'}
    $taskToken=Get-WindowsProcessToken -Process $Process -ExpectedExecutable $taskImage
    Assert-WindowsProcessMode -Observation $taskToken -Mode ordinary
    $taskDigest=(Get-FileHash -LiteralPath $taskImage).Hash.ToLowerInvariant()
    $taskSource=switch($taskDigest){
        '870cb30c62c0cf563edacf7fe30a35d3b66cbffbb574beeb6b354ed2c7fab15a' {'8e69ff6db69ea13b8cb29feb9f7a1abacd202a7b'}
        '626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608' {'de6862d40389d527527b9db8ef52e186a58b625f'}
        default {throw 'Exact admitted8e MSI or official019 image required'}
    }
    if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count -ne 1){
        throw 'Exactly one isolated native product required'
    }
    $taskListeners=@(Get-NetTCPConnection -State Listen -LocalPort $DebugPort -ErrorAction Stop)
    if(!$taskListeners.Count -or @($taskListeners.OwningProcess|Sort-Object -Unique).Count -ne 1){
        throw 'Unique native debugger listener required'
    }
    foreach($taskListener in $taskListeners){
        if(![Net.IPAddress]::IsLoopback([Net.IPAddress]::Parse($taskListener.LocalAddress))){
            throw 'Debugger listener must be loopback only'
        }
    }
    $taskPortOwner=[int]$taskListeners[0].OwningProcess
    $taskNext=$taskPortOwner
    $taskChain=[Collections.Generic.List[object]]::new()
    $taskHandles=[Collections.Generic.List[Diagnostics.Process]]::new()
    try{
        for($taskDepth=0;$taskDepth -lt 32 -and $taskNext -ne $Process.Id;$taskDepth++){
            $taskHandle=Get-Process -Id $taskNext -ErrorAction Stop
            $taskHandles.Add($taskHandle)
            $null=$taskHandle.SafeHandle
            $taskNative=Get-CimInstance Win32_Process -Filter ("ProcessId="+$taskNext)
            if(!$taskNative -or $taskHandle.HasExited){
                throw 'Debugger ancestry process generation differs'
            }
            if($taskHandle.StartTime.ToUniversalTime() -lt [datetime]::Parse($taskToken.creationTimeUtc).ToUniversalTime()){
                throw 'Debugger ancestor predates retained product'
            }
            $taskChain.Add([ordered]@{processId=$taskNext;parentProcessId=[int]$taskNative.ParentProcessId
                creationTimeUtc=$taskHandle.StartTime.ToUniversalTime().ToString('o')
                imageBasename=[IO.Path]::GetFileName($taskNative.ExecutablePath)})
            $taskNext=[int]$taskNative.ParentProcessId
        }
        if($taskNext -ne $Process.Id -or !$taskChain.Count -or
            $taskChain[0].imageBasename -ine 'msedgewebview2.exe'){
            throw 'Debugger listener is not a retained product WebView descendant'
        }
        $taskAfter=Get-WindowsProcessToken -Process $Process -ExpectedExecutable $taskImage
        if($taskAfter.creationTimeUtc -cne $taskToken.creationTimeUtc){throw 'Retained product identity changed'}
        foreach($taskHandle in $taskHandles){if($taskHandle.HasExited){throw 'Debugger ancestry exited during verification'}}
        $taskReceipt=[ordered]@{
            schemaVersion=1;observedAtUtc=[datetime]::UtcNow.ToString('o')
            source=$taskSource;exeSha256=$taskDigest
            product=$taskAfter;debugPort=$DebugPort;listenerOwner=$taskPortOwner;ancestry=$taskChain.ToArray()
            retainedProductHandleVerified=$true;loopbackListenerVerified=$true
            listenerIsOwnedWebViewDescendant=$true;nativeProductCount=1
            commandLineOrRawSidPublished=$false;productStartedByThisMethod=$false
        }
        $taskJson=$taskReceipt|ConvertTo-Json -Depth 8
        [IO.File]::WriteAllText($taskOutput,$taskJson,[Text.UTF8Encoding]::new($false))
        [ordered]@{path=$taskOutput;sha256=(Get-FileHash -LiteralPath $taskOutput).Hash.ToLowerInvariant()}
    }finally{
        foreach($taskHandle in $taskHandles){$taskHandle.Dispose()}
    }
}

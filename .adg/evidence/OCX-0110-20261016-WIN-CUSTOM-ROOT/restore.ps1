$ErrorActionPreference='Stop'
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Do not restore binding while a product still owns its roots'}
$taskPreparation=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'preparation.original.json') -Raw | ConvertFrom-Json
$taskMetadata=Join-Path $taskPreparation.anchor 'manager-state/data-root.json'
$taskBinding=Get-Content -LiteralPath $taskMetadata -Raw | ConvertFrom-Json
if($taskBinding.active_data_root -cne $taskPreparation.custom -or $taskBinding.opencodex_home_path -cne $taskPreparation.externalHome){throw 'Binding changed independently; stop restoration'}
Copy-Item -LiteralPath $taskMetadata -Destination (Join-Path $PSScriptRoot 'anchor-bound.original.json')
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'anchor-before.original.json') -Destination $taskMetadata
if((Get-FileHash -LiteralPath $taskMetadata).Hash -cne
    (Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'anchor-before.original.json')).Hash){throw 'Original anchor bytes not restored'}
Copy-Item -LiteralPath (Join-Path $taskPreparation.anchor 'logs/app.log') -Destination (Join-Path $PSScriptRoot 'anchor-log-after.original.log')
Copy-Item -LiteralPath (Join-Path $taskPreparation.custom 'logs/app.log') -Destination (Join-Path $PSScriptRoot 'custom-log.original.log')
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');anchorOriginalBytesRestored=$true
    originalAnchorLogUnchanged=((Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'anchor-log-before.original.log')).Hash -ceq
        (Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'anchor-log-after.original.log')).Hash)
    startupPointerUnchanged=((Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop').StartupDataRootV1 -ceq $taskPreparation.startupPointer)
    customFixtureRetained=$true;nativeCount=0;fullStorageGatePassed=$false
} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'restoration.json') -Encoding utf8

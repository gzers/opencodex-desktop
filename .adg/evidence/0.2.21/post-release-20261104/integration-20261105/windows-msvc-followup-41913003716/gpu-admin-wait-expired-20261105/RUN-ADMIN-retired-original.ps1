$ErrorActionPreference='Stop'
$taskCollector='D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/post021-windows-materials/test/smoke/post021-windows-final/collect-gpu-trace.ps1'
Write-Output 'After the recorder announces it is active, return to the isolated OpenCodeX Desktop window within10s and leave it foreground for45s.'
& $taskCollector -TargetIdentityFile (Join-Path $PSScriptRoot 'target.json') -OutputDirectory (Join-Path $PSScriptRoot 'capture') -DurationSeconds 45

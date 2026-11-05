$ErrorActionPreference='Stop'
Write-Output 'After trace active: return to the isolated OpenCodeX Desktop window within10s; keep foreground45s.'
& 'D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/post021-windows-materials/test/smoke/post021-windows-final/collect-gpu-trace.ps1' -TargetIdentityFile (Join-Path $PSScriptRoot 'target.json') -OutputDirectory (Join-Path $PSScriptRoot 'capture') -DurationSeconds 45

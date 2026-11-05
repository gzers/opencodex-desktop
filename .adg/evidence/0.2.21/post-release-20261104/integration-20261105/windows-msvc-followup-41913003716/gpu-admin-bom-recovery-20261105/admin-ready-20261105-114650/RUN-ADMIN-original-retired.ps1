$ErrorActionPreference='Stop'
$taskReady=Join-Path $PSScriptRoot 'scene-ready.json'
if(!(Test-Path -LiteralPath $taskReady)){throw 'Sampler setup is not ready; no recording started'}
if(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'scene-result.json')){throw 'Sampler already ended; ask for a fresh ready sampler; no recording started'}
$taskExpected=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'sampler.json') -Raw -Encoding utf8|ConvertFrom-Json
$taskP=Get-CimInstance Win32_Process -Filter ('ProcessId = '+[uint32]$taskExpected.pid)
if(!$taskP -or $taskP.ExecutablePath -ine $taskExpected.exe -or [string]$taskP.CreationDate.ToUniversalTime().Ticks -cne $taskExpected.createdAtUtcTicks){throw 'Waiting sampler identity changed; no recording started'}
Write-Output 'After trace active: switch to the isolated OpenCodeX Desktop window within10s and keep it foreground45s.'
& 'D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/post021-windows-materials/test/smoke/post021-windows-final/collect-gpu-trace.ps1' -TargetIdentityFile (Join-Path $PSScriptRoot 'target.json') -OutputDirectory (Join-Path $PSScriptRoot 'capture') -DurationSeconds 45

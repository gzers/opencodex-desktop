$ErrorActionPreference='Stop'
if(!(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'scene-ready.json'))){throw 'Sampler not ready'}
if(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'scene-result.json')){throw 'Sampler ended; no recording started'}
$taskExpected=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'sampler.json') -Raw -Encoding utf8|ConvertFrom-Json
$taskP=Get-CimInstance Win32_Process -Filter ('ProcessId='+[uint32]$taskExpected.pid)
if(!$taskP -or $taskP.ExecutablePath -ine $taskExpected.exe -or [string]$taskP.CreationDate.ToUniversalTime().Ticks -cne $taskExpected.createdAtUtcTicks -or $taskP.CommandLine -cne $taskExpected.commandLine -or $taskP.SessionId -ne $taskExpected.sessionId){throw 'Sampler identity changed'}
$taskOwner=Invoke-CimMethod -InputObject $taskP -MethodName GetOwner
if($taskOwner.ReturnValue -ne 0 -or $taskOwner.User -ine $taskExpected.ownerUser -or $taskOwner.Domain -ine $taskExpected.ownerDomain){throw 'Sampler owner changed'}
Write-Output 'Switch to isolated OpenCodeX now. Recorder waits up to120s for stable native foreground; keep window foreground60s.'
& 'D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/post021-windows-materials/test/smoke/post021-windows-final/collect-gpu-trace.ps1' -TargetIdentityFile (Join-Path $PSScriptRoot 'target.json') -OutputDirectory (Join-Path $PSScriptRoot 'capture') -DurationSeconds 45 -ForegroundReadyFile (Join-Path $PSScriptRoot 'scene-foreground.json')

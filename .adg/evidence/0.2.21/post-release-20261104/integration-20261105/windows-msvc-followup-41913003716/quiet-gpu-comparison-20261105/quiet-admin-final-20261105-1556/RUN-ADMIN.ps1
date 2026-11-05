$ErrorActionPreference='Stop'
if(!(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'scene-ready.json'))){throw 'Sampler not ready'}
if(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'scene-result.json')){throw 'Sampler ended; no recording started'}
$taskExpected=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'sampler.json') -Raw -Encoding utf8|ConvertFrom-Json
$taskP=Get-CimInstance Win32_Process -Filter ('ProcessId='+[uint32]$taskExpected.pid)
if(!$taskP -or $taskP.ExecutablePath -ine $taskExpected.exe -or [string]$taskP.CreationDate.ToUniversalTime().Ticks -cne $taskExpected.createdAtUtcTicks -or $taskP.CommandLine -cne $taskExpected.commandLine -or $taskP.SessionId -ne $taskExpected.sessionId){throw 'Sampler identity changed'}
$taskOwner=Invoke-CimMethod -InputObject $taskP -MethodName GetOwner
if($taskOwner.ReturnValue -ne 0 -or $taskOwner.User -ine $taskExpected.ownerUser -or $taskOwner.Domain -ine $taskExpected.ownerDomain){throw 'Sampler owner changed'}
$taskMarker=Join-Path $PSScriptRoot ('admin-focus-wait-'+[guid]::NewGuid().ToString('N')+'.json')
[IO.File]::WriteAllText($taskMarker,([ordered]@{at=[datetimeoffset]::UtcNow.ToString('o');samplerPid=$taskExpected.pid;samplerCreatedAtUtcTicks=$taskExpected.createdAtUtcTicks;administratorShellPid=$PID}|ConvertTo-Json),[Text.UTF8Encoding]::new($false))
Write-Output 'Waiting for isolated test window. The assistant will raise its verified window; keep it foreground60s.'
$taskReleasePath=Join-Path $PSScriptRoot 'recording-release.json'
$taskReleaseDeadline=[datetimeoffset]::UtcNow.AddMinutes(30)
$taskReleased=$false
Write-Output 'Waiting for assistant release after your reply; no recording yet.'
while([datetimeoffset]::UtcNow -lt $taskReleaseDeadline){try{$taskRelease=Get-Content -LiteralPath $taskReleasePath -Raw -Encoding utf8|ConvertFrom-Json;if($taskRelease.samplerPid -eq $taskExpected.pid -and [string]$taskRelease.samplerCreatedAtUtcTicks -ceq [string]$taskExpected.createdAtUtcTicks){$taskReleased=$true;break}}catch{};Start-Sleep -Milliseconds 200}
if(!$taskReleased){throw 'No assistant release; no recording started'}
& 'D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/post021-windows-materials/test/smoke/post021-windows-final/collect-gpu-trace.ps1' -TargetIdentityFile (Join-Path $PSScriptRoot 'target.json') -OutputDirectory (Join-Path $PSScriptRoot 'capture') -DurationSeconds 45 -ForegroundWaitSeconds 1800 -ForegroundReadyFile (Join-Path $PSScriptRoot 'scene-foreground.json')

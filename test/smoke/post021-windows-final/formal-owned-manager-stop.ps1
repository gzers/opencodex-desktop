param([Parameter(Mandatory=$true)][string]$IdentityFile)
$ErrorActionPreference='Stop'
$taskExpected=Get-Content -LiteralPath $IdentityFile -Raw|ConvertFrom-Json
$taskOwner=[Security.Principal.WindowsIdentity]::GetCurrent().Name
if($taskOwner -ine 'EZIO-MINI\Administrator' -or $taskExpected.owner -ine $taskOwner){throw 'Administrator owner required'}
if($taskExpected.pid -ne 30804 -or $taskExpected.createdAtUtcTicks -cne '639294604444368260'){throw 'Recorded test-manager generation required'}
$taskImage=[IO.Path]::GetFullPath('D:/Users/Administrator/WorkSpace/Test/post021-formal-20261105-1518/opencodex-desktop.exe')
if($taskExpected.path -ine $taskImage -or (Get-FileHash -Algorithm SHA256 -LiteralPath $taskImage).Hash -ine 'fa9df83a55153f764673781e85167093d20602fd2e34742c69f6b2f5b37f9c68'){throw 'Original test manager image changed'}
$taskProcess=Get-CimInstance Win32_Process -Filter 'ProcessId=30804' -ErrorAction Stop
if(!$taskProcess -or $taskProcess.SessionId -ne 1 -or [string]$taskProcess.CreationDate.ToUniversalTime().Ticks -cne $taskExpected.createdAtUtcTicks){throw 'Test manager session/generation changed'}
$taskWho=Invoke-CimMethod -InputObject $taskProcess -MethodName GetOwner -ErrorAction Stop
if($taskWho.ReturnValue -ne 0 -or ($taskWho.Domain+'\'+$taskWho.User) -ine $taskOwner -or $taskProcess.ExecutablePath -ine $taskImage){throw 'Test manager owner/path changed'}
$taskNative=Get-Process -Id 30804
[void]$taskNative.Handle
$taskDelta=$taskNative.StartTime.ToUniversalTime().Ticks-$taskProcess.CreationDate.ToUniversalTime().Ticks
if($taskNative.Path -ine $taskImage -or $taskDelta -lt 0 -or $taskDelta -gt 9){throw 'Native manager generation changed'}
Stop-Process -InputObject $taskNative
if(!$taskNative.WaitForExit(10000)){throw 'Owned test manager did not exit'}
[ordered]@{pid=30804;owner=$taskOwner;sessionId=1;path=$taskImage;createdAtUtcTicks=$taskExpected.createdAtUtcTicks;nativeTickDelta=$taskDelta}|ConvertTo-Json -Compress

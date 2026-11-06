param([Parameter(Mandatory=$true)][string]$IdentityFile)
$ErrorActionPreference='Stop'
$taskExpected=Get-Content -LiteralPath $IdentityFile -Raw|ConvertFrom-Json
$taskOwner=[Security.Principal.WindowsIdentity]::GetCurrent().Name
if($taskOwner -ine 'EZIO-MINI\Administrator' -or $taskExpected.owner -ine $taskOwner){throw 'Administrator owner required'}
if($taskExpected.sessionId -ne 1 -or !$taskExpected.createdAtUtcTicks){throw 'Recorded test generation required'}
$taskProcess=Get-CimInstance Win32_Process -Filter ('ProcessId='+[int]$taskExpected.pid) -ErrorAction Stop
if(!$taskProcess -or $taskProcess.SessionId -ne 1){throw 'Owned service absent or session changed'}
if([string]$taskProcess.CreationDate.ToUniversalTime().Ticks -cne $taskExpected.createdAtUtcTicks){throw 'Test service generation changed'}
$taskWho=Invoke-CimMethod -InputObject $taskProcess -MethodName GetOwner -ErrorAction Stop
if($taskWho.ReturnValue -ne 0 -or ($taskWho.Domain+'\'+$taskWho.User) -ine $taskOwner){throw 'Test service owner changed'}
$taskRoot=[IO.Path]::GetFullPath('D:/Users/Administrator/WorkSpace/Test/post021-formal-20261105-1518').TrimEnd([char]92)+[char]92
$taskImage=[IO.Path]::GetFullPath($taskExpected.path)
if(!$taskImage.StartsWith($taskRoot,[StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetFileName($taskImage) -ine 'bun.exe'){throw 'Image outside authorized test root'}
if($taskProcess.ExecutablePath -ine $taskImage -or $taskProcess.CommandLine -notlike '*formal-msix-service.ts*' -or $taskProcess.CommandLine -notlike '*post021-formal-20261105-1518*'){throw 'Test service path/command mismatch'}
$taskNative=Get-Process -Id $taskProcess.ProcessId
[void]$taskNative.Handle
$taskDelta=$taskNative.StartTime.ToUniversalTime().Ticks-$taskProcess.CreationDate.ToUniversalTime().Ticks
if($taskNative.Path -ine $taskImage -or $taskDelta -lt 0 -or $taskDelta -gt 9){throw 'Native generation mismatch'}
Stop-Process -InputObject $taskNative
if(!$taskNative.WaitForExit(10000)){throw 'Owned test service did not exit'}
[ordered]@{pid=$taskExpected.pid;createdAtUtcTicks=$taskExpected.createdAtUtcTicks;owner=$taskOwner;sessionId=1;path=$taskImage;nativeTickDelta=$taskDelta}|ConvertTo-Json -Compress

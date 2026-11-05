param([Parameter(Mandatory=$true)][string]$IdentityFile)
$ErrorActionPreference='Stop'
$taskE=Get-Content -LiteralPath $IdentityFile -Raw|ConvertFrom-Json
$taskSelf=[Security.Principal.WindowsIdentity]::GetCurrent()
if($taskSelf.Name -ine 'EZIO-MINI\Administrator' -or $taskE.owner -ine $taskSelf.Name -or $taskE.sid -cne $taskSelf.User.Value -or $taskE.sessionId -ne 1){throw 'Exact Administrator test owner/SID/session required'}
$taskRoot=[IO.Path]::GetFullPath('D:/Users/Administrator/WorkSpace/Test').TrimEnd([char]92)+[char]92
$taskImage=[IO.Path]::GetFullPath($taskE.path)
if(!$taskImage.StartsWith($taskRoot,[StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetFileName($taskImage) -inotmatch '^(bun|opencodex-desktop)[.]exe$'){throw 'Test image outside allowed root'}
if((Get-FileHash -LiteralPath $taskImage -Algorithm SHA256).Hash -ine $taskE.sha256){throw 'Test image bytes changed'}
$taskP=Get-CimInstance Win32_Process -Filter ('ProcessId='+[int]$taskE.pid) -ErrorAction Stop
if(!$taskP -or $taskP.SessionId -ne 1 -or $taskP.ExecutablePath -ine $taskImage -or [string]$taskP.CreationDate.ToUniversalTime().Ticks -cne $taskE.createdAtUtcTicks){throw 'Test process path/session/generation changed'}
$taskSid=Invoke-CimMethod -InputObject $taskP -MethodName GetOwnerSid -ErrorAction Stop
if($taskSid.ReturnValue -ne 0 -or $taskSid.Sid -cne $taskE.sid){throw 'Test process SID changed'}
if([IO.Path]::GetFileName($taskImage) -ieq 'bun.exe' -and $taskP.CommandLine -notlike ('*'+$taskE.commandMarker+'*')){throw 'Test service command changed'}
$taskH=Get-Process -Id $taskP.ProcessId
[void]$taskH.Handle
$taskDelta=$taskH.StartTime.ToUniversalTime().Ticks-$taskP.CreationDate.ToUniversalTime().Ticks
if($taskH.Path -ine $taskImage -or $taskDelta -lt 0 -or $taskDelta -gt 9){throw 'Native test generation changed'}
Stop-Process -InputObject $taskH
if(!$taskH.WaitForExit(10000)){throw 'Exact owned process did not exit'}
[ordered]@{pid=$taskE.pid;path=$taskImage;createdAtUtcTicks=$taskE.createdAtUtcTicks;sid=$taskE.sid;sessionId=1;nativeTickDelta=$taskDelta;sha256=$taskE.sha256}|ConvertTo-Json -Compress

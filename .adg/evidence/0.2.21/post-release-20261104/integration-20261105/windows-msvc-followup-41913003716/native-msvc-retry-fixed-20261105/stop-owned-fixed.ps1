param([Parameter(Mandatory=$true)][string]$IdentityFile,[Parameter(Mandatory=$true)][string]$ResultFile)
$ErrorActionPreference='Stop'
$taskAdmission=Get-Content -LiteralPath $IdentityFile -Raw|ConvertFrom-Json -DateKind String
if(($taskAdmission.path -notlike '*post021-windows-final-msvc-41913003716*') -and ($taskAdmission.path -notlike '*post021-windows-final-msvc-retry-41944607861*') -and ($taskAdmission.commandLine -notlike '*post021-msvc-followup*')){throw 'Identity lacks owned test root'}
$taskCurrent=Get-CimInstance Win32_Process -Filter ('ProcessId = '+$taskAdmission.pid)
if(-not $taskCurrent -or $taskCurrent.ExecutablePath -cne $taskAdmission.path -or $taskCurrent.CreationDate.ToString('o') -cne $taskAdmission.createdAt -or $taskCurrent.SessionId -ne $taskAdmission.session){throw 'CIM identity mismatch'}
if($taskAdmission.commandLine -and $taskCurrent.CommandLine -cne $taskAdmission.commandLine){throw 'Command line mismatch'}
$taskOwner=Invoke-CimMethod -InputObject $taskCurrent -MethodName GetOwner
if($taskOwner.ReturnValue -ne 0 -or $taskOwner.User -cne $taskAdmission.owner.User -or $taskOwner.Domain -cne $taskAdmission.owner.Domain){throw 'Owner mismatch'}
$taskProcess=Get-Process -Id $taskAdmission.pid
$taskDelta=$taskProcess.StartTime.ToUniversalTime().Ticks-$taskCurrent.CreationDate.ToUniversalTime().Ticks
if($taskProcess.Path -cne $taskAdmission.path -or $taskDelta -lt 0 -or $taskDelta -gt 9){throw 'Handle identity mismatch beyond CIM microsecond precision'}
$taskCreated=$taskProcess.StartTime.ToString('o')
Stop-Process -InputObject $taskProcess
$taskProcess.WaitForExit(10000)|Out-Null
[pscustomobject]@{result='stopped-exact-owned-process';admission=$taskAdmission;handleCreated=$taskCreated;tickDelta=$taskDelta;exited=$taskProcess.HasExited;at=(Get-Date).ToString('o')}|ConvertTo-Json -Depth 6|Set-Content -LiteralPath $ResultFile -Encoding utf8

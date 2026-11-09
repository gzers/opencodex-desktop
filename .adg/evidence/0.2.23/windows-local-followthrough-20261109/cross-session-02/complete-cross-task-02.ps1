$ErrorActionPreference='Stop'
$b=Join-Path $PSScriptRoot 'cross-session-02'
$r=Get-Content -LiteralPath (Join-Path $b 'task-registration.json') -Raw|ConvertFrom-Json
$report=[ordered]@{observedUtc=[datetime]::UtcNow.ToString('o');taskName=$r.taskName;scope='Only own exact completed cross-session task and readback; no normal Stop inferred from cleanup'}
try{
 foreach($role in @('console','rdp')){$f=Get-Content -LiteralPath (Join-Path $b ($role+'/finished.json')) -Raw|ConvertFrom-Json;if(!$f.privateFixtureRemoved -or $f.cleanup -cne 'tracked-generations-exited' -or !$f.dailyConfigBytesUnchanged){throw 'Own actors not fully cleaned'}}
 if(!$r.taskName.StartsWith('OCXD023-cross-')){throw 'Exact task name required'}
 $t=Get-ScheduledTask -TaskName $r.taskName
 if($t.State -eq 'Running' -or !$t.Actions.Arguments.Contains((Join-Path $b 'console-launch.ps1'))){throw 'Task running or action changed'}
 $report.lastTaskResult=(Get-ScheduledTaskInfo -TaskName $r.taskName).LastTaskResult
 Unregister-ScheduledTask -TaskName $r.taskName -Confirm:$false
 $report.taskRemoved=!(Get-ScheduledTask -TaskName $r.taskName -ErrorAction SilentlyContinue)
 $report.formerServiceProcesses=@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @((Get-Content -LiteralPath (Join-Path $b 'controller-trace.json') -Raw|ConvertFrom-Json|Where-Object record).record.pid)}|Select-Object ProcessId,ExecutablePath,SessionId)
 $report.formerListeners=@(Get-NetTCPConnection -LocalPort @((Get-Content -LiteralPath (Join-Path $b 'controller-trace.json') -Raw|ConvertFrom-Json|Where-Object record).record.port|Select-Object -Unique) -State Listen -ErrorAction SilentlyContinue|Select-Object LocalAddress,LocalPort,OwningProcess)
 $report.privateFixturesAbsent=(!(Test-Path -LiteralPath (Join-Path $b 'console/private')) -and !(Test-Path -LiteralPath (Join-Path $b 'rdp/private')))
}catch{$report.failure=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
finally{$report.finishedUtc=[datetime]::UtcNow.ToString('o');$report|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $b 'task-cleanup.json') -Encoding utf8}

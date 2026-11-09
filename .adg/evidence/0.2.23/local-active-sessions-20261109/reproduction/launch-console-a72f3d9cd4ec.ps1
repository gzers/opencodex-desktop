param([ValidatePattern('^sessions-[0-9]{2}$')][string]$RunName='sessions-02',[string]$RunDirectory='')
$ErrorActionPreference='Stop'
$taskRoot=if($RunDirectory){[IO.Path]::GetFullPath($RunDirectory)}else{Join-Path $PSScriptRoot $RunName}
$taskAgent='C:/Users/15119/.codex/worktrees/ocxd-023-windows-r1/code/test/smoke/windows023-active-session.ps1'
$taskPowerShell='C:/Users/15119/.cache/codex-runtimes/codex-primary-runtime/dependencies/native/powershell/pwsh.exe'
$taskConsoleSid='S-1-5-21-587408064-856152790-2583328096-500'
$taskName='ocxd023-r34-console-'+$RunName.Substring(9)
$taskIdentity=[Security.Principal.WindowsIdentity]::GetCurrent()
if(!([Security.Principal.WindowsPrincipal]::new($taskIdentity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'Actual elevated administrator required; no credential provision here'}
$taskArgs='-NoLogo -NoProfile -NonInteractive -File "'+$taskAgent+'" -RunDirectory "'+$taskRoot+'" -ProgramDirectory "'+(Join-Path $PSScriptRoot 'program')+'" -ProvisionTool "'+(Join-Path $PSScriptRoot 'fixture/ocxd-local-provision.exe')+'" -Role console -ExpectedSid '+$taskConsoleSid+' -ExpectedSessionId 1'
$taskAction=New-ScheduledTaskAction -Execute $taskPowerShell -Argument $taskArgs -WorkingDirectory $PSScriptRoot
$taskPrincipal=New-ScheduledTaskPrincipal -UserId $taskConsoleSid -LogonType Interactive -RunLevel Highest
$taskSettings=New-ScheduledTaskSettingsSet -ExecutionTimeLimit ([TimeSpan]::FromMinutes(40)) -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
if(Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue){throw 'Never replace an existing task'}
$taskRegistered=$false
try{
 Register-ScheduledTask -TaskName $taskName -Action $taskAction -Principal $taskPrincipal -Settings $taskSettings -Description 'Temporary exact 0.2.23 console diagnostic; no daily configuration' | Out-Null
 $taskRegistered=$true
 Start-ScheduledTask -TaskName $taskName
 @{name=$taskName;principal=$taskConsoleSid;requestedSession=1;launcherSid=$taskIdentity.User.Value;launcherSession=[Diagnostics.Process]::GetCurrentProcess().SessionId;createdAt=[datetime]::UtcNow.ToString('o')} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRoot 'console-launch.json')
 $until=[datetime]::UtcNow.AddMinutes(38)
 while(!(Test-Path -LiteralPath (Join-Path $taskRoot 'console/finished.json'))){
  $task=Get-ScheduledTask -TaskName $taskName
  $info=Get-ScheduledTaskInfo -TaskName $taskName
  if($task.State -ne 'Running' -and $info.LastRunTime -gt [datetime]::Now.AddMinutes(-10)){break}
  if([datetime]::UtcNow -gt $until){throw 'Diagnostic still running; retain exact task for safe cleanup'}
  Start-Sleep -Seconds 2
 }
 $info=Get-ScheduledTaskInfo -TaskName $taskName
 $info | Select-Object LastRunTime,LastTaskResult | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRoot 'console-task-exit.json')
}catch{
 @{error=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRoot 'console-launch-failure.json')
}finally{
 if($taskRegistered){
  $task=Get-ScheduledTask -TaskName $taskName
  if($task.State -ne 'Running' -and $task.Principal.UserId -in @($taskConsoleSid,'Administrator',($env:COMPUTERNAME+[char]92+'Administrator')) -and $task.Actions.Execute -eq $taskPowerShell){
   Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
   @{name=$taskName;removed=$true} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRoot 'console-task-cleanup.json')
  }
 }
}

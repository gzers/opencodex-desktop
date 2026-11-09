$ErrorActionPreference='Stop'
$taskRoot=Join-Path $PSScriptRoot 'console-exit-only02'
$taskName='ocxd023-r34-console-exit02'
$taskPowerShell='C:/Users/15119/.cache/codex-runtimes/codex-primary-runtime/dependencies/native/powershell/pwsh.exe'
$taskScript=Join-Path $PSScriptRoot 'console-exit-check.ps1'
if(Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue){throw 'Do not replace tasks'}
$action=New-ScheduledTaskAction -Execute $taskPowerShell -Argument ('-NoLogo -NoProfile -NonInteractive -File "'+$taskScript+'"') -WorkingDirectory $PSScriptRoot
$principal=New-ScheduledTaskPrincipal -UserId 'S-1-5-21-587408064-856152790-2583328096-500' -LogonType Interactive -RunLevel Highest
Register-ScheduledTask -TaskName $taskName -Action $action -Principal $principal -Settings (New-ScheduledTaskSettingsSet -ExecutionTimeLimit ([TimeSpan]::FromMinutes(12))) | Out-Null
Start-ScheduledTask -TaskName $taskName
while(!(Test-Path -LiteralPath (Join-Path $taskRoot 'finished.json'))){Start-Sleep -Seconds 1}
while((Get-ScheduledTask -TaskName $taskName).State -eq 'Running'){Start-Sleep -Seconds 1}
Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
@{name=$taskName;removed=$true} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRoot 'task-cleanup.json')

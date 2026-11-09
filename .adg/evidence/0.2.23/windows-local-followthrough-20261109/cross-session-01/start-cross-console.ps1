$ErrorActionPreference='Stop'
$root=$PSScriptRoot;$b=Join-Path $root 'cross-session-01'
if(Test-Path -LiteralPath $b){throw 'Fresh cross-session evidence required'}
New-Item -ItemType Directory -Path $b|Out-Null
$admin=Get-LocalUser|Where-Object {$_.SID.Value.EndsWith('-500')}
if(@($admin).Count -ne 1){throw 'Exact builtin console account required'}
$taskName='OCXD023-cross-'+[guid]::NewGuid().ToString('N').Substring(0,10)
$launcher=@'
$ErrorActionPreference='Stop'
$b=$PSScriptRoot
@{pid=$PID;sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value;session=[Diagnostics.Process]::GetCurrentProcess().SessionId;startedUtc=[datetime]::UtcNow.ToString('o')}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $b 'console-task-launch.json') -Encoding utf8
try{
 & 'C:\Users\15119\.codex\worktrees\ocxd-023-windows-r1\code\test\smoke\windows023-current-session.ps1' -RunDirectory $b -Role console -ProgramDirectory 'D:\ocx13-stop-build-20261109-220455\program' -ProvisionTool 'D:\ocx-win-followthrough-20261109-230145\console-fixture-build\tool\ocxd-local-provision.exe' -ExpectedSid 'EXPECTEDSID' -ExpectedSessionId 1 -ExpectedSource fdf5d4db551f71cbf493dc97b428f8b9a9364a7a
}catch{@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber;atUtc=[datetime]::UtcNow.ToString('o')}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $b 'console-task-failure.json') -Encoding utf8;throw}
'@
$launcher=$launcher.Replace('EXPECTEDSID',$admin.SID.Value)
[IO.File]::WriteAllText((Join-Path $b 'console-launch.ps1'),$launcher)
$action=New-ScheduledTaskAction -Execute 'D:\ocxu1-20261109-191803\powershell\pwsh.exe' -Argument ('-NoLogo -NoProfile -NonInteractive -WindowStyle Hidden -File "'+(Join-Path $b 'console-launch.ps1')+'"') -WorkingDirectory $b
$principal=New-ScheduledTaskPrincipal -UserId ($env:COMPUTERNAME+[char]92+$admin.Name) -LogonType Interactive -RunLevel Highest
$settings=New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit ([timespan]::FromMinutes(30))
Register-ScheduledTask -TaskName $taskName -Action $action -Principal $principal -Settings $settings|Out-Null
@{taskName=$taskName;sid=$admin.SID.Value;account=$admin.Name;expectedSession=1;harnessSource='e023355b5';productSource='fdf5d4db551f71cbf493dc97b428f8b9a9364a7a';scope='Only fresh owned Sandbox; console/RDP real overlap, fixed conflict and normal Stop peer preservation';registeredUtc=[datetime]::UtcNow.ToString('o')}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $b 'task-registration.json') -Encoding utf8
Start-ScheduledTask -TaskName $taskName

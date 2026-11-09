$ErrorActionPreference='Stop'
$taskRoot=$PSScriptRoot
@{controllerPid=$PID;sessionId=[Diagnostics.Process]::GetCurrentProcess().SessionId;directory=$taskRoot;launchUtc=[datetime]::UtcNow.ToString('o');scope='Fresh lifecycle.3 Desktop; one actual user1 Start only'}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $taskRoot 'controller-launch.json') -Encoding utf8
try{
 & 'C:\Users\15119\.codex\worktrees\ocxd-023-windows-r1\code\test\smoke\windows023-user1-start-stop.ps1' -Mode Controller -Directory $taskRoot -ProgramDirectory 'D:\ocxu1-lifecycle3-build-20261109-212335\program' -Node 'D:\ocxu1-lifecycle3-build-20261109-212335\tools\runtime\node\node.exe' -ExpectedSource 'd3b34a234e32b63365672052772c2c72c719f97c' -ExpectedBuildRun '42625073747'
}catch{
 @{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber;atUtc=[datetime]::UtcNow.ToString('o')}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $taskRoot 'controller-launch-failure.json') -Encoding utf8
 throw
}
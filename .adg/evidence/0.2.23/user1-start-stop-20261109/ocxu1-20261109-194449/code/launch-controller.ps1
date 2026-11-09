$ErrorActionPreference='Stop'
$taskRoot=$PSScriptRoot
@{controllerPid=$PID;sessionId=[Diagnostics.Process]::GetCurrentProcess().SessionId;directory=$taskRoot;launchUtc=[datetime]::UtcNow.ToString('o');scope='New active-RDP comparison, one user1; old evidence retained'}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $taskRoot 'controller-launch.json') -Encoding utf8
try{
 & 'C:\Users\15119\.codex\worktrees\ocxd-023-windows-r1\code\test\smoke\windows023-user1-start-stop.ps1' -Mode Controller -Directory $taskRoot -ProgramDirectory 'D:\ocxu1-20261109-190955\program' -Node 'D:\ocxu1-20261109-190955\tools\runtime\node\node.exe'
}catch{
 @{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber;atUtc=[datetime]::UtcNow.ToString('o')}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $taskRoot 'controller-launch-failure.json') -Encoding utf8
 throw
}
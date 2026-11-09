$ErrorActionPreference='Stop'
$taskRoot=$PSScriptRoot
@{controllerPid=$PID;sessionId=[Diagnostics.Process]::GetCurrentProcess().SessionId;directory=$taskRoot;launchUtc=[datetime]::UtcNow.ToString('o');scope='Item13 HOME comparison fix; one actual user1 Start then normal Stop'}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $taskRoot 'controller-launch.json') -Encoding utf8
try{
 & 'C:\Users\15119\.codex\worktrees\ocxd-023-windows-r1\code\test\smoke\windows023-user1-start-stop.ps1' -Mode Controller -Directory $taskRoot -ProgramDirectory 'D:\ocx13-stop-build-20261109-220455\program' -Node 'D:\ocx13-stop-build-20261109-220455\tools\runtime\node\node.exe' -ExpectedSource 'fdf5d4db551f71cbf493dc97b428f8b9a9364a7a' -ExpectedBuildRun '42631183420'
}catch{
 @{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber;atUtc=[datetime]::UtcNow.ToString('o')}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $taskRoot 'controller-launch-failure.json') -Encoding utf8
 throw
}
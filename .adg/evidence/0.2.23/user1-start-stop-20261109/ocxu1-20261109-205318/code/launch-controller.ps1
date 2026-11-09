$ErrorActionPreference='Stop'
$taskRoot=$PSScriptRoot
@{controllerPid=$PID;sessionId=[Diagnostics.Process]::GetCurrentProcess().SessionId;directory=$taskRoot;launchUtc=[datetime]::UtcNow.ToString('o');scope='New-source output diagnosis; one actual user1 Start only'}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $taskRoot 'controller-launch.json') -Encoding utf8
try{
 & 'C:\Users\15119\.codex\worktrees\ocxd-023-windows-r1\code\test\smoke\windows023-user1-start-stop.ps1' -Mode Controller -Directory $taskRoot -ProgramDirectory 'D:\ocxu1-output-build-20261109-204221\program' -Node 'D:\ocxu1-output-build-20261109-204221\tools\runtime\node\node.exe' -ExpectedSource 'a4abf17aa7aa2b9f677c20aa3d30cda59deed5b4' -ExpectedBuildRun '42619237040'
}catch{
 @{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber;atUtc=[datetime]::UtcNow.ToString('o')}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $taskRoot 'controller-launch-failure.json') -Encoding utf8
 throw
}
$ErrorActionPreference='Stop'
$batch=$PSScriptRoot
@{controllerPid=$PID;sessionId=[Diagnostics.Process]::GetCurrentProcess().SessionId;launchedUtc=[datetime]::UtcNow.ToString('o');directory=$batch}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $batch 'controller-launch.json') -Encoding utf8
try{
 & 'C:\Users\15119\.codex\worktrees\ocxd-023-windows-r1\code\test\smoke\windows023-local-two-user.ps1' -LocalWorkstation -ElevatedFirstAccount -Directory $batch -ProgramDirectory 'D:\ocx13-stop-build-20261109-220455\program' -Node 'D:\ocx13-stop-build-20261109-220455\tools\runtime\node\node.exe' -ExpectedSource fdf5d4db551f71cbf493dc97b428f8b9a9364a7a -ExpectedBuildRun 42631183420
}catch{@{error=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber;timeUtc=[datetime]::UtcNow.ToString('o')}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $batch 'launch-failure.json') -Encoding utf8;throw}
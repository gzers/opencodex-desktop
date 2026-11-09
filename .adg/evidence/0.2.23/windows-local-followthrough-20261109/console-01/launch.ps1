$ErrorActionPreference='Stop'
$b=$PSScriptRoot
@{pid=$PID;sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value;session=[Diagnostics.Process]::GetCurrentProcess().SessionId;startedUtc=[datetime]::UtcNow.ToString('o')}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $b 'task-launch.json') -Encoding utf8
try{
 & 'C:\Users\15119\.codex\worktrees\ocxd-023-windows-r1\code\test\smoke\windows023-console-lifecycle.ps1' -Directory $b -ProgramDirectory 'D:\ocx13-stop-build-20261109-220455\program' -ProvisionTool 'D:\ocx-win-followthrough-20261109-230145\console-fixture-build\tool\ocxd-local-provision.exe' -ExpectedSid 'S-1-5-21-587408064-856152790-2583328096-500' -ExpectedSessionId 1 -ExpectedSource fdf5d4db551f71cbf493dc97b428f8b9a9364a7a -ExpectedBuildRun 42631183420
}catch{@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber;atUtc=[datetime]::UtcNow.ToString('o')}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $b 'task-failure.json') -Encoding utf8;throw}
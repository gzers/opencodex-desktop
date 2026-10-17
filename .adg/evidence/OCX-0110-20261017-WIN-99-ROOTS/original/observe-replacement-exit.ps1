param([string]$RunDirectory)
$ErrorActionPreference='Stop'
$taskContract=Get-Content -LiteralPath (Join-Path $RunDirectory 'contract.json') -Raw | ConvertFrom-Json
$taskOrder=Get-Content -LiteralPath (Join-Path $RunDirectory 'recovery-process-order.json') -Raw | ConvertFrom-Json
if(!$taskOrder.processSubsetPassed){throw 'Recovery order not passed'}
. (Join-Path $RunDirectory 'retained-process-times.ps1')
$taskProcess=[Diagnostics.Process]::GetProcessById([int]$taskOrder.newPid)
try {
 [void]$taskProcess.SafeHandle
 $taskBefore=Get-WindowsRetainedTimesEvidence -Process $taskProcess -ExpectedCreationFileTime ([long]$taskOrder.newKernelTimes.creationFileTime)
 $taskBefore | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $RunDirectory 'replacement-retained-handle.json') -Encoding utf8
 $taskHuman=Get-Content -LiteralPath (Join-Path $RunDirectory 'human-window.json') -Raw | ConvertFrom-Json
 $taskDuration=(([datetimeoffset]$taskHuman.expiresAtUtc).AddSeconds(60)-[datetimeoffset]::UtcNow).TotalMilliseconds
 if($taskDuration -le 0 -or !$taskProcess.WaitForExit([int]$taskDuration)){throw 'Normal close not observed'}
 $taskTimes=Get-WindowsRetainedTimesEvidence -Process $taskProcess -ExpectedCreationFileTime ([long]$taskOrder.newKernelTimes.creationFileTime) -RequireExited
 [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskProcess.Id;kernelTimes=$taskTimes;exitCode=$taskProcess.ExitCode;nativeCount=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count;forcedTermination=$false} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $RunDirectory 'replacement-exit.json') -Encoding utf8
} finally {$taskProcess.Dispose()}
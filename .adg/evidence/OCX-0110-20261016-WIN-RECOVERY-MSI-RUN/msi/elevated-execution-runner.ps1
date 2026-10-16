$ErrorActionPreference='Stop'
$taskRun='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop\test\runtime\host-msi-prepared-20261016T1740'
try {
    & 'C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop\test\runtime\host-msi-execution-20261016T1740.ps1' -RunDirectory $taskRun -Execute *> (Join-Path $taskRun 'elevated-execution.original.log')
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');completed=$true;exitCode=0} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRun 'runner-result.json') -Encoding utf8
    exit 0
} catch {
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');completed=$false;exceptionType=$_.Exception.GetType().FullName;message=$_.Exception.Message;scriptStackTrace=$_.ScriptStackTrace;automaticRetry=$false} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRun 'runner-result.json') -Encoding utf8
    exit 1
}
param([string]$RunDirectory)
$ErrorActionPreference='Stop'
$taskLaunch=Get-Content -LiteralPath (Join-Path $RunDirectory 'recovery-launch.json') -Raw | ConvertFrom-Json
$taskSource='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop'
. (Join-Path $taskSource 'test/smoke/windows-gpu-resource.ps1')
try {
    $taskSnapshot=Get-WindowsGpuResourceSnapshot -Launch $taskLaunch -Mode ordinary
    $taskSnapshot | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $RunDirectory 'gpu-snapshot.original.json') -Encoding utf8
} catch {
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');exceptionType=$_.Exception.GetType().FullName;message=$_.Exception.Message;hresult=$_.Exception.HResult;gatePassed=$false;retryAttempted=$false} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $RunDirectory 'gpu-snapshot.failure.json') -Encoding utf8
    throw
}

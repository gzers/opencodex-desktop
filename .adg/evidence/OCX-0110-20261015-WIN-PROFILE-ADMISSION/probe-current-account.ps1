$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskSource=(& git -C $taskCode rev-parse HEAD).Trim()
if ($taskSource -cne '493da80675e6508e46e0fd83d86ee7216abf96fe') { throw 'Probe helper source changed' }
if (@(& git -C $taskCode status --porcelain).Count) { throw 'Code worktree is not clean' }
$taskMethod=Join-Path $taskCode 'test/smoke/windows-acceptance-profile.ps1'
. $taskMethod
$taskIdentity=[Security.Principal.WindowsIdentity]::GetCurrent()
try { $taskDailySid=$taskIdentity.User.Value } finally { $taskIdentity.Dispose() }
# Intentionally bind both SIDs to the actual daily account for a native negative
# control. This does not assert that a proposed test account exists or is ready.
$taskResult=Get-WindowsAcceptanceProfile -DailySid $taskDailySid -ExpectedSid $taskDailySid `
    -ExpectedProfile 'C:\Users\ocx0110_accept'
$taskResult.negativeControl='actual-current-daily-account; ExpectedSid intentionally equals DailySid'
$taskResult.sourceCommit=$taskSource
$taskResult.proposedTestAccountExists=($null -ne (Get-LocalUser -Name 'ocx0110_accept' -ErrorAction SilentlyContinue))
$taskResult.methodMatchesCurrentFile=($taskResult.methodSha256 -ceq (Get-FileHash -LiteralPath $taskMethod).Hash.ToLowerInvariant())
$taskOutput=Join-Path $PSScriptRoot 'native-current-account-rejection.original.json'
if (Test-Path -LiteralPath $taskOutput) { throw 'Do not overwrite the original native probe' }
$taskJson=$taskResult | ConvertTo-Json -Depth 10
[IO.File]::WriteAllText($taskOutput,$taskJson+"`n",[Text.UTF8Encoding]::new($false))
$taskJson
if ($taskResult.admitted -or 'daily-account' -notin $taskResult.reasons -or
    !$taskResult.dailySidMatched -or !$taskResult.methodMatchesCurrentFile) {
    throw 'Actual daily account was not rejected with the expected method identity'
}

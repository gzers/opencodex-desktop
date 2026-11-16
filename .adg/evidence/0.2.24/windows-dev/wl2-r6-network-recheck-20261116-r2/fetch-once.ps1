$taskEvidence=$PSScriptRoot
$taskStarted=[DateTimeOffset]::Now
$taskWatch=[Diagnostics.Stopwatch]::StartNew()
$taskArguments=@('fetch','origin','main','docs/governance-main')
& git @taskArguments 1> "$taskEvidence/fetch.stdout.log" 2> "$taskEvidence/fetch.stderr.log"
$taskRawExit=$LASTEXITCODE
$taskElapsed=$taskWatch.ElapsedMilliseconds
$taskCachedMain=(& git rev-parse origin/main)
$taskCachedGovernance=(& git rev-parse origin/docs/governance-main)
$taskSource=(& git rev-parse HEAD)
@{started=$taskStarted.ToString('o');finished=([DateTimeOffset]::Now).ToString('o');elapsedMs=$taskElapsed;program='git';arguments=$taskArguments;gitRawExitCode=$taskRawExit;latestRemoteInputsConfirmed=($taskRawExit -eq 0);main=$taskCachedMain;governanceMain=$taskCachedGovernance;sourceHead=$taskSource;persistentConfigChanged=$false;certificateVerificationDisabled=$false;newNativeTestsRun=$false;newPushAttempted=$false;publicStartOpen=$false} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath "$taskEvidence/fetch-result.json" -Encoding utf8
exit $taskRawExit

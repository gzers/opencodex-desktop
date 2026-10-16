param(
    [Parameter(Mandatory)][ValidateSet('baseline019','candidate0110')][string]$Version,
    [Parameter(Mandatory)][string]$RunDirectory
)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'run-binding.ps1')
$taskRun=Assert-PreparedIdleRun -RunDirectory $RunDirectory -RequireCurrentHumanWindow
$taskDiagnostic='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop'
$taskLaunch=Join-Path $taskRun ('idle-'+$Version+'-launch.json')
$taskReady=Join-Path $taskRun ($Version+'-sky-settled-ready.json')
$taskOutput=Join-Path $taskRun $Version
$taskContract=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'contract.json') -Raw | ConvertFrom-Json
$taskIdentity=Get-Content -LiteralPath $taskLaunch -Raw | ConvertFrom-Json
$taskExpectedSource=if($Version -eq 'baseline019'){$taskContract.baselineSource}else{$taskContract.candidateSource}
$taskExpectedHash=if($Version -eq 'baseline019'){$taskContract.baselineExeSha256}else{$taskContract.candidateExeSha256}
if($taskIdentity.source -cne $taskExpectedSource -or $taskIdentity.exeSha256 -cne $taskExpectedHash){
    throw 'Launch identity differs from the prepared pair'
}
$taskHuman=Get-Content -LiteralPath (Join-Path $taskRun 'human-window.json') -Raw | ConvertFrom-Json
if((([datetimeoffset]$taskHuman.expiresAtUtc)-[datetimeoffset]::UtcNow).TotalSeconds -lt 620){
    throw 'Fewer than 620 seconds remain in the human window; do not start another measurement'
}
& (Join-Path $taskDiagnostic 'test/smoke/windows-idle-observe.ps1') `
    -LaunchPath $taskLaunch -ReadinessPath $taskReady -OutputDirectory $taskOutput

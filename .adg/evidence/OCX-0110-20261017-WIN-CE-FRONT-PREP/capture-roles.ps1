param(
    [Parameter(Mandatory)][ValidateSet('baseline019','candidate0110')][string]$Version,
    [Parameter(Mandatory)][ValidateSet('settled','postspan')][string]$Moment,
    [Parameter(Mandatory)][string]$RunDirectory
)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'run-binding.ps1')
$taskRun=Assert-PreparedIdleRun -RunDirectory $RunDirectory -RequireCurrentHumanWindow
$taskContract=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'contract.json') -Raw | ConvertFrom-Json
if((Get-FileHash -LiteralPath $taskContract.rolesMethodPath).Hash.ToLowerInvariant() -cne $taskContract.rolesMethodSha256){throw 'Committed role observation method changed'}
$taskLaunch=Get-Content -LiteralPath (Join-Path $taskRun ('idle-'+$Version+'-launch.json')) -Raw | ConvertFrom-Json
$taskPrefix=if($Version -eq 'baseline019'){'baseline'}else{'candidate'}
if($taskLaunch.source -cne $taskContract.($taskPrefix+'Source') -or
   $taskLaunch.exeSha256 -cne $taskContract.($taskPrefix+'ExeSha256')){throw 'Actual generation must bind to this version'}
$taskResult=Join-Path $taskRun ($Version+'-'+$Moment+'-roles.original.json')
$taskAttempt=Join-Path $taskRun ($Version+'-'+$Moment+'-roles-attempt.json')
if((Test-Path -LiteralPath $taskAttempt) -or (Test-Path -LiteralPath $taskResult)){throw 'One role snapshot per declared moment; do not retry'}
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');pid=$taskLaunch.pid
    creationTimeUtc=$taskLaunch.creationTimeUtc;methodSha256=$taskContract.rolesMethodSha256
    snapshotOnly=$true;formalCounterSpanMeasured=$false;automaticRetry=$false} |
    ConvertTo-Json | Set-Content -LiteralPath $taskAttempt -Encoding utf8
. $taskContract.rolesMethodPath
try {
    Get-WindowsGpuResourceSnapshot -Launch $taskLaunch -Mode ordinary |
        ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $taskResult -Encoding utf8
}catch{
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');exceptionType=$_.Exception.GetType().FullName
        message=$_.Exception.Message;hresult=$_.Exception.HResult;automaticRetry=$false
        acceptanceClaimed=$false} | ConvertTo-Json |
        Set-Content -LiteralPath (Join-Path $taskRun ($Version+'-'+$Moment+'-roles-failure.json')) -Encoding utf8
    throw
}

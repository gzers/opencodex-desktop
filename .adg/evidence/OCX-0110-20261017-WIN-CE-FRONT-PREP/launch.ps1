param(
    [Parameter(Mandatory)][ValidateSet('baseline019','candidate0110')][string]$Version,
    [Parameter(Mandatory)][string]$RunDirectory
)
$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskDiagnostic='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop'
. (Join-Path $PSScriptRoot 'run-binding.ps1')
$taskRun=Assert-PreparedIdleRun -RunDirectory $RunDirectory -RequireCurrentHumanWindow
$taskSource=(git -c core.longpaths=true -C $taskDiagnostic rev-parse HEAD).Trim()
if($taskSource -cne '768e231811a2363b32f5fa67c8e74cbc31d751e1'){throw 'Collector source changed'}
$taskCi=Get-Content -LiteralPath (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261016-WIN-IDLE-D94-F227/ci-status.json') -Raw | ConvertFrom-Json
if($taskCi.status -ne 'completed' -or $taskCi.conclusion -ne 'success'){throw 'Exact collector CI is not passed'}
$taskOutput=Join-Path $taskRun ('idle-'+$Version+'-launch.json')
if(Test-Path -LiteralPath $taskOutput){throw 'Do not overwrite or retry this declared attempt'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native product already exists'}
$taskContract=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'contract.json') -Raw | ConvertFrom-Json
$taskCandidate=Get-Content -LiteralPath (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261017-WIN-CE-CANDIDATE/original/candidate-descriptor.json') -Raw | ConvertFrom-Json
if(!(Test-Path -LiteralPath (Join-Path $taskRun 'fixture-ready.json'))){throw 'Future window fixture clone not recorded'}
if($Version -eq 'candidate0110'){
    $taskHuman=Get-Content -LiteralPath (Join-Path $taskRun 'human-window.json') -Raw | ConvertFrom-Json
    if((([datetimeoffset]$taskHuman.expiresAtUtc)-[datetimeoffset]::UtcNow).TotalSeconds -lt 840){throw 'Insufficient candidate settling, counting and normal-exit margin'}
    $taskPrevious=Get-Content -LiteralPath (Join-Path $taskRun 'baseline019-exit.json') -Raw | ConvertFrom-Json
    if(!$taskPrevious.exitObserved -or $taskPrevious.exitCode -ne 0){throw 'Baseline normal exit not proven'}
}
$taskExe=if($Version -eq 'candidate0110'){$taskCandidate.executable}else{Join-Path $taskCode 'test/software/acceptance-kit-02a0d4-20261015/bin/baseline019/opencodex-desktop.exe'}
$taskHash=if($Version -eq 'candidate0110'){$taskContract.candidateExeSha256}else{$taskContract.baselineExeSha256}
$taskProductSource=if($Version -eq 'candidate0110'){$taskContract.candidateSource}else{$taskContract.baselineSource}
if((Get-FileHash -LiteralPath $taskExe).Hash.ToLowerInvariant() -cne $taskHash){throw 'Executable digest changed'}
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
. $taskContract.sidMethodPath
$taskInfo=[Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName=$taskExe
$taskInfo.WorkingDirectory=Split-Path -Parent $taskExe
$taskInfo.UseShellExecute=$false
foreach($taskName in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY','OPENCODEX_WINDOWS_SMOKE_CDP_PORT','OPENCODEX_WINDOWS_SMOKE_SCALE','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','WEBVIEW2_USER_DATA_FOLDER')){
    [void]$taskInfo.Environment.Remove($taskName)
}
$taskProcess=[Diagnostics.Process]::Start($taskInfo)
try {
    [void]$taskProcess.SafeHandle
    $taskToken=Get-AcceptanceProcessSidEvidence -Process $taskProcess -ExpectedExecutable $taskExe
    Assert-WindowsProcessMode -Observation $taskToken -Mode ordinary
    if($taskToken.sidSha256 -cne '373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310'){throw 'User changed'}
    [ordered]@{
        atUtc=[datetimeoffset]::UtcNow.ToString('o');phase='idle-'+$Version;mode='ordinary'
        source=$taskProductSource;executable=$taskExe;exeSha256=$taskHash
        pid=$taskProcess.Id;creationTimeUtc=$taskToken.creationTimeUtc
        applicationToken=$taskToken;sandboxUsed=$false;cdpUsed=$false
        collectorSource=$taskSource;collectorMethodSource=$taskContract.collectorSource
        humanWindowReceiptSha256=(Get-FileHash -LiteralPath (Join-Path $taskRun 'human-window.json')).Hash.ToLowerInvariant()
    } | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath $taskOutput -Encoding utf8
}finally{$taskProcess.Dispose()}

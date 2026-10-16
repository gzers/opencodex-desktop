param([Parameter(Mandatory)][string]$TestRunDirectory)
$ErrorActionPreference='Stop'
$taskContract=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'contract.json') -Raw | ConvertFrom-Json
$taskAllowed='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop\test\runtime'
$taskRun=[IO.Path]::GetFullPath($TestRunDirectory)
if(!$taskRun.StartsWith($taskAllowed+'\',[StringComparison]::OrdinalIgnoreCase) -or
   (Test-Path -LiteralPath $taskRun)){throw 'Use a fresh task-owned rejection-test directory'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native product exists'}
New-Item -ItemType Directory -Path $taskRun | Out-Null
$taskParsed=foreach($taskFile in Get-ChildItem -LiteralPath $PSScriptRoot -Filter '*.ps1'){
    $taskTokens=$null;$taskErrors=$null
    [void][Management.Automation.Language.Parser]::ParseFile($taskFile.FullName,[ref]$taskTokens,[ref]$taskErrors)
    if($taskErrors.Count){throw ('Parse failed: '+$taskFile.Name)}
    [ordered]@{path=$taskFile.Name;errors=0;sha256=(Get-FileHash -LiteralPath $taskFile.FullName).Hash.ToLowerInvariant()}
}
$taskKey='HKCU:\Software\com.gzers.opencodex.desktop'
$taskPointerBefore=(Get-ItemProperty -LiteralPath $taskKey).StartupDataRootV1
$taskAclBefore=(Get-Acl -LiteralPath $taskContract.recoveryDirectory).GetSecurityDescriptorSddlForm([Security.AccessControl.AccessControlSections]::Access)
$taskRejected=@()
foreach($taskEntry in @(
    @{path='launch.ps1';parameters=@{Version='baseline019';RunDirectory=$taskRun}},
    @{path='prepare-fixture.ps1';parameters=@{RunDirectory=$taskRun}},
    @{path='prepare-recovery.ps1';parameters=@{RunDirectory=$taskRun}},
    @{path='launch-recovery.ps1';parameters=@{RunDirectory=$taskRun}}
)){
    $taskFailure=$null
    $taskParameters=$taskEntry.parameters
    try{& (Join-Path $PSScriptRoot $taskEntry.path) @taskParameters}catch{$taskFailure=$_}
    if($null -eq $taskFailure -or $taskFailure.ToString() -notmatch 'human-window\.json'){
        [ordered]@{path=$taskEntry.path;expected='missing human-window.json'
            actualType=if($taskFailure){$taskFailure.Exception.GetType().FullName}else{$null}
            actualMessage=if($taskFailure){$taskFailure.Exception.Message}else{$null}} |
            ConvertTo-Json | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'guard-check-failure.json') -Encoding utf8
        throw ('Missing-human refusal not established: '+$taskEntry.path)
    }
    $taskRejected+=@{path=$taskEntry.path;rejected=$true;reason='missing human-window.json';exceptionType=$taskFailure.Exception.GetType().FullName}
}
if((Get-ItemProperty -LiteralPath $taskKey).StartupDataRootV1 -cne $taskPointerBefore){throw 'Guard check changed pointer'}
if((Get-Acl -LiteralPath $taskContract.recoveryDirectory).GetSecurityDescriptorSddlForm([Security.AccessControl.AccessControlSections]::Access) -cne $taskAclBefore){throw 'Guard check changed ACL'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Guard check started native product'}
if(@(Get-ChildItem -LiteralPath $taskRun -Force).Count){throw 'Guard check wrote action records'}
foreach($taskPair in @(@($taskContract.baselineExecutable,$taskContract.baselineExeSha256),@($taskContract.candidateExecutable,$taskContract.candidateExeSha256))){
    if((Get-FileHash -LiteralPath $taskPair[0]).Hash.ToLowerInvariant() -cne $taskPair[1]){throw 'Exact paired executable changed'}
}
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');status='passed';parsedFiles=$taskParsed
    rejectedWithoutHuman=$taskRejected;invocation='explicit named-parameter hashtable splats'
    candidateSource=$taskContract.candidateSource;candidateExeSha256=$taskContract.candidateExeSha256
    baselineExeSha256=$taskContract.baselineExeSha256
    pointerUnchanged=$true;aclUnchanged=$true;actionRecordsWritten=$false
    profileMoved=$false;productStarted=$false;foregroundActivated=$false
    freshHumanWindowGranted=$false;nativeRecoveryGatePassed=$false;fullPerformanceGatePassed=$false
} | ConvertTo-Json -Depth 9 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'preparation-checks.json') -Encoding utf8
Write-Output ('Preparation checks passed: '+$taskParsed.Count+' parsed scripts and '+$taskRejected.Count+' missing-human rejections.')

param([Parameter(Mandatory=$true)][ValidateSet('baseline019','candidate0110')][string]$Version)
$ErrorActionPreference='Stop'
$taskOutput=Join-Path $PSScriptRoot ($Version+'-role-snapshot.json')
if(Test-Path -LiteralPath $taskOutput){throw 'Keep original role observation; do not resample to select a result'}
$taskLaunch=Get-Content -LiteralPath (Join-Path $PSScriptRoot ('idle-'+$Version+'-launch.json')) -Raw | ConvertFrom-Json
$taskSample=Get-Content -LiteralPath (Join-Path $PSScriptRoot ($Version+'-samples.original.jsonl')) -Tail 1 | ConvertFrom-Json
if($taskSample.elapsedSeconds -lt 120 -or $taskSample.elapsedSeconds -gt 300){throw 'Role observation is bounded to the predeclared 120-300s part of this idle window'}
$taskRoles=@()
foreach($taskMember in $taskSample.members){
    if($taskMember.exited){
        $taskRoles+=@{pid=$taskMember.pid;creationUtc=$taskMember.creationUtc;role='exited-role-unavailable';counterCpuRetained=$true;commandLinePublished=$false}
        continue
    }
    $taskRow=Get-CimInstance Win32_Process -Filter ('ProcessId='+[int]$taskMember.pid)
    if(!$taskRow -or [Math]::Abs($taskRow.CreationDate.ToUniversalTime().Ticks-([datetimeoffset]$taskMember.creationUtc).UtcTicks) -gt 10){
        $taskRoles+=@{pid=$taskMember.pid;creationUtc=$taskMember.creationUtc;role='role-unavailable-after-sample';counterCpuRetained=$true;commandLinePublished=$false}
        continue
    }
    $taskRole=if($taskMember.pid -eq $taskLaunch.pid){'application'}elseif($taskRow.CommandLine -match '--type=([^\s"]+)'){$Matches[1]}else{'browser'}
    $taskSubtype=if($taskRow.CommandLine -match '--utility-sub-type=([^\s"]+)'){$Matches[1]}else{$null}
    $taskRoles+=@{pid=$taskMember.pid;creationUtc=$taskMember.creationUtc;parentPid=$taskRow.ParentProcessId;role=$taskRole;utilitySubtype=$taskSubtype;commandLinePublished=$false}
}
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');version=$Version;source=$taskLaunch.source
    sampleElapsedSeconds=$taskSample.elapsedSeconds;members=$taskRoles
    inputInjected=$false;counterSamplingChanged=$false;originalCommandLinesPublished=$false
} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $taskOutput -Encoding utf8
Write-Output ('Recorded '+$Version+' observed process roles without publishing command lines.')

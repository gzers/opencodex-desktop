param(
    [Parameter(Mandatory)][string]$LaunchManifest,
    [Parameter(Mandatory)][string]$OutputFile
)
$ErrorActionPreference='Stop'
$taskRepo=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../..'))
$taskWorkspace=Split-Path -Parent $taskRepo
$taskTools=Join-Path $taskWorkspace '.toolchain'
$taskEvidence=Join-Path $taskTools 'post021-msvc-followup'
$taskLaunchPath=[IO.Path]::GetFullPath($LaunchManifest)
$taskOutput=[IO.Path]::GetFullPath($OutputFile)
if(!$taskOutput.StartsWith($taskEvidence+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)){
    throw 'Target identity must be saved below the owned followup directory'
}
if(Test-Path -LiteralPath $taskOutput){throw 'Fresh target identity required; do not overwrite stale process evidence'}
$taskAllowedRoots=@(
    (Join-Path $taskTools 'post021-windows-final-msvc-41913003716'),
    (Join-Path $taskTools 'post021-windows-final-msvc-retry-41944607861')
)
if(!@($taskAllowedRoots|Where-Object {$taskLaunchPath.StartsWith($_+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)}).Count){
    throw 'Explicit verified MSVC test launch manifest required'
}
$taskLaunch=Get-Content -LiteralPath $taskLaunchPath -Raw -Encoding utf8|ConvertFrom-Json
$taskExe=[IO.Path]::GetFullPath([string]$taskLaunch.exe)
if((Split-Path -Parent $taskExe) -ine (Split-Path -Parent $taskLaunchPath)){
    throw 'Launch manifest and executable must be in the same admitted directory'
}
$taskP=Get-CimInstance Win32_Process -Filter ('ProcessId = '+[uint32]$taskLaunch.pid)
if(!$taskP -or $taskP.ExecutablePath -ine $taskExe){throw 'Recorded launch PID/path is not running; no identity written'}
$taskHash=(Get-FileHash -LiteralPath $taskExe -Algorithm SHA256).Hash.ToLowerInvariant()
if($taskHash -cne [string]$taskLaunch.sha256){throw 'Recorded launch binary hash no longer matches'}
if($taskHash -notin @(
    'dd6f37cf2f6094cf9254ead08217cf8456e190cb213d3198c1b528172f1a0184',
    'fa9df83a55153f764673781e85167093d20602fd2e34742c69f6b2f5b37f9c68'
)){throw 'Unreviewed native executable hash'}
$taskOwner=Invoke-CimMethod -InputObject $taskP -MethodName GetOwner
if($taskOwner.ReturnValue -ne 0){throw 'Target owner is unknown'}
$taskParent=Split-Path -Parent $taskOutput
if(!(Test-Path -LiteralPath $taskParent)){throw 'Create the explicit owned handoff directory first'}
$taskIdentity=[ordered]@{
    pid=$taskP.ProcessId;exe=$taskExe;sha256=$taskHash
    createdAtUtcTicks=[string]$taskP.CreationDate.ToUniversalTime().Ticks
    sessionId=$taskP.SessionId;ownerDomain=$taskOwner.Domain;ownerUser=$taskOwner.User
    launchManifest=$taskLaunchPath;observedAtUtc=[datetimeoffset]::UtcNow.ToString('o')
    scope='Read-only identity for this exact live test process; never reusable after restart'
}
$taskIdentity|ConvertTo-Json -Depth 5|Set-Content -LiteralPath $taskOutput -Encoding utf8
Write-Output ('Saved live target PID '+$taskP.ProcessId+' with creation time, owner, session and exact binary hash')

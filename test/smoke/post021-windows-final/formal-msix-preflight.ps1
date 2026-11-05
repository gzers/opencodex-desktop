param(
    [string]$ExpectedUser='Administrator',
    [string]$TestRoot,
    [Parameter(Mandatory)][string]$OutputDirectory
)
$ErrorActionPreference='Stop'
# Read-only admission. This script never starts, stops or restarts an app,
# changes configuration, authenticates a user, or invokes the restart ladder.
$taskIdentity=[Security.Principal.WindowsIdentity]::GetCurrent()
$taskUser=$taskIdentity.Name.Split('\')[-1]
if($taskUser -ine $ExpectedUser){throw 'Wrong Windows owner; run this preflight inside the authorized test account'}
$taskWorkspace=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../../..'))
$taskBoundary=Join-Path $taskWorkspace '.toolchain/post021-msvc-followup'
if($TestRoot){$taskBoundary=[IO.Path]::GetFullPath($TestRoot)}
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
if(!$taskOutput.StartsWith($taskBoundary+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)){
    throw 'Output must be a fresh child of the explicit test root'
}
$taskAncestor=$taskOutput
while($taskAncestor){
    if((Test-Path -LiteralPath $taskAncestor) -and ((Get-Item -LiteralPath $taskAncestor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Reparse-point output boundary refused'}
    $taskParent=Split-Path -Parent $taskAncestor
    if($taskParent -eq $taskAncestor){break}
    $taskAncestor=$taskParent
}
if(Test-Path -LiteralPath $taskOutput){throw 'Fresh preflight output directory required'}
$taskSelf=Get-CimInstance Win32_Process -Filter ('ProcessId='+$PID)
$taskHome=[Environment]::GetFolderPath('UserProfile')
$taskConfig=Join-Path $taskHome '.codex/config.toml'
$taskExistingConfig=Test-Path -LiteralPath $taskConfig
$taskConfigState=[ordered]@{path=$taskConfig;exists=$taskExistingConfig}
if($taskExistingConfig){
    $taskConfigState.bytes=(Get-Item -LiteralPath $taskConfig).Length
    $taskConfigState.sha256=(Get-FileHash -LiteralPath $taskConfig -Algorithm SHA256).Hash.ToLowerInvariant()
}
$taskCodexHome=Join-Path $taskHome '.codex'
$taskExistingState=(Test-Path -LiteralPath $taskCodexHome) -and (@(Get-ChildItem -LiteralPath $taskCodexHome -Force|Select-Object -First 1).Count -gt 0)
$taskPackages=@();$taskPackageError=$null
try{
    Import-Module Appx -ErrorAction Stop
    $taskPackages=@(Get-AppxPackage -Name OpenAI.Codex)
    if(!$taskPackages.Count){$taskPackages=@(Get-AppxPackage -Name OpenAI.CodexBeta)}
}catch{$taskPackageError=$_.Exception.Message}
$taskOwn=@();$taskOtherCount=0;$taskUnknownCount=0
foreach($taskP in @(Get-CimInstance Win32_Process -Filter "Name='ChatGPT.exe'")){
    $taskOwner=Invoke-CimMethod -InputObject $taskP -MethodName GetOwner
    if($taskOwner.ReturnValue -ne 0){$taskUnknownCount++;continue}
    $taskOwnerName=$taskOwner.Domain+'\'+$taskOwner.User
    if($taskOwnerName -ine $taskIdentity.Name){$taskOtherCount++;continue}
    $taskOwn += [ordered]@{pid=$taskP.ProcessId;parentPid=$taskP.ParentProcessId;path=$taskP.ExecutablePath;createdAtUtcTicks=[string]$taskP.CreationDate.ToUniversalTime().Ticks;sessionId=$taskP.SessionId;owner=$taskOwnerName}
}
$taskAncestry=@();$taskSeen=@{};$taskCurrent=$taskSelf;$taskAncestryError=$null
for($taskHop=0;$taskCurrent -and $taskHop -lt 24;$taskHop++){
    if($taskSeen.ContainsKey([string]$taskCurrent.ProcessId)){$taskAncestryError='cycle';break}
    $taskSeen[[string]$taskCurrent.ProcessId]=$true
    $taskOwner=Invoke-CimMethod -InputObject $taskCurrent -MethodName GetOwner
    $taskAncestry += [ordered]@{pid=$taskCurrent.ProcessId;parentPid=$taskCurrent.ParentProcessId;path=$taskCurrent.ExecutablePath;createdAtUtcTicks=[string]$taskCurrent.CreationDate.ToUniversalTime().Ticks;sessionId=$taskCurrent.SessionId;ownerUser=$taskOwner.User;ownerDomain=$taskOwner.Domain;ownerResult=$taskOwner.ReturnValue}
    $taskCurrent=Get-CimInstance Win32_Process -Filter ('ProcessId='+$taskCurrent.ParentProcessId)
}
if($taskCurrent -and !$taskAncestryError){$taskAncestryError='ancestry-not-established'}
$taskReport=[ordered]@{
    result='read-only-preflight';acceptance=$false;restartAuthorizedByThisReport=$false
    owner=$taskIdentity.Name;sid=$taskIdentity.User.Value;sessionId=$taskSelf.SessionId
    profile=$taskHome;config=$taskConfigState;existingCodexState=$taskExistingState
    packages=@($taskPackages|ForEach-Object {[ordered]@{name=$_.Name;version=[string]$_.Version;family=$_.PackageFamilyName;installLocation=$_.InstallLocation}})
    packageProbeError=$taskPackageError;ownCodexProcesses=$taskOwn;otherOwnerCodexCount=$taskOtherCount;unknownOwnerCodexCount=$taskUnknownCount
    ancestry=$taskAncestry;ancestryError=$taskAncestryError
    existingStateRequiresIsolationReview=($taskExistingState -or $taskOwn.Count -gt 0)
    next='Review disposable ownership and isolated config before preparing the actual product Save/restart test. No restart performed.'
    atUtc=[datetimeoffset]::UtcNow.ToString('o')
}
New-Item -ItemType Directory -Path $taskOutput|Out-Null
[IO.File]::WriteAllText((Join-Path $taskOutput 'preflight.json'),($taskReport|ConvertTo-Json -Depth 8),[Text.UTF8Encoding]::new($false))
Write-Output ('Read-only preflight saved to '+(Join-Path $taskOutput 'preflight.json'))

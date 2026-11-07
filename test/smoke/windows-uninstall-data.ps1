param([Parameter(Mandatory)][string]$InstallDirectory)
$ErrorActionPreference='Stop'
if(!$IsWindows -or $env:GITHUB_ACTIONS -cne 'true' -or $env:RUNNER_OS -cne 'Windows' -or
   $env:RUNNER_ENVIRONMENT -cne 'github-hosted' -or $env:GITHUB_REPOSITORY -cne 'gzers/opencodex-desktop'){
    throw 'Explicit cleanup smoke requires the disposable repository hosted runner'
}
. (Join-Path $PSScriptRoot 'windows023-two-user-native-helper.ps1')
$taskInstall=[IO.Path]::GetFullPath($InstallDirectory)
$taskAllowed=[IO.Path]::GetFullPath($env:RUNNER_TEMP)
$null=Assert-TwoUserUnder $taskAllowed $taskInstall
Assert-TwoUserNoReparseAncestors $taskInstall
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Require native0'}
$taskExe=Join-Path $taskInstall 'ocxd.exe'
$taskIdentity=[Security.Principal.WindowsIdentity]::GetCurrent()
$taskSid=$taskIdentity.User
$taskProfile=[Environment]::GetFolderPath([Environment+SpecialFolder]::UserProfile)
$taskRegistered=(Get-ItemProperty -LiteralPath ('HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList\'+$taskSid.Value)).ProfileImagePath
if(!(Test-TwoUserSamePath $taskProfile $taskRegistered) -or !(Test-TwoUserSamePath $taskProfile $env:USERPROFILE)){
    throw 'Actual TokenUser registered profile required; environment redirection is not identity'
}
Assert-TwoUserNoReparseAncestors $taskProfile
$taskUser=Join-Path $taskProfile '.codex-desktop'
$taskRoot=Join-Path $taskUser 'desktop'
$taskCustom=$taskInstall+'-cleanup-current'
$taskCustomD=Join-Path $taskCustom 'desktop'
$taskHome=$taskInstall+'-external-home'
$taskLegacy=Join-Path $taskInstall 'data'
foreach($taskPath in @($taskUser,$taskCustom,$taskHome,$taskLegacy)){
    Assert-TwoUserNoReparseAncestors $taskPath
    if(Test-Path -LiteralPath $taskPath){throw 'Never overwrite an existing cleanup fixture or current-user aggregate'}
}
foreach($taskPath in @($taskCustom,$taskHome,$taskLegacy)){$null=Assert-TwoUserUnder $taskAllowed $taskPath}
foreach($taskRootPath in @($taskRoot,$taskCustomD)){
    foreach($taskPartition in @('manager-state','backups','logs','exports','cache','sync-state')){
        New-Item -ItemType Directory -Path (Join-Path $taskRootPath $taskPartition) -Force | Out-Null
    }
}
New-Item -ItemType Directory -Path $taskHome | Out-Null
New-Item -ItemType Directory -Path $taskLegacy | Out-Null
$taskEncoding=[Text.UTF8Encoding]::new($false)
foreach($taskRootPath in @($taskRoot,$taskCustomD)){
    [IO.File]::WriteAllText((Join-Path $taskRootPath 'manager-state/data-root.json'),
        ([ordered]@{structure_version='2';opencodex_home_mode='external';opencodex_home_path=$taskHome}|ConvertTo-Json -Compress),$taskEncoding)
}
[IO.File]::WriteAllText((Join-Path $taskRoot 'manager-location.json'),
    ([ordered]@{schemaVersion=1;activeRoot=$taskCustom}|ConvertTo-Json -Compress),$taskEncoding)
$taskSentinels=@(
    (Join-Path $taskRoot 'manager-state/preferences.json'),
    (Join-Path $taskCustomD 'backups/owned-payload'),
    (Join-Path $taskHome 'outside.sentinel'),
    (Join-Path $taskRoot '.backup-w2.lock'),
    (Join-Path $taskRoot 'audit.log'),
    (Join-Path $taskCustomD 'audit.log.5'),
    (Join-Path $taskLegacy 'legacy-path-must-remain.sentinel')
)
foreach($taskPath in $taskSentinels){[IO.File]::WriteAllText($taskPath,'explicit v2 cleanup runner fixture',$taskEncoding)}
# The guard above proved these roots absent before this invocation created them.
# Explicit owner/DACL assignment belongs only to the NEW synthetic fixtures,
# never to an existing profile, data root, installation or foreign object.
foreach($taskNewRoot in @($taskUser,$taskCustom,$taskHome)){
    $taskItems=@((Get-Item -LiteralPath $taskNewRoot -Force))+@(Get-ChildItem -LiteralPath $taskNewRoot -Recurse -Force)
    foreach($taskItem in $taskItems){
        if($taskItem.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Linked synthetic fixture refused'}
        $taskAcl=if($taskItem.PSIsContainer){[Security.AccessControl.DirectorySecurity]::new()}else{[Security.AccessControl.FileSecurity]::new()}
        $taskAcl.SetOwner($taskSid);$taskAcl.SetAccessRuleProtection($true,$false)
        $taskInheritance=if($taskItem.PSIsContainer){[Security.AccessControl.InheritanceFlags]'ContainerInherit,ObjectInherit'}else{[Security.AccessControl.InheritanceFlags]::None}
        foreach($taskPrincipal in @($taskSid,[Security.Principal.SecurityIdentifier]::new('S-1-5-18'))){
            $taskAcl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($taskPrincipal,[Security.AccessControl.FileSystemRights]::FullControl,
                $taskInheritance,[Security.AccessControl.PropagationFlags]::None,[Security.AccessControl.AccessControlType]::Allow))
        }
        Set-Acl -LiteralPath $taskItem.FullName -AclObject $taskAcl
        if((Get-Acl -LiteralPath $taskItem.FullName).GetOwner([Security.Principal.SecurityIdentifier]).Value -cne $taskSid.Value){throw 'New synthetic fixture actual owner differs'}
    }
}
$taskExpected=@($taskSentinels|ForEach-Object {[ordered]@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash.ToLowerInvariant()}})
$taskPointer=Get-Content -LiteralPath (Join-Path $taskRoot 'manager-location.json') -Raw
function Invoke-CleanupHelper([string[]]$Arguments){
    $taskInfo=[Diagnostics.ProcessStartInfo]::new()
    $taskInfo.FileName=$taskExe;$taskInfo.UseShellExecute=$false;$taskInfo.CreateNoWindow=$true
    $taskInfo.RedirectStandardOutput=$true;$taskInfo.RedirectStandardError=$true
    foreach($taskArgument in $Arguments){$taskInfo.ArgumentList.Add($taskArgument)}
    foreach($taskName in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY')){[void]$taskInfo.Environment.Remove($taskName)}
    $taskProcess=[Diagnostics.Process]::Start($taskInfo)
    try{
        $taskStdout=$taskProcess.StandardOutput.ReadToEndAsync();$taskStderr=$taskProcess.StandardError.ReadToEndAsync()
        if(!$taskProcess.WaitForExit(30000)){throw 'Bound cleanup observation expired; no kill or retry'}
        [ordered]@{exitCode=$taskProcess.ExitCode;stdout=$taskStdout.GetAwaiter().GetResult();stderr=$taskStderr.GetAwaiter().GetResult()}
    }finally{$taskProcess.Dispose()}
}
$taskUnconfirmed=Invoke-CleanupHelper @('--uninstall-manager-data')
if($taskUnconfirmed.exitCode -ne 2){throw 'Unconfirmed cleanup did not refuse'}
foreach($taskRow in $taskExpected){if((Get-FileHash -LiteralPath $taskRow.path).Hash.ToLowerInvariant() -cne $taskRow.sha256){throw 'Unconfirmed cleanup changed a fixture'}}
if((Get-Content -LiteralPath (Join-Path $taskRoot 'manager-location.json') -Raw) -cne $taskPointer){throw 'Unconfirmed cleanup changed the v2 pointer'}
foreach($taskNewRoot in @($taskUser,$taskCustom)){
    if(Test-Path -LiteralPath (Join-Path $taskNewRoot '.desktop-removal.lock')){throw 'Unconfirmed cleanup created a removal gate'}
}
$taskConfirmed=Invoke-CleanupHelper @('--uninstall-manager-data','--confirm')
if($taskConfirmed.exitCode -ne 0){throw ('Confirmed v2 cleanup failed: '+$taskConfirmed.stderr)}
if((Test-Path -LiteralPath $taskRoot) -or (Test-Path -LiteralPath $taskCustomD)){throw 'Confirmed cleanup left fixed or selected v2 desktop data'}
foreach($taskRow in @($taskExpected[2],$taskExpected[6])){
    if((Get-FileHash -LiteralPath $taskRow.path).Hash.ToLowerInvariant() -cne $taskRow.sha256){throw 'External HOME or unselected legacy fixture changed'}
}
$taskGates=@()
foreach($taskNewRoot in @($taskUser,$taskCustom)){
    $taskGate=Join-Path $taskNewRoot '.desktop-removal.lock'
    Assert-TwoUserNoReparseAncestors $taskGate
    if(!(Test-Path -LiteralPath $taskGate -PathType Leaf) -or (Get-Item -LiteralPath $taskGate).Length -ne 0){throw 'Durable external removal gate missing or changed'}
    $taskEntries=@(Get-ChildItem -LiteralPath $taskNewRoot -Force)
    if($taskEntries.Count -ne 1 -or !(Test-TwoUserSamePath $taskEntries[0].FullName $taskGate)){throw 'Unexpected fixture residue retained for inspection'}
    $taskGates+=@{path=$taskGate;sha256=(Get-FileHash -LiteralPath $taskGate).Hash.ToLowerInvariant()}
}
$taskRepeat=Invoke-CleanupHelper @('--uninstall-manager-data','--confirm')
if($taskRepeat.exitCode -ne 0){throw 'Repeated cleanup of absent v2 D failed'}
foreach($taskGate in $taskGates){if((Get-FileHash -LiteralPath $taskGate.path).Hash.ToLowerInvariant() -cne $taskGate.sha256){throw 'Repeat replaced a durable gate'}}
if(!(Test-Path -LiteralPath (Join-Path $taskInstall 'opencodex-desktop.exe'))){throw 'Helper removed the application'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Cleanup left a native process'}
# Product removal retained the gates. Only this harness may now clean its
# exact, newly created, fully inventoried fixtures after all helper handles exit.
foreach($taskNewRoot in @($taskUser,$taskCustom,$taskHome)){Remove-Item -LiteralPath $taskNewRoot -Recurse -Force}
[ordered]@{
    scope='disposable hosted runner actual TokenUser profile; current v2 fixed pointer and selected aggregate'
    structureVersion=2;tokenUser=$taskSid.Value;profile=$taskProfile
    unconfirmedExitCode=$taskUnconfirmed.exitCode;confirmedExitCode=$taskConfirmed.exitCode;repeatExitCode=$taskRepeat.exitCode
    fixedAndSelectedDesktopRemoved=$true;externalHomeUnchanged=$true;unselectedLegacyPathUnchanged=$true
    durableExternalGatesRetainedByProduct=$true;newFixturesCleanedByHarness=$true
    applicationRetained=$true;legacyCompatibilityTested=$false;hostAcceptanceClaimed=$false
}

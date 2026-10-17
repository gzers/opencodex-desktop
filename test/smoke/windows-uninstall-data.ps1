param([Parameter(Mandatory)][string]$InstallDirectory)
$ErrorActionPreference='Stop'
if(!$IsWindows -or $env:GITHUB_ACTIONS -cne 'true' -or
   $env:GITHUB_REPOSITORY -cne 'gzers/opencodex-desktop'){
    throw 'Explicit cleanup smoke is restricted to the disposable repository runner'
}
$taskInstall=[IO.Path]::GetFullPath($InstallDirectory)
$taskAllowed=[IO.Path]::GetFullPath($env:RUNNER_TEMP)
if(!$taskInstall.StartsWith($taskAllowed+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned runner installation required'}
$taskCursor=Get-Item -LiteralPath $taskInstall
while($taskCursor){if($taskCursor.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Installation alias refused'};$taskCursor=$taskCursor.Parent}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Require native0'}
$taskExe=Join-Path $taskInstall 'ocxd.exe'
$taskRoot=Join-Path $taskInstall 'data'
$taskCustom=$taskInstall+'-cleanup-custom'
$taskHome=$taskInstall+'-external-home'
$taskKey='HKCU:\Software\com.gzers.opencodex.desktop'
if((Get-ItemProperty -LiteralPath $taskKey -ErrorAction SilentlyContinue).psobject.Properties.Name -contains 'StartupDataRootV1'){throw 'Existing startup pointer must not be replaced'}
foreach($taskPath in @($taskRoot,$taskCustom,$taskHome)){
    if(Test-Path -LiteralPath $taskPath){throw 'Dedicated cleanup fixture already exists'}
    if(![IO.Path]::GetFullPath($taskPath).StartsWith($taskAllowed+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Fixture outside runner boundary'}
}
foreach($taskRootPath in @($taskRoot,$taskCustom)){
    foreach($taskPartition in @('manager-state','opencodex-home','backups','logs','exports','cache','sync-state')){
        New-Item -ItemType Directory -Path (Join-Path $taskRootPath $taskPartition) -Force | Out-Null
    }
}
New-Item -ItemType Directory -Path $taskHome | Out-Null
$taskEncoding=[Text.UTF8Encoding]::new($false)
[IO.File]::WriteAllText((Join-Path $taskCustom 'manager-state/data-root.json'),'{"structure_version":"1"}',$taskEncoding)
[IO.File]::WriteAllText((Join-Path $taskRoot 'manager-state/data-root.json'),
    ([ordered]@{structure_version='1';active_data_root=$taskCustom;opencodex_home_mode='external';opencodex_home_path=$taskHome} | ConvertTo-Json -Compress),$taskEncoding)
$taskSentinels=@(
    Join-Path $taskRoot 'manager-state/preferences.json'
    Join-Path $taskCustom 'backups/owned-payload'
    Join-Path $taskHome 'outside.sentinel'
    Join-Path $taskRoot '.backup-w2.lock'
    Join-Path $taskRoot 'audit.log'
    Join-Path $taskRoot 'audit.log.1'
    Join-Path $taskCustom 'audit.log.5'
)
foreach($taskPath in $taskSentinels){[IO.File]::WriteAllText($taskPath,'explicit cleanup runner fixture',$taskEncoding)}
$taskExpected=@($taskSentinels | ForEach-Object {[ordered]@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash.ToLowerInvariant()}})
New-Item -Path $taskKey -Force | Out-Null
$taskPointer=[ordered]@{schema=1;anchor=$taskRoot} | ConvertTo-Json -Compress
New-ItemProperty -LiteralPath $taskKey -Name StartupDataRootV1 -PropertyType String -Value $taskPointer | Out-Null
function Invoke-CleanupHelper([string[]]$Arguments){
    $taskInfo=[Diagnostics.ProcessStartInfo]::new()
    $taskInfo.FileName=$taskExe;$taskInfo.UseShellExecute=$false;$taskInfo.CreateNoWindow=$true
    $taskInfo.RedirectStandardOutput=$true;$taskInfo.RedirectStandardError=$true
    foreach($taskArgument in $Arguments){$taskInfo.ArgumentList.Add($taskArgument)}
    foreach($taskName in @('OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY')){
        [void]$taskInfo.Environment.Remove($taskName)
    }
    $taskProcess=[Diagnostics.Process]::Start($taskInfo)
    try{
        $taskStdout=$taskProcess.StandardOutput.ReadToEndAsync()
        $taskStderr=$taskProcess.StandardError.ReadToEndAsync()
        if(!$taskProcess.WaitForExit(30000)){throw 'Bound cleanup observation expired; no kill or retry'}
        [ordered]@{exitCode=$taskProcess.ExitCode;stdout=$taskStdout.GetAwaiter().GetResult();stderr=$taskStderr.GetAwaiter().GetResult()}
    }finally{$taskProcess.Dispose()}
}
$taskUnconfirmed=Invoke-CleanupHelper @('--uninstall-manager-data')
if($taskUnconfirmed.exitCode -ne 2){throw 'Unconfirmed cleanup did not refuse'}
foreach($taskRow in $taskExpected){if((Get-FileHash -LiteralPath $taskRow.path).Hash.ToLowerInvariant() -cne $taskRow.sha256){throw 'Unconfirmed cleanup changed a fixture'}}
if((Get-ItemProperty -LiteralPath $taskKey).StartupDataRootV1 -cne $taskPointer){throw 'Unconfirmed cleanup changed the pointer'}
$taskConfirmed=Invoke-CleanupHelper @('--uninstall-manager-data','--confirm')
if($taskConfirmed.exitCode -ne 0){throw ('Confirmed cleanup failed: '+$taskConfirmed.stderr)}
if((Test-Path -LiteralPath $taskRoot) -or (Test-Path -LiteralPath $taskCustom)){throw 'Confirmed cleanup left manager data'}
if((Get-ItemProperty -LiteralPath $taskKey).StartupDataRootV1){throw 'Confirmed cleanup left the startup pointer'}
if((Get-FileHash -LiteralPath $taskExpected[2].path).Hash.ToLowerInvariant() -cne $taskExpected[2].sha256){throw 'External HOME was changed'}
if(!(Test-Path -LiteralPath (Join-Path $taskInstall 'opencodex-desktop.exe'))){throw 'Helper removed the application'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Cleanup left a native process'}
[ordered]@{
    scope='disposable runner real installed helper with dedicated seeded roots; no native UI root selection'
    unconfirmedExitCode=$taskUnconfirmed.exitCode;confirmedExitCode=$taskConfirmed.exitCode
    startupPointerRemoved=$true;anchorAndActiveRootRemoved=$true;externalHomeUnchanged=$true
    applicationRetained=$true;applicationRootBindingTested=$false;hostAcceptanceClaimed=$false
}

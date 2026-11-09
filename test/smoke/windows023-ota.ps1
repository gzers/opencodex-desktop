# Explicit022->023 NSIS signed updater, new paths/no migration, original022 rollback.
# MSI022 machine->023 user remains blocked by its own retention scenario.
param([Parameter(Mandatory=$true)][string]$OutputDirectory,[ValidateSet('stable','beta')][string]$Channel='beta',[Parameter(Mandatory=$true)][ValidatePattern('^[a-fA-F0-9]{64}$')][string]$ExpectedBuiltSha256)
$ErrorActionPreference='Stop'
if(!$IsWindows -or $env:GITHUB_ACTIONS -cne 'true' -or $env:RUNNER_ENVIRONMENT -cne 'github-hosted' -or $env:GITHUB_REPOSITORY -cne 'gzers/opencodex-desktop' -or $env:GITHUB_RUN_ATTEMPT -cne '1'){throw 'Fresh first-attempt hosted project Windows023 OTA only'}
$out=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $out){throw 'Fresh OTA evidence required; preserve previous attempts'}
New-Item -ItemType Directory -Path $out|Out-Null
$install=Join-Path $env:RUNNER_TEMP ('OpenCodeX023 OTA 中文 '+[guid]::NewGuid().ToString('N'))
$rollbackInstall=Join-Path $env:RUNNER_TEMP ('OpenCodeX022 Rollback 中文 '+[guid]::NewGuid().ToString('N'))
$sandbox=Join-Path $out 'actual-sandbox'
$baseline=Join-Path $out 'OpenCodeX.Desktop_0.2.22_x64-setup.exe'
$baselineHash='0b824645ac1b1c3eacab58a3a7fcfededfc910104ac282cfd99c7855b4f97895'
$release=Invoke-RestMethod 'https://api.github.com/repos/gzers/opencodex-desktop/releases/tags/v0.2.22'
$asset=@($release.assets|Where-Object id -eq 715405138)
if($release.id -ne 435948372 -or $release.draft -or $release.prerelease -or $asset.Count -ne 1 -or $asset[0].size -ne 5323941 -or $asset[0].digest -cne ('sha256:'+$baselineHash)){throw 'Original published022 baseline identity differs'}
Invoke-WebRequest -Uri $asset[0].browser_download_url -OutFile $baseline
if((Get-FileHash -LiteralPath $baseline).Hash.ToLowerInvariant() -cne $baselineHash){throw 'Baseline original bytes differ'}
$listen=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0);$listen.Start();$port=$listen.LocalEndpoint.Port;$listen.Stop()
function Invoke-OriginalInstaller([string]$Package,[string]$Directory){
 $p=Start-Process -FilePath $Package -ArgumentList ('/S /D='+$Directory) -Wait -PassThru
 if($p.ExitCode -ne 0){throw ('Original installer exit '+$p.ExitCode)}
}
function Start-OwnedApplication([string]$Directory){
 $i=[Diagnostics.ProcessStartInfo]::new();$i.FileName=Join-Path $Directory 'opencodex-desktop.exe';$i.WorkingDirectory=$Directory;$i.UseShellExecute=$false
 $i.Environment['OPENCODEX_SANDBOX']='1';$i.Environment['OPENCODEX_SANDBOX_ROOT']=$sandbox;$i.Environment['OPENCODEX_WINDOWS_SMOKE_CDP_PORT']=[string]$port
 foreach($key in @('HOME','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','WEBVIEW2_USER_DATA_FOLDER')){[void]$i.Environment.Remove($key)}
 [Diagnostics.Process]::Start($i)
}
function Stop-OwnedApplication($Process){if($Process -and !$Process.HasExited){$Process.Kill($true);if(!$Process.WaitForExit(10000)){throw 'Owned app did not exit'}}}
function Data-Tree([string]$Root){
 if(!(Test-Path -LiteralPath $Root)){return @()}
 @(Get-ChildItem -LiteralPath $Root -Recurse -Force|Sort-Object FullName|ForEach-Object{
  if($_.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Unexpected reparse point in owned OTA data'}
  if(!$_.PSIsContainer){@{path=[IO.Path]::GetRelativePath($Root,$_.FullName);bytes=$_.Length;sha256=(Get-FileHash -LiteralPath $_.FullName).Hash.ToLowerInvariant()}}
 })
}
$app=$null;$record=[ordered]@{result='fail';kind='nsis';from='0.2.22';to='0.2.23';channel=$Channel;baselineSha256=$baselineHash;automaticRestartObserved=$false;explicitRollbackObserved=$false;migration='none';scope='published original022 + real production-channel old-key signed023 updater; isolated synthetic business config; explicit independent original022 rollback; no manual fallback for automatic restart'}
try{
 Invoke-OriginalInstaller $baseline $install
 $exe=Join-Path $install 'opencodex-desktop.exe';$originalExeHash=(Get-FileHash -LiteralPath $exe).Hash.ToLowerInvariant()
 if((Get-Item -LiteralPath $exe).VersionInfo.ProductVersion -cne '0.2.22'){throw 'Original baseline install version differs'}
 $app=Start-OwnedApplication $install
 & node (Join-Path $PSScriptRoot 'windows-ota.mjs') ('http://127.0.0.1:'+$port) $out $sandbox 'prepare-exit' '0.2.23' '0.2.22' $Channel
 if($LASTEXITCODE -ne 0 -or !$app.WaitForExit(30000)){throw 'Original signed preparation/ordinary exit failed'}
 Start-Sleep -Seconds 5
 if((Get-Item -LiteralPath $exe).VersionInfo.ProductVersion -cne '0.2.22'){throw 'Unconfirmed ordinary exit installed new version'}
 $record.ordinaryExitPreservedOriginal=$true
 $app=Start-OwnedApplication $install;$oldPid=$app.Id;$oldStarted=$app.StartTime
 & node (Join-Path $PSScriptRoot 'windows-ota.mjs') ('http://127.0.0.1:'+$port) $out $sandbox 'prepare-restart' '0.2.23' '0.2.22' $Channel
 if($LASTEXITCODE -ne 0 -or !$app.WaitForExit(30000)){throw 'Explicit real updater handoff failed'}
 $deadline=[datetimeoffset]::UtcNow.AddMinutes(3);$automatic=@()
 do{
  Start-Sleep -Milliseconds 500
  $image=Get-Item -LiteralPath $exe -ErrorAction SilentlyContinue
  $automatic=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'"|Where-Object {$_.ExecutablePath -ieq $exe -and $_.ProcessId -ne $oldPid -and $_.CreationDate -gt $oldStarted})
 }while([datetimeoffset]::UtcNow -lt $deadline -and (!$image -or $image.VersionInfo.ProductVersion -cne '0.2.23' -or !$automatic.Count))
 if(!$image -or $image.VersionInfo.ProductVersion -cne '0.2.23' -or $automatic.Count -ne 1){throw 'Real updater did not replace/reopen023 automatically; no manual launch fallback'}
 $app=Get-Process -Id $automatic[0].ProcessId
 $record.automaticRestartObserved=$true;$record.automaticRestart=@{pid=$app.Id;path=$automatic[0].ExecutablePath;creation=$automatic[0].CreationDate;version=$image.VersionInfo.ProductVersion}
 Stop-OwnedApplication $app
 & node (Join-Path $PSScriptRoot 'windows-bundle-identity.mjs') '--installed-hash' $exe 'nsis' $ExpectedBuiltSha256.ToLowerInvariant() > (Join-Path $out 'updated-bundle-identity.json')
 if($LASTEXITCODE -ne 0){throw 'Updated EXE differs from exact signed release original'}
 $app=Start-OwnedApplication $install
 & node (Join-Path $PSScriptRoot 'windows023-ota-readback.mjs') ('http://127.0.0.1:'+$port) $out $sandbox 'verify023' $Channel
 if($LASTEXITCODE -ne 0){throw 'Installed023 actual new-path/no-migration readback failed'}
 Stop-OwnedApplication $app
 $newTree=Data-Tree (Join-Path $sandbox 'data');$newTree|ConvertTo-Json -Depth 5|Set-Content (Join-Path $out 'new023-data-before-rollback.json')
 # Explicit manual rollback uses the exact published original package in a
 # separate owned folder; neither root is imported, cleaned or migrated.
 Invoke-OriginalInstaller $baseline $rollbackInstall
 $oldExe=Join-Path $rollbackInstall 'opencodex-desktop.exe'
 if((Get-FileHash -LiteralPath $oldExe).Hash.ToLowerInvariant() -cne $originalExeHash){throw 'Rollback original022 executable identity changed'}
 $app=Start-OwnedApplication $rollbackInstall
 & node (Join-Path $PSScriptRoot 'windows023-ota-readback.mjs') ('http://127.0.0.1:'+$port) $out $sandbox 'rollback022' $Channel
 if($LASTEXITCODE -ne 0){throw 'Explicit original022 rollback failed its own saved config readback'}
 Stop-OwnedApplication $app
 if(($newTree|ConvertTo-Json -Depth 5 -Compress) -cne ((Data-Tree (Join-Path $sandbox 'data'))|ConvertTo-Json -Depth 5 -Compress)){throw 'Manual022 rollback changed023 new data'}
 $record.explicitRollbackObserved=$true;$record.original022ExeSha256=$originalExeHash;$record.oldAndNewRootsPreserved=$true;$record.result='pass'
}catch{$record.error=$_.Exception.Message;throw}finally{
 try{Stop-OwnedApplication $app;$record.ownedProcessCleanup='exited'}catch{$record.cleanupError=$_.Exception.Message;$record.result='fail'}
 # Task-owned installed program folders are removed only after their process
 # handle exit. Data/evidence stays in the exact disposable output directory.
 if($record.ownedProcessCleanup -ceq 'exited'){
  foreach($dir in @($install,$rollbackInstall)){if(Test-Path -LiteralPath $dir){$uninstaller=Join-Path $dir 'uninstall.exe';if(Test-Path -LiteralPath $uninstaller){$p=Start-Process $uninstaller -ArgumentList '/S' -Wait -PassThru;if($p.ExitCode -ne 0){$record.result='fail';$record.cleanupError='Owned original uninstaller failed'}; $until=[datetimeoffset]::UtcNow.AddSeconds(15);while((Test-Path -LiteralPath (Join-Path $dir 'opencodex-desktop.exe')) -and [datetimeoffset]::UtcNow -lt $until){Start-Sleep -Milliseconds 200};if(Test-Path -LiteralPath (Join-Path $dir 'opencodex-desktop.exe')){$record.result='fail';$record.cleanupError='Owned original uninstaller retained application image'}}}}
 }
 $record|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $out 'ota023-result.json') -Encoding utf8
}
if($record.result -cne 'pass'){throw '023 signed OTA/rollback cleanup incomplete'}

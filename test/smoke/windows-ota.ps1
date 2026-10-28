param([Parameter(Mandatory=$true)][ValidateSet('msi','nsis')][string]$Kind,[Parameter(Mandatory=$true)][string]$OutputDirectory,[string]$Version='0.2.17')
$ErrorActionPreference='Stop'
if(!$IsWindows -or $env:GITHUB_ACTIONS -cne 'true' -or $env:GITHUB_REPOSITORY -cne 'gzers/opencodex-desktop'){throw 'OTA acceptance is restricted to the disposable project runner'}
if($Version -cne '0.2.17'){throw 'Only authorized 0.2.16 to 0.2.17 acceptance is admitted'}
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Force $taskOutput|Out-Null
$taskInstall=Join-Path $env:RUNNER_TEMP ('OpenCodeX OTA 中文 '+$Kind+' '+[guid]::NewGuid().ToString('N'))
$taskSandbox=Join-Path $taskOutput 'actual-sandbox'
$taskAsset=if($Kind -ceq 'msi'){'OpenCodeX.Desktop_0.2.16_x64_en-US.msi'}else{'OpenCodeX.Desktop_0.2.16_x64-setup.exe'}
$taskPackage=Join-Path $taskOutput $taskAsset
Invoke-WebRequest -Uri ('https://github.com/gzers/opencodex-desktop/releases/download/v0.2.16/'+$taskAsset) -OutFile $taskPackage
$taskRelease=Invoke-RestMethod 'https://api.github.com/repos/gzers/opencodex-desktop/releases/tags/v0.2.16'
$taskExpected=@($taskRelease.assets|Where-Object name -ceq $taskAsset)
$taskHash=(Get-FileHash -LiteralPath $taskPackage).Hash.ToLowerInvariant()
if($taskExpected.Count -ne 1 -or $taskExpected[0].digest -cne ('sha256:'+$taskHash)){throw 'Public baseline asset digest mismatch'}
$taskArgs=if($Kind -ceq 'msi'){'/i "'+$taskPackage+'" /qn /norestart INSTALLDIR="'+$taskInstall+'"'}else{'/S /D='+$taskInstall}
$taskFile=if($Kind -ceq 'msi'){'msiexec.exe'}else{$taskPackage}
$taskInstaller=Start-Process -FilePath $taskFile -ArgumentList $taskArgs -PassThru -Wait
if($taskInstaller.ExitCode -ne 0){throw 'Baseline installation failed'}
$taskExe=Join-Path $taskInstall 'opencodex-desktop.exe'
if((Get-Item -LiteralPath $taskExe).VersionInfo.ProductVersion -cne '0.2.16'){throw 'Installed baseline version mismatch'}
$taskListener=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0);$taskListener.Start();$taskPort=$taskListener.LocalEndpoint.Port;$taskListener.Stop()
function Start-OwnedApp {
 $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$taskExe;$info.WorkingDirectory=$taskInstall;$info.UseShellExecute=$false
 $info.Environment['OPENCODEX_SANDBOX']='1';$info.Environment['OPENCODEX_SANDBOX_ROOT']=$taskSandbox;$info.Environment['OPENCODEX_WINDOWS_SMOKE_CDP_PORT']=[string]$taskPort
 [void]$info.Environment.Remove('HOME');[void]$info.Environment.Remove('WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS');[void]$info.Environment.Remove('WEBVIEW2_USER_DATA_FOLDER')
 return [Diagnostics.Process]::Start($info)
}
$taskApp=$null
$taskResult=[ordered]@{result='fail';kind=$Kind;from='0.2.16';to=$Version;baselinePackageSha256=$taskHash;installDirectory=$taskInstall;sandbox=$taskSandbox;scope='real public stable updater and published baseline; isolated production model file, synthetic credentials and prewrite backup; no user machine changes'}
try{
 $taskApp=Start-OwnedApp
 & node (Join-Path $PSScriptRoot 'windows-ota.mjs') ('http://127.0.0.1:'+$taskPort) $taskOutput $taskSandbox 'prepare-exit' $Version
 if($LASTEXITCODE -ne 0){throw 'Actual updater preparation failed'}
 if(!$taskApp.WaitForExit(30000)){throw 'Ordinary exit failed'}
 Start-Sleep -Seconds 5
 if((Get-Item -LiteralPath $taskExe).VersionInfo.ProductVersion -cne '0.2.16'){throw 'Ordinary exit installed an unconfirmed update'}
 $taskResult.ordinaryExitPreservedOldVersion=$true
 $taskApp=Start-OwnedApp
 & node (Join-Path $PSScriptRoot 'windows-ota.mjs') ('http://127.0.0.1:'+$taskPort) $taskOutput $taskSandbox 'prepare-restart' $Version
 if($LASTEXITCODE -ne 0){throw 'Explicit update handoff failed'}
 if(!$taskApp.WaitForExit(30000)){throw 'Updater did not exit the old process'}
 $deadline=[datetimeoffset]::UtcNow.AddMinutes(3)
 do{
  Start-Sleep -Milliseconds 500
  $newImage=Get-Item -LiteralPath $taskExe -ErrorAction SilentlyContinue
  $newApp=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'"|Where-Object {$_.ExecutablePath -ieq $taskExe -and $_.ProcessId -ne $taskApp.Id})
 }while([datetimeoffset]::UtcNow -lt $deadline -and (!$newImage -or $newImage.VersionInfo.ProductVersion -cne $Version -or !$newApp.Count))
 if(!$newImage -or $newImage.VersionInfo.ProductVersion -cne $Version -or !$newApp.Count){throw 'OTA did not replace and automatically reopen the new application'}
 $taskApp=Get-Process -Id $newApp[0].ProcessId
 $taskResult.automaticRestartObserved=$true
 $taskResult.automaticRestartProcess=[ordered]@{pid=$newApp[0].ProcessId;path=$newApp[0].ExecutablePath;creationDate=$newApp[0].CreationDate;version=$newImage.VersionInfo.ProductVersion}
 # The native installer reopens through the Windows Shell, whose environment
 # does not carry the old application's test-only CDP/sandbox variables. The
 # system process above proves automatic restart. Inspect the immutable fixture
 # using one explicitly isolated relaunch, without changing the real updater.
 $taskApp.Kill($true);[void]$taskApp.WaitForExit(10000)
 $taskApp=Start-OwnedApp
 $taskResult.readbackMethod='automatic new process observed by OS; exact updated EXE then relaunched with explicit isolated test environment for Tauri/file readback'

 & node (Join-Path $PSScriptRoot 'windows-ota.mjs') ('http://127.0.0.1:'+$taskPort) $taskOutput $taskSandbox 'verify' $Version
 if($LASTEXITCODE -ne 0){throw 'Reopened OTA result verification failed'}
 $taskResult.installedExecutableSha256=(Get-FileHash -LiteralPath $taskExe).Hash.ToLowerInvariant();$taskResult.result='pass'
}catch{$taskResult.error=$_.Exception.Message;throw}finally{
 if($taskApp -and !$taskApp.HasExited){$taskApp.Kill($true);[void]$taskApp.WaitForExit(10000)}
 $taskResult|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $taskOutput 'ota-result.json') -Encoding utf8
}

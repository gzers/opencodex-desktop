$ErrorActionPreference='Stop'
. C:/Users/15119/.codex/worktrees/ocxd-023-windows-r1/code/test/smoke/windows023-two-user-native-helper.ps1
. C:/Users/15119/.codex/worktrees/ocxd-023-windows-r1/code/test/smoke/windows-process-token.ps1
$taskSid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
if($taskSid -cne 'S-1-5-21-587408064-856152790-2583328096-500' -or [Diagnostics.Process]::GetCurrentProcess().SessionId -ne 1){throw 'Actual Administrator console required'}
$taskRoot=Join-Path $PSScriptRoot 'console-exit-only02'
$taskPrivate=Join-Path $taskRoot 'private'
if(Test-Path -LiteralPath $taskPrivate){throw 'Fresh private fixture required'}
New-Item -ItemType Directory -Path $taskPrivate | Out-Null
& icacls.exe $taskPrivate /setowner ('*'+$taskSid) /T /C *> (Join-Path $taskRoot 'private-owner.log')
if($LASTEXITCODE -ne 0){throw 'Fresh fixture owner failed'}
& icacls.exe $taskPrivate /inheritance:r /grant:r ('*'+$taskSid+':(OI)(CI)F') '*S-1-5-18:(OI)(CI)F' *> (Join-Path $taskRoot 'private-acl.log')
if($LASTEXITCODE -ne 0){throw 'Fresh fixture ACL failed'}
$taskExe=Join-Path $taskPrivate 'opencodex-desktop.exe'
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'program/opencodex-desktop.exe') -Destination $taskExe
if((Get-FileHash $taskExe).Hash -ine '598f4eb4f62bea022dc73d45ae2112563630cb2d9e6f1b26ae4ff23939885f27'){throw 'Original executable differs'}
$taskProcess=$null;$taskFact=$null;$taskResult=@{accepted=$false;scope='native console tray normal exit only; no runtime supplied or service started'}
try{
 $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$taskExe;$info.UseShellExecute=$false;$info.CreateNoWindow=$true
 $info.Environment['OPENCODEX_SANDBOX']='1';$info.Environment['OPENCODEX_SANDBOX_ROOT']=Join-Path $taskPrivate 'sandbox'
 foreach($key in @('HOME','OPENCODEX_HOME','CODEX_HOME','OPENCODEX_WINDOWS_SMOKE_CDP_PORT','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','WEBVIEW2_USER_DATA_FOLDER')){[void]$info.Environment.Remove($key)}
 $taskProcess=[Diagnostics.Process]::Start($info)
 $taskFact=Get-TwoUserProcessFact $taskProcess $taskSid
 $token=Get-WindowsProcessToken $taskProcess $taskExe;Assert-WindowsProcessMode $token administrator
 $until=[datetime]::UtcNow.AddSeconds(30);$window=$null
 do{try{$window=Get-TwoUserWindow $taskFact}catch{};if(!$window){Start-Sleep -Milliseconds 250}}while(!$window -and [datetime]::UtcNow -lt $until)
 if(!$window){throw 'Owned console window missing'}
 $taskResult.manager=$taskFact;$taskResult.token=$token;$taskResult.window=$window
 Write-TwoUserJson (Join-Path $taskRoot 'ready.json') $taskResult
 $until=[datetime]::UtcNow.AddMinutes(10)
 while(!(Test-Path -LiteralPath (Join-Path $taskRoot 'observe-exit'))){if([datetime]::UtcNow -gt $until){throw 'Operator deadline'};Start-Sleep -Milliseconds 250}
 Write-TwoUserJson (Join-Path $taskRoot 'awaiting-native-exit.json') @{ready=$true;manager=$taskFact;accepted=$false}
 # Human observation envelope only; original product start/stop/status budgets
 # do not apply to waiting for the user to choose the native Exit menu.
 $until=[datetime]::UtcNow.AddMinutes(10)
 while(!$taskProcess.WaitForExit(1000) -and [datetime]::UtcNow -lt $until){}
 $taskResult.originalHandleExited=$taskProcess.HasExited
 $taskResult.exitCode=if($taskProcess.HasExited){$taskProcess.ExitCode}else{$null}
 $taskResult.result=if($taskResult.originalHandleExited -and $taskResult.exitCode -eq 0){'pass-native-console-tray-exit'}else{'fail'}
}catch{$taskResult.result='fail';$taskResult.error=$_.Exception.Message}
finally{
 if($taskProcess){
  if(!$taskProcess.HasExited -and $taskFact){Stop-TwoUserTracked $taskFact;$taskResult.forcedCleanup=$true}
  $taskProcess.Dispose()
 }
 Close-TwoUserProcessHandles
 $taskResult.privateRetainedUntilWebViewReleased=$true
 Write-TwoUserJson (Join-Path $taskRoot 'finished.json') $taskResult
}

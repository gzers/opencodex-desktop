param([ValidateSet('Auto','MSI-default','MSI-custom','NSIS-default','NSIS-custom')][string]$Cell='Auto')
$ErrorActionPreference='Stop'
$id=[Security.Principal.WindowsIdentity]::GetCurrent()
$sid=$id.User.Value
$session=[Diagnostics.Process]::GetCurrentProcess().SessionId
$admin=([Security.Principal.WindowsPrincipal]::new($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if($sid -eq 'S-1-5-21-587408064-856152790-2583328096-500' -and $session -eq 1 -and $env:SESSIONNAME -ieq 'Console'){$role='console'}
elseif($sid -eq 'S-1-5-21-587408064-856152790-2583328096-1003' -and $session -eq 3 -and $env:SESSIONNAME -like 'RDP-*' -and !$admin){$role='rdp'}
else{throw 'Use the Administrator console or the marenben standard RDP desktop.'}
if(![Environment]::UserInteractive){throw 'An interactive desktop is required.'}
$dest=Join-Path $PSScriptRoot "receipts-$role"
$default=Join-Path $env:LOCALAPPDATA 'Programs\OpenCodeX Desktop'
$privateRoot=Join-Path $env:USERPROFILE 'ocx-w2-team79-r1'
$vendor='Software\gzers\OpenCodeX Desktop'
$arp='Software\Microsoft\Windows\CurrentVersion\Uninstall'
function Save-New($path,$value) {
  $data=[Text.UTF8Encoding]::new($false).GetBytes(($value | ConvertTo-Json -Depth 12))
  $file=[IO.File]::Open($path,'CreateNew','Write','None');try{$file.Write($data,0,$data.Length)}finally{$file.Dispose()}
}
function Hash-File($path){(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()}
function Assert-PlainPath($path){
  $part=[IO.Path]::GetFullPath($path)
  while($part){if(Test-Path -LiteralPath $part){if((Get-Item -LiteralPath $part -Force).Attributes -band [IO.FileAttributes]::ReparsePoint){throw "Reparse point in target: $part"}};$part=[IO.Path]::GetDirectoryName($part)}
}
function Read-Context {
  $r=[ordered]@{utc=[DateTime]::UtcNow.ToString('o');sid=$sid;session=$session;effectiveAdmin=$admin;profile=$env:USERPROFILE;registrations=@();vendor=@();policies=@();shortcuts=@();runValues=@();defaultRoot=$default;defaultExists=(Test-Path -LiteralPath $default);sessions=@(quser.exe)}
  foreach($view in 'Registry64','Registry32'){
    foreach($hive in 'CurrentUser','LocalMachine'){
      $reg=[Microsoft.Win32.RegistryKey]::OpenBaseKey($hive,$view)
      try{
        $k=$reg.OpenSubKey($arp)
        if($k){try{foreach($name in $k.GetSubKeyNames()){
          $item=$k.OpenSubKey($name);if(!$item){continue}
          try{if([string]$item.GetValue('DisplayName') -like '*OpenCodeX*'){$r.registrations+=[ordered]@{hive=$hive;view=$view;key=$name;name=$item.GetValue('DisplayName');version=$item.GetValue('DisplayVersion');location=$item.GetValue('InstallLocation');uninstall=$item.GetValue('UninstallString')}}}finally{$item.Dispose()}
        }}finally{$k.Dispose()}}
        $k=$reg.OpenSubKey('SOFTWARE\Policies\Microsoft\Windows\Installer');try{$r.policies+=[ordered]@{hive=$hive;view=$view;keyPresent=($null -ne $k);disableMSI=if($k){$k.GetValue('DisableMSI')}}}finally{if($k){$k.Dispose()}}
        if($hive -eq 'CurrentUser'){
          foreach($keyName in $vendor,($vendor+'\023'),($vendor+'\023\Components')){
            $k=$reg.OpenSubKey($keyName);try{
              $values=@();if($k){foreach($name in $k.GetValueNames()){$values+=[ordered]@{name=$name;value=if($name -in '','InstallDir','Desktop Shortcut','Start Menu Shortcut','Uninstaller Shortcut','Desktop','ocx','ocxd','NativeShim','NativeConsole','NativeEntrySource','Documents','RuntimeBundle'){$k.GetValue($name,$null,[Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)};kind=$k.GetValueKind($name).ToString()}}}
              $r.vendor+=[ordered]@{view=$view;key=$keyName;present=($null -ne $k);values=$values}
            }finally{if($k){$k.Dispose()}}
          }
          $k=$reg.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Run');try{if($k -and $k.GetValueNames() -contains 'OpenCodeX Desktop'){$r.runValues+=[ordered]@{view=$view;present=$true}}}finally{if($k){$k.Dispose()}}
        }
      }finally{$reg.Dispose()}
    }
  }
  $desktop=[Environment]::GetFolderPath('DesktopDirectory');$programs=[Environment]::GetFolderPath('Programs')
  foreach($name in @((Join-Path $desktop 'OpenCodeX Desktop.lnk'),(Join-Path $programs 'OpenCodeX Desktop.lnk'),(Join-Path $programs 'OpenCodeX Desktop\OpenCodeX Desktop.lnk'),(Join-Path $programs 'OpenCodeX Desktop\Uninstall OpenCodeX Desktop.lnk'))){if(Test-Path -LiteralPath $name){$r.shortcuts+=[ordered]@{path=$name;sha256=Hash-File $name}}}
  return $r
}
function Install-Cell($which) {
  $receipt=Join-Path $dest ("install-"+$which+".json");if(Test-Path -LiteralPath $receipt){throw "Receipt already exists: $which"}
  $r=[ordered]@{cell=$which;utc=[DateTime]::UtcNow.ToString('o');sid=$sid;session=$session;stage='preflight';outcome='blocked';firstError=$null;exitCode=$null}
  try{
    $r.before=Read-Context
    $isMsi=$which.StartsWith('MSI');$custom=$which.EndsWith('custom');$target=if($custom){Join-Path $privateRoot ($which+'\OpenCodeX Desktop')}else{$default};$r.expectedRoot=$target
    Assert-PlainPath $target
    if($r.before.registrations.Count -or $r.before.shortcuts.Count -or (Test-Path -LiteralPath $target)){throw 'Existing product registration, shortcut or target directory. See before snapshot.'}
    if($r.before.runValues.Count -and !$isMsi){throw 'Existing OpenCodeX autostart value in current user profile.'}
    foreach($entry in $r.before.vendor){
      if($isMsi -and $entry.key -like '*\023*' -and $entry.values.Count){throw 'Existing MSI installation/component values. See before snapshot.'}
      if($entry.key -ne $vendor){continue}
      foreach($v in $entry.values){
        if($isMsi -and $v.name -in 'Desktop Shortcut','Start Menu Shortcut','Uninstaller Shortcut'){throw 'Existing MSI shortcut marker. See before snapshot.'}
        if(!$isMsi -and $v.name -eq '' -and $v.value -and ([string]$v.value).TrimEnd('\') -ine $default.TrimEnd('\')){throw 'Existing NSIS remembered directory. See before snapshot.'}
      }
    }
    if($isMsi){$pkg='OpenCodeX Desktop_0.2.24_x64_en-US.msi';$expected='780055175cf325f5a4f452806cb0042552a216506be558c9216e371af02b6e78'}else{$pkg='OpenCodeX Desktop_0.2.24_x64-setup.exe';$expected='988a55ecbbe662ce46260238401e007e9ec9ad266b69f9b375b443852ebaf65b'}
    $inputPath=Join-Path $PSScriptRoot $pkg;$r.packageSha256=Hash-File $inputPath;if($r.packageSha256 -ne $expected){throw 'Frozen installer digest mismatch.'}
    if($custom){Write-Host "In the wizard choose this exact folder: $target"}else{Write-Host "Keep the default folder: $target"}
    Write-Host 'At Finish, clear Launch / Run OpenCodeX Desktop. If asked for an administrator password, cancel.'
    if($isMsi){
      $log=Join-Path $env:USERPROFILE ("ocx-w2-team79-r1-"+$which+".msi.log");if(Test-Path -LiteralPath $log){throw 'Private MSI log path already exists.'}
      $r.privateLog=$log;$argsText='/i "'+$inputPath+'" /norestart /L*V "'+$log+'"'
    }
    $r.stage='installer';$r.outcome='fail';$clock=[Diagnostics.Stopwatch]::StartNew()
    if($isMsi){$proc=Start-Process -FilePath (Join-Path $env:SystemRoot 'System32\msiexec.exe') -ArgumentList $argsText -PassThru -Wait -ErrorAction Stop}
    else{$proc=Start-Process -FilePath $inputPath -PassThru -Wait -ErrorAction Stop}
    $clock.Stop();$r.exitCode=$proc.ExitCode;$r.elapsedMs=$clock.ElapsedMilliseconds
    if($isMsi -and (Test-Path -LiteralPath $log)){
      $r.logSha256=Hash-File $log
      $r.msiErrorLines=@(Select-String -LiteralPath $log -Pattern 'forbidden by system policy|installation is forbidden|Error [1-9][0-9]{3}[:. ]|returned actual error code [1-9]|MainEngineThread is returning [1-9]|Return value 3' | Select-Object -First 3 | ForEach-Object {$_.Line.Trim()})
    }
    if($proc.ExitCode -ne 0){$detail=if($isMsi -and $r.msiErrorLines){' | '+$r.msiErrorLines[0]}else{''};throw ("Installer exit="+$proc.ExitCode+$detail)}
    $r.stage='installed payload';$r.after=Read-Context
    $rows=@($r.after.registrations | Where-Object {$_.hive -eq 'CurrentUser' -and $_.version -eq '0.2.24' -and $_.location -and ([string]$_.location).Trim('"').TrimEnd('\') -ieq $target.TrimEnd('\')});if(!$rows.Count){throw 'Current-user candidate registration at expected target is missing.'}
    Assert-PlainPath $target
    $r.files=@();foreach($rel in 'opencodex-desktop.exe','resources\LICENSE','resources\THIRD-PARTY-NOTICES.txt','ocx.exe','ocxd.exe','ocx-native-entry.json','runtime-bundle\official-runtime.tgz','runtime-bundle\runtime-bundle-0.2.24-win32-x64.tar.gz','runtime-bundle\runtime-bundle.manifest.json','runtime-bundle\runtime-bundle.manifest.sha256'){
      $file=Join-Path $target $rel;Assert-PlainPath $file;$r.files+=[ordered]@{name=$rel;bytes=(Get-Item -LiteralPath $file).Length;sha256=Hash-File $file}
    }
    $official=@($r.files | Where-Object {$_['name'] -eq 'runtime-bundle\official-runtime.tgz'});if($official[0].sha256 -ne 'b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c'){throw 'Fixed official runtime SHA-256 mismatch.'}
    if($isMsi){
      $matched=$false
      foreach($entry in $r.after.vendor){
        if($entry.key -ne ($vendor+'\023\Components')){continue}
        $names=@($entry['values'] | Where-Object {([string]$_.value).TrimEnd('\') -ieq $target.TrimEnd('\')} | ForEach-Object {$_.name})
        $missing=@('Desktop','ocx','ocxd','NativeShim','NativeConsole','NativeEntrySource','Documents','RuntimeBundle' | Where-Object {$_ -notin $names});if(!$missing.Count){$matched=$true}
      }
      if(!$matched){throw 'MSI component registration does not bind all eight components to the selected root.'}
    }
    $r.outcome='pass';$r.stage='installed'
  }catch{$r.firstError=$_.Exception.Message}
  if(!$r.Contains('after')){try{$r.after=Read-Context}catch{$r.afterReadError=$_.Exception.Message}}
  $r.finishedUtc=[DateTime]::UtcNow.ToString('o');Save-New $receipt $r
  Write-Host "$which $($r.outcome) $($r.firstError)";return $r
}
if($role -eq 'console'){$path=Join-Path $dest 'installer-context.json';Save-New $path (Read-Context);Write-Host 'Administrator installation inventory recorded.';exit 0}
if($Cell -ne 'Auto'){$result=Install-Cell $Cell;if($result.outcome -eq 'pass'){exit 0};exit 1}
$first=Install-Cell 'MSI-default'
if($first.outcome -eq 'pass'){Write-Host 'Leave the installation in place.';exit 0}
if($first.exitCode -eq 1625 -or $first.stage -eq 'preflight'){
  Write-Host 'Checking the independent NSIS default installation.'
  $next=Install-Cell 'NSIS-default';if($next.outcome -eq 'pass'){Write-Host 'Leave the installation in place.';exit 0}
}
exit 1

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

if($role -ne 'rdp'){throw 'Run this read-only check in the marenben RDP desktop.'}
$receipt=Join-Path $dest 'rdp-installed-live.json'
if(Test-Path -LiteralPath $receipt){Write-Host 'EXISTING-RECEIPT';exit 0}
$r=[ordered]@{utc=[DateTime]::UtcNow.ToString('o');context=(Read-Context);processes=@();files=@();config=$null;log=$null;firstError=$null;result='unverified'}
try{
  $rows=@($r.context.registrations | Where-Object {$_.hive -eq 'CurrentUser' -and $_.name -eq 'OpenCodeX Desktop' -and $_.version -eq '0.2.24' -and ([string]$_.location).Trim('"').TrimEnd('\') -ieq $default -and ([string]$_.uninstall).Trim('"') -ieq (Join-Path $default 'uninstall.exe')})
  $r.nsisDefaultRegistrations=$rows.Count
  $app=Join-Path $default 'opencodex-desktop.exe'
  foreach($proc in @(Get-CimInstance Win32_Process -Filter "SessionId=$session" | Where-Object {$_.Name -in 'opencodex-desktop.exe','ocxd.exe','ocx.exe'})){
    $owner=Invoke-CimMethod -InputObject $proc -MethodName GetOwnerSid
    $r.processes+=[ordered]@{pid=$proc.ProcessId;session=$proc.SessionId;image=$proc.ExecutablePath;sid=$owner.Sid;ownerQueryCode=$owner.ReturnValue;parentPid=$proc.ParentProcessId}
  }
  if(!$rows.Count){throw 'No 0.2.24 NSIS current-user uninstaller registration at the exact default root.'}
  if(!@($r.processes | Where-Object {$_.image -ieq $app -and $_.sid -eq $sid -and $_.ownerQueryCode -eq 0}).Count){throw 'No running candidate desktop from the registered root with the expected SID.'}
  foreach($rel in 'opencodex-desktop.exe','resources\LICENSE','resources\THIRD-PARTY-NOTICES.txt','ocx.exe','ocxd.exe','ocx-native-entry.json','uninstall.exe','runtime-bundle\official-runtime.tgz','runtime-bundle\runtime-bundle-0.2.24-win32-x64.tar.gz','runtime-bundle\runtime-bundle.manifest.json','runtime-bundle\runtime-bundle.manifest.sha256'){
    $file=Join-Path $default $rel;Assert-PlainPath $file
    $r.files+=[ordered]@{name=$rel;bytes=(Get-Item -LiteralPath $file).Length;sha256=Hash-File $file}
  }
  $official=@($r.files | Where-Object {$_['name'] -eq 'runtime-bundle\official-runtime.tgz'})
  if($official[0].sha256 -ne 'b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c'){throw 'Fixed official runtime SHA-256 mismatch.'}
  $r.result='nsis-default-live-verified'
}catch{$r.firstError=$_.Exception.Message}
try{
  $config=Join-Path $env:USERPROFILE '.codex\config.toml'
  $r.config=[ordered]@{path=$config;exists=(Test-Path -LiteralPath $config -PathType Leaf)}
  if($r.config.exists){$configFile=Get-Item -LiteralPath $config;$r.config.mtimeUtc=$configFile.LastWriteTimeUtc.ToString('o');$r.config.sha256=Hash-File $config}
}catch{$r.configReadError=$_.Exception.Message}
try{
  $log=Join-Path $env:USERPROFILE 'ocx-w2-team79-r1-MSI-default.msi.log'
  $r.log=[ordered]@{path=$log;sha256=Hash-File $log;excerpts=@(Select-String -LiteralPath $log -Pattern '^(?:MSI \([sc]\).+?:\s+)(?:Machine policy value .DisableMsi.|User policy value .DisableMsi.|.*(?:forbidden by system policy|MainEngineThread is returning 1625|restricted by policy))' | Select-Object -First 15 | ForEach-Object {$_.Line.Trim()})}
}catch{$r.logReadError=$_.Exception.Message}
$r.finishedUtc=[DateTime]::UtcNow.ToString('o')
Save-New $receipt $r
Write-Host ('RESULT: '+$r.result)
if($r.firstError){Write-Host $r.firstError}

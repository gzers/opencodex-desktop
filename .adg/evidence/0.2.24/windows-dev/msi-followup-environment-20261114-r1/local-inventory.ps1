$ErrorActionPreference='Continue'
$e=Split-Path -Parent $PSCommandPath
Start-Transcript -LiteralPath "$e/local-inventory.transcript.log" -ErrorAction Stop|Out-Null
$report=[ordered]@{observedAt=[datetimeoffset]::Now.ToString('o');scope='read-only current host capability facts; no MSI execution';commands=@();errors=@()}
try{
 $id=[Security.Principal.WindowsIdentity]::GetCurrent();$report.token=@{sid=$id.User.Value;elevated=([Security.Principal.WindowsPrincipal]::new($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)}
 $report.os=Get-CimInstance Win32_OperatingSystem|Select-Object Caption,Version,BuildNumber,OSArchitecture
 foreach($hive in @([Microsoft.Win32.RegistryHive]::LocalMachine,[Microsoft.Win32.RegistryHive]::CurrentUser)){
  foreach($view in @([Microsoft.Win32.RegistryView]::Registry64,[Microsoft.Win32.RegistryView]::Registry32)){
   $base=[Microsoft.Win32.RegistryKey]::OpenBaseKey($hive,$view);$key=$base.OpenSubKey('SOFTWARE\Policies\Microsoft\Windows\Installer',$false)
   $row=@{hive=$hive.ToString();view=$view.ToString();keyExists=$null -ne $key;values=@{}}
   if($key){foreach($name in @('DisableMSI','AlwaysInstallElevated','DisableUserInstalls')){$row.values[$name]=$key.GetValue($name,$null,[Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)};$key.Dispose()}
   $report.commands+=@{command='read-only Installer policy registry';result=$row};$base.Dispose()
  }
 }
 $report.sessionCount=@(Get-PSSession).Count
 $report.sshConfigExists=Test-Path -LiteralPath 'C:/Users/15119/.ssh/config'
 $report.vmwareCommand=Get-Command vmrun -ErrorAction SilentlyContinue|Select-Object Name,Source
 $report.virtualBoxCommand=Get-Command VBoxManage -ErrorAction SilentlyContinue|Select-Object Name,Source
 $report.windowsSandboxCommand=Get-Command WindowsSandbox.exe -ErrorAction SilentlyContinue|Select-Object Name,Source
 try{Get-VM -ErrorAction Stop|Select-Object Name,State|ConvertTo-Json|Write-Output}catch{$report.errors+=@{command='Get-VM non-elevated';message=$_.Exception.Message};$_|Out-String|Write-Output}
 $msi='C:/Users/15119/.codex/worktrees/024-windows-native/code/test/windows024-native/target/w1d-bundle01/tauri/target/release/bundle/msi/OpenCodeX W1D Proof_0.2.24_x64_en-US.msi'
 $report.package=@{path=$msi;bytes=(Get-Item -LiteralPath $msi).Length;sha256=(Get-FileHash -LiteralPath $msi -Algorithm SHA256).Hash.ToLowerInvariant();matchesContract=$false}
 $report.package.matchesContract=($report.package.bytes -eq 316784640 -and $report.package.sha256 -ceq '9a2af03c31f35e1c87bcfe2c839a01142ca50cad8b58d1c79450a05381755282')
 $report.accountsRemaining=@(Get-LocalUser|Where-Object Name -like 'ocxw1d*').Count
 $report.profilesRemaining=@(Get-CimInstance Win32_UserProfile|Where-Object LocalPath -like 'C:\Users\ocxw1d*').Count
 $report.policyChanged=$false;$report.accountsCreated=0;$report.msiExecutions=0;$report.terminalTestsExecuted=0;$report.officialTestsExecuted=0
}catch{$report.errors+=@{command='local inventory';message=$_.Exception.Message}}
finally{$report|ConvertTo-Json -Depth 10|Set-Content -LiteralPath "$e/local-environment.json" -Encoding utf8;Stop-Transcript|Out-Null}

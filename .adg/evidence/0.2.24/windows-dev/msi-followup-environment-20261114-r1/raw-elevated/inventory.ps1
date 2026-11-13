$ErrorActionPreference='Continue'
$root='C:/Users/Public/ocx-msi-env-20261114-r1'
Start-Transcript -LiteralPath (Join-Path $root 'elevated-inventory.transcript.log') -ErrorAction Stop|Out-Null
$report=[ordered]@{at=[datetimeoffset]::Now.ToString('o');scope='read-only environment capability inspection, no VM start/create/change, no MSI execution';commands=@();errors=@();policyChanged=$false}
try {
 $id=[Security.Principal.WindowsIdentity]::GetCurrent();$report.token=@{sid=$id.User.Value;elevated=([Security.Principal.WindowsPrincipal]::new($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)}
 $report.os=Get-CimInstance Win32_OperatingSystem|Select-Object Caption,Version,BuildNumber,OSArchitecture
 $report.services=@(Get-Service vmms,vmcompute,WinRM,sshd -ErrorAction SilentlyContinue|Select-Object Name,Status,StartType)
 foreach($view in @([Microsoft.Win32.RegistryView]::Registry64,[Microsoft.Win32.RegistryView]::Registry32)){
  $base=[Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::LocalMachine,$view)
  $key=$base.OpenSubKey('SOFTWARE\Policies\Microsoft\Windows\Installer',$false)
  $policy=@{view=$view.ToString();keyExists=$null -ne $key;values=@{}}
  if($key){foreach($name in @('DisableMSI','AlwaysInstallElevated','DisableUserInstalls')){$policy.values[$name]=$key.GetValue($name,$null,[Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)};$key.Dispose()}
  $report.commands+=@{command='read-only Installer machine policy';result=$policy};$base.Dispose()
 }
 try{$vms=@(Get-VM -ErrorAction Stop|Select-Object Name,State,Version,Generation,Id);$report.virtualMachines=$vms;$report.commands+=@{command='Get-VM';succeeded=$true;count=$vms.Count};$vms|ConvertTo-Json -Depth 5|Write-Output}catch{$report.errors+=@{command='Get-VM';message=$_.Exception.Message}}
 try{$report.vmHost=Get-VMHost -ErrorAction Stop|Select-Object VirtualMachinePath,VirtualHardDiskPath}catch{$report.errors+=@{command='Get-VMHost';message=$_.Exception.Message}}
 $report.remoteSessions=@(Get-PSSession|Select-Object ComputerName,State,Availability)
 & wsl.exe --list --verbose 1> "$root/wsl.stdout.log" 2> "$root/wsl.stderr.log";$report.commands+=@{command='wsl --list --verbose';exitCode=$LASTEXITCODE}
 & docker.exe context ls --format '{{.Name}} {{.DockerEndpoint}}' 1> "$root/docker-context.stdout.log" 2> "$root/docker-context.stderr.log";$report.commands+=@{command='docker context ls';exitCode=$LASTEXITCODE}
 & docker.exe --context default info --format '{{.OSType}}' 1> "$root/docker-default.stdout.log" 2> "$root/docker-default.stderr.log";$report.commands+=@{command='docker --context default info';exitCode=$LASTEXITCODE}
} catch{$report.errors+=@{command='inventory';message=$_.Exception.Message}}
finally{$report|ConvertTo-Json -Depth 12|Set-Content -LiteralPath "$root/environment.json" -Encoding utf8;Stop-Transcript|Out-Null}

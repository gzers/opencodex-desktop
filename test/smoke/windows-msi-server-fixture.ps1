param(
 [Parameter(Mandatory=$true)][ValidateSet('prepare','restore')][string]$Mode,
 [Parameter(Mandatory=$true)][string]$EvidenceDirectory
)
$ErrorActionPreference='Stop'
if(!$IsWindows -or $env:GITHUB_ACTIONS -cne 'true' -or $env:RUNNER_ENVIRONMENT -cne 'github-hosted' -or $env:GITHUB_REPOSITORY -cne 'gzers/opencodex-desktop' -or $env:GITHUB_RUN_ATTEMPT -cne '1'){throw 'Disposable first-attempt hosted project Windows fixture only'}
. (Join-Path $PSScriptRoot 'windows-process-token.ps1')
$token=Get-WindowsHarnessToken
if(!$token.elevated){throw 'Server policy fixture requires the elevated harness, never the application token'}
$root=[IO.Path]::GetFullPath($EvidenceDirectory)
New-Item -ItemType Directory -Force $root|Out-Null
$statePath=Join-Path $root 'msi-server-policy.original.json'
$receiptPath=Join-Path $root 'msi-server-policy.restored.json'
$subkey='SOFTWARE\Policies\Microsoft\Windows\Installer'
$machine=[Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::LocalMachine,[Microsoft.Win32.RegistryView]::Registry64)
function Read-Policy{
 $key=$machine.OpenSubKey($subkey,$false)
 try{
  $present=$key -and 'DisableMSI' -in $key.GetValueNames()
  @{keyExisted=($null -ne $key);valueExisted=[bool]$present;value=$(if($present){$key.GetValue('DisableMSI')}else{$null});kind=$(if($present){$key.GetValueKind('DisableMSI').ToString()}else{$null})}
 }finally{if($key){$key.Dispose()}}
}
try{
 if($Mode -ceq 'prepare'){
  if(Test-Path -LiteralPath $statePath){throw 'Preserve existing fixture attempt; no retry'}
  $server=(Get-CimInstance Win32_OperatingSystem).ProductType -ne 1
  $prior=Read-Policy
  if($prior.valueExisted -and ($prior.kind -cne 'DWord' -or $prior.value -notin @(0,1,2))){throw 'Unknown installer policy; not changed'}
  $state=@{schemaVersion=1;run=$env:GITHUB_RUN_ID;attempt=$env:GITHUB_RUN_ATTEMPT;server=[bool]$server;changed=([bool]$server -and (!$prior.valueExisted -or $prior.value -ne 0));original=$prior;scope='temporary disposable Server fixture for unmanaged per-user MSI; normal Windows client behavior; no AlwaysInstallElevated or product privilege changes'}
  $state|ConvertTo-Json -Depth 5|Set-Content -LiteralPath $statePath -Encoding utf8
  if($state.changed){$key=$machine.CreateSubKey($subkey);try{$key.SetValue('DisableMSI',0,[Microsoft.Win32.RegistryValueKind]::DWord)}finally{$key.Dispose()};$read=Read-Policy;if(!$read.valueExisted -or $read.value -ne 0){throw 'Prepared Server policy readback differs'}}
 }else{
  if(!(Test-Path -LiteralPath $statePath)){throw 'Original Server policy evidence required'}
  $state=Get-Content -LiteralPath $statePath -Raw|ConvertFrom-Json
  if($state.schemaVersion -ne 1 -or $state.run -cne $env:GITHUB_RUN_ID -or $state.attempt -cne $env:GITHUB_RUN_ATTEMPT){throw 'Foreign policy fixture receipt refused'}
  $current=Read-Policy
  $alreadyOriginal=$current.keyExisted -eq $state.original.keyExisted -and $current.valueExisted -eq $state.original.valueExisted -and (!$current.valueExisted -or ($current.value -eq $state.original.value -and $current.kind -ceq $state.original.kind))
  if($state.changed -and !$alreadyOriginal){
   if(!$current.valueExisted -or $current.kind -cne 'DWord' -or $current.value -ne 0){throw 'Policy externally changed; original evidence retained, no overwrite'}
   $key=$machine.OpenSubKey($subkey,$true)
   try{if($state.original.valueExisted){$key.SetValue('DisableMSI',[int]$state.original.value,[Microsoft.Win32.RegistryValueKind]::DWord)}else{$key.DeleteValue('DisableMSI',$true)}}finally{$key.Dispose()}
   if(!$state.original.keyExisted){$key=$machine.OpenSubKey($subkey,$false);try{$empty=$key.GetValueNames().Count -eq 0 -and $key.GetSubKeyNames().Count -eq 0}finally{$key.Dispose()};if($empty){$machine.DeleteSubKey($subkey,$true)}}
  }
  $actual=Read-Policy
  if($actual.valueExisted -ne $state.original.valueExisted -or $actual.keyExisted -ne $state.original.keyExisted -or ($actual.valueExisted -and ($actual.value -ne $state.original.value -or $actual.kind -cne $state.original.kind))){throw 'Exact original installer policy was not restored'}
  @{result='pass';run=$env:GITHUB_RUN_ID;changed=$state.changed;restored=$actual;alwaysInstallElevatedChanged=$false}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath $receiptPath -Encoding utf8
 }
}finally{$machine.Dispose()}

param([Parameter(Mandatory)][string]$Config,[Parameter(Mandatory)][string]$ExpectedComputer)
$ErrorActionPreference='Stop'
$c=Get-Content -LiteralPath $Config -Raw|ConvertFrom-Json
if($env:COMPUTERNAME -ine $ExpectedComputer -or $c.expectedGuestComputer -ine $ExpectedComputer -or !$c.authorizedLocalAdministratorMsi -or !$c.testAdministrator -or !$c.msiOnly -or $c.componentOnly -or $c.testName -cne 'isolated_msi_uninstall_administrator'){throw 'explicit local disposable administrator MSI configuration required'}
$id=[Security.Principal.WindowsIdentity]::GetCurrent()
if(!([Security.Principal.WindowsPrincipal]::new($id)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'elevated controller required to create and clean one disposable test administrator'}
if(Test-Path -LiteralPath (Join-Path $c.output 'controller.json')){throw 'fresh controller evidence required'}
[void][IO.Directory]::CreateDirectory($c.output)
function Daily-Facts {
 $rows=@()
 foreach($target in @('User','Machine')){
  $value=[Environment]::GetEnvironmentVariable('Path',$target)
  $rows+=@{scope=$target;present=$null -ne $value;sha256=if($null -ne $value){[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::Unicode.GetBytes($value))).ToLowerInvariant()}else{$null}}
 }
 return ConvertTo-Json -InputObject $rows -Compress
}
$before=Daily-Facts
$before|Set-Content -LiteralPath (Join-Path $c.output 'daily-path-before.json') -Encoding utf8
try{
 $policies=@()
 foreach($hive in @('HKLM:','HKCU:')){
  $path=$hive+'/SOFTWARE/Policies/Microsoft/Windows/Installer'
  $value=if(Test-Path -LiteralPath $path){Get-ItemProperty -LiteralPath $path|Select-Object DisableMSI,AlwaysInstallElevated,DisableUserInstalls}else{$null}
  $policies+=@{path=$path;exists=Test-Path -LiteralPath $path;values=$value;noPolicyChanges=$true}
 }
 $policies|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $c.output 'controller-policy.json') -Encoding utf8
 if(@($policies|Where-Object {$_.values -and (($null -ne $_.values.DisableMSI -and $_.values.DisableMSI -notin @(0,1)) -or $_.values.AlwaysInstallElevated -eq 1 -or $_.values.DisableUserInstalls -eq 1)}).Count){throw 'explicit restrictive guest policy; no policy edit or MSI run'}
 & (Join-Path $PSScriptRoot 'query-package-msi.ps1') -Msi (Join-Path $c.input 'w1d.msi') -Output (Join-Path $c.output 'package-facts.json')
 if(!(Test-Path -LiteralPath $c.test -PathType Leaf)){throw 'actual MSVC administrator MSI test executable missing'}
 # A child pwsh keeps account controller exit from skipping this readback.
 & (Join-Path $PSHOME 'pwsh.exe') -NoLogo -NoProfile -NonInteractive -File (Join-Path $PSScriptRoot 'account-w1d.ps1') -Config $Config 1> (Join-Path $c.output 'account-controller.stdout.log') 2> (Join-Path $c.output 'account-controller.stderr.log')
 $code=$LASTEXITCODE
}finally{
 $after=Daily-Facts
 $after|Set-Content -LiteralPath (Join-Path $c.output 'daily-path-after.json') -Encoding utf8
 @{dailyUserAndMachinePathUnchanged=$before -ceq $after;scope='only controller daily user and machine PATH; private test SID recorded separately'}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $c.output 'daily-path-result.json') -Encoding utf8
 if($before -cne $after){throw 'external daily PATH change observed; no overwrite attempted'}
}
exit $code

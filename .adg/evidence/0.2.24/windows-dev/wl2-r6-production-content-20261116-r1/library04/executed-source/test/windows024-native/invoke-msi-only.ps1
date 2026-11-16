param([Parameter(Mandatory)][string]$Config,[Parameter(Mandatory)][string]$ExpectedGuestComputer)
$ErrorActionPreference='Stop'
if($env:COMPUTERNAME -ieq 'EZIO-MINI' -or $env:COMPUTERNAME -ine $ExpectedGuestComputer){throw 'daily host excluded; exact prepared isolated Windows guest required'}
$c=Get-Content -LiteralPath $Config -Raw|ConvertFrom-Json
if(!$c.preparedPerUserMsiGuest -or !$c.msiOnly -or $c.expectedGuestComputer -ine $ExpectedGuestComputer -or $c.componentOnly -or $c.testName -cne 'isolated_msi_uninstall_only'){throw 'explicit MSI-only guest config required; no fallback or policy change'}
if(Test-Path -LiteralPath (Join-Path $c.output 'controller.json')){throw 'fresh controller evidence required'}
# The caller supplies an already permitted guest. Never infer policy permission
# from an absent registry key and never edit Installer or machine security policy.
[void][IO.Directory]::CreateDirectory($c.output)
foreach($hive in @('HKLM:','HKCU:')){
 $path=$hive+'/SOFTWARE/Policies/Microsoft/Windows/Installer'
 $facts=if(Test-Path -LiteralPath $path){Get-ItemProperty -LiteralPath $path|Select-Object DisableMSI,AlwaysInstallElevated,DisableUserInstalls}else{$null}
 @{path=$path;exists=Test-Path -LiteralPath $path;values=$facts;absenceDoesNotProvePermission=$true}|ConvertTo-Json -Depth 4|Set-Content -LiteralPath (Join-Path $c.output ($hive.TrimEnd(':')+'-policy.json')) -Encoding utf8
 if($facts -and (($null -ne $facts.DisableMSI -and $facts.DisableMSI -ne 0) -or $facts.AlwaysInstallElevated -eq 1 -or $facts.DisableUserInstalls -eq 1)){throw 'guest policy incompatible with ordinary per-user test; no account or MSI run'}
}
& (Join-Path $PSScriptRoot 'query-package-msi.ps1') -Msi (Join-Path $c.input 'w1d.msi') -Output (Join-Path $c.output 'package-facts.json')
if(!(Test-Path -LiteralPath $c.test -PathType Leaf)){throw 'actual MSVC MSI-only test executable missing'}
& (Join-Path $PSScriptRoot 'account-w1d.ps1') -Config $Config
exit $LASTEXITCODE

param([Parameter(Mandatory)][string]$Config,[Parameter(Mandatory)][string]$ExpectedComputer,[switch]$Elevated)
$ErrorActionPreference='Stop'
$c=Get-Content -LiteralPath $Config -Raw|ConvertFrom-Json
if(!$c.authorizedLocalAdministratorMsi -or !$c.testAdministrator -or !$c.msiOnly -or $c.expectedGuestComputer -ine $ExpectedComputer -or $env:COMPUTERNAME -ine $ExpectedComputer){throw 'explicit local administrator scenario required'}
if($Elevated){
 $code=1
 try{
  & (Join-Path $PSHOME 'pwsh.exe') -NoLogo -NoProfile -NonInteractive -File (Join-Path $PSScriptRoot 'invoke-msi-local-admin.ps1') -Config $Config -ExpectedComputer $ExpectedComputer 1> (Join-Path $c.output 'wrapper.stdout.log') 2> (Join-Path $c.output 'wrapper.stderr.log')
  $code=$LASTEXITCODE
 }catch{$_|Out-String|Set-Content -LiteralPath (Join-Path $c.output 'launcher-error.log') -Encoding utf8}
 finally{@{exitCode=$code;at=[datetime]::Now.ToString('o');scope='dedicated administrator MSI scenario; not ordinary-user acceptance'}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $c.output 'launch-result.json') -Encoding utf8}
 exit $code
}
if(Test-Path -LiteralPath (Join-Path $c.output 'launch.json')){throw 'fresh launcher evidence required'}
$process=Start-Process -FilePath (Join-Path $PSHOME 'pwsh.exe') -Verb RunAs -WindowStyle Hidden -ArgumentList @('-NoLogo','-NoProfile','-NonInteractive','-File',('"'+$PSCommandPath+'"'),'-Config',('"'+$Config+'"'),'-ExpectedComputer',$ExpectedComputer,'-Elevated') -PassThru
@{pid=$process.Id;creation=$process.StartTime.ToString('o');config=$Config;expectedComputer=$ExpectedComputer}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $c.output 'launch.json') -Encoding utf8
$process.Dispose()

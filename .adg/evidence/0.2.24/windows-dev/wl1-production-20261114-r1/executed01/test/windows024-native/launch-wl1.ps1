param([Parameter(Mandatory)][string]$Config,[switch]$Elevated)
$ErrorActionPreference='Stop'
$c=Get-Content -LiteralPath $Config -Raw|ConvertFrom-Json
if($c.testName -cne 'isolated_production_lifecycle_wl1' -or $c.msiOnly -or $c.testAdministrator -or $c.componentOnly){throw 'one ordinary W-L1 Profile required; no MSI/CLI scenario'}
if($Elevated){
 $code=1
 try{
  & (Join-Path $PSHOME 'pwsh.exe') -NoLogo -NoProfile -NonInteractive -File (Join-Path $PSScriptRoot 'account-w1d.ps1') -Config $Config 1> (Join-Path $c.output 'controller.stdout.log') 2> (Join-Path $c.output 'controller.stderr.log')
  $code=$LASTEXITCODE
 }catch{$_|Out-String|Set-Content -LiteralPath (Join-Path $c.output 'launcher-error.log') -Encoding utf8}
 finally{@{exitCode=$code;at=[datetime]::Now.ToString('o');scope='one fresh ordinary W-L1 account; no full acceptance inferred from launcher exit'}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $c.output 'launch-result.json') -Encoding utf8}
 exit $code
}
if(Test-Path -LiteralPath (Join-Path $c.output 'launch.json')){throw 'fresh launch evidence required'}
$p=Start-Process -FilePath (Join-Path $PSHOME 'pwsh.exe') -Verb RunAs -WindowStyle Hidden -ArgumentList @('-NoLogo','-NoProfile','-NonInteractive','-File',('"'+$PSCommandPath+'"'),'-Config',('"'+$Config+'"'),'-Elevated') -PassThru
@{pid=$p.Id;creation=$p.StartTime.ToString('o');config=$Config}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $c.output 'launch.json') -Encoding utf8
$p.Dispose()

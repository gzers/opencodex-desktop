param([Parameter(Mandatory)][string]$Config,[switch]$Elevated)
$ErrorActionPreference='Stop'
$c=Get-Content -LiteralPath $Config -Raw|ConvertFrom-Json
if($c.testName -cne 'isolated_wl2_native_boundaries' -or $c.msiOnly -or $c.testAdministrator -or !$c.componentOnly){throw 'one ordinary W-L2 prerequisites Profile required; no MSI/CLI scenario'}
if($Elevated){
 $code=1
 function DailyPathFacts {
  @('User','Machine')|ForEach-Object {$value=[Environment]::GetEnvironmentVariable('Path',$_);@{scope=$_;present=$null -ne $value;sha256=if($null -ne $value){[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::Unicode.GetBytes($value))).ToLowerInvariant()}else{$null}}}|ConvertTo-Json -Compress
 }
 $before=DailyPathFacts
 $before|Set-Content -LiteralPath (Join-Path $c.output 'daily-path-before.json') -Encoding utf8
 try{
  & (Join-Path $PSHOME 'pwsh.exe') -NoLogo -NoProfile -NonInteractive -File (Join-Path $PSScriptRoot 'account-w1d.ps1') -Config $Config 1> (Join-Path $c.output 'controller.stdout.log') 2> (Join-Path $c.output 'controller.stderr.log')
  $code=$LASTEXITCODE
 }catch{$_|Out-String|Set-Content -LiteralPath (Join-Path $c.output 'launcher-error.log') -Encoding utf8}
 finally{
  $after=DailyPathFacts
  $after|Set-Content -LiteralPath (Join-Path $c.output 'daily-path-after.json') -Encoding utf8
  @{unchanged=$before -ceq $after;scope='daily controller user and machine PATH digests'}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $c.output 'daily-path-result.json') -Encoding utf8
  if($before -cne $after){$code=1}
  @{exitCode=$code;at=[datetime]::Now.ToString('o');scope='one fresh ordinary W-L2 prerequisites account; no full acceptance inferred from launcher exit'}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $c.output 'launch-result.json') -Encoding utf8
 }
 exit $code
}
if(Test-Path -LiteralPath (Join-Path $c.output 'launch.json')){throw 'fresh launch evidence required'}
$p=Start-Process -FilePath (Join-Path $PSHOME 'pwsh.exe') -Verb RunAs -WindowStyle Hidden -ArgumentList @('-NoLogo','-NoProfile','-NonInteractive','-File',('"'+$PSCommandPath+'"'),'-Config',('"'+$Config+'"'),'-Elevated') -PassThru
@{pid=$p.Id;creation=$p.StartTime.ToString('o');config=$Config}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $c.output 'launch.json') -Encoding utf8
$p.Dispose()

param([Parameter(Mandatory=$true)][ValidateSet('console','rdp')][string]$Role,[Parameter(Mandatory=$true)][ValidateSet('start','conflict','observe','stop','inspect','await-tray-exit','finish')][string]$Mode,[int]$Port=0,[int]$ExpectedPid=0,[string]$RunName='sessions-02')
$ErrorActionPreference='Stop'
$dir=Join-Path $PSScriptRoot ($RunName+'/'+$Role)
if(!(Test-Path -LiteralPath (Join-Path $dir 'ready.json'))){throw 'Original session readiness receipt required'}
if(Test-Path -LiteralPath (Join-Path $dir 'finished.json')){throw 'Original session already finished'}
$commands=@(Get-ChildItem -LiteralPath $dir -Filter 'command-*.json')
$sequence=$commands.Count+1
$path=Join-Path $dir ('command-'+$sequence+'.json')
@{mode=$Mode;port=$Port;expectedPid=$ExpectedPid} | ConvertTo-Json | Set-Content -LiteralPath ($path+'.writing')
[IO.File]::Move(($path+'.writing'),$path)
@{role=$Role;mode=$Mode;sequence=$sequence;reply=(Join-Path $dir ('reply-'+$sequence+'.json'))} | ConvertTo-Json

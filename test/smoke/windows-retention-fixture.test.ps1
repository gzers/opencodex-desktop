param([Parameter(Mandatory=$true)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'windows-retention-fixture.ps1')
$base=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $base){throw 'New output required'}
New-Item -ItemType Directory -Path $base|Out-Null
function Refuses([scriptblock]$Action){$refused=$false;try{& $Action}catch{$refused=$true};if(!$refused){throw 'Unsafe retention fixture was accepted'}}
$own=New-InstallerRetentionFixture (Join-Path $base 'own')
$before=[IO.File]::ReadAllBytes($own.path)
Refuses {New-InstallerRetentionFixture $own.root}
if((Get-FileHash $own.path).Hash -cne $own.hash){throw 'Existing fixture changed'}
$done=Remove-InstallerRetentionFixture $own
if(!$done.removed -or (Test-Path $own.root)){throw 'Exact cleanup failed'}
$extra=New-InstallerRetentionFixture (Join-Path $base 'extra')
[IO.File]::WriteAllText((Join-Path $extra.directory 'other'),'keep')
Refuses {Remove-InstallerRetentionFixture $extra}
if(!(Test-Path $extra.path) -or [IO.File]::ReadAllText((Join-Path $extra.directory 'other')) -cne 'keep'){throw 'Additional data was changed'}
Remove-Item -LiteralPath (Join-Path $extra.directory 'other')
$null=Remove-InstallerRetentionFixture $extra
$changed=New-InstallerRetentionFixture (Join-Path $base 'changed')
[IO.File]::WriteAllText($changed.path,'changed')
Refuses {Remove-InstallerRetentionFixture $changed}
if([IO.File]::ReadAllText($changed.path) -cne 'changed'){throw 'Changed data removed'}
[IO.File]::WriteAllBytes($changed.path,$before)
$null=Remove-InstallerRetentionFixture $changed
@{result='pass';checks=5;scope='actual exact filesystem fixture lifecycle and refusal; no installation accepted'}|ConvertTo-Json|Set-Content (Join-Path $base 'checks.json')
Write-Output 'PASS exact new retention fixture, existing/extra/changed refusal; no recursive removal'

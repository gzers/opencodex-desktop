param([Parameter(Mandatory)][string]$Target,[Parameter(Mandatory)][string]$Output)
$ErrorActionPreference='Stop'
$r=@{pid=$PID;attempted=$true;written=$false;refused=$false;failureType=$null}
try{[IO.File]::WriteAllText($Target,'external-recreated-file');$r.written=$true}catch{$r.refused=$true;$r.failureType=$_.Exception.GetType().FullName}
[IO.File]::WriteAllText($Output,($r|ConvertTo-Json))

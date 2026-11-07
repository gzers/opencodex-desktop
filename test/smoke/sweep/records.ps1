# Test orchestration only. A diagnostic record can never authorize acceptance.
function Write-SweepRecord {
 param([string]$Directory,[string]$Step,[string]$Name,[ValidateSet('pass','fail')][string]$Status,[string]$FirstError='',[object]$Evidence=$null,[string[]]$Dependencies=@())
 if($Step -notmatch '^[0-9]{2,3}$' -or $Name -notmatch '^[a-z0-9][a-z0-9-]*$'){throw 'Invalid diagnostic record identity'}
 $directoryPath=[IO.Path]::GetFullPath($Directory)
 [IO.Directory]::CreateDirectory($directoryPath)|Out-Null
 $record=[ordered]@{schemaVersion=1;step=$Step;name=$Name;status=$Status;firstError=($FirstError -split "\r?\n",2)[0];accepted=$false;diagnosticOnly=$true;dependencies=@($Dependencies);evidence=$Evidence;atUtc=[datetime]::UtcNow.ToString('o')}
 if($Status -ceq 'fail' -and !$record.firstError){throw 'A diagnostic failure needs its actual error or unmet prerequisite'}
 $path=Join-Path $directoryPath ($Step+'-'+$Name+'.json')
 # CreateNew prevents successful re-execution from replacing an earlier failure.
 $stream=[IO.File]::Open($path,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try{$bytes=[Text.UTF8Encoding]::new($false).GetBytes(($record|ConvertTo-Json -Depth 15));$stream.Write($bytes,0,$bytes.Length)}finally{$stream.Dispose()}
 return $record
}
function Invoke-SweepCheck {
 param([string]$Directory,[string]$Step,[string]$Name,[scriptblock]$Action,[string[]]$Dependencies=@())
 try{ $value=& $Action; Write-SweepRecord $Directory $Step $Name pass -Evidence $value -Dependencies $Dependencies }
 catch{ Write-SweepRecord $Directory $Step $Name fail -FirstError $_.Exception.Message -Evidence @{errorType=$_.Exception.GetType().FullName;line=$_.InvocationInfo.ScriptLineNumber} -Dependencies $Dependencies }
}

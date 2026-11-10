param([Parameter(Mandatory)][string]$Directory,[Parameter(Mandatory)][string]$HistoricalEvidence,[Parameter(Mandatory)][string]$Bun)
$ErrorActionPreference='Stop';Set-StrictMode -Version Latest
Add-Type -Path (Join-Path $PSScriptRoot Native.cs)
if(Test-Path -LiteralPath $Directory){throw 'fresh bridge test required'};$null=New-Item -ItemType Directory -Path $Directory
$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value;[Official299Proof.Native]::SetFixtureDacl($Directory,('D:P(A;OICI;GA;;;'+$sid+')(A;OICI;GA;;;SY)'),$true)
$held=[Collections.Generic.List[IDisposable]]::new();$rows=@();$pass=$false;$errorFact=$null
function Seal([string]$path){$held.Add([IO.File]::Open($path,'Open','Read','Read'));$held.Add([Official299Proof.Guard]::new($path,$false))}
function Run-Bridge([string]$output){
 $ev=@{};foreach($e in [Environment]::GetEnvironmentVariables().GetEnumerator()){$ev[$e.Key]=$e.Value}
 $args=@((Join-Path $PSScriptRoot cache-candidate-cli.ts),(Join-Path $Directory before.json),(Join-Path $Directory ready.json),(Join-Path $Directory pre-stop.json),(Join-Path $Directory official-stop.json),(Join-Path $Directory context.json),(Join-Path $Directory scope.json),$output)
 $cmd='"'+$Bun+'" run';foreach($arg in $args){$cmd+=' "'+$arg+'"'}
 $label=[guid]::NewGuid().ToString('N');$p=[Official299Proof.Owned]::new($Bun,$cmd,$Directory,$ev,(Join-Path $Directory ($label+'.stdout.log')),(Join-Path $Directory ($label+'.stderr.log')),$sid)
 try{$p.Resume();$sw=[Diagnostics.Stopwatch]::StartNew();while((!$p.Exited() -or $p.Active() -gt 0) -and $sw.ElapsedMilliseconds -lt 4000){Start-Sleep -Milliseconds 10};if(!$p.Exited() -or $p.Active() -ne 0){throw 'small bridge did not finish'};return @{identity=$p.Identity;exitCode=$p.ExitCode();jobActive=$p.Active();elapsedMs=$sw.ElapsedMilliseconds;stderr=(Get-Content (Join-Path $Directory ($label+'.stderr.log')) -Raw)}}finally{if($p.Active() -gt 0){$p.ForceOwnedCleanup()};$p.Dispose()}
}
try{
 $prefix=Join-Path $HistoricalEvidence attempt-r4-01/ordinary
 foreach($stage in @('before','ready','pre-stop','official-stop')){$path=Join-Path $Directory ($stage+'.json');Copy-Item -LiteralPath (Join-Path $prefix ('capture-'+$stage+'.json')) -Destination $path;Seal $path}
 $c=Get-Content (Join-Path $prefix result.json) -Raw|ConvertFrom-Json
 # The historical elapsedMs is used only in replay. The live host must supply its actual current stopwatch.
 $cp=Join-Path $Directory context.json;[IO.File]::WriteAllText($cp,($c|ConvertTo-Json -Depth 40));Seal $cp
 $scope=@{fixture=$c.fixture;sid=$c.runtime.sid;declaredRoutedSlugs=@('fixture/gpt-6.1-sol');scope='historical immutable synthetic replay, not new native authority'};$sp=Join-Path $Directory scope.json;[IO.File]::WriteAllText($sp,($scope|ConvertTo-Json));Seal $sp
 $denied=0;foreach($path in @($cp,$sp,(Join-Path $Directory ready.json))){try{[IO.File]::WriteAllText($path,'forbidden');throw 'sealed input was writable'}catch [Management.Automation.MethodInvocationException]{$denied++}}
 if($denied -ne 3){throw 'protected stage/context/scope write lock not proven'};$rows+=@{case='native-no-write-or-delete-sharing-input-handles';pass=$true;targets=3}
 $output=Join-Path $Directory authorization.json;$first=Run-Bridge $output;$auth=Get-Content $output -Raw|ConvertFrom-Json
 if($first.exitCode -ne 0 -or !$auth.accepted -or !$auth.cacheCandidate.accepted){throw 'candidate CLI rejected immutable r4 replay'}
 $digest=(Get-FileHash $output).Hash;$second=Run-Bridge $output
 if($second.exitCode -eq 0 -or (Get-FileHash $output).Hash -cne $digest){throw 'create-only output overwritten'}
 $rows+=@{case='create-only-cli-output';pass=$true;first=$first;second=$second;perPathOnly=$true;historicalServiceAcceptedRemains=$false}
 $pass=$true
}catch{$errorFact=@{type=$_.Exception.GetType().FullName;message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
finally{foreach($h in $held){$h.Dispose()};[IO.File]::WriteAllText((Join-Path $Directory bridge-result.json),(@{pass=$pass;checks=$rows;error=$errorFact;scope='protected historical stage replay and native input locks only; no live service';utc=[datetime]::UtcNow.ToString('o')}|ConvertTo-Json -Depth 20))}
if(!$pass){exit 1}

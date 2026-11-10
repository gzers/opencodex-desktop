param([Parameter(Mandatory)][string]$Directory,[Parameter(Mandatory)][string]$Bun,[Parameter(Mandatory)][string]$Official)
$ErrorActionPreference='Stop';Set-StrictMode -Version Latest
Add-Type -Path (Join-Path $PSScriptRoot Native.cs),(Join-Path $PSScriptRoot Recovery.cs),(Join-Path $PSScriptRoot FiveRecovery.cs)
if(Test-Path -LiteralPath $Directory){throw 'new coordinator fixture required'};$null=New-Item -ItemType Directory -Path $Directory
$currentSid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
[Official299Proof.Native]::SetFixtureDacl($Directory,('D:P(A;OICI;GA;;;'+$currentSid+')(A;OICI;GA;;;SY)'),$true)
foreach($leaf in @('codex','grok','runtime')){$null=New-Item -ItemType Directory -Path (Join-Path $Directory $leaf)}
[IO.File]::WriteAllText((Join-Path $Directory runtime/config.json),(@{providers=@{}}|ConvertTo-Json))
$paths=[Official299Proof.FiveRecovery]::Paths
foreach($rel in $paths){[IO.File]::WriteAllText((Join-Path $Directory $rel),'synthetic-preimage')}
[IO.File]::WriteAllText((Join-Path $Directory codex/config.toml),'model = "synthetic-original"')
[IO.File]::WriteAllText((Join-Path $Directory codex/opencodex-catalog.json),'{"models":[]}')
[IO.File]::WriteAllText((Join-Path $Directory codex/models_cache.json),'{"models":[]}')
$before=[Official299Proof.FiveRecovery]::Capture($Directory,'before',$paths)
foreach($rel in $paths){[IO.File]::WriteAllText((Join-Path $Directory $rel),'synthetic-produced-after')}
[IO.File]::WriteAllText((Join-Path $Directory codex/config.toml),'model = "synthetic-produced"')
[IO.File]::WriteAllText((Join-Path $Directory codex/opencodex-catalog.json),'{"models":[]}')
[IO.File]::WriteAllText((Join-Path $Directory codex/models_cache.json),'{"models":[]}')
$after=[Official299Proof.FiveRecovery]::Capture($Directory,'after',$paths)
$plan=[Official299Proof.FiveRecovery]::Plan($Directory,$before,$after,[guid]::NewGuid().ToString('N'),('b'*64),'synthetic-coordinator-fixture')
$pf=Join-Path $Directory plan.json;$digest=[Official299Proof.FiveRecovery]::SavePlan($pf,$plan)
$ev=@{};foreach($e in [Environment]::GetEnvironmentVariables().GetEnumerator()){$ev[$e.Key]=$e.Value}
foreach($key in @($ev.Keys)){if($key -match '^(OPENCODEX|CODEX|CLAUDE|GROK|OCX_)'){$ev.Remove($key)}}
$ev['OPENCODEX_HOME']=Join-Path $Directory runtime;$ev['CODEX_HOME']=Join-Path $Directory codex
$ready=Join-Path $Directory ready.json;$release=Join-Path $Directory release;$result=Join-Path $Directory coordinator-result.json
$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value;$p=$null;$pass=$false;$r=@{pass=$false;error=$null}
try{
 $p=[Official299Proof.Owned]::new($Bun,('"'+$Bun+'" run "'+(Join-Path $PSScriptRoot coordinator-r4.ts)+'" "'+$Official+'" "'+$ready+'" "'+$release+'" "'+$result+'" 2000'),$Directory,$ev,(Join-Path $Directory stdout.log),(Join-Path $Directory stderr.log),$sid);$p.Resume()
 $sw=[Diagnostics.Stopwatch]::StartNew();while(!(Test-Path -LiteralPath $ready) -and !$p.Exited() -and $sw.ElapsedMilliseconds -lt 6000){Start-Sleep -Milliseconds 10}
 if(!(Test-Path -LiteralPath $ready)){throw 'official coordination did not reach native barrier'}
 $held=Get-Content $ready -Raw|ConvertFrom-Json;if(!$held.claim -or !$held.n -or !$held.k){throw 'missing official lock hold'}
 $ops=[Official299Proof.FiveRecovery]::Apply($pf,$digest,(Join-Path $Directory intent.jsonl),$false,0,0,$null)
 [IO.File]::WriteAllText($release,'native-host-completed')
 $sw.Restart();while((!$p.Exited() -or $p.Active() -gt 0) -and $sw.ElapsedMilliseconds -lt 4000){Start-Sleep -Milliseconds 10}
 $actual=Get-Content $result -Raw|ConvertFrom-Json
 if(!$p.Exited() -or $p.ExitCode() -ne 0 -or $p.Active() -ne 0 -or !$actual.released -or ![Official299Proof.FiveRecovery]::Equal($before,([Official299Proof.FiveRecovery]::Capture($Directory,'final',$paths)),$false)){throw 'coordinated native apply or release failed'}
 $r=@{pass=$true;held=$held;released=$actual;workerIdentity=$p.Identity;exitCode=$p.ExitCode();jobActive=$p.Active();operations=$ops;scope='bounded test rendezvous while unchanged official claim/N/K held, no subprocess spawned under lock callback; no product integration claim'}
}catch{$r.error=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
finally{if($p){if($p.Active() -gt 0){$p.ForceOwnedCleanup()};$p.Dispose()};[IO.File]::WriteAllText((Join-Path $Directory prototype-result.json),($r|ConvertTo-Json -Depth 20))}
if(!$r.pass){exit 1}

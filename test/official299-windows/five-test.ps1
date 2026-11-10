param([Parameter(Mandatory)][string]$Directory)
$ErrorActionPreference='Stop';Set-StrictMode -Version Latest
Add-Type -Path (Join-Path $PSScriptRoot Native.cs),(Join-Path $PSScriptRoot Recovery.cs),(Join-Path $PSScriptRoot FiveRecovery.cs)
if(Test-Path -LiteralPath $Directory){throw 'fresh prototype only'};$null=New-Item -ItemType Directory -Path $Directory
$paths=[Official299Proof.FiveRecovery]::Paths;$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
[Official299Proof.Native]::SetFixtureDacl($Directory,('D:P(A;OICI;GA;;;'+$sid+')(A;OICI;GA;;;SY)'),$true)
$checks=[Collections.Generic.List[object]]::new();$cases=[Collections.Generic.List[object]]::new();$pass=$false
function Take([string]$root,[string]$stage){[Official299Proof.FiveRecovery]::Capture($root,$stage,$paths)}
function Setup([string]$label,[bool]$existing=$false){
 $root=Join-Path $Directory $label;$null=New-Item -ItemType Directory -Path (Join-Path $root codex),(Join-Path $root grok) -Force
 foreach($rel in $paths){if($existing -or $rel -in @('codex/config.toml','codex/opencodex-catalog.json')){[IO.File]::WriteAllText((Join-Path $root $rel),('synthetic-original:'+ $rel))}}
 $before=Take $root before
 foreach($rel in $paths){[IO.File]::WriteAllText((Join-Path $root $rel),('synthetic-producer-postimage:'+ $rel))}
 $after=Take $root postimage
 $plan=[Official299Proof.FiveRecovery]::Plan($root,$before,$after,[guid]::NewGuid().ToString('N'),('a'*64),'small-controlled-synthetic-producer')
 $pf=Join-Path $root protected-plan.json;$digest=[Official299Proof.FiveRecovery]::SavePlan($pf,$plan)
 $f=@{root=$root;before=$before;after=$after;plan=$pf;digest=$digest;journal=(Join-Path $root intent.jsonl)};$cases.Add($f);return $f
}
function Run-Child($f,[bool]$recover,[int]$crash=0){
 $i=[Diagnostics.ProcessStartInfo]::new();$i.FileName=Join-Path $PSHOME pwsh.exe;$i.UseShellExecute=$false;$i.CreateNoWindow=$true;$i.RedirectStandardOutput=$true;$i.RedirectStandardError=$true
 foreach($arg in @('-NoLogo','-NoProfile','-NonInteractive','-File',(Join-Path $PSScriptRoot five-worker.ps1),'-Plan',$f.plan,'-Digest',$f.digest,'-Journal',$f.journal,'-CrashAfter',[string]$crash)){$i.ArgumentList.Add($arg)};if($recover){$i.ArgumentList.Add('-Recover')}
 $p=[Diagnostics.Process]::Start($i);$p.SafeHandle|Out-Null;$id=[Official299Proof.Native]::Observe($p.Handle);$stdout=$p.StandardOutput.ReadToEndAsync();$stderr=$p.StandardError.ReadToEndAsync();if(!$p.WaitForExit(10000)){throw 'test child timeout'};$result=@{identity=$id;exitCode=$p.ExitCode;stdout=$stdout.GetAwaiter().GetResult();stderr=$stderr.GetAwaiter().GetResult()};$p.Dispose();return $result
}
try{
 $capture=Setup capture
 [IO.File]::WriteAllText((Join-Path $capture.root 'codex/opencodex-catalog.json'),'{invalid-changed-json')
 $cache=Join-Path $capture.root codex/models_cache.json;$saved=(Get-Acl $cache).GetSecurityDescriptorSddlForm('Access');[Official299Proof.Native]::SetFixtureDacl($cache,'D:P(A;;GA;;;SY)',$true)
 Remove-Item -LiteralPath (Join-Path $capture.root codex/opencodex-catalog-audit.jsonl)
 Remove-Item -LiteralPath (Join-Path $capture.root grok/config.toml.bak-opencodex)
 $null=New-Item -ItemType SymbolicLink -Path (Join-Path $capture.root grok/config.toml.bak-opencodex) -Target (Join-Path $capture.root codex/config.toml)
 try{$rows=Take $capture.root failed-finally}finally{[Official299Proof.Native]::SetFixtureDacl($cache,$saved,$true)}
 if(($rows|Where-Object Path -eq 'codex/models_cache.json').State -cne 'unknown' -or ($rows|Where-Object Path -eq 'codex/opencodex-catalog-audit.jsonl').State -cne 'absent' -or ($rows|Where-Object Path -eq 'grok/config.toml.bak-opencodex').State -cne 'unknown'){throw 'capture misclassified unknown/absence/link'}
 $invalid=$rows|Where-Object Path -eq 'codex/opencodex-catalog.json';if([Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($invalid.Image.Bytes)) -cne '{invalid-changed-json'){throw 'invalid different postimage lost'}
 foreach($row in @($rows|Where-Object State -eq file)){if([Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Convert]::FromBase64String($row.Image.Bytes))).ToLowerInvariant() -cne $row.Image.Sha256 -or [Convert]::FromBase64String($row.Image.Bytes).LongLength -ne $row.Length){throw 'raw capture digest/length mismatch'}}
 try{[void][Official299Proof.FiveRecovery]::Capture($capture.root,'bad',@('runtime/runtime-port.json'));throw 'unregistered secret target admitted'}catch [Management.Automation.MethodInvocationException]{}
 $checks.Add(@{case='changed-invalid-unreadable-absent-link-continued-raw-capture';pass=$true;rows=$rows;unregisteredRuntimeSecretRejected=$true})
 foreach($existing in @($false,$true)){$f=Setup ('normal-'+$existing) $existing;[IO.File]::WriteAllText((Join-Path $f.root codex/models_cache-user.json),'keep-unknown-user');$ops=[Official299Proof.FiveRecovery]::Apply($f.plan,$f.digest,$f.journal,$false,0,0,$null);if(![Official299Proof.FiveRecovery]::Equal($f.before,(Take $f.root restored),$false)){throw 'five original objects differ'};if([IO.File]::ReadAllText((Join-Path $f.root codex/models_cache-user.json)) -cne 'keep-unknown-user'){throw 'unknown extra erased'};$checks.Add(@{case=('normal-five-existing-aux-'+$existing);pass=$true;operations=$ops;unknownSamePrefixExtraRetained=$true;auditPostimageArchivedInPlan=$true})}
 foreach($label in @('external-edit','audit-append','symbolic-link','hardlink','unreadable','missing','tampered-plan','tampered-original','parent-link','second-write-failure','absent-slot-race','separate-writer')){
  $f=Setup $label ($label -eq 'absent-slot-race');$target=Join-Path $f.root grok/config.toml.bak-opencodex;$first=Join-Path $f.root codex/config.toml;$firstHash=(Get-FileHash $first).Hash;$acl=$null;$hook=$null;$failed=$null;$parentMoved=$null
  switch($label){
   'external-edit' {[IO.File]::WriteAllText($target,'external-third-content')}
   'audit-append' {[IO.File]::AppendAllText((Join-Path $f.root codex/opencodex-catalog-audit.jsonl),'external-append')}
   'symbolic-link' {Remove-Item -LiteralPath $target;$null=New-Item -ItemType SymbolicLink -Path $target -Target $first}
   'hardlink' {Remove-Item -LiteralPath $target;$null=New-Item -ItemType HardLink -Path $target -Target $first}
   'unreadable' {$acl=(Get-Acl $target).GetSecurityDescriptorSddlForm('Access');[Official299Proof.Native]::SetFixtureDacl($target,'D:P(A;;GA;;;SY)',$true)}
   'missing' {Remove-Item -LiteralPath $target}
   'tampered-plan' {[IO.File]::AppendAllText($f.plan,' ')}
   'tampered-original' {$p=Get-Content $f.plan -Raw|ConvertFrom-Json;$p.Before[0].Image.Bytes=[Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes('bad-original'));[IO.File]::WriteAllText($f.plan,($p|ConvertTo-Json -Depth 15))}
   'parent-link' {$parentMoved=Join-Path $f.root moved-grok;Move-Item -LiteralPath (Join-Path $f.root grok) -Destination $parentMoved;$null=New-Item -ItemType SymbolicLink -Path (Join-Path $f.root grok) -Target $parentMoved}
   'absent-slot-race' {$p=Get-Content $f.plan -Raw|ConvertFrom-Json;$p.After[4].State='absent';$p.After[4].Image=$null;Remove-Item -LiteralPath $target;[IO.File]::WriteAllText($f.plan,($p|ConvertTo-Json -Depth 15));$f.digest=[Official299Proof.FiveRecovery]::Hash($f.plan);$script:raceTarget=$target;$hook=[Action]{[IO.File]::WriteAllText($script:raceTarget,'external-recreated-file')}}
   'separate-writer' {$script:raceTarget=$first;$script:raceOutput=Join-Path $f.root race.json;$hook=[Action]{$i=[Diagnostics.ProcessStartInfo]::new();$i.FileName=Join-Path $PSHOME pwsh.exe;$i.UseShellExecute=$false;$i.CreateNoWindow=$true;foreach($a in @('-NoProfile','-File',(Join-Path $PSScriptRoot race-worker.ps1),'-Target',$script:raceTarget,'-Output',$script:raceOutput)){$i.ArgumentList.Add($a)};$p=[Diagnostics.Process]::Start($i);$p.SafeHandle|Out-Null;if(!$p.WaitForExit(3000)-or $p.ExitCode-ne 0){throw 'race helper failed'};$p.Dispose();$race=Get-Content $script:raceOutput -Raw|ConvertFrom-Json;if(!$race.refused -or $race.written){throw 'separate writer not excluded'}}}
  }
  try{[void][Official299Proof.FiveRecovery]::Apply($f.plan,$f.digest,$f.journal,$false,$(if($label -eq 'second-write-failure'){2}else{0}),0,$hook)}catch{$failed=@{type=$_.Exception.GetType().FullName;message=$_.Exception.Message}}
  finally{if($acl){[Official299Proof.Native]::SetFixtureDacl($target,$acl,$true)}}
  if($label -eq 'separate-writer'){if($failed){throw 'exclusive safe case failed'};if(![Official299Proof.FiveRecovery]::Equal($f.before,(Take $f.root restored),$false)){throw 'separate writer restore differs'}}else{if(!$failed -or (Get-FileHash $first).Hash -cne $firstHash){throw ('batch refusal or rollback changed first target: '+$label)}}
  if($label -eq 'second-write-failure' -and ![Official299Proof.FiveRecovery]::Equal($f.after,(Take $f.root rolledback),$true)){throw 'second-write rollback failed'}
  $checks.Add(@{case=$label;pass=$true;error=$failed;batchRefusedBeforeEarlierChange=($label -ne 'separate-writer');wholeTreeClaimed=$false})
 }
 foreach($third in @($false,$true)){
  $f=Setup ('crash-'+$third);$dead=Run-Child $f $false 1;if($dead.exitCode -ne 91){throw 'actual host interruption did not occur'}
  $first=Join-Path $f.root codex/config.toml;$firstHash=(Get-FileHash $first).Hash
  if($third){[IO.File]::WriteAllText((Join-Path $f.root grok/config.toml.bak-opencodex),'third-content-after-crash')}
  $recovery=Run-Child $f $true
  if($third){if($recovery.exitCode -eq 0 -or (Get-FileHash $first).Hash -cne $firstHash){throw 'crash recovery overwrote third state or earlier item'}}else{if($recovery.exitCode -ne 0 -or ![Official299Proof.FiveRecovery]::Equal($f.after,(Take $f.root recovered),$false)){throw 'fresh process failed durable-intent recovery'}}
  $checks.Add(@{case=('actual-crash-recovery-third-state-'+$third);pass=$true;crashed=$dead;freshRecovery=$recovery})
 }
 $pass=$true
}catch{$checks.Add(@{case='first-error';pass=$false;message=$_.Exception.Message;type=$_.Exception.GetType().FullName;line=$_.InvocationInfo.ScriptLineNumber})}
finally{$src=@{};foreach($f in @('Native.cs','Recovery.cs','FiveRecovery.cs','five-worker.ps1','five-test.ps1','race-worker.ps1')){$src[$f]=(Get-FileHash (Join-Path $PSScriptRoot $f)).Hash.ToLowerInvariant()};[IO.File]::WriteAllText((Join-Path $Directory five-prototype.json),(@{pass=$pass;checks=$checks.ToArray();cases=$cases.ToArray();sourceSha256=$src;utc=[datetime]::UtcNow.ToString('o');scope='native small synthetic fixtures only; no official service launched'}|ConvertTo-Json -Depth 30))}
if(!$pass){exit 1}

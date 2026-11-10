param([Parameter(Mandatory)][string]$Directory,[Parameter(Mandatory)][string]$Bun,[Parameter(Mandatory)][string]$Official)
$ErrorActionPreference='Stop';Set-StrictMode -Version Latest
Add-Type -Path (Join-Path $PSScriptRoot 'Native.cs'),(Join-Path $PSScriptRoot 'Recovery.cs')
if(Test-Path -LiteralPath $Directory){throw 'new prototype directory required'};$null=New-Item -ItemType Directory -Path $Directory
$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$checks=[Collections.Generic.List[object]]::new();$cases=[Collections.Generic.List[object]]::new();$pass=$false
function Fixture([string]$label){
 $root=Join-Path $Directory $label;$agents=Join-Path $root 'agents';$null=New-Item -ItemType Directory -Path $agents -Force
 [IO.File]::WriteAllText((Join-Path $agents 'ocx-old.md'),"<!-- generated-by: opencodex -->"+[char]10+'old-pruned')
 [IO.File]::WriteAllText((Join-Path $agents 'ocx-gpt-6-1-sol.md'),"<!-- generated-by: opencodex -->"+[char]10+'old-overwritten')
 [IO.File]::WriteAllText((Join-Path $agents 'ocx-user.md'),'user-owned-same-prefix')
 [IO.File]::WriteAllText((Join-Path $agents 'manual.md'),'manual-definition')
 $before=[Official299Proof.Recovery]::Capture($agents)
 $ev=@{};foreach($e in [Environment]::GetEnvironmentVariables().GetEnumerator()){$ev[$e.Key]=$e.Value}
 foreach($key in @($ev.Keys)){if($key -match '^(OPENCODEX|CODEX|CLAUDE|GROK|OCX_)'){$ev.Remove($key)}}
 $ev['OPENCODEX_HOME']=Join-Path $root 'runtime';$ev['CODEX_HOME']=Join-Path $root 'codex';$ev['CLAUDE_CONFIG_DIR']=$root
 $p=[Official299Proof.Owned]::new($Bun,('"'+$Bun+'" run "'+(Join-Path $PSScriptRoot 'agent-producer.ts')+'" "'+$Official+'" "'+$root+'" "'+(Join-Path $root 'producer.json')+'"'),$root,$ev,(Join-Path $root 'producer.stdout.log'),(Join-Path $root 'producer.stderr.log'),$sid)
 try{$p.Resume();$sw=[Diagnostics.Stopwatch]::StartNew();while((!$p.Exited() -or $p.Active() -gt 0) -and $sw.ElapsedMilliseconds -lt 5000){Start-Sleep -Milliseconds 10};if(!$p.Exited() -or $p.ExitCode() -ne 0 -or $p.Active() -ne 0){throw 'actual official small producer failed'};$handle=@{identity=$p.Identity;originalExit=$p.ExitCode();jobActive=$p.Active()}}finally{if($p.Active() -gt 0){$p.ForceOwnedCleanup()};$p.Dispose()}
 $after=[Official299Proof.Recovery]::Capture($agents);$plan=[Official299Proof.Recovery]::Plan($before,$after,[guid]::NewGuid().ToString('N'))
 if($plan.Changes.Count -ne 3){throw 'official create/overwrite/prune fixture incomplete'}
 $case=@{label=$label;root=$root;agents=$agents;before=$before;after=$after;plan=$plan;producer=$handle};$cases.Add($case);return $case
}
function Content($im){@($im.Files|Sort-Object Name|ForEach-Object {@{name=$_.Name;sha256=$_.Sha256;sddl=$_.Sddl;owner=$_.Owner}})|ConvertTo-Json -Depth 6 -Compress}
function Invoke-Race {
 $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=Join-Path $PSHOME 'pwsh.exe';$info.UseShellExecute=$false;$info.CreateNoWindow=$true
 foreach($arg in @('-NoLogo','-NoProfile','-NonInteractive','-File',(Join-Path $PSScriptRoot 'race-worker.ps1'),'-Target',$script:racePath,'-Output',$script:raceOutput)){$info.ArgumentList.Add($arg)}
 $p=[Diagnostics.Process]::Start($info);$p.SafeHandle|Out-Null;$fact=[Official299Proof.Native]::Observe($p.Handle)
 if(!$p.WaitForExit(3000) -or $p.ExitCode -ne 0){throw 'race helper did not finish normally'}
 $script:raceReceipt=@{identity=$fact;originalExitCode=$p.ExitCode;result=(Get-Content -LiteralPath $script:raceOutput -Raw|ConvertFrom-Json)};$p.Dispose()
}
try {
 $normal=Fixture 'normal';$ops=[Official299Proof.Recovery]::Restore($normal.plan,0,$null);$final=[Official299Proof.Recovery]::Capture($normal.agents)
 if((Content $normal.before) -cne (Content $final)){throw 'normal operation compensation differs'}
 $checks.Add(@{case='real-official-create-overwrite-prune-restored-with-acl';pass=$true;operations=$ops;userPrefixedFilePreserved=$true})
 foreach($label in @('external-edit','symbolic-link','hardlink','unreadable','rollback','concurrent-write','absent-slot-race','unexpected-user-file')){
  $f=Fixture $label;$target=Join-Path $f.agents 'ocx-new.md';$extra=Join-Path $f.agents 'unrelated-user.md';$unreadableAcl=$null
  switch($label){
   'external-edit' {[IO.File]::WriteAllText($target,'external-edited-postimage')}
   'symbolic-link' {Remove-Item -LiteralPath $target;$null=New-Item -ItemType SymbolicLink -Path $target -Target (Join-Path $f.agents 'manual.md')}
   'hardlink' {Remove-Item -LiteralPath $target;$null=New-Item -ItemType HardLink -Path $target -Target (Join-Path $f.agents 'manual.md')}
   'unreadable' {$unreadableAcl=(Get-Acl -LiteralPath $target).GetSecurityDescriptorSddlForm('Access');[Official299Proof.Native]::SetFixtureDacl($target,'D:P(A;;GA;;;SY)',$true)}
   'unexpected-user-file' {[IO.File]::WriteAllText($extra,'keep-other-user-content')}
  }
  $shared=Join-Path $f.agents 'ocx-gpt-6-1-sol.md';$postShared=[IO.File]::ReadAllBytes($shared);$errorSeen=$null
  try{
   if($label -eq 'rollback'){[void][Official299Proof.Recovery]::Restore($f.plan,2,$null);throw 'injected failure missing'}
   elseif($label -eq 'concurrent-write'){
    $script:racePath=$shared;$script:raceOutput=Join-Path $f.root race.json
    $hook=[Action]{Invoke-Race}
    [void][Official299Proof.Recovery]::Restore($f.plan,0,$hook);if(!$script:raceReceipt.result.refused -or $script:raceReceipt.result.written){throw 'native held handle did not refuse separate-process racing write'}
   }elseif($label -eq 'absent-slot-race'){
    $script:racePath=Join-Path $f.agents 'ocx-old.md';$script:raceOutput=Join-Path $f.root race.json;$hook=[Action]{Invoke-Race}
    [void][Official299Proof.Recovery]::Restore($f.plan,0,$hook);throw 'CREATE_NEW collision not refused'
   }else{[void][Official299Proof.Recovery]::Restore($f.plan,0,$null);if($label -ne 'unexpected-user-file'){throw 'unsafe target unexpectedly granted restoration'}}
  }catch{$errorSeen=@{type=$_.Exception.GetType().FullName;message=$_.Exception.Message}}
  finally{if($unreadableAcl){[Official299Proof.Native]::SetFixtureDacl($target,$unreadableAcl,$true)}}
  if($label -in @('concurrent-write','unexpected-user-file')){
   if($errorSeen){throw ('safe path failed: '+$label+' '+$errorSeen.message)}
   if($label -eq 'unexpected-user-file' -and [IO.File]::ReadAllText($extra) -cne 'keep-other-user-content'){throw 'unrelated user file erased'}
  }else{
   if(!$errorSeen -or [Convert]::ToBase64String([IO.File]::ReadAllBytes($shared)) -cne [Convert]::ToBase64String($postShared)){throw 'refusal or transaction rollback changed earlier file'}
   if($label -eq 'rollback' -and (Content ([Official299Proof.Recovery]::Capture($f.agents))) -cne (Content $f.after)){throw 'transaction rollback did not restore sealed postimage'}
   if($label -eq 'absent-slot-race' -and [IO.File]::ReadAllText($script:racePath) -cne 'external-recreated-file'){throw 'CREATE_NEW collision overwrote user content'}
  }
  $checks.Add(@{case=$label;pass=$true;error=$errorSeen;fullOriginalTreeClaimed=($label -eq 'concurrent-write');scope='native small-file operation only';separateProcessRace=$(if($label -in @('concurrent-write','absent-slot-race')){$script:raceReceipt}else{$null})})
 }
 $pass=$true
}catch{$checks.Add(@{case='first-error';pass=$false;type=$_.Exception.GetType().FullName;message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber})}
finally{
 $src=@{};foreach($leaf in @('Native.cs','Recovery.cs','agent-producer.ts','recovery-test.ps1','race-worker.ps1')){$src[$leaf]=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $leaf)).Hash.ToLowerInvariant()}
 [IO.File]::WriteAllText((Join-Path $Directory 'recovery-prototype.json'),(@{pass=$pass;checks=$checks.ToArray();cases=$cases.ToArray();sourceSha256=$src;utc=[datetime]::UtcNow.ToString('o')}|ConvertTo-Json -Depth 30))
}
if(!$pass){exit 1}

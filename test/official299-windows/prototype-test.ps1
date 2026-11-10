param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
Add-Type -Path (Join-Path $PSScriptRoot 'Native.cs')
. (Join-Path $PSScriptRoot 'policy.ps1')
if(Test-Path -LiteralPath $OutputDirectory){throw 'preserve old prototype evidence; new directory required'}
$null=New-Item -ItemType Directory -Path $OutputDirectory
$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$envs=@{};foreach($e in [Environment]::GetEnvironmentVariables().GetEnumerator()){$envs[$e.Key]=$e.Value}
$checks=[Collections.Generic.List[object]]::new();$p=$null;$g=$null;$passed=$false
try {
 $exe=Join-Path $PSHOME 'pwsh.exe'
 $p=[Official299Proof.Owned]::new($exe,('"'+$exe+'" -NoLogo -NoProfile -NonInteractive -Command "exit 0"'),$OutputDirectory,$envs,(Join-Path $OutputDirectory 'zero.stdout.log'),(Join-Path $OutputDirectory 'zero.stderr.log'),$sid)
 if($p.Resumed -or $p.Exited() -or $p.Active() -ne 1 -or @($p.Members()).Count -ne 1){throw 'suspended identity/Job pre-resume failed'}
 $checks.Add(@{case='suspended-assigned-before-execution';pass=$true;identity=$p.Identity;members=@($p.Members());resumed=$p.Resumed})
 $p.Resume();$sw=[Diagnostics.Stopwatch]::StartNew();while((!$p.Exited() -or $p.Active() -ne 0) -and $sw.ElapsedMilliseconds -lt 5000){Start-Sleep -Milliseconds 20}
 if(!$p.Exited() -or $p.ExitCode() -ne 0 -or $p.Active() -ne 0 -or @($p.Members()).Count -ne 0){throw 'original handle and empty Job disagree'}
 $checks.Add(@{case='normal-zero-exit';pass=$true;exitCode=$p.ExitCode();jobActive=$p.Active();members=@($p.Members())});$p.Dispose();$p=$null
 # Root exits while a directly spawned, automatically inherited child stays alive.
 $childScript=Join-Path $OutputDirectory 'child-root.ps1'
 [IO.File]::WriteAllText($childScript,('$null=Start-Process -FilePath (Join-Path $PSHOME ''pwsh.exe'') -WindowStyle Hidden -ArgumentList @(''-NoProfile'',''-NonInteractive'',''-Command'',''Start-Sleep -Seconds 30''); exit 0'))
 $p=[Official299Proof.Owned]::new($exe,('"'+$exe+'" -NoLogo -NoProfile -NonInteractive -File "'+$childScript+'"'),$OutputDirectory,$envs,(Join-Path $OutputDirectory 'child.stdout.log'),(Join-Path $OutputDirectory 'child.stderr.log'),$sid)
 $root=$p.Identity;$p.Resume();$sw.Restart();while(!$p.Exited() -and $sw.ElapsedMilliseconds -lt 5000){Start-Sleep -Milliseconds 20}
 if(!$p.Exited() -or $p.ExitCode() -ne 0 -or $p.Active() -eq 0 -or @($p.Members()).Count -eq 0){throw 'leftover controlled child counterexample not established'}
 $checks.Add(@{case='root-exit-with-child-does-not-mean-clean';pass=$true;root=$root;rootExitCode=$p.ExitCode();jobActive=$p.Active();members=@($p.Members());cleanupDecision='refused-until-empty'})
 $p.ForceOwnedCleanup();$sw.Restart();while($p.Active() -gt 0 -and $sw.ElapsedMilliseconds -lt 5000){Start-Sleep -Milliseconds 20}
 if($p.Active() -ne 0 -or @($p.Members()).Count -ne 0){throw 'owned counterexample cleanup failed'}
 $checks.Add(@{case='separate-forced-counterexample-cleanup';pass=$true;jobActive=$p.Active();members=@($p.Members())});$p.Dispose();$p=$null
 $file=Join-Path $OutputDirectory 'one-link.txt';[IO.File]::WriteAllText($file,'synthetic')
 $g=[Official299Proof.Guard]::new($file,$false)
 try{Move-Item -LiteralPath $file -Destination (Join-Path $OutputDirectory 'moved.txt');throw 'guard did not prevent rename'}catch [IO.IOException]{$checks.Add(@{case='retained-path-rename-denied';pass=$true;fileIdentity=$g.Identity})};$g.Dispose();$g=$null
 $link=Join-Path $OutputDirectory 'hardlink.txt';$null=New-Item -ItemType HardLink -Path $link -Target $file
 try{$g=[Official299Proof.Guard]::new($link,$false);throw 'hardlink accepted'}catch [Management.Automation.MethodInvocationException]{if($_.Exception.InnerException.Message -notlike '*linked or wrong-kind*'){throw};$checks.Add(@{case='hardlink-refused';pass=$true})}
 $expected=@{schema='official299-host-binding/1';pid=123;filetime='456';sid=$sid;home='C:\synthetic\runtime';homeIdentity='1:2:3'}
 $observed=@{pid=123;filetime='456';sid=$sid;elevated=$false}
 if((Get-ProofDecision $expected @{readable=$true;value=$expected.Clone()} $observed '1:2:3') -cne 'admitted'){throw 'matching original generation refused'}
 foreach($key in @('schema','pid','filetime','sid','home','homeIdentity')){$bad=$expected.Clone();$bad[$key]='wrong';if((Get-ProofDecision $expected @{readable=$true;value=$bad} $observed '1:2:3') -cne 'refused'){throw ('wrong '+$key+' admitted')};$checks.Add(@{case=('binding-wrong-'+$key);pass=$true;decision='refused';destructiveRequests=0})}
 $unknown=Read-ProofBinding (Join-Path $OutputDirectory 'missing-proof.json')
 if((Get-ProofDecision $expected $unknown $observed '1:2:3') -cne 'unknown'){throw 'missing proof granted authority'}
 $checks.Add(@{case='missing-binding';pass=$true;readback=$unknown;decision='unknown';destructiveRequests=0})
 $unreadable=Join-Path $OutputDirectory 'unreadable-proof.json';[IO.File]::WriteAllText($unreadable,'{}')
 $acl=Get-Acl -LiteralPath $unreadable;$savedAcl=$acl.GetSecurityDescriptorSddlForm('Access');$acl.SetAccessRuleProtection($true,$false)
 foreach($ace in @($acl.Access)){$null=$acl.RemoveAccessRuleSpecific($ace)}
 $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new('S-1-5-18'),'FullControl','Allow'));Set-Acl -LiteralPath $unreadable -AclObject $acl
 try {$read=Read-ProofBinding $unreadable;if($read.readable -or (Get-ProofDecision $expected $read $observed '1:2:3') -cne 'unknown'){throw 'unreadable proof granted authority'};$checks.Add(@{case='actual-unreadable-binding';pass=$true;readback=$read;decision='unknown';destructiveRequests=0})}
 finally {[Official299Proof.Native]::RestoreSyntheticDacl($unreadable,$savedAcl);if((Get-Acl -LiteralPath $unreadable).GetSecurityDescriptorSddlForm('Access') -cne $savedAcl){throw 'synthetic DACL restoration readback differs'}}
 $passed=$true
} catch {$checks.Add(@{case='first-error';pass=$false;type=$_.Exception.GetType().FullName;message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber})}
finally {
 if($p){$p.ForceOwnedCleanup();$checks.Add(@{case='failure-owned-cleanup';members=@($p.Members());active=$p.Active()});$p.Dispose()};if($g){$g.Dispose()}
 [IO.File]::WriteAllText((Join-Path $OutputDirectory 'prototype-result.json'),(@{pass=$passed;checks=$checks.ToArray();scope='test host native primitive counterexamples; no official runtime launched';utc=[datetime]::UtcNow.ToString('o')}|ConvertTo-Json -Depth 20))
}
if(!$passed){exit 1}

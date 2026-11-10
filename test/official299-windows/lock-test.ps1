param([Parameter(Mandatory)][string]$Directory,[Parameter(Mandatory)][string]$Bun,[Parameter(Mandatory)][string]$Official)
$ErrorActionPreference='Stop';Set-StrictMode -Version Latest
Add-Type -Path (Join-Path $PSScriptRoot 'Native.cs')
if(Test-Path -LiteralPath $Directory){throw 'fresh lock prototype directory required'};$null=New-Item -ItemType Directory -Path $Directory
$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value;$checks=@();$pass=$false;$holder=$null;$probe=$null
$ev=@{};foreach($e in [Environment]::GetEnvironmentVariables().GetEnumerator()){$ev[$e.Key]=$e.Value}
foreach($key in @($ev.Keys)){if($key -match '^(OPENCODEX|CODEX|CLAUDE|GROK|OCX_)'){$ev.Remove($key)}}
foreach($leaf in @('runtime','codex','claude','temp')){$path=Join-Path $Directory $leaf;$null=New-Item -ItemType Directory -Path $path}
$ev['OPENCODEX_HOME']=Join-Path $Directory runtime;$ev['CODEX_HOME']=Join-Path $Directory codex;$ev['CLAUDE_CONFIG_DIR']=Join-Path $Directory claude;$ev['TEMP']=Join-Path $Directory temp;$ev['TMP']=$ev['TEMP']
function Worker([string]$kind,[string]$mode,[string]$label){$output=Join-Path $Directory ($label+'.json');$release=Join-Path $Directory ($label+'.release');$p=[Official299Proof.Owned]::new($Bun,('"'+$Bun+'" run "'+(Join-Path $PSScriptRoot 'lock-worker.ts')+'" "'+$Official+'" '+$kind+' '+$mode+' "'+$output+'" "'+$release+'"'),$Directory,$ev,(Join-Path $Directory ($label+'.stdout.log')),(Join-Path $Directory ($label+'.stderr.log')),$sid);$p.Resume();return @{process=$p;output=$output;release=$release}}
function Wait-Worker($w){$sw=[Diagnostics.Stopwatch]::StartNew();while((!$w.process.Exited() -or $w.process.Active() -gt 0) -and $sw.ElapsedMilliseconds -lt 6000){Start-Sleep -Milliseconds 10};if(!$w.process.Exited() -or $w.process.ExitCode() -ne 0 -or $w.process.Active() -ne 0){throw 'lock helper did not exit normally'};return @{identity=$w.process.Identity;exitCode=$w.process.ExitCode();jobActive=$w.process.Active();result=(Get-Content -LiteralPath $w.output -Raw|ConvertFrom-Json)}}
try{
 foreach($kind in @('claim','owner')){
  $holder=Worker $kind hold ($kind+'-holder');$sw=[Diagnostics.Stopwatch]::StartNew();while(!(Test-Path -LiteralPath $holder.output) -and $sw.ElapsedMilliseconds -lt 3000){Start-Sleep -Milliseconds 10};$first=Get-Content -LiteralPath $holder.output -Raw|ConvertFrom-Json;if(!$first.acquired){throw 'live official holder not established'}
  $path=Join-Path $ev['CODEX_HOME'] ('.opencodex-native-main.'+$kind+'.sqlite');$g=[Official299Proof.Guard]::new($path,$false);try{$idBefore=$g.Identity}finally{$g.Dispose()};$aclBefore=(Get-Acl -LiteralPath $path).Sddl
  $probe=Worker $kind probe ($kind+'-blocked');$blocked=Wait-Worker $probe;$probe.process.Dispose();$probe=$null
  if($blocked.result.acquired -or $blocked.result.code -cne ('NATIVE_MAIN_'+$kind.ToUpper()+'_BUSY')){throw 'official live-holder competitor did not refuse'}
  [IO.File]::WriteAllText($holder.release,'release-owned-holder');$heldExit=Wait-Worker $holder;$holder.process.Dispose();$holder=$null
  $probe=Worker $kind probe ($kind+'-after-release');$after=Wait-Worker $probe;$probe.process.Dispose();$probe=$null
  $g=[Official299Proof.Guard]::new($path,$false);try{$idAfter=$g.Identity}finally{$g.Dispose()};$aclAfter=(Get-Acl -LiteralPath $path).Sddl
  if(!$after.result.acquired -or !$after.result.released -or $idAfter -cne $idBefore -or $aclBefore -cne $aclAfter){throw 'official release/reacquisition changed identity/ACL or failed'}
  $checks+=@{kind=$kind;pass=$true;fileIdentityBefore=$idBefore;fileIdentityAfter=$idAfter;sddlBefore=$aclBefore;sddlAfter=$aclAfter;blocked=$blocked;holderExit=$heldExit;reacquired=$after;fileRetained=$true}
 }
 $pass=$true
}catch{$checks+=@{case='first-error';pass=$false;message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber;type=$_.Exception.GetType().FullName}}
finally{foreach($w in @($holder,$probe)){if($w){if($w.process.Active() -gt 0){$w.process.ForceOwnedCleanup()};$w.process.Dispose()}};[IO.File]::WriteAllText((Join-Path $Directory lock-prototype.json),(@{pass=$pass;checks=$checks;utc=[datetime]::UtcNow.ToString('o');nativeSourceSha256=(Get-FileHash (Join-Path $PSScriptRoot Native.cs)).Hash.ToLowerInvariant();workerSourceSha256=(Get-FileHash (Join-Path $PSScriptRoot lock-worker.ts)).Hash.ToLowerInvariant()}|ConvertTo-Json -Depth 30))}
if(!$pass){exit 1}

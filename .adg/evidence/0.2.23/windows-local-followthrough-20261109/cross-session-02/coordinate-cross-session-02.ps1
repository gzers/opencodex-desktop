$ErrorActionPreference='Stop'
. 'C:\Users\15119\.codex\worktrees\ocxd-023-windows-r1\code\test\smoke\windows023-two-user-native-helper.ps1'
$b=Join-Path $PSScriptRoot 'cross-session-02';$seq=@{console=0;rdp=0};$trace=@()
$summary=[ordered]@{accepted=$false;hypothesis='H-W7';scope='Only actual console/RDP overlap, fixed peer-port conflict and normal Stop preservation';startedUtc=[datetime]::UtcNow.ToString('o');passed=$false;harnessCommit='4d783585645206c52b49bce8656475d26940c1d7';sandbox=$true;defaultProfileOrCredentialAcceptance=$false}
function Read-Ready([string]$role){
 $path=Join-Path $b ($role+'/ready.json');$until=[datetime]::UtcNow.AddSeconds(400)
 while(!(Test-Path -LiteralPath $path)){
  if((Test-Path -LiteralPath (Join-Path $b ($role+'/finished.json'))) -or (Test-Path -LiteralPath (Join-Path $b ($role+'/failure.json')))){throw ('Actor failed before ready: '+$role)}
  if([datetime]::UtcNow -gt $until){throw 'Fixture readiness envelope; service Start not attempted'}
  Start-Sleep -Milliseconds 300
 }
 Get-Content -LiteralPath $path -Raw|ConvertFrom-Json
}
function Command([string]$role,[string]$mode,[int]$port=0,[int]$servicePid=0){
 $seq[$role]++;$n=$seq[$role];$d=Join-Path $b $role
 Write-TwoUserJson (Join-Path $d ('command-'+$n+'.json')) @{mode=$mode;port=$port;expectedPid=$servicePid}
 $reply=Join-Path $d ('reply-'+$n+'.json');$until=[datetime]::UtcNow.AddSeconds(125)
 while(!(Test-Path -LiteralPath $reply)){
  if(Test-Path -LiteralPath (Join-Path $d 'finished.json')){throw ('Actor exited during '+$role+' '+$mode)}
  if([datetime]::UtcNow -gt $until){throw ('Actor transport envelope: '+$role+' '+$mode)}
  Start-Sleep -Milliseconds 150
 }
 $v=Get-Content -LiteralPath $reply -Raw|ConvertFrom-Json
 $script:trace+=@{atUtc=[datetime]::UtcNow.ToString('o');role=$role;mode=$mode;reply=$reply;record=$v.value.receipt.record;result=$v.value.receipt.result;driverExitCode=$v.value.exitCode;sessions=@(quser)}
 Write-TwoUserJson (Join-Path $b 'controller-trace.json') $script:trace

 if($role -eq 'rdp' -and $mode -eq 'stop'){
  $client=Join-Path $d 'private/sandbox/home/.codex/config.toml'
  $text=if(Test-Path -LiteralPath $client){[IO.File]::ReadAllText($client)}else{''}
  $fields=@{};foreach($name in @('openai_base_url','experimental_realtime_ws_base_url','model_catalog_json')){$fields[$name]=[regex]::IsMatch($text,('(?m)^[ \t]*'+$name+'\s*='))}
  Write-TwoUserJson (Join-Path $d 'native-client-stop-readback.json') @{observedUtc=[datetime]::UtcNow.ToString('o');exists=(Test-Path -LiteralPath $client);sha256=$(if(Test-Path -LiteralPath $client){(Get-FileHash -LiteralPath $client).Hash.ToLowerInvariant()}else{$null});routeFieldsPresent=$fields;opencodexProviderPresent=[regex]::IsMatch($text,'(?m)^[ \t]*model_provider\s*=\s*["'']opencodex["'']');formerPortTextPresent=$text.Contains('127.0.0.1:'+([string]$port));rawContentsOmitted=$true;ownFreshSandboxOnly=$true}
 }
 if($v.value.exitCode -ne 0 -or $v.value.receipt.result -cne 'pass-runtime-step'){throw ('Original runtime gate failed: '+$role+' '+$mode)}
 if($v.managerBefore.pid -ne $v.managerAfter.pid -or $v.managerBefore.handle -cne $v.managerAfter.handle -or !$v.managerAfter.ownedVisible){throw 'Original actor window changed'}
 if($mode -eq 'stop' -and !$v.restore.passed){throw 'Original service exit0 and actual restore not proved'}
 $v
}
try{
 $a=Read-Ready console;$z=Read-Ready rdp
 if($a.sid -ceq $z.sid -or $a.sessionId -ne 1 -or $z.sessionId -ne 2 -or !$a.applicationToken.elevated -or $z.applicationToken.elevated){throw 'Actual SID/session/elevation requirements failed'}
 $summary.identities=@{console=$a;rdp=$z}
 $as=Command console start;$ar=$as.value.receipt.record
 if(!$ar){throw 'No actual console record'}
 $conflict=Command rdp conflict $ar.port $ar.pid
 $listeners=@(Get-NetTCPConnection -LocalPort $ar.port -State Listen -ErrorAction SilentlyContinue)
 if(!$conflict.value.receipt.conflict.rejected -or !$listeners.Count -or @($listeners|Where-Object {$_.OwningProcess -ne $ar.pid}).Count){throw 'Peer fixed-port rejection changed original console listener'}
 $summary.fixedConflict=$true
 $survivor=Command console observe $ar.port $ar.pid
 $bs=Command rdp start;$br=$bs.value.receipt.record
 if(!$br -or $br.port -eq $ar.port){throw 'Independent actual port not proved'}
 $ao=Command console observe $ar.port $ar.pid
 $bo=Command rdp observe $br.port $br.pid
 $summary.overlap=$true
 $bstop=Command rdp stop $br.port $br.pid
 $aafter=Command console observe $ar.port $ar.pid
 $summary.peerSurvivedNormalStop=$true
 $astop=Command console stop $ar.port $ar.pid
 $summary.console=@{start=$as.value.receipt.startBudget;record=$ar;stop=$astop.value.receipt.stopBudget;restore=$astop.restore;totalStartActions=1;successfulStarts=1;normalStops=1}
 $summary.rdp=@{start=$bs.value.receipt.startBudget;record=$br;stop=$bstop.value.receipt.stopBudget;restore=$bstop.restore;totalStartActions=2;expectedConflicts=1;successfulStarts=1;normalStops=1}
 $summary.passed=$true
}catch{$summary.firstFailure=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber;atUtc=[datetime]::UtcNow.ToString('o')}}
finally{
 foreach($role in @('rdp','console')){
  $d=Join-Path $b $role
  if((Test-Path -LiteralPath $d) -and !(Test-Path -LiteralPath (Join-Path $d 'finished.json'))){$seq[$role]++;Write-TwoUserJson (Join-Path $d ('command-'+$seq[$role]+'.json')) @{mode='finish';reason='Controller done; exact owned fixture cleanup only'}}
 }
 $summary.finishedUtc=[datetime]::UtcNow.ToString('o')
 Write-TwoUserJson (Join-Path $b 'controller-finished.json') $summary
}

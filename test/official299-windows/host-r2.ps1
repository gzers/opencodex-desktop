param([Parameter(Mandatory)][string]$InputFile)
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
$c=Get-Content -LiteralPath $InputFile -Raw | ConvertFrom-Json
Add-Type -Path (Join-Path $PSScriptRoot 'Native.cs'),(Join-Path $PSScriptRoot 'Recovery.cs')
. (Join-Path $PSScriptRoot 'policy.ps1')
$identity=[Security.Principal.WindowsIdentity]::GetCurrent();$sid=$identity.User.Value
$out=$c.output
function Write-Proof($name,$value){[IO.File]::WriteAllText((Join-Path $out $name),($value|ConvertTo-Json -Depth 30))}
function Require($condition,[string]$message){if(!$condition){throw $message}}
function Digest([string]$path){(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()}
function Private-Directory([string]$path){
 Require (!(Test-Path -LiteralPath $path)) 'fresh private directory required'
 $null=New-Item -ItemType Directory -Path $path
 $acl=[Security.AccessControl.DirectorySecurity]::new();$owner=[Security.Principal.SecurityIdentifier]::new($sid)
 $acl.SetOwner($owner);$acl.SetAccessRuleProtection($true,$false)
 foreach($s in @($sid,'S-1-5-18')){$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($s),'FullControl','ContainerInherit,ObjectInherit','None','Allow'))}
 Set-Acl -LiteralPath $path -AclObject $acl
}
function Check-Private([string]$path){
 $acl=Get-Acl -LiteralPath $path
 Require ($acl.GetOwner([Security.Principal.SecurityIdentifier]).Value -ceq $sid) 'TokenUser owner mismatch'
 foreach($ace in $acl.Access){Require ($ace.AccessControlType -eq 'Allow' -and $ace.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value -cin @($sid,'S-1-5-18')) 'unexpected private DACL ACE'}
 Require (@($acl.Access | Where-Object {$_.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value -ceq $sid}).Count -gt 0) 'missing owner access'
 return $acl.Sddl
}
$guards=[Collections.Generic.List[IDisposable]]::new();$runtime=$null;$http=$null;$bindingLock=$null
$r=[ordered]@{schema='official299-windows-restoration/2';accepted=$false;restorationChainPassed=$false;stableLockExceptionAdopted=$false;stage='ordinary-identity';runtimeCreationEntered=$false;start=$null;status=$null;stop=$null;firstError=$null;counterexamples=@();cleanup=@();budgetsMs=@{start=30000;status=5000;stop=10000};utc=[datetime]::UtcNow.ToString('o')}
$trace=[Collections.Generic.List[object]]::new()
$ownedListening=[Collections.Generic.Dictionary[string,object]]::new();$lockWorkers=[Collections.Generic.List[object]]::new();$backupLock=$null
function Observe-OwnedListeners {
 $members=@($runtime.TrackMembers());$ids=@($members|ForEach-Object {$_.pid});$rows=@([Official299Proof.Native]::TcpListeners()|Where-Object {$_.pid -in $ids})
 foreach($row in $rows){$fact=@($members|Where-Object {$_.pid -eq $row.pid})[0];$key=$row.family+'/'+$row.address+'/'+$row.port+'/'+$row.pid+'/'+$fact.filetime;$ownedListening[$key]=@{family=$row.family;address=$row.address;port=$row.port;pid=$row.pid;filetime=$fact.filetime;sid=$fact.sid;sampledUtc=[datetime]::UtcNow.ToString('o');jobIdentity='same-retained-unnamed-job'}}
 return @{members=$members;listeners=$rows}
}
function Agent-Same($a,$b){if($a.RootId -cne $b.RootId){return $false};return (($a.Files|Sort-Object Name|Select-Object Name,Id,Sha256,Sddl,Owner|ConvertTo-Json -Compress) -ceq ($b.Files|Sort-Object Name|Select-Object Name,Id,Sha256,Sddl,Owner|ConvertTo-Json -Compress))}
try {
 Require ($sid -ceq $c.sid) 'new account SID differs'
 Require (!([Security.Principal.WindowsPrincipal]::new($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) 'ordinary account required'
 Require ($env:USERPROFILE -ieq $c.profile) 'actual profile differs'
 Require ((Split-Path -Leaf $c.profile) -cmatch '^ocx299[a-f0-9]{10}$') 'disposable profile required'
 $self=[Official299Proof.Native]::Observe([Diagnostics.Process]::GetCurrentProcess().Handle)
 Require (!$self.elevated -and $self.sid -ceq $sid) 'actual ordinary host token differs'
 $r.host=$self;$r.hostSourceSha256=Digest $PSCommandPath;$r.nativeSourceSha256=Digest (Join-Path $PSScriptRoot 'Native.cs')
 $fixture=Join-Path $c.profile 'official299-proof';Private-Directory $fixture
 foreach($leaf in @('home','runtime','codex','claude','grok','temp','program')){Private-Directory (Join-Path $fixture $leaf)}
 $isolatedHome=Join-Path $fixture 'home';$runtimeHome=Join-Path $fixture 'runtime';$codex=Join-Path $fixture 'codex';$claude=Join-Path $fixture 'claude';$grok=Join-Path $fixture 'grok'
 $r.fixture=$fixture
 $p=$fixture
 while($p){$guards.Add([Official299Proof.Guard]::new($p,$true));$next=Split-Path -Parent $p;if($next -eq $p){break};$p=$next}
 foreach($path in @($isolatedHome,$runtimeHome,$codex,$claude,$grok)){$guards.Add([Official299Proof.Guard]::new($path,$true));$null=Check-Private $path}
 $r.privateRoot=@{path=$fixture;sddl=(Check-Private $fixture);protected=(Get-Acl $fixture).AreAccessRulesProtected}
 Require $r.privateRoot.protected 'private root still inherits'
 $r.stage='private-package-copy'
 $program=Join-Path $fixture 'program'
 Copy-Item -LiteralPath (Join-Path $c.inputRoot 'node_modules') -Destination $program -Recurse
 $package=Join-Path $program 'node_modules/@bitkyc08/opencodex';$bun=Join-Path $program 'node_modules/@oven/bun-windows-x64/bin/bun.exe'
 Require ((Digest $bun) -ceq $c.bunSha256) 'platform Bun differs'
 $mismatches=@();$srcCount=0
 foreach($f in Get-ChildItem -LiteralPath (Join-Path $c.archiveRoot 'package') -File -Recurse){$rel=[IO.Path]::GetRelativePath((Join-Path $c.archiveRoot 'package'),$f.FullName);$dest=Join-Path $package $rel;if(!(Test-Path -LiteralPath $dest) -or (Digest $f.FullName) -cne (Digest $dest)){$mismatches+=$rel};$srcCount++}
 Require ($mismatches.Count -eq 0) 'official package differs from fixed archive'
 [Official299Proof.Native]::SetFixtureDacl($program,('D:P(A;OICI;GRGX;;;'+$sid+')(A;OICI;GA;;;SY)'),$true)
 $r.programReadOnlySddl=(Get-Acl -LiteralPath $program).Sddl
 $guards.Add([Official299Proof.Guard]::new($program,$true));$guards.Add([Official299Proof.Guard]::new($package,$true));$guards.Add([Official299Proof.Guard]::new($bun,$false))
 $r.package=@{version='2.99.0';archiveSha256=$c.archiveSha256;filesCompared=$srcCount;mismatches=$mismatches;bunSha256=$c.bunSha256;bunVersion=$c.bunVersion;lockSha256=$c.lockSha256;privateProgram=$program}
 $nl=[string][char]10
 $originals=@{
  'codex/config.toml'=('# synthetic original'+$nl+'model = "gpt-5.6"'+$nl+'model_provider = "openai"'+$nl+'model_catalog_json = '+(ConvertTo-Json (Join-Path $codex 'opencodex-catalog.json') -Compress)+$nl);
  'codex/auth.json'='{ "OPENAI_API_KEY": "synthetic-not-a-real-credential" }';
  'claude/settings.json'='{ "env": { "SYNTHETIC_SENTINEL": "original" } }';
  'grok/config.toml'=('# synthetic fixture'+$nl+'model = "native"'+$nl+$nl+'[model.native]'+$nl+'model = "grok-fixture"'+$nl+'name = "Synthetic native"'+$nl+'api_backend = "xai"'+$nl)
 }
 $before=@();foreach($rel in $originals.Keys){$path=Join-Path $fixture $rel;[IO.File]::WriteAllText($path,$originals[$rel]);$before+=@{relative=$rel;sha256=(Digest $path);bytesBase64=[Convert]::ToBase64String([IO.File]::ReadAllBytes($path))}}
 Write-Proof 'client-originals.json' $before
 $catalogPath=Join-Path $codex 'opencodex-catalog.json';$catalogSource=Join-Path $package 'src/codex/data/upstream-models.json';[IO.File]::WriteAllBytes($catalogPath,[IO.File]::ReadAllBytes($catalogSource));$catalogOriginal=Digest $catalogPath
 $r.catalogSource=@{relative='src/codex/data/upstream-models.json';sha256=(Digest $catalogSource);originalSha256=$catalogOriginal;originalBytes=[Convert]::ToBase64String([IO.File]::ReadAllBytes($catalogSource))}
 $agents=Join-Path $claude 'agents';Private-Directory $agents
 [IO.File]::WriteAllText((Join-Path $agents 'ocx-old.md'),('<!-- generated-by: opencodex -->'+$nl+'synthetic old generated, pruned by current roster'+$nl))
 [IO.File]::WriteAllText((Join-Path $agents 'ocx-gpt-6-1-sol.md'),('<!-- generated-by: opencodex -->'+$nl+'synthetic prior generated body, overwritten by current roster'+$nl))
 [IO.File]::WriteAllText((Join-Path $agents 'ocx-user.md'),'synthetic user-owned same prefix, no generated marker')
 [IO.File]::WriteAllText((Join-Path $agents 'manual.md'),'synthetic manual definition')
 $agentBefore=[Official299Proof.Recovery]::Capture($agents)
 $transactionId=[guid]::NewGuid().ToString('N');$backup=Join-Path $fixture 'protected-agent-preimage.json';[IO.File]::WriteAllText($backup,($agentBefore|ConvertTo-Json -Depth 12));$backupLock=[IO.File]::Open($backup,'Open','Read','Read');$guards.Add([Official299Proof.Guard]::new($backup,$false));$r.agentBackup=@{transactionId=$transactionId;sha256=(Digest $backup);sddl=(Check-Private $backup);retainedReadHandleNoWriteOrDelete=$true;originalAbsentNames=@('ocx-new.md');before=$agentBefore}
 function Client-Inventory([switch]$AllowUnreadable) {
  $entries=@();foreach($client in @('codex','claude','grok')){foreach($file in Get-ChildItem -LiteralPath (Join-Path $fixture $client) -File -Recurse){$guard=$null;$rel=[IO.Path]::GetRelativePath($fixture,$file.FullName).Replace('\','/');try{$guard=[Official299Proof.Guard]::new($file.FullName,$false);$entries+=@{relative=$rel;length=$file.Length;sha256=(Digest $file.FullName)}}catch{if(!$AllowUnreadable){throw};$entries+=@{relative=$rel;length=$file.Length;sha256=$null;readState='unknown';failureType=$_.Exception.GetType().FullName;failureHResult=$_.Exception.HResult}}finally{if($guard){$guard.Dispose()}}}};return @($entries|Sort-Object relative)
 }
 $clientBaseline=@(Client-Inventory);Write-Proof 'client-tree-before.json' $clientBaseline
 $listener=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0);$listener.Start();$port=$listener.LocalEndpoint.Port;$listener.Stop();$r.port=$port
 $config=@{hostname='127.0.0.1';port=$port;defaultProvider='fixture';providers=@{fixture=@{adapter='openai-responses';baseUrl='https://fixture.invalid/v1';apiKey='synthetic-never-sent';liveModels=$false;models=@('gpt-6.1-sol');selectedModels=@('gpt-6.1-sol')}};subagentModels=@('fixture/gpt-6.1-sol','fixture/new');syncResumeHistory=$false;shutdownTimeoutMs=5000}
 [IO.File]::WriteAllText((Join-Path $runtimeHome 'config.json'),($config|ConvertTo-Json -Depth 8))
 $admin=[Convert]::ToHexString([Security.Cryptography.RandomNumberGenerator]::GetBytes(32)).ToLowerInvariant()
 $ev=@{SystemRoot=$env:SystemRoot;WINDIR=$env:WINDIR;COMSPEC=(Join-Path $env:SystemRoot 'System32/cmd.exe');PATH=((Split-Path -Parent $bun)+';'+$env:SystemRoot+'\System32;'+$PSHOME);PATHEXT='.COM;.EXE;.BAT;.CMD';USERPROFILE=$c.profile;HOME=$isolatedHome;OPENCODEX_HOME=$runtimeHome;CODEX_HOME=$codex;CLAUDE_CONFIG_DIR=$claude;GROK_HOME=$grok;APPDATA=(Join-Path $c.profile 'AppData/Roaming');LOCALAPPDATA=(Join-Path $c.profile 'AppData/Local');TEMP=(Join-Path $fixture 'temp');TMP=(Join-Path $fixture 'temp');USERNAME=$env:USERNAME;USERDOMAIN=$env:USERDOMAIN;OPENCODEX_ADMIN_AUTH_TOKEN=$admin;NO_COLOR='1';CI='1'}
 $handler=[Net.Http.HttpClientHandler]::new();$handler.UseProxy=$false;$http=[Net.Http.HttpClient]::new($handler);$http.Timeout=[Threading.Timeout]::InfiniteTimeSpan
 function Request([string]$path,[string]$method,[long]$remaining,[hashtable]$headers){
  Require ($remaining -gt 0) 'action budget exhausted before HTTP'
  $cancel=[Threading.CancellationTokenSource]::new([int]$remaining);$req=[Net.Http.HttpRequestMessage]::new([Net.Http.HttpMethod]::new($method),('http://127.0.0.1:'+$port+$path))
  foreach($key in $headers.Keys){$null=$req.Headers.TryAddWithoutValidation($key,[string]$headers[$key])}
  try{$res=$http.SendAsync($req,$cancel.Token).GetAwaiter().GetResult();try{$body=$res.Content.ReadAsStringAsync($cancel.Token).GetAwaiter().GetResult();@{status=[int]$res.StatusCode;body=$body;proof=$(if($res.Headers.Contains('x-opencodex-attestation-proof')){@($res.Headers.GetValues('x-opencodex-attestation-proof'))[0]}else{$null})}}finally{$res.Dispose()}}finally{$req.Dispose();$cancel.Dispose()}
 }
 function Verify-Proof([string]$secret,[string]$challenge,[int]$processId,[int]$endpoint,[string]$proof){
  if($secret -cnotmatch '^[A-Za-z0-9_-]{43}$' -or $challenge -cnotmatch '^[A-Za-z0-9_-]{43}$' -or $proof -cnotmatch '^[A-Za-z0-9_-]{43}$'){return $false}
  $mac=[Security.Cryptography.HMACSHA256]::new([Text.Encoding]::UTF8.GetBytes($secret))
  try{$expected=[Convert]::ToBase64String($mac.ComputeHash([Text.Encoding]::UTF8.GetBytes(('opencodex-local-management-v1',$challenge,$processId,$endpoint -join [char]10)))).TrimEnd('=').Replace('+','-').Replace('/','_');return [Security.Cryptography.CryptographicOperations]::FixedTimeEquals([Text.Encoding]::ASCII.GetBytes($expected),[Text.Encoding]::ASCII.GetBytes($proof))}finally{$mac.Dispose()}
 }
 $r.stage='start';$watch=[Diagnostics.Stopwatch]::StartNew()
 $r.runtimeCreationEntered=$true
 $runtime=[Official299Proof.Owned]::new($bun,('"'+$bun+'" run "'+(Join-Path $package 'src/cli/index.ts')+'" start --port '+$port),$package,$ev,(Join-Path $out 'startup.stdout.log'),(Join-Path $out 'startup.stderr.log'),$sid)
 Require (($runtime.Limits() -band 0x1800) -eq 0) 'Job permits child breakaway'
 $r.job=@{unnamed=$true;retainedHandle=$true;limitFlags=$runtime.Limits();breakawayAllowed=$false;sddl=('O:'+$sid+'D:P(A;;GA;;;'+$sid+')(A;;GA;;;SY)')}
 $r.runtime=$runtime.Identity;$r.start=[ordered]@{pass=$false;elapsedMs=$null;receiptCount=0;receipt=$null;createdSuspended=$true;assignedBeforeResume=$true;jobInitialMembers=@($runtime.Members());jobInitialActive=$runtime.Active()}
 $homeGuard=[Official299Proof.Guard]::new($runtimeHome,$true);$guards.Add($homeGuard)
 $binding=@{schema='official299-host-binding/1';pid=$runtime.Identity.pid;filetime=$runtime.Identity.filetime;sid=$sid;home=$runtimeHome;homeIdentity=$homeGuard.Identity}
 $record=Join-Path $fixture 'binding.json';[IO.File]::WriteAllText($record,($binding|ConvertTo-Json));$bindingLock=[IO.File]::Open($record,'Open','Read','Read');$guards.Add([Official299Proof.Guard]::new($record,$false));$r.binding=@{value=$binding;sddl=(Check-Private $record);writeAndDeleteSharingDenied=$true}
 $runtime.Resume();$trace.Add(@{stage='resumed';elapsedMs=$watch.ElapsedMilliseconds;pid=$runtime.Identity.pid})
 $statePath=Join-Path $runtimeHome 'runtime-port.json';$state=$null;$ready=$null;$health=$null;$challenge=$null;$lines=@()
 while($watch.ElapsedMilliseconds -lt 30000){
  if($runtime.Exited()){throw ('official exited during start, original exit='+$runtime.ExitCode())}
  $observed=Observe-OwnedListeners
  if(Test-Path -LiteralPath $statePath){
   $null=Check-Private $statePath;$g=[Official299Proof.Guard]::new($statePath,$false)
   try{$state=Get-Content -LiteralPath $statePath -Raw|ConvertFrom-Json}finally{$g.Dispose()}
   Require ($state.pid -eq $runtime.Identity.pid -and $state.port -eq $port) 'runtime record PID/port differs'
   $challenge=[Convert]::ToBase64String([Security.Cryptography.RandomNumberGenerator]::GetBytes(32)).TrimEnd('=').Replace('+','-').Replace('/','_')
   $health=Request '/healthz' 'GET' (30000-$watch.ElapsedMilliseconds) @{'x-opencodex-attestation-challenge'=$challenge}
   Require (Verify-Proof $state.attestationSecret $challenge $state.pid $port $health.proof) 'official health HMAC rejected'
   $hb=$health.body|ConvertFrom-Json;Require ($health.status -eq 200 -and $hb.service -ceq 'opencodex' -and $hb.pid -eq $state.pid -and $hb.port -eq $port -and $hb.version -ceq '2.99.0') 'official health identity differs'
   $ready=Request '/readyz' 'GET' (30000-$watch.ElapsedMilliseconds) @{};$rb=$ready.body|ConvertFrom-Json
   $lines=@(Get-Content -LiteralPath (Join-Path $out 'startup.stdout.log')|Where-Object {$_ -ceq 'Client startup work complete.'})
   $trace.Add(@{stage='ready-probe';elapsedMs=$watch.ElapsedMilliseconds;http=$ready.status;body=$ready.body;completionLines=$lines.Count})
   if($ready.status -eq 200 -and $rb.status -ceq 'ready' -and $rb.service -ceq 'opencodex' -and $rb.pid -eq $state.pid -and $rb.port -eq $port -and $lines.Count -eq 1){break}
  }
  Start-Sleep -Milliseconds 100
 }
 Require ($watch.ElapsedMilliseconds -lt 30000 -and $null -ne $ready -and $ready.status -eq 200 -and $lines.Count -eq 1) 'original start budget or ready/completion condition failed'
 $owners=@([Official299Proof.Native]::Listeners($port));Require ($owners.Count -gt 0 -and @($owners|Where-Object {$_ -ne $runtime.Identity.pid}).Count -eq 0) 'listener owner differs'
 $receipt=[ordered]@{kind='host-start-receipt';pid=$runtime.Identity.pid;filetime=$runtime.Identity.filetime;sid=$sid;home=$runtimeHome;homeIdentity=$binding.homeIdentity;port=$port;jobMembers=@($runtime.Members());officialCompletionLine=$lines[0];challenge=$challenge;health=$health;ready=$ready}
 $receiptText=$receipt|ConvertTo-Json -Depth 20 -Compress
 Require ($watch.ElapsedMilliseconds -lt 30000) 'late start proof rejected'
 [IO.File]::WriteAllText((Join-Path $out 'startup-receipts.jsonl'),$receiptText+[char]10)
 $r.start.pass=$true;$r.start.elapsedMs=$watch.ElapsedMilliseconds;$r.start.receiptCount=1;$r.start.receipt=$receiptText
 $agentReady=[Official299Proof.Recovery]::Capture($agents);Write-Proof 'agent-after-ready.json' $agentReady
 $r.stage='status';$sw=[Diagnostics.Stopwatch]::StartNew()
 $now=[Official299Proof.Native]::Observe($runtime.Process)
 Require ($now.sid -ceq $sid -and $now.filetime -ceq $binding.filetime -and $now.pid -eq $binding.pid) 'same-generation identity differs'
 $challenge=[Convert]::ToBase64String([Security.Cryptography.RandomNumberGenerator]::GetBytes(32)).TrimEnd('=').Replace('+','-').Replace('/','_')
 $health=Request '/healthz' 'GET' (5000-$sw.ElapsedMilliseconds) @{'x-opencodex-attestation-challenge'=$challenge};Require (Verify-Proof $state.attestationSecret $challenge $state.pid $port $health.proof) 'status HMAC rejected'
 $ready=Request '/readyz' 'GET' (5000-$sw.ElapsedMilliseconds) @{};$rb=$ready.body|ConvertFrom-Json
 Require ($health.status -eq 200 -and $ready.status -eq 200 -and $rb.status -ceq 'ready' -and $rb.pid -eq $binding.pid -and $rb.port -eq $port -and $sw.ElapsedMilliseconds -lt 5000) 'original status budget/identity failed'
 $r.status=@{pass=$true;elapsedMs=$sw.ElapsedMilliseconds;identity=$now;health=$health;challenge=$challenge;ready=$ready}
 $r.counterexamples=@{reused='revision1 native identity and signature checks; no dependency or assertion removed';newRecoveryPrototype='recovery-test.ps1';newLockPrototype='lock-test.ps1'}
 $during=@();foreach($v in $before){$during+=@{relative=$v.relative;sha256=(Digest (Join-Path $fixture $v.relative));changed=((Digest (Join-Path $fixture $v.relative)) -cne $v.sha256)}};Write-Proof 'clients-during.json' $during;Write-Proof 'client-tree-during.json' @(Client-Inventory -AllowUnreadable)
 $r.injection=@{grokChanged=(@($during|Where-Object {$_.relative -eq 'grok/config.toml' -and $_.changed}).Count -eq 1);codexConfigChanged=(@($during|Where-Object {$_.relative -eq 'codex/config.toml' -and $_.changed}).Count -eq 1);catalogChanged=((Digest $catalogPath) -cne $catalogOriginal);catalogRoutedModelPresent=(@((Get-Content -LiteralPath $catalogPath -Raw|ConvertFrom-Json).models|Where-Object {$_.slug -ceq 'fixture/gpt-6.1-sol'}).Count -gt 0);catalogDuringSha256=(Digest $catalogPath)}
 Write-Proof 'injection.json' $r.injection
 $agentSealed=[Official299Proof.Recovery]::Capture($agents);Require (Agent-Same $agentReady $agentSealed) 'late Agent changes are unknown; do not silently extend write-set'
 $agentPlan=[Official299Proof.Recovery]::Plan($agentBefore,$agentSealed,$transactionId);$r.agentPlan=$agentPlan;Write-Proof 'agent-operation-plan.json' $agentPlan
 $r.runningListeners=Observe-OwnedListeners;Write-Proof 'listener-history.json' @($ownedListening.Values)
 $lockFacts=@();foreach($kind in @('claim','owner')){$path=Join-Path $codex ('.opencodex-native-main.'+$kind+'.sqlite');$g=[Official299Proof.Guard]::new($path,$false);try{$id=$g.Identity}finally{$g.Dispose()};$lockFacts+=@{kind=$kind;path=$path;id=$id;sddl=(Check-Private $path)}};$r.stableLocksBefore=$lockFacts
 $r.stage='stop';$sw=[Diagnostics.Stopwatch]::StartNew();$r.stop=[ordered]@{pass=$false;normalRequested=$false;response=$null;originalExited=$false;originalExitCode=$null;jobActive=$null;jobMembers=$null;listenerOwners=$null;restored=$null;elapsedMs=$null}
 Require ((Get-ProofDecision $binding (Read-ProofBinding $record) ([Official299Proof.Native]::Observe($runtime.Process)) $homeGuard.Identity) -ceq 'admitted' -and !$runtime.Exited()) 'stop authority lost'
 $r.stop.normalRequested=$true;$stop=Request '/api/stop' 'POST' (10000-$sw.ElapsedMilliseconds) @{'x-opencodex-api-key'=$admin};$r.stop.response=$stop
 $sb=$stop.body|ConvertFrom-Json;Require ($stop.status -eq 200 -and $sb.success -eq $true -and $sb.sharedTeardown -ceq 'performed') 'server restoration unconfirmed'
 while($sw.ElapsedMilliseconds -lt 10000 -and (!$runtime.Exited() -or $runtime.Active() -ne 0)){Start-Sleep -Milliseconds 25}
 $r.stop.originalExited=$runtime.Exited();if($r.stop.originalExited){$r.stop.originalExitCode=$runtime.ExitCode()};$r.stop.jobActive=$runtime.Active();$r.stop.jobMembers=@($runtime.Members());$r.stop.listenerOwners=@([Official299Proof.Native]::Listeners($port));$r.stop.retainedMemberExits=@($runtime.TrackedExits())
 Require ($r.stop.originalExited -and $r.stop.originalExitCode -eq 0 -and $r.stop.jobActive -eq 0 -and $r.stop.jobMembers.Count -eq 0) 'normal exit and complete Job proof missing before compensation'
 $allNow=@([Official299Proof.Native]::TcpListeners());$r.stop.allOriginalListeners=@($ownedListening.Values|ForEach-Object {$v=$_;$remaining=@($allNow|Where-Object {$_.family -ceq $v.family -and $_.address -ceq $v.address -and $_.port -eq $v.port});@{original=$v;remaining=$remaining;gone=($remaining.Count -eq 0)}})
 $agentExit=[Official299Proof.Recovery]::Capture($agents);Require (Agent-Same $agentSealed $agentExit) 'Agent postimage changed after seal; batch compensation refused'
 Require ((Digest $backup) -ceq $r.agentBackup.sha256) 'protected preimage backup changed'
 $r.stop.agentCompensation=@{kind='Desktop-test-host-compensation';officialApiReceiptUnchanged=$true;operations=@([Official299Proof.Recovery]::Restore($agentPlan,0,$null));transactionId=$transactionId}
 $agentFinal=[Official299Proof.Recovery]::Capture($agents);Write-Proof 'agent-after-compensation.json' $agentFinal
 $r.stop.agentRestored=(($agentBefore.Files|Sort-Object Name|Select-Object Name,Sha256,Sddl,Owner|ConvertTo-Json -Compress) -ceq ($agentFinal.Files|Sort-Object Name|Select-Object Name,Sha256,Sddl,Owner|ConvertTo-Json -Compress))
 $r.stop.stableLockReacquisition=@()
 foreach($lf in $lockFacts){
  Require ($sw.ElapsedMilliseconds -lt 10000) 'no remaining original Stop budget for native lock proof'
  $label='post-stop-'+$lf.kind;$lp=Join-Path $out ($label+'.json')
  $worker=[Official299Proof.Owned]::new($bun,('"'+$bun+'" run "'+(Join-Path $PSScriptRoot 'lock-worker.ts')+'" "'+$package+'" '+$lf.kind+' probe "'+$lp+'" "'+(Join-Path $fixture ($label+'.release'))+'"'),$package,$ev,(Join-Path $out ($label+'.stdout.log')),(Join-Path $out ($label+'.stderr.log')),$sid);$lockWorkers.Add($worker);$worker.Resume()
  while((!$worker.Exited() -or $worker.Active() -gt 0) -and $sw.ElapsedMilliseconds -lt 10000){Start-Sleep -Milliseconds 10}
  Require ($worker.Exited() -and $worker.ExitCode() -eq 0 -and $worker.Active() -eq 0) 'post-stop official lock worker incomplete within original budget'
  $lockResult=Get-Content -LiteralPath $lp -Raw|ConvertFrom-Json;$g=[Official299Proof.Guard]::new($lf.path,$false);try{$afterId=$g.Identity}finally{$g.Dispose()};$afterSddl=Check-Private $lf.path
  $r.stop.stableLockReacquisition+=@{kind=$lf.kind;workerIdentity=$worker.Identity;originalExitCode=$worker.ExitCode();jobActive=$worker.Active();result=$lockResult;fileIdentityBefore=$lf.id;fileIdentityAfter=$afterId;sddlBefore=$lf.sddl;sddlAfter=$afterSddl;elapsedWithinStopMs=$sw.ElapsedMilliseconds}
  Require ($lockResult.acquired -and $lockResult.released -and $afterId -ceq $lf.id -and $afterSddl -ceq $lf.sddl) 'stable lock reacquisition/identity/ACL proof failed'
 }
 $after=@();foreach($v in $before){$path=Join-Path $fixture $v.relative;$digest=if(Test-Path -LiteralPath $path){Digest $path}else{$null};$after+=@{relative=$v.relative;beforeSha256=$v.sha256;afterSha256=$digest;restored=($v.sha256 -ceq $digest);bytesBase64=$(if($digest -ceq $v.sha256){[Convert]::ToBase64String([IO.File]::ReadAllBytes($path))}else{$null})}}
 $clientFinal=@(Client-Inventory);$treeSame=(($clientBaseline|ConvertTo-Json -Compress) -ceq ($clientFinal|ConvertTo-Json -Compress));$r.stop.clientTreeRestored=$treeSame;$r.stop.clientConfigWasChanged=(@($during|Where-Object {$_.relative -eq 'codex/config.toml' -and $_.changed}).Count -eq 1);Write-Proof 'client-tree-after.json' $clientFinal
 $r.stop.restored=$after;$r.stop.elapsedMs=$sw.ElapsedMilliseconds
 $r.stop.catalogRestored=((Digest $catalogPath) -ceq $catalogOriginal);$r.stop.catalogAfterSha256=Digest $catalogPath
 $r.stop.allListenersGone=(@($r.stop.allOriginalListeners|Where-Object {!$_.gone}).Count -eq 0)
 $r.stop.stableLocksReleased=($r.stop.stableLockReacquisition.Count -eq 2)
 $r.restorationChainPassed=($r.stop.agentRestored -and $r.stop.catalogRestored -and $r.stop.allListenersGone -and $r.stop.stableLocksReleased -and $r.injection.grokChanged -and $r.injection.catalogChanged -and $r.injection.catalogRoutedModelPresent -and $r.injection.codexConfigChanged -and @($after|Where-Object {!$_.restored}).Count -eq 0 -and $sw.ElapsedMilliseconds -lt 10000)
 Write-Proof 'client-restored.json' $after
 Require ($r.stop.originalExited -and $r.stop.originalExitCode -eq 0 -and $r.stop.jobActive -eq 0 -and $r.stop.jobMembers.Count -eq 0 -and $r.stop.listenerOwners.Count -eq 0 -and @($after|Where-Object {!$_.restored}).Count -eq 0 -and $treeSame -and $r.stop.clientConfigWasChanged -and $sw.ElapsedMilliseconds -lt 10000) 'normal Stop full proof failed'
 $r.stop.pass=$true;$r.accepted=$true;$r.stage='complete'
} catch {
 $r.firstError=@{stage=$r.stage;type=$_.Exception.GetType().FullName;message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber;hresult=$_.Exception.HResult}
 if($r.stage -eq 'start' -and $r.start){$r.start.elapsedMs=$watch.ElapsedMilliseconds}
 if($r.stage -eq 'stop' -and $r.stop){$r.stop.elapsedMs=$sw.ElapsedMilliseconds}
} finally {
 if(!$r.runtimeCreationEntered){$r.cleanup+=@{kind='no-runtime-creation-entered';jobActive=0;jobMembers=@();reason='failure before native service constructor; no runtime/version subprocess launched'}}
 foreach($worker in $lockWorkers){try{if($worker.Active() -gt 0){$worker.ForceOwnedCleanup();$r.cleanup+=@{kind='separate-owned-lock-worker-force';identity=$worker.Identity;normalStopUnchanged=$true}};$worker.Dispose()}catch{$r.cleanup+=@{kind='unknown';scope='lock-worker';error=$_.Exception.Message}}}
 if($runtime){try{$pre=@{kind='before-forced-cleanup';originalExited=$runtime.Exited();jobActive=$runtime.Active();jobMembers=@($runtime.Members());memberIdentities=@($runtime.MemberIdentities());exitCode=$(if($runtime.Exited()){$runtime.ExitCode()}else{$null})};$r.cleanup+=$pre;if($pre.jobActive -gt 0){$runtime.ForceOwnedCleanup();$wait=[Diagnostics.Stopwatch]::StartNew();while((!$runtime.Exited() -or $runtime.Active() -gt 0) -and $wait.ElapsedMilliseconds -lt 5000){Start-Sleep -Milliseconds 25};$r.cleanup+=@{kind='separate-forced-owned-job';originalExited=$runtime.Exited();jobActive=$runtime.Active();jobMembers=@($runtime.Members());exitCode=$(if($runtime.Exited()){$runtime.ExitCode()}else{$null});listenerOwners=@([Official299Proof.Native]::Listeners($port));normalStopUnchanged=$true}}}catch{$r.cleanup+=@{kind='unknown';error=$_.Exception.Message}}}
 if($backupLock){$backupLock.Dispose()};if($bindingLock){$bindingLock.Dispose()};foreach($g in $guards){$g.Dispose()};if($runtime){$runtime.Dispose()};if($http){$http.Dispose()}
 Write-Proof 'listener-history.json' @($ownedListening.Values)
 Write-Proof 'startup-trace.json' $trace.ToArray();Write-Proof 'result.json' $r
}
if(!$r.accepted){exit 1}

param(
 [Parameter(Mandatory=$true)][string]$Directory,
 [Parameter(Mandatory=$true)][string]$ProgramDirectory,
 [Parameter(Mandatory=$true)][string]$ProvisionTool,
 [Parameter(Mandatory=$true)][string]$ExpectedSid,
 [Parameter(Mandatory=$true)][int]$ExpectedSessionId,
 [Parameter(Mandatory=$true)][string]$ExpectedSource,
 [Parameter(Mandatory=$true)][string]$ExpectedBuildRun
)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'windows023-two-user-native-helper.ps1')
. (Join-Path $PSScriptRoot 'windows-process-token.ps1')
$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$session=[Diagnostics.Process]::GetCurrentProcess().SessionId
$token=Get-WindowsHarnessToken
if($sid -cne $ExpectedSid -or $session -ne $ExpectedSessionId -or $session -ne 1){throw 'Exact local console SID/session required'}
Assert-WindowsProcessMode $token administrator
$taskRoot=Assert-TwoUserLocalPath $Directory;$source=Assert-TwoUserLocalPath $ProgramDirectory
Assert-TwoUserNoReparseAncestors $taskRoot;Assert-TwoUserNoReparseAncestors $source
if(Test-Path -LiteralPath (Join-Path $taskRoot 'identity.json')){throw 'Fresh console evidence required'}
$manifest=Get-Content -LiteralPath (Join-Path $source 'core-build.json') -Raw|ConvertFrom-Json
if($manifest.source -cne $ExpectedSource -or $manifest.run -cne $ExpectedBuildRun){throw 'Exact sealed product required'}
foreach($f in $manifest.files){$p=Join-Path $source $f.path;$null=Assert-TwoUserUnder $source $p;if((Get-Item -LiteralPath $p).Length -ne $f.bytes -or (Get-FileHash -LiteralPath $p).Hash -ine $f.sha256){throw 'Sealed member differs'}}
$toolBuild=Get-Content -LiteralPath (Join-Path (Split-Path -Parent $ProvisionTool) 'fixture-build.json') -Raw|ConvertFrom-Json
$descriptor=Join-Path $source 'runtime-bundle/runtime-bundle.manifest.json'
if(!$toolBuild.fixtureOnly -or $toolBuild.appsTree -cne $manifest.appsTree -or (Get-FileHash -LiteralPath $ProvisionTool).Hash -ine $toolBuild.sha256 -or (Get-FileHash -LiteralPath $descriptor).Hash -ine $toolBuild.descriptorSha256){throw 'Fixture product tree/resource/source identity differs'}
$private=Join-Path $taskRoot 'private';$fixture=Join-Path $private 'sandbox';$installed=Join-Path $fixture 'program'
New-Item -ItemType Directory -Path $installed|Out-Null
foreach($f in Get-ChildItem -LiteralPath $source -Force){Copy-Item -LiteralPath $f.FullName -Destination $installed -Recurse}
New-Item -ItemType Directory -Path (Join-Path $fixture 'home')|Out-Null
& icacls.exe $private /setowner ('*'+$sid) /T /C|Out-Null;if($LASTEXITCODE -ne 0){throw 'Own fixture owner failed'}
& icacls.exe $private /inheritance:r /grant:r ('*'+$sid+':(OI)(CI)F') '*S-1-5-18:(OI)(CI)F'|Out-Null;if($LASTEXITCODE -ne 0){throw 'Own fixture DACL failed'}
$code=Join-Path $private 'code';New-Item -ItemType Directory -Path $code|Out-Null
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'windows-cdp.mjs') -Destination $code
$control=Get-TwoUserRuntimeDriverText
$from=$control.IndexOf('    await diagnosticReadiness();',[StringComparison]::Ordinal);$to=$control.IndexOf('    const confirmedLate=',$from,[StringComparison]::Ordinal)
if($from -lt 0 -or $to -le $from){throw 'Original failure diagnostic boundary changed'}
$control=$control.Remove($from,$to-$from)
foreach($pair in @(
 @('join(process.env.USERPROFILE,''.codex-desktop/opencodex/home'')','join(config.fixtureRoot,''data/opencodex/home'')'),
 @('join(process.env.USERPROFILE,''.codex-desktop/desktop'')','join(config.fixtureRoot,''data/desktop'')'),
 @('[process.env.USERPROFILE,join(process.env.USERPROFILE,''.codex-desktop''),join(process.env.USERPROFILE,''.codex-desktop/opencodex''),c.home]','[config.fixtureRoot,join(config.fixtureRoot,''data''),join(config.fixtureRoot,''data/opencodex''),c.home]')
)){if(!$control.Contains($pair[0])){throw 'Reviewed Sandbox path adaptation changed'};$control=$control.Replace($pair[0],$pair[1])}
$driver=(Get-Content -LiteralPath (Join-Path $PSScriptRoot 'windows023-user1-start-stop.mjs') -Raw).Replace('/*USER1_RUNTIME_CONTROL*/',$control)
[IO.File]::WriteAllText((Join-Path $code 'console-driver.mjs'),$driver)
$executed=Join-Path $taskRoot 'executed-code';New-Item -ItemType Directory -Path $executed|Out-Null
Copy-Item -LiteralPath (Join-Path $code 'console-driver.mjs') -Destination $executed
foreach($n in @('windows023-console-lifecycle.ps1','windows023-two-user-native-helper.ps1','windows-process-token.ps1','windows-cdp.mjs')){Copy-Item -LiteralPath (Join-Path $PSScriptRoot $n) -Destination $executed}
Write-TwoUserJson (Join-Path $taskRoot 'fixture-build.json') $toolBuild
$node=Join-Path $installed 'runtime/node/node.exe';$exe=Join-Path $installed 'opencodex-desktop.exe'
$ledger=[Collections.Generic.Dictionary[int,object]]::new();$manager=$null;$managerFact=$null;$driverProcess=$null
$summary=[ordered]@{accepted=$false;source=$ExpectedSource;appsTree=$manifest.appsTree;fixtureSource=$toolBuild.source;sid=$sid;sessionId=$session;harnessToken=$token;sandbox=$true;productionProfileVerified=$false;scope='Actual elevated local console, same-SID WebView, isolated daily client directories, one Start then immediate normal Stop';startedUtc=[datetime]::UtcNow.ToString('o');runtimePassed=$false}
Write-TwoUserJson (Join-Path $taskRoot 'identity.json') $summary
function Daily-Hashes {
 $profile=[Environment]::GetFolderPath([Environment+SpecialFolder]::UserProfile)
 @(foreach($relative in @('.codex/config.toml','.codex/auth.json','.claude/settings.json','.claude.json','.codex-desktop/desktop/manager-location.json','.codex-desktop/desktop/manager-state/preferences.json')){$p=Join-Path $profile $relative;@{relative=$relative;exists=(Test-Path -LiteralPath $p -PathType Leaf);sha256=$(if(Test-Path -LiteralPath $p -PathType Leaf){(Get-FileHash -LiteralPath $p).Hash.ToLowerInvariant()}else{$null})}})
}
$daily=Daily-Hashes;Write-TwoUserJson (Join-Path $taskRoot 'daily-before.json') $daily
function Start-ConsoleManager {
 $listener=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0);$listener.Start();$port=$listener.LocalEndpoint.Port;$listener.Stop()
 $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$exe;$info.WorkingDirectory=$installed;$info.UseShellExecute=$false;$info.CreateNoWindow=$true
 $info.Environment['OPENCODEX_SANDBOX']='1';$info.Environment['OPENCODEX_SANDBOX_ROOT']=$fixture;$info.Environment['OPENCODEX_WINDOWS_SMOKE_CDP_PORT']=[string]$port
 foreach($key in @('HOME','OPENCODEX_HOME','CODEX_HOME','CLAUDE_CONFIG_DIR','WEBVIEW2_USER_DATA_FOLDER','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS')){[void]$info.Environment.Remove($key)}
 $p=[Diagnostics.Process]::Start($info);$fact=Get-TwoUserProcessFact $p $sid;$ledger[[int]$fact.pid]=$fact
 $appToken=Get-WindowsProcessToken $p $exe;Assert-WindowsProcessMode $appToken administrator
 [pscustomobject]@{process=$p;fact=$fact;port=$port;token=$appToken}
}
function Observe-ConsoleFamily {
 Add-TwoUserDescendants $managerFact $ledger
 $record=Join-Path $fixture 'data/opencodex/home/runtime-port.json'
 if(Test-Path -LiteralPath $record){
  $r=Get-Content -LiteralPath $record -Raw|ConvertFrom-Json
  if(!$ledger.ContainsKey([int]$r.pid)){throw 'Service PID not observed in original console launch family'}
  $fact=Assert-TwoUserLive $ledger[[int]$r.pid]
  Write-TwoUserJson (Join-Path $taskRoot 'runtime-record-native.json') @{pid=$r.pid;port=$r.port;native=$fact;recordAcl=(Get-Acl -LiteralPath $record).Sddl;listeners=@(Get-NetTCPConnection -LocalPort $r.port -State Listen -ErrorAction SilentlyContinue|Select-Object LocalAddress,LocalPort,OwningProcess);secretOmitted=$true;observedUtc=[datetime]::UtcNow.ToString('o')}
 }
}
try{
 # Let this exact product initialize its own recognized v2 data before supply.
 $initial=Start-ConsoleManager;$manager=$initial.process;$managerFact=$initial.fact
 $until=[datetime]::UtcNow.AddSeconds(40)
 do{Observe-ConsoleFamily;$ready=(Test-Path -LiteralPath (Join-Path $fixture 'data/desktop/manager-state/data-root.json'));if(!$ready){Start-Sleep -Milliseconds 250}}while(!$ready -and [datetime]::UtcNow -lt $until)
 if(!$ready){throw 'Product did not initialize own Sandbox v2 metadata'}
 foreach($f in @($ledger.Values|Sort-Object {[long]$_.creationTicks} -Descending)){Stop-TwoUserTracked $f};$manager.Dispose();$manager=$null;Close-TwoUserProcessHandles;$ledger.Clear()
 $pi=[Diagnostics.ProcessStartInfo]::new();$pi.FileName=$ProvisionTool;$pi.UseShellExecute=$false;$pi.CreateNoWindow=$true;$pi.RedirectStandardOutput=$true;$pi.RedirectStandardError=$true;$pi.ArgumentList.Add($fixture)
 $provision=[Diagnostics.Process]::Start($pi);$out=$provision.StandardOutput.ReadToEndAsync();$err=$provision.StandardError.ReadToEndAsync();$pf=Get-TwoUserProcessFact $provision $sid;$ledger[[int]$pf.pid]=$pf
 try{if(!$provision.WaitForExit(300000)){throw 'Fixture provision deadline; no runtime Start'};Write-TwoUserJson (Join-Path $taskRoot 'offline-provision.json') @{identity=$pf;exitCode=$provision.ExitCode;stdout=$out.GetAwaiter().GetResult();stderr=$err.GetAwaiter().GetResult()};if($provision.ExitCode -ne 0){throw 'Actual offline provisioner failed'}}finally{$provision.Dispose()}
 $active=Start-ConsoleManager;$manager=$active.process;$managerFact=$active.fact;$summary.applicationToken=$active.token;$summary.manager=$managerFact
 $until=[datetime]::UtcNow.AddSeconds(40);$window=$null
 do{Observe-ConsoleFamily;try{$window=Get-TwoUserWindow $managerFact}catch{};if(!$window){Start-Sleep -Milliseconds 250}}while(!$window -and [datetime]::UtcNow -lt $until)
 if(!$window){throw 'Actual elevated console window missing'}
 $summary.window=$window;Write-TwoUserJson (Join-Path $taskRoot 'active-console.json') $summary
 & $node --check (Join-Path $code 'console-driver.mjs');if($LASTEXITCODE -ne 0){throw 'Generated console driver syntax failed'}
 $config=Join-Path $taskRoot 'driver-input.json';$release=Join-Path $taskRoot 'driver-release'
 Write-TwoUserJson $config @{commit=$ExpectedSource;root=(Join-Path $fixture 'data/desktop');home=(Join-Path $fixture 'data/opencodex/home');fixtureRoot=$fixture;driverRelease=$release}
 $di=[Diagnostics.ProcessStartInfo]::new();$di.FileName=$node;$di.UseShellExecute=$false;$di.CreateNoWindow=$true;$di.Environment['USERPROFILE']=Join-Path $fixture 'home';$di.Environment['HOME']=Join-Path $fixture 'home'
 $di.RedirectStandardOutput=$true;$di.RedirectStandardError=$true
 foreach($a in @((Join-Path $code 'console-driver.mjs'),('http://127.0.0.1:'+$active.port),$taskRoot,$config)){$di.ArgumentList.Add($a)}
 $driverProcess=[Diagnostics.Process]::Start($di);$driverOut=$driverProcess.StandardOutput.ReadToEndAsync();$driverErr=$driverProcess.StandardError.ReadToEndAsync();$df=Get-TwoUserProcessFact $driverProcess $sid;$ledger[[int]$df.pid]=$df
 $until=[datetime]::UtcNow.AddSeconds(120)
 while(!$driverProcess.HasExited){Observe-ConsoleFamily;if(Test-Path -LiteralPath (Join-Path $taskRoot 'user1-result.json')){[IO.File]::WriteAllText($release,'result observed')};if([datetime]::UtcNow -gt $until){throw 'Driver envelope elapsed; no retry'};Start-Sleep -Milliseconds 200}
 Write-TwoUserJson (Join-Path $taskRoot 'driver-exit.json') @{identity=$df;originalHandleExited=$driverProcess.HasExited;exitCode=$driverProcess.ExitCode}
 [IO.File]::WriteAllText((Join-Path $taskRoot 'driver-stderr.log'),$driverErr.GetAwaiter().GetResult())
 $summary.runtimePassed=($driverProcess.ExitCode -eq 0)
}catch{$summary.firstFailure=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber;atUtc=[datetime]::UtcNow.ToString('o')}}
finally{
 $exits=@(foreach($f in $ledger.Values){try{@{identity=$f;exit=(Get-TwoUserOriginalExit $f)}}catch{@{identity=$f;exitCode=$null;unknown=$_.Exception.Message}}})
 Write-TwoUserJson (Join-Path $taskRoot 'original-exits-before-cleanup.json') $exits
 $cleanup=@();$failed=$false
 foreach($f in @($ledger.Values|Sort-Object {[long]$_.creationTicks} -Descending)){try{Stop-TwoUserTracked $f;$cleanup+=@{identity=$f;result='original-exited-or-exact-owned-cleanup'}}catch{$failed=$true;$cleanup+=@{identity=$f;result='retained-unproven';error=$_.Exception.Message}}}
 $log=Join-Path $fixture 'data/desktop/logs/app.log';if(Test-Path -LiteralPath $log){Copy-Item -LiteralPath $log -Destination (Join-Path $taskRoot 'app.log')}
 $after=Daily-Hashes;Write-TwoUserJson (Join-Path $taskRoot 'daily-after.json') $after
 $summary.dailyConfigBytesUnchanged=((ConvertTo-Json -InputObject $daily -Compress) -ceq (ConvertTo-Json -InputObject $after -Compress))
 if($manager){$manager.Dispose()};if($driverProcess){$driverProcess.Dispose()};Close-TwoUserProcessHandles
 $summary.cleanup=$cleanup;$summary.cleanupIdentitiesProven=(!$failed);$summary.finishedUtc=[datetime]::UtcNow.ToString('o')
 if(!$failed -and $summary.dailyConfigBytesUnchanged){try{$null=Assert-TwoUserUnder $taskRoot $private;Assert-TwoUserNoReparseAncestors $private;Remove-Item -LiteralPath $private -Recurse -Force;$summary.privateFixtureRemoved=!(Test-Path -LiteralPath $private)}catch{$summary.privateFixtureRemoved=$false;$summary.privateCleanupFailure=$_.Exception.Message}}
 Write-TwoUserJson (Join-Path $taskRoot 'finished.json') $summary
}

param(
 [Parameter(Mandatory=$true)][string]$Step,
 [Parameter(Mandatory=$true)][string]$Name,
 [Parameter(Mandatory=$true)][string]$Directory,
 [Parameter(Mandatory=$true)][ValidateSet('inputs','build','fixture-schema','state-observer','attested-status','runtime-port-save','bundle-roots','policy-prepare','policy-restore','installer','native','two-user','collect')][string]$Task,
 [string]$Kind,[string]$Mode,[string]$Probe
)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'records.ps1')
$repo=(Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$root=[IO.Path]::GetFullPath($Directory)
$recordDirectory=Join-Path $root 'sweep'
$evidence=Join-Path $root 'evidence'
$receipt=Invoke-SweepCheck $recordDirectory $Step $Name {
 switch($Task){
  inputs {
   $inputRoot=Join-Path $root 'input'
   $original=Get-Content -LiteralPath (Join-Path $inputRoot 'build-identities.json') -Raw|ConvertFrom-Json
   if($original.source -cne '0c869d9497c102b714dea1ee89394f999da8c4ec' -or $original.status -cne 'unverified-build-only' -or @($original.files).Count -ne 3){throw 'Exact original diagnostic inputs required; not an accepted candidate'}
   foreach($file in $original.files){
    if($file.name -notin @('opencodex-desktop.exe','OpenCodeX Desktop_0.2.23_x64-setup.exe','OpenCodeX Desktop_0.2.23_x64_en-US.msi')){throw 'Unexpected original artifact member'}
    $path=Join-Path $inputRoot $file.name
    if((Get-Item -LiteralPath $path).Length -ne $file.bytes -or (Get-FileHash -LiteralPath $path).Hash.ToLowerInvariant() -cne $file.sha256){throw 'Original diagnostic input bytes differ'}
   }
   $bundle=Join-Path $inputRoot 'bundle'
   foreach($type in @('msi','nsis')){[IO.Directory]::CreateDirectory((Join-Path $bundle $type))|Out-Null}
   Copy-Item -LiteralPath (Join-Path $inputRoot 'OpenCodeX Desktop_0.2.23_x64_en-US.msi') -Destination (Join-Path $bundle 'msi')
   Copy-Item -LiteralPath (Join-Path $inputRoot 'OpenCodeX Desktop_0.2.23_x64-setup.exe') -Destination (Join-Path $bundle 'nsis')
   $payload=Join-Path $env:RUNNER_TEMP ('ocxd023-sweep-payload-'+[guid]::NewGuid().ToString('N'))
   [IO.Directory]::CreateDirectory($evidence)|Out-Null
   $p=Start-Process msiexec.exe -ArgumentList ('/a "'+(Join-Path $bundle 'msi/OpenCodeX Desktop_0.2.23_x64_en-US.msi')+'" /qn TARGETDIR="'+$payload+'" /L*v "'+(Join-Path $evidence 'administrative-extract.log')+'"') -Wait -PassThru
   if($p.ExitCode -ne 0){throw ('Original payload extraction failed: '+$p.ExitCode)}
   $images=@(Get-ChildItem -LiteralPath $payload -Recurse -Filter opencodex-desktop.exe)
   if($images.Count -ne 1){throw 'One original MSI payload required'}
   $resources=Join-Path $images[0].DirectoryName 'runtime-bundle'
   $descriptor=Get-Content -LiteralPath (Join-Path $resources 'runtime-bundle.manifest.json') -Raw|ConvertFrom-Json
   if((Get-FileHash -LiteralPath (Join-Path $resources $descriptor.archiveName)).Hash.ToLowerInvariant() -cne $descriptor.archiveSha256){throw 'Original complete resource archive digest differs'}
   $destination=Join-Path $repo 'apps/desktop/tauri/resources/runtime-bundle'
   if(Test-Path -LiteralPath $destination){throw 'Fresh source resource destination required'}
   # Original resources stay bound to the old installer cells only. Current
   # product probes must stage the newly adopted policy, never reuse old bytes.
   $candidate=Join-Path $env:RUNNER_TEMP ('ocxd023-sweep-complete-'+[guid]::NewGuid().ToString('N'))
   [IO.Directory]::CreateDirectory($candidate)|Out-Null
   'OCXD_SWEEP_PROGRAM='+$candidate >> $env:GITHUB_ENV
   'OCXD_REQUIRE_RUNTIME_BUNDLE=1' >> $env:GITHUB_ENV
   @{productOriginal=$original.source;originalBuild=$original;resourceDescriptorSha256=(Get-FileHash (Join-Path $resources 'runtime-bundle.manifest.json')).Hash.ToLowerInvariant();candidateDirectory=$candidate}
  }
  build {
   if(!$env:OCXD_SWEEP_PROGRAM){throw 'Original complete resources prerequisite unavailable'}
   $node=(Get-Command node.exe).Source;$npm=Join-Path (Split-Path -Parent $node) 'node_modules/npm/bin/npm-cli.js'
   & python .github/scripts/stage_runtime_bundle_023.py --platform win32-x64 --build-node $node --npm-cli $npm --output (Join-Path $root 'current-runtime-stage') *> (Join-Path $evidence 'current-runtime-stage.log')
   if($LASTEXITCODE -ne 0){throw 'Current adopted runtime resources staging failed; old installers cannot substitute'}
   $resources=Join-Path $repo 'apps/desktop/tauri/resources/runtime-bundle'
   Copy-Item -LiteralPath $resources -Destination $env:OCXD_SWEEP_PROGRAM -Recurse
   $descriptor=Get-Content -LiteralPath (Join-Path $resources 'runtime-bundle.manifest.json') -Raw|ConvertFrom-Json
   $policy=Get-Content -LiteralPath (Join-Path $repo '.github/runtime-platforms-023.json') -Raw|ConvertFrom-Json -AsHashtable
   if($descriptor.runtimeVersion -cne $policy.runtime.version -or $descriptor.runtimeArchiveSha256 -cne $policy.runtime.sha256){throw 'Current resource runtime identity differs from adopted policy'}
   @{runtimeVersion=$descriptor.runtimeVersion;runtimeArchiveSha256=$descriptor.runtimeArchiveSha256;descriptorSha256=(Get-FileHash (Join-Path $resources 'runtime-bundle.manifest.json')).Hash.ToLowerInvariant();archiveSha256=$descriptor.archiveSha256}|ConvertTo-Json|Set-Content (Join-Path $evidence 'current-runtime-resource-identity.json')
   & npm --prefix apps/desktop/ui ci *> (Join-Path $evidence 'npm-ci.log');if($LASTEXITCODE -ne 0){throw 'Frontend dependency install failed'}
   & npm --prefix apps/desktop/ui run build *> (Join-Path $evidence 'frontend-build.log');if($LASTEXITCODE -ne 0){throw 'Frontend build failed'}
   & cargo build --manifest-path apps/desktop/tauri/Cargo.toml --locked --features custom-protocol --bins --jobs 2 --message-format=json 1> (Join-Path $evidence 'native-build-messages.json') 2> (Join-Path $evidence 'native-build.log');if($LASTEXITCODE -ne 0){throw 'Current native binary build failed'}
   & rustc --edition 2021 test/smoke/sweep/filesystem.rs -o (Join-Path $env:OCXD_SWEEP_PROGRAM 'sweep-filesystem.exe') *> (Join-Path $evidence 'filesystem-probe-build.log');if($LASTEXITCODE -ne 0){throw 'Test-only filesystem observer build failed'}
   foreach($file in @('opencodex-desktop.exe','ocxd.exe')){Copy-Item -LiteralPath (Join-Path $repo ('apps/desktop/tauri/target/debug/'+$file)) -Destination $env:OCXD_SWEEP_PROGRAM}
   @{source=(git -C $repo rev-parse HEAD).Trim();appsTree=(git -C $repo rev-parse HEAD:apps).Trim();profile='debug custom-protocol; diagnostic-only';files=@(Get-ChildItem $env:OCXD_SWEEP_PROGRAM -File|ForEach-Object{@{name=$_.Name;sha256=(Get-FileHash $_.FullName).Hash.ToLowerInvariant()}})}
  }
  fixture-schema {
   $schemaRuntime=Join-Path $root 'current-runtime-stage/platform-bundle/runtime-bundle/runtime/opencodex/node_modules/@bitkyc08/opencodex'
   $schemaBun=Join-Path $root 'current-runtime-stage/platform-bundle/runtime-bundle/runtime/bun/bun.exe'
   & python test/smoke/sweep/check_runtime_fixture.py --runtime $schemaRuntime --bun $schemaBun *> (Join-Path $evidence 'actual-runtime-fixture-schema.log')
   if($LASTEXITCODE -ne 0){throw 'Real-user synthetic fixture rejected by actual adopted runtime schema'}
   @{checks=2;source='actual adopted runtime configSchema';temporaryRoots='cleaned by Python context';accepted=$false}
  }
  state-observer {
   # Cargo carries its platform linker/native flags. A transient example keeps
   # this diagnostic source owned by test/, without adding a product command.
   $exampleDirectory=Join-Path $repo 'apps/desktop/tauri/examples'
   $example=Join-Path $exampleDirectory 'ocxd-sweep-state.rs'
   if(Test-Path -LiteralPath $example){throw 'Fresh transient diagnostic example required'}
   [IO.Directory]::CreateDirectory($exampleDirectory)|Out-Null
   Copy-Item -LiteralPath (Join-Path $repo 'test/smoke/sweep/state.rs') -Destination $example
   try{
    & cargo build --manifest-path apps/desktop/tauri/Cargo.toml --locked --features custom-protocol --example ocxd-sweep-state *> (Join-Path $evidence 'state-probe-build.log')
    if($LASTEXITCODE -ne 0){throw 'Independent Cargo backend observer build failed; product candidate remains available'}
    Copy-Item -LiteralPath (Join-Path $repo 'apps/desktop/tauri/target/debug/examples/ocxd-sweep-state.exe') -Destination (Join-Path $env:OCXD_SWEEP_PROGRAM 'sweep-state.exe')
   }finally{Remove-Item -LiteralPath $example -Force}
   @{source=(git -C $repo rev-parse HEAD).Trim();builtBy='Cargo exact product dependency/feature/linker graph';transientSourceRemoved=(!(Test-Path -LiteralPath $example));observerSha256=(Get-FileHash (Join-Path $env:OCXD_SWEEP_PROGRAM 'sweep-state.exe')).Hash.ToLowerInvariant();accepted=$false}
  }
  attested-status {
   & cargo test --manifest-path apps/desktop/tauri/Cargo.toml --locked --lib infrastructure::attested_status -- --nocapture *> (Join-Path $evidence 'attested-status-regression.log')
   if($LASTEXITCODE -ne 0){throw 'Actual Windows attested status transport/consumer regression failed'}
   $log=Get-Content -LiteralPath (Join-Path $evidence 'attested-status-regression.log') -Raw
   if($log -notmatch '8 passed; 0 failed'){throw 'All eight Windows attested status/official stop tests must actually execute'}
   & cargo test --manifest-path apps/desktop/tauri/Cargo.toml --locked --lib infrastructure::windows_task_scheduler -- --nocapture *> (Join-Path $evidence 'native-task-scheduler-regression.log')
   if($LASTEXITCODE -ne 0){throw 'Actual native read-only Task Scheduler observation failed'}
   $scheduler=Get-Content -LiteralPath (Join-Path $evidence 'native-task-scheduler-regression.log') -Raw
   if($scheduler -notmatch '2 passed; 0 failed'){throw 'Both native Task Scheduler and fail-closed query tests must execute'}
   @{source=(git -C $repo rev-parse HEAD).Trim();tests=8;taskSchedulerTests=2;platform='Windows actual loopback TCP/private files/consumer including official owner-only ACL and authenticated same-connection stop';accepted=$false}
  }
  runtime-port-save {
   & cargo test --manifest-path apps/desktop/tauri/Cargo.toml --locked --lib modules::runtime::port::tests:: -- --nocapture *> (Join-Path $evidence 'runtime-port-save-regression.log')
   if($LASTEXITCODE -ne 0){throw 'Actual Windows port persistence/SQLite coordination regression failed'}
   $log=Get-Content -LiteralPath (Join-Path $evidence 'runtime-port-save-regression.log') -Raw
   if($log -notmatch '8 passed; 0 failed'){throw 'All eight real port persistence and contention tests must execute'}
   @{source=(git -C $repo rev-parse HEAD).Trim();tests=8;accepted=$false}
  }
  bundle-roots {
   & cargo test --manifest-path apps/desktop/tauri/Cargo.toml --locked --lib windows_bundle_roots -- --nocapture *> (Join-Path $evidence 'windows-bundle-root-regression.log')
   if($LASTEXITCODE -ne 0){throw 'Actual Windows bundle root/reparse regression failed; original log retained'}
   $log=Get-Content -LiteralPath (Join-Path $evidence 'windows-bundle-root-regression.log') -Raw
   if($log -notmatch '3 passed; 0 failed'){throw 'All three Windows filesystem regressions must actually execute'}
   @{source=(git -C $repo rev-parse HEAD).Trim();tests=3;platform='actual Windows filesystem';accepted=$false}
  }
  policy-prepare { & (Join-Path $repo 'test/smoke/windows-msi-server-fixture.ps1') -Mode prepare -EvidenceDirectory (Join-Path $evidence 'server-policy');if(!$?){throw 'Server MSI preparation failed'};Get-Content (Join-Path $evidence 'server-policy/msi-server-policy.original.json') -Raw|ConvertFrom-Json }
  policy-restore { & (Join-Path $repo 'test/smoke/windows-msi-server-fixture.ps1') -Mode restore -EvidenceDirectory (Join-Path $evidence 'server-policy');if(!$?){throw 'Exact Server MSI restoration failed'};Get-Content (Join-Path $evidence 'server-policy/msi-server-policy.restored.json') -Raw|ConvertFrom-Json }
  installer {
   # Each cell gets its own actual ordinary account and fresh evidence directory;
   # one cell cannot suppress another or contaminate its remembered install path.
   $out=Join-Path $evidence ($Kind+'-'+$Mode)
   & (Join-Path $repo 'test/smoke/windows-installer.ps1') -BundleDirectory (Join-Path $root 'input/bundle') -BuiltExecutable (Join-Path $root 'input/opencodex-desktop.exe') -Node (Get-Command node.exe).Source -OutputDirectory $out -ExpectedCommit '0c869d9497c102b714dea1ee89394f999da8c4ec' -Kind $Kind -Mode $Mode
   if(!$? -or $LASTEXITCODE -ne 0){throw 'Actual ordinary-owner installer cell failed; original receipts retained'}
   $r=Get-Content -LiteralPath (Join-Path $out 'installer-verification.json') -Raw|ConvertFrom-Json
   if($r.result -cne 'pass'){throw ('Actual installer cell did not pass: '+$r.error)}
   @{receipt=$r;cleanup=(Get-Content (Join-Path $out 'installation-owner-cleanup.json') -Raw|ConvertFrom-Json)}
  }
  native {
   if(!$env:OCXD_SWEEP_PROGRAM){throw 'Current complete binary prerequisite unavailable'}
   $out=Join-Path $evidence ('native-'+$Probe)
   $node=(Get-Command node.exe).Source;$npm=Join-Path (Split-Path -Parent $node) 'node_modules/npm/bin/npm-cli.js'
   $nativeError=$null
   try{
   & (Join-Path $repo 'test/smoke/windows-install-recovery.ps1') -Executable (Join-Path $env:OCXD_SWEEP_PROGRAM 'opencodex-desktop.exe') -Node $node -NpmCli $npm -OutputDirectory $out -SweepProbe $Probe
   }catch{$nativeError=$_}
   $nativeReport=Join-Path $out 'native-sweep.json'
   if(!(Test-Path -LiteralPath $nativeReport)){if($nativeError){throw $nativeError};throw 'Actual native probe produced no receipt'}
   $r=Get-Content (Join-Path $out 'native-sweep.json') -Raw|ConvertFrom-Json
   if($r.status -cne 'pass'){$first=@($r.checks|Where-Object status -ceq 'fail'|Select-Object -First 1);throw ($first[0].name+': '+$first[0].firstError)}
   $r
  }
  two-user {
   if(!$env:OCXD_SWEEP_PROGRAM){throw 'Current complete binary prerequisite unavailable'}
   $out=Join-Path $env:RUNNER_TEMP ('ocxd023-sweep-users-'+[guid]::NewGuid().ToString('N'))
   try{& (Join-Path $repo 'test/smoke/windows023-two-user-native.ps1') -Executable (Join-Path $env:OCXD_SWEEP_PROGRAM 'opencodex-desktop.exe') -Node (Get-Command node.exe).Source -OutputDirectory $out -ExpectedCommit (git -C $repo rev-parse HEAD).Trim() -VerifyRuntimePorts -DiagnosticSweep}
   finally{if(Test-Path -LiteralPath $out){Copy-Item -LiteralPath $out -Destination (Join-Path $evidence 'two-user') -Recurse}}
   # This top-level step means actual overlapping distinct-SID Managers, not runtime readiness.
   $r=Get-Content (Join-Path $out 'two-user-native.json') -Raw|ConvertFrom-Json
   if(!$r.overlap -or $r.overlap.managers.Count -ne 2 -or $r.overlap.managers[0].sid -ceq $r.overlap.managers[1].sid -or $r.overlap.observedSeconds -lt 6){throw ('Two real overlapping SID Managers not proved: '+$r.failureCode)}
   $r.overlap
  }
  collect {throw 'Collector invoked through separate report adapter'}
 }
}
$receipt|ConvertTo-Json -Depth 15
if($receipt.status -cne 'pass'){exit 1}

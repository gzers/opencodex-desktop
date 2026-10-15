$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskRoot=Join-Path $taskCode 'test/runtime/foreground-02-current-20261015/install-a/data'
$taskRoaming=Join-Path $env:APPDATA 'com.gzers.opencodex.desktop'
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native product exists'}
if(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'preparation.original.json')){throw 'Idle preparation already exists'}
if(Test-Path -LiteralPath $taskRoaming){throw 'Unexpected new roaming root; inspect before altering it'}
$taskOriginal=Get-Content -LiteralPath (Join-Path $taskRoot 'manager-state/preferences.json') -Raw | ConvertFrom-Json
Copy-Item -LiteralPath (Join-Path $taskRoot 'manager-state/preferences.json') -Destination (Join-Path $PSScriptRoot 'candidate-preferences-before.original.json')
if($taskOriginal.schema_version -ne 2){throw 'Expected migrated candidate fixture'}
$taskOriginal.appearance.theme='light'
$taskOriginal.appearance.visual_effects='high'
$taskOriginal.appearance.glow_render='mesh'
$taskOriginal.appearance.interface_scale=100
$taskOriginal.shell.auto_panel=$false
$taskOriginal.shell.launch_with_codex=$false
$taskOriginal.shell.keep_proxy_on_close=$false
$taskOriginal.updates.app_update_auto_check=$false
$taskOriginal.maintenance.startup_cleanup=$false
$taskOriginal | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $taskRoot 'manager-state/preferences.json') -Encoding utf8
# Copy only the owned manager fixture to the now unused application root.
# Never follow links, merge an unknown directory, or copy the daily sidecars.
$taskItems=@(Get-ChildItem -LiteralPath $taskRoot -Recurse -Force)
if(@($taskItems | Where-Object {$_.Attributes -band [IO.FileAttributes]::ReparsePoint}).Count){throw 'Fixture alias refused'}
Copy-Item -LiteralPath $taskRoot -Destination $taskRoaming -Recurse
$taskBaselineText=(& git -C $taskCode show 'de6862d40389d527527b9db8ef52e186a58b625f:apps/desktop/tauri/config/preferences.defaults.json') -join "`n"
if($LASTEXITCODE -ne 0){throw 'Frozen baseline preferences unavailable'}
$taskBaseline=$taskBaselineText | ConvertFrom-Json
$taskBaseline.auto_panel=$false
$taskBaseline.launch_with_codex=$false
$taskBaseline.keep_proxy_on_close=$false
$taskBaseline.app_update_auto_check=$false
$taskBaseline.startup_cleanup=$false
$taskBaseline.theme='light'
$taskBaseline.visual_effects='high'
$taskBaseline.glow_render='mesh'
$taskBaseline.interface_scale=100
$taskBaseline | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $taskRoaming 'manager-state/preferences.json') -Encoding utf8
Copy-Item -LiteralPath (Join-Path $taskRoot 'manager-state/preferences.json') -Destination (Join-Path $PSScriptRoot 'candidate-preferences-seeded.json')
Copy-Item -LiteralPath (Join-Path $taskRoaming 'manager-state/preferences.json') -Destination (Join-Path $PSScriptRoot 'baseline-preferences-seeded.json')
Copy-Item -LiteralPath (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-FG-02-ROOT/native.ps1') -Destination (Join-Path $PSScriptRoot 'native.ps1')
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');nativeCount=0
    candidateRoot=$taskRoot;baselineRoot=$taskRoaming
    candidateReadVia='existing HKCU anchor';baselineReadVia='original ordinary AppData startup'
    fixtureCopiedFromOwnedRoot=$true;dailySidecarsCopied=$false
    accountChanged=$false;knownFolderOrHomeEnvironmentRedirected=$false
    theme='light';visualEffects='high';glowRender='mesh';interfaceScale=100
    autoChecksAndPanelAndLifecycleStartDisabled=$true
    candidateOrderFirst=$true;performanceSamplesCollected=0
} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'preparation.original.json') -Encoding utf8

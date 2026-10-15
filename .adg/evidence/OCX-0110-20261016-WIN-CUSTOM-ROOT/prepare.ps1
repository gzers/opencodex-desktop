$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskRuntime=Join-Path $taskCode 'test/runtime/foreground-02-current-20261015'
$taskAnchor=Join-Path $taskRuntime 'install-a/data'
$taskCustom=Join-Path $taskRuntime 'custom-root'
$taskHome=Join-Path $taskRuntime 'external-home'
if(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'preparation.original.json')){throw 'Custom binding already prepared'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native instance exists'}
$taskFiles=@(
    'apps/desktop/tauri/src/infrastructure/windows_startup_root.rs',
    'apps/desktop/tauri/src/modules/data_root/bootstrap.rs',
    'apps/desktop/tauri/src/modules/data_root/mod.rs',
    'apps/desktop/tauri/src/commands/data_root.rs',
    'apps/desktop/tauri/src/lib.rs'
)
$taskBlobs=@()
foreach($taskFile in $taskFiles){
    $taskOld=(& git -C $taskCode rev-parse ('02a0d4a8cbae8ef1450908d86b2c75fcad65f04f:'+$taskFile)).Trim()
    $taskNew=(& git -C $taskCode rev-parse ('8be87e2d54a895d4fe02e31300100a1ffed90121:'+$taskFile)).Trim()
    if($taskOld -cne $taskNew){throw 'Storage implementation changed; old candidate cannot cover this source scope'}
    $taskBlobs+=@{path=$taskFile;blob=$taskOld;identical=$true}
}
$taskPointer=(Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop').StartupDataRootV1
if(($taskPointer | ConvertFrom-Json).anchor -notlike '*foreground-02-current-20261015\install-a\data'){throw 'Unexpected startup pointer'}
if(Test-Path -LiteralPath $taskCustom){throw 'Old unexpected custom root exists'}
if(!(Test-Path -LiteralPath $taskHome)){throw 'Owned external HOME missing'}
$taskMetadata=Join-Path $taskAnchor 'manager-state/data-root.json'
$taskOriginal=Get-Content -LiteralPath $taskMetadata -Raw | ConvertFrom-Json
if($taskOriginal.structure_version -cne '1' -or $taskOriginal.active_data_root){throw 'Expected standalone anchor'}
Copy-Item -LiteralPath $taskMetadata -Destination (Join-Path $PSScriptRoot 'anchor-before.original.json')
Copy-Item -LiteralPath (Join-Path $taskAnchor 'logs/app.log') -Destination (Join-Path $PSScriptRoot 'anchor-log-before.original.log')
New-Item -ItemType Directory -Path (Join-Path $taskCustom 'manager-state') -Force | Out-Null
[ordered]@{structure_version='1'} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskCustom 'manager-state/data-root.json') -Encoding utf8
$taskPreferences=Get-Content -LiteralPath (Join-Path $taskAnchor 'manager-state/preferences.json') -Raw | ConvertFrom-Json
$taskPreferences.appearance.theme='light';$taskPreferences.appearance.visual_effects='mid'
$taskPreferences.shell.keep_proxy_on_close=$false;$taskPreferences.shell.auto_panel=$false
$taskPreferences.shell.launch_with_codex=$false;$taskPreferences.updates.app_update_auto_check=$false
$taskPreferences.maintenance.startup_cleanup=$false
$taskPreferences | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $taskCustom 'manager-state/preferences.json') -Encoding utf8
[ordered]@{structure_version='1';active_data_root=$taskCustom;opencodex_home_mode='external';opencodex_home_path=$taskHome} |
    ConvertTo-Json | Set-Content -LiteralPath $taskMetadata -Encoding utf8
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');actualExeSource='02a0d4a8cbae8ef1450908d86b2c75fcad65f04f'
    laterSource='8be87e2d54a895d4fe02e31300100a1ffed90121';unchangedStorageBlobs=$taskBlobs
    anchor=$taskAnchor;custom=$taskCustom;externalHome=$taskHome;startupPointer=$taskPointer
    bindingSeededAtRestInOwnedFixture=$true;uiSavingNotClaimed=$true
    realUserDataCopiedOrMigrated=$false;nativeCount=0;fullStorageGatePassed=$false
} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'preparation.original.json') -Encoding utf8

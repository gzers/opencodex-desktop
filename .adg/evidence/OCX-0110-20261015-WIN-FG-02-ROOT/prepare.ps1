$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskRuntime=Join-Path $taskCode 'test/runtime/foreground-02-current-20261015'
$taskOutput=Join-Path $PSScriptRoot 'preparation.original.json'
if ((Test-Path -LiteralPath $taskOutput) -or (Test-Path -LiteralPath $taskRuntime)) {
    throw 'Preparation already exists; inspect instead of repeating'
}
if (@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count) {
    throw 'Native product exists'
}
if ((& git -C $taskCode rev-parse HEAD).Trim() -cne '02a0d4a8cbae8ef1450908d86b2c75fcad65f04f' -or
    @(& git -C $taskCode status --porcelain).Count) { throw 'Code identity changed' }
if (Test-Path -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop') {
    if ((Get-Item -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop').Property -contains 'StartupDataRootV1') {
        throw 'Unexpected saved startup pointer; do not reset blindly'
    }
}
$taskCi=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'ci-readback.json') -Raw | ConvertFrom-Json
if ($taskCi.runs.Count -ne 3 -or @($taskCi.runs | Where-Object conclusion -ne success).Count) {
    throw 'Matching CI not verified'
}
New-Item -ItemType Directory -Path (Join-Path $taskRuntime 'install-a') | Out-Null
$taskBin=Join-Path $taskCode 'test/software/acceptance-kit-02a0d4-20261015/bin/candidate0110'
$taskImages=@()
foreach ($taskName in @('opencodex-desktop.exe','ocxd.exe')) {
    $taskFrom=Join-Path $taskBin $taskName
    $taskExpected=if($taskName -eq 'opencodex-desktop.exe'){'7383c275f58fed7e25678420a6d3f5c739f9b23d7373acc40fab1e7cd98db6d9'}else{'d90c6f17006b30ca5c8891b32438520dd4d074a8523bc98f98bd3d5d1bc824b5'}
    if((Get-FileHash -LiteralPath $taskFrom).Hash.ToLowerInvariant() -cne $taskExpected){throw 'Candidate changed'}
    $taskTo=Join-Path $taskRuntime ('install-a/'+$taskName)
    Copy-Item -LiteralPath $taskFrom -Destination $taskTo
    if((Get-FileHash -LiteralPath $taskTo).Hash.ToLowerInvariant() -cne $taskExpected){throw 'Copied image changed'}
    $taskImages+=@{name=$taskName;sha256=$taskExpected}
}
$taskMoved=@()
foreach($taskBase in @($env:APPDATA,$env:LOCALAPPDATA)){
    $taskFrom=[IO.Path]::GetFullPath((Join-Path $taskBase 'com.gzers.opencodex.desktop'))
    if(Test-Path -LiteralPath $taskFrom){
        $taskItem=Get-Item -LiteralPath $taskFrom -Force
        if(!$taskItem.PSIsContainer -or $taskItem.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'App data alias refused'}
        # Both absolute endpoints are confined to the explicitly named application
        # directory and its sibling; no recursive shell expansion is used.
        $taskTo=$taskFrom+'.acceptance-save-20261015T2206'
        if((Split-Path -Parent $taskTo) -cne (Split-Path -Parent $taskFrom) -or
            (Test-Path -LiteralPath $taskTo)){throw 'Unsafe or existing sidecar destination'}
        Move-Item -LiteralPath $taskFrom -Destination $taskTo
        if((Test-Path -LiteralPath $taskFrom) -or !(Test-Path -LiteralPath $taskTo)){throw 'Application sidecar move incomplete'}
        $taskMoved+=@{from=$taskFrom;to=$taskTo;deleted=$false}
    }
}
# Prevent lifecycle auto-starts during this root-admission subset. The data root
# is otherwise new; this is a documented preference seed, not an empty-root test.
$taskPrefs=Get-Content -LiteralPath (Join-Path $taskCode 'apps/desktop/tauri/config/preferences.defaults.json') -Raw | ConvertFrom-Json
$taskPrefs.auto_panel=$false
$taskPrefs.launch_with_codex=$false
$taskPrefs.app_update_auto_check=$false
$taskPrefs.startup_cleanup=$false
$taskPrefs.keep_proxy_on_close=$false
$taskPrefs.visual_effects='medium'
$taskPrefs.theme='light'
New-Item -ItemType Directory -Path (Join-Path $taskRuntime 'install-a/data/manager-state') -Force | Out-Null
$taskPrefs | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskRuntime 'install-a/data/manager-state/preferences.json') -Encoding utf8
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o')
    humanAuthorization='现在开始前台验证持续完成后续任务'
    configurationAuthorization='目前windows还没有用起来，你直接删掉都可以'
    source='02a0d4a8cbae8ef1450908d86b2c75fcad65f04f';runtime=$taskRuntime
    candidateRun=38737628201;images=$taskImages;sidecarMoves=$taskMoved
    preferenceSeed=@('auto_panel=false','launch_with_codex=false','app_update_auto_check=false','startup_cleanup=false','keep_proxy_on_close=false','medium','light')
    sandboxUsed=$false;cdpUsed=$false;hostInstallerExecuted=$false
    codexUserDirectoriesChanged=$false;dailyInstallationBinaryChanged=$false
    startupPointerBefore='absent';nativeCountBefore=0;fullVersionGatePassed=$false
} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $taskOutput -Encoding utf8

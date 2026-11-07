# Compiler diagnosis only. The payload is Windows where.exe, NEVER a product
# candidate. No installer/application execution or installation is performed.
$ErrorActionPreference = 'Stop'
$taskOut = [IO.Path]::GetFullPath((Join-Path $PWD '../../../test/out/windows023-msi-compiler'))
New-Item -ItemType Directory -Path $taskOut -Force | Out-Null
$taskReport = [ordered]@{source=(git rev-parse HEAD);fixture='Windows where.exe plus synthetic offline resource bytes';productAcceptance=$false;installerExecuted=$false;bundleExit=$null;candleExit=$null}
try {
    foreach ($taskDir in @('target/release','../ui/dist','resources/runtime-bundle')) {
        New-Item -ItemType Directory -Path $taskDir -Force | Out-Null
    }
    $taskSystemExe = Join-Path $env:SystemRoot 'System32/where.exe'
    foreach ($taskName in @('opencodex-desktop.exe','ocxd.exe')) {
        Copy-Item -LiteralPath $taskSystemExe -Destination (Join-Path 'target/release' $taskName)
    }
    '<html>MSI compiler fixture only</html>' | Set-Content ../ui/dist/index.html
    foreach ($taskName in @('runtime-bundle-0.2.23-win32-x64.tar.gz','maintained-runtime.tgz','runtime-bundle.manifest.json','runtime-bundle.manifest.sha256')) {
        'synthetic compiler fixture, not an installable runtime' | Set-Content (Join-Path 'resources/runtime-bundle' $taskName)
    }
    $taskCli = Join-Path (npm root --global) '@tauri-apps/cli/tauri.js'
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    # Bundle renders the actual repository template without a Rust build.
    & node $taskCli bundle --features custom-protocol --bundles msi --config tauri.runtime-bundle.conf.json --no-sign --verbose *> (Join-Path $taskOut 'tauri-bundle.log')
    $taskReport.bundleExit = $LASTEXITCODE
    $taskWix = Join-Path $env:LOCALAPPDATA 'tauri/WixTools314'
    $taskGenerated = Join-Path $PWD 'target/release/wix/x64'
    if (!(Test-Path (Join-Path $taskGenerated 'main.wxs'))) { throw 'Actual CLI did not render a WiX input; inspect tauri-bundle.log' }
    Copy-Item -LiteralPath $taskGenerated -Destination (Join-Path $taskOut 'generated') -Recurse
    # A direct replay preserves candle's original stdout/stderr even when the
    # bundler only surfaces its generic command failure. Use the same native
    # SourceDir and x64 architecture as the pinned bundler.
    Push-Location $taskGenerated
    try {
        & (Join-Path $taskWix 'candle.exe') -arch x64 ('-dSourceDir=' + [IO.Path]::GetFullPath((Join-Path $PWD '../../opencodex-desktop.exe'))) -ext (Join-Path $taskWix 'WixUIExtension.dll') -ext (Join-Path $taskWix 'WixUtilExtension.dll') main.wxs *> (Join-Path $taskOut 'candle.log')
        $taskReport.candleExit = $LASTEXITCODE
    } finally { Pop-Location }
    if ($taskReport.bundleExit -ne 0 -or $taskReport.candleExit -ne 0) { throw 'MSI compiler fixture failed; original generated source and output retained' }
} finally {
    $taskReport | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $taskOut 'compiler-result.json')
}

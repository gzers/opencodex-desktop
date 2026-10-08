param(
    [Parameter(Mandatory=$true)][string]$BundleDirectory,
    [Parameter(Mandatory=$true)][string]$BuiltExecutable,
    [Parameter(Mandatory=$true)][string]$Node,
    [Parameter(Mandatory=$true)][string]$OutputDirectory
)
$ErrorActionPreference = 'Stop'
if (!$IsWindows -or $env:GITHUB_ACTIONS -ne 'true') {
    throw 'Installer smoke is restricted to an ephemeral Windows Actions runner'
}
$taskBundle = (Resolve-Path -LiteralPath $BundleDirectory).Path
$taskBuilt = (Resolve-Path -LiteralPath $BuiltExecutable).Path
$taskOutput = [System.IO.Path]::GetFullPath($OutputDirectory)
$taskSource = (& git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $taskSource -notmatch '^[0-9a-f]{40}$') { throw 'Source commit missing' }
$taskHash = (Get-FileHash -LiteralPath $taskBuilt -Algorithm SHA256).Hash.ToLower()
$taskVersion = (Get-Content -LiteralPath (Join-Path $PSScriptRoot '../../apps/desktop/tauri/tauri.conf.json') -Raw | ConvertFrom-Json).version
if ([string]::IsNullOrWhiteSpace($taskVersion)) { throw 'Expected version missing from tauri.conf.json' }
New-Item -ItemType Directory -Force $taskOutput | Out-Null
$taskResult = [ordered]@{
    scope = 'ephemeral runner fresh NSIS/MSI install, final EXE first-screen and uninstall; no human upgrade/DPI/OTA acceptance'
    sourceCommit = $taskSource; builtExecutableSha256 = $taskHash
    version = $taskVersion; codeSigned = $false; releaseAccepted = $false
    result = 'fail'; installers = @()
}
function Invoke-Installer([string]$File, [string]$Arguments) {
    $process = Start-Process -FilePath $File -ArgumentList $Arguments -Wait -PassThru
    if ($process.ExitCode -notin @(0, 3010)) { throw "Installer failed with exit code $($process.ExitCode)" }
    return $process.ExitCode
}
try {
    # MSI reads the shared remembered InstallDir even after NSIS uninstall.
    # Test MSI on the fresh runner first; NSIS /D explicitly chooses its own root.
    foreach ($kind in @('msi', 'nsis')) {
        $pattern = if ($kind -eq 'nsis') { '*-setup.exe' } else { '*.msi' }
        $packages = @(Get-ChildItem -LiteralPath (Join-Path $taskBundle $kind) -Filter $pattern -File)
        if ($packages.Count -ne 1) { throw "Expected one $kind package, found $($packages.Count)" }
        $package = $packages[0]
        $installRoot = Join-Path $env:RUNNER_TEMP ('ocxd-candidate-' + $kind + '-' + [guid]::NewGuid().ToString('N'))
        $entry = [ordered]@{
            kind = $kind; name = $package.Name; bytes = $package.Length
            sha256 = (Get-FileHash -LiteralPath $package.FullName -Algorithm SHA256).Hash.ToLower()
            installDirectory = $installRoot; result = 'fail'
        }
        $taskResult.installers += $entry
        $installed = Join-Path $installRoot 'opencodex-desktop.exe'
        $installLog = Join-Path $taskOutput ($kind + '-install.log')
        if ($kind -eq 'nsis') {
            $entry.installExitCode = Invoke-Installer $package.FullName ('/S /D=' + $installRoot)
        } else {
            $entry.installExitCode = Invoke-Installer 'msiexec.exe' ('/i "' + $package.FullName + '" /qn /norestart INSTALLDIR="' + $installRoot + '" /L*v "' + $installLog + '"')
        }
        try {
            if (!(Test-Path -LiteralPath $installed)) { throw 'Installed main EXE missing' }
            $identityJson = & $Node (Join-Path $PSScriptRoot 'windows-bundle-identity.mjs') $taskBuilt $installed $kind
            if ($LASTEXITCODE -ne 0) { throw 'Installed EXE failed exact bundle identity verification' }
            $identity = $identityJson | ConvertFrom-Json
            if ($identity.builtSha256 -ne $taskHash) { throw 'Built EXE changed during installer verification' }
            $entry.installedExecutableSha256 = $identity.installedSha256
            $entry.expectedInstalledExecutableSha256 = $identity.expectedSha256
            $entry.bundleMarker = $identity.bundleMarker
            $entry.bundleMarkerOffset = $identity.bundleMarkerOffset
            $version = (Get-Item -LiteralPath $installed).VersionInfo.ProductVersion
            $entry.productVersion = $version
            if ($version -cne $taskVersion) { throw "Unexpected version $version; expected $taskVersion from tauri.conf.json" }
            & (Join-Path $PSScriptRoot 'windows-startup.ps1') -Executable $installed -Node $Node -OutputDirectory (Join-Path $taskOutput ($kind + '-startup')) -ExpectedCommit $taskSource
            $entry.firstScreen = 'pass'
        } finally {
            if ($kind -eq 'nsis') {
                $uninstaller = Join-Path $installRoot 'uninstall.exe'
                if (Test-Path -LiteralPath $uninstaller) { $entry.uninstallExitCode = Invoke-Installer $uninstaller '/S' }
            } else {
                $uninstallLog = Join-Path $taskOutput ($kind + '-uninstall.log')
                $entry.uninstallExitCode = Invoke-Installer 'msiexec.exe' ('/x "' + $package.FullName + '" /qn /norestart /L*v "' + $uninstallLog + '"')
            }
            $deadline = (Get-Date).AddSeconds(30)
            while ((Test-Path -LiteralPath $installed) -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 250 }
            if (Test-Path -LiteralPath $installed) { throw 'Uninstall left the main executable behind' }
            $entry.uninstallRemovedExecutable = $true
        }
        $entry.result = 'pass'
    }
    $taskResult.result = 'pass'
} catch {
    $taskResult.error = $_.Exception.Message
    throw
} finally {
    $taskResult | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskOutput 'installer-verification.json') -Encoding utf8
}

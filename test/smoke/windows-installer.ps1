param(
    [Parameter(Mandatory=$true)][string]$BundleDirectory,
    [Parameter(Mandatory=$true)][string]$BuiltExecutable,
    [Parameter(Mandatory=$true)][string]$Node,
    [Parameter(Mandatory=$true)][string]$OutputDirectory,
    [string]$ExpectedCommit = '',
    [ValidateSet('all','msi','nsis')][string]$Kind='all',
    [ValidateSet('all','default','custom')][string]$Mode='all'
)
$ErrorActionPreference = 'Stop'
if (!$IsWindows -or $env:GITHUB_ACTIONS -ne 'true') {
    throw 'Installer smoke is restricted to an ephemeral Windows Actions runner'
}
$taskBundle = (Resolve-Path -LiteralPath $BundleDirectory).Path
$taskBuilt = (Resolve-Path -LiteralPath $BuiltExecutable).Path
$taskOutput = [System.IO.Path]::GetFullPath($OutputDirectory)
$taskSource = if ($ExpectedCommit) { $ExpectedCommit } else { (& git rev-parse HEAD).Trim() }
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
function Find-DefaultInstalledDirectory {
    $roots=@('HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*','HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*','HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*')
    $entries=@(Get-ItemProperty -Path $roots -ErrorAction SilentlyContinue | Where-Object DisplayName -ceq 'OpenCodeX Desktop')
    $directories=@()
    foreach($item in $entries){
        if($item.InstallLocation){$directories+=([string]$item.InstallLocation).Trim().Trim('"')}
        if($item.DisplayIcon -match '^"?([^"\r\n]+?\.exe)'){$directories+=Split-Path -Parent $Matches[1]}
        if($item.UninstallString -match '^"([^"\r\n]+uninstall\.exe)"'){$directories+=Split-Path -Parent $Matches[1]}
    }
    $found=@($directories | Where-Object {![string]::IsNullOrWhiteSpace($_)} | ForEach-Object {$_.Trim().Trim('"')} | Sort-Object -Unique | Where-Object {Test-Path -LiteralPath (Join-Path $_ 'opencodex-desktop.exe')})
    if($found.Count -ne 1){throw 'Could not identify one product-owned default install directory'}
    return $found[0]
}
try {
    # No /D or INSTALLDIR override: exercise the installer's actual default/remembered path.
    $taskResult.defaultInstallChecks=@()
    $taskKinds=if($Kind -ceq 'all'){@('msi','nsis')}else{@($Kind)}
    if($Mode -cne 'custom'){
    foreach($kind in $taskKinds){
        $pattern=if($kind -eq 'nsis'){'*-setup.exe'}else{'*.msi'}
        $package=@(Get-ChildItem -LiteralPath (Join-Path $taskBundle $kind) -Filter $pattern -File)
        if($package.Count -ne 1){throw 'Exact default installer package missing'}
        if($kind -eq 'nsis'){[void](Invoke-Installer $package[0].FullName '/S')}
        else{[void](Invoke-Installer 'msiexec.exe' ('/i "'+$package[0].FullName+'" /qn /norestart'))}
        $defaultRoot=Find-DefaultInstalledDirectory
        try{
            $defaultExe=Join-Path $defaultRoot 'opencodex-desktop.exe'
            & (Join-Path $PSScriptRoot 'windows-startup.ps1') -Executable $defaultExe -Node $Node -OutputDirectory (Join-Path $taskOutput ($kind+'-default-startup')) -ExpectedCommit $taskSource
            $taskResult.defaultInstallChecks+=@{kind=$kind;result='pass';installDirectory=$defaultRoot;pathOverride=$false;boundary=$(if($Kind -ceq 'all'){'MSI fresh first; NSIS may honor the existing product remembered directory'}else{'fresh isolated runner; no earlier installed product or directory override'})}
            foreach($resource in @('LICENSE','THIRD-PARTY-NOTICES.txt')){
                if(!(Test-Path -LiteralPath (Join-Path $defaultRoot ('resources/'+$resource)))){throw 'Installed local documentation resource missing'}
            }
        }finally{
            if($kind -eq 'nsis'){[void](Invoke-Installer (Join-Path $defaultRoot 'uninstall.exe') '/S')}
            else{[void](Invoke-Installer 'msiexec.exe' ('/x "'+$package[0].FullName+'" /qn /norestart'))}
        }
    }
    }
    # MSI reads the shared remembered InstallDir even after NSIS uninstall.
    # Test MSI on the fresh runner first; NSIS /D explicitly chooses its own root.
    if($Mode -cne 'default'){
    foreach ($kind in $taskKinds) {
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
        $retentionFile = $null
        $retentionHash = $null
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
            $entry.explicitCleanupHelper = & (Join-Path $PSScriptRoot 'windows-uninstall-data.ps1') -InstallDirectory $installRoot
            # After confirmed cleanup, default uninstall must still retain data
            # written later. This fixture is never treated as an app binding.
            $retentionFile = Join-Path $installRoot 'data/default-uninstall-retain.sentinel'
            New-Item -ItemType Directory -Path (Split-Path -Parent $retentionFile) -Force | Out-Null
            [IO.File]::WriteAllText($retentionFile,'default uninstall retains data',[Text.UTF8Encoding]::new($false))
            $retentionHash = (Get-FileHash -LiteralPath $retentionFile).Hash.ToLowerInvariant()
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
            if ($retentionFile) {
                if ((Get-FileHash -LiteralPath $retentionFile).Hash.ToLowerInvariant() -cne $retentionHash) {
                    throw 'Default uninstall changed retained data'
                }
                $entry.defaultUninstallRetainedData = $true
            }
        }
        $entry.result = 'pass'
    }
    }
    $taskResult.result = 'pass'
} catch {
    $taskResult.error = $_.Exception.Message
    throw
} finally {
    $taskResult | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskOutput 'installer-verification.json') -Encoding utf8
}

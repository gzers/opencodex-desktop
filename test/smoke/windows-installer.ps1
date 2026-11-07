param(
    [Parameter(Mandatory=$true)][string]$BundleDirectory,
    [Parameter(Mandatory=$true)][string]$BuiltExecutable,
    [Parameter(Mandatory=$true)][string]$Node,
    [Parameter(Mandatory=$true)][string]$OutputDirectory,
    [string]$ExpectedCommit = '',
    [ValidateSet('all','msi','nsis')][string]$Kind='all',
    [ValidateSet('all','default','custom')][string]$Mode='all',
    [switch]$AsInstallationOwner
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'windows-installer-matrix.ps1')
if (!$IsWindows -or $env:GITHUB_ACTIONS -ne 'true') {
    throw 'Installer smoke is restricted to an ephemeral Windows Actions runner'
}
$taskBundle = (Resolve-Path -LiteralPath $BundleDirectory).Path
$taskBuilt = (Resolve-Path -LiteralPath $BuiltExecutable).Path
$taskOutput = [System.IO.Path]::GetFullPath($OutputDirectory)
$taskSource = Get-WindowsInstallerSource $ExpectedCommit
$taskHash = (Get-FileHash -LiteralPath $taskBuilt -Algorithm SHA256).Hash.ToLower()
$taskVersion = (Get-Content -LiteralPath (Join-Path $PSScriptRoot '../../apps/desktop/tauri/tauri.conf.json') -Raw | ConvertFrom-Json).version
if ([string]::IsNullOrWhiteSpace($taskVersion)) { throw 'Expected version missing from tauri.conf.json' }
New-Item -ItemType Directory -Force $taskOutput | Out-Null
if ($taskVersion -ceq '0.2.23' -and !$AsInstallationOwner) {
    # Both 023 installers are per-user. Install, start and uninstall as ONE
    # ordinary account; a second SID cannot read the installer's private I.
    $suiteInput = Join-Path $taskOutput 'ordinary-installer-input.json'
    @{bundle=$taskBundle;built=$taskBuilt;kind=$Kind;mode=$Mode;source=$taskSource;output=$taskOutput} |
        ConvertTo-Json | Set-Content -LiteralPath $suiteInput -Encoding utf8
    & (Join-Path $PSScriptRoot 'windows-ordinary-startup.ps1') -Executable $taskBuilt -Node $Node -OutputDirectory $taskOutput -ExpectedCommit $taskSource -InstallerSuiteConfig $suiteInput
    if ($LASTEXITCODE -ne 0) { throw 'Ordinary installation-owner suite failed' }
    return
}
if ($AsInstallationOwner) {
    . (Join-Path $PSScriptRoot 'windows-process-token.ps1')
    $ownerToken = Get-WindowsHarnessToken
    if ($taskVersion -cne '0.2.23' -or $ownerToken.elevated -ne $false) { throw '023 installer owner must be an ordinary native token' }
}
$taskResult = [ordered]@{
    scope = 'ephemeral runner fresh NSIS/MSI install, final EXE first-screen and uninstall; no human upgrade/DPI/OTA acceptance'
    sourceCommit = $taskSource; builtExecutableSha256 = $taskHash
    version = $taskVersion; codeSigned = $false; releaseAccepted = $false
    result = 'fail'; installers = @()
}
if ($AsInstallationOwner) { $taskResult.installationOwnerToken=$ownerToken; $taskResult.installationOwnerProfile=$env:USERPROFILE }
# Read-only policy evidence. Never relax a machine/user installer policy.
$taskResult.installerPolicies=@(foreach($policyPath in @('HKLM:\Software\Policies\Microsoft\Windows\Installer','HKCU:\Software\Policies\Microsoft\Windows\Installer')){
 $properties=Get-ItemProperty -LiteralPath $policyPath -ErrorAction SilentlyContinue
 @{path=$policyPath;exists=($null -ne $properties);DisableMSI=$properties.DisableMSI;DisableUserInstalls=$properties.DisableUserInstalls;AlwaysInstallElevated=$properties.AlwaysInstallElevated}
})

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
function Clear-ProductRememberedInstallDirectory {
    if ($AsInstallationOwner) {
        # Only this test owner's NEW-version key; no other SID/HKLM/old root.
        $key='HKCU:\Software\gzers\OpenCodeX Desktop\023'
        if (Test-Path -LiteralPath $key) { Remove-ItemProperty -LiteralPath $key -Name InstallDir -ErrorAction SilentlyContinue }
        if ((Get-ItemProperty -LiteralPath $key -Name InstallDir -ErrorAction SilentlyContinue).InstallDir) { throw 'Owner remembered install directory remains' }
        return @()
    }
    # WiX AppSearch restores the remembered location before resolving INSTALLDIR.
    # Clear only this product-owned value in both views/hives, then prove it is
    # absent so a later MSI command cannot silently be redirected by stale state.
    $locations=[System.Collections.Generic.List[string]]::new()
    foreach($view in @([Microsoft.Win32.RegistryView]::Registry32,[Microsoft.Win32.RegistryView]::Registry64)){
      foreach($hive in @([Microsoft.Win32.RegistryHive]::CurrentUser,[Microsoft.Win32.RegistryHive]::LocalMachine)){
        $base=[Microsoft.Win32.RegistryKey]::OpenBaseKey($hive,$view)
        try{
          $subkey='Software\gzers\OpenCodeX Desktop'
          $key=$base.OpenSubKey($subkey,$true)
          if($key){try{if($null -ne $key.GetValue('InstallDir',$null)){$locations.Add("$hive\$subkey ($view)");$key.DeleteValue('InstallDir',$false)}}finally{$key.Dispose()}}
          $verify=$base.OpenSubKey($subkey,$false)
          if($verify){try{if($null -ne $verify.GetValue('InstallDir',$null)){throw "Could not clear product InstallDir from $hive ($view)"}}finally{$verify.Dispose()}}
        }finally{$base.Dispose()}
      }
      # MSI AppSearch can run against a user hive left loaded by a prior
      # per-user install. Enumerate loaded hives but touch only this product key.
      $users=[Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::Users,$view)
      try{
        foreach($sid in $users.GetSubKeyNames() | Where-Object {$_ -match '^S-1-' -and $_ -notmatch '_Classes$'}){
          $subkey="$sid\Software\gzers\OpenCodeX Desktop"
          $key=$users.OpenSubKey($subkey,$true)
          if($key){try{if($null -ne $key.GetValue('InstallDir',$null)){$locations.Add("HKEY_USERS\$subkey ($view)");$key.DeleteValue('InstallDir',$false)}}finally{$key.Dispose()}}
          $verify=$users.OpenSubKey($subkey,$false)
          if($verify){try{if($null -ne $verify.GetValue('InstallDir',$null)){throw "Could not clear product InstallDir from HKEY_USERS\$subkey ($view)"}}finally{$verify.Dispose()}}
        }
      }finally{$users.Dispose()}
    }
    return @($locations)
}
try {
    # No /D or INSTALLDIR override: exercise the installer's actual default/remembered path.
    $taskResult.defaultInstallChecks=@()
    $taskKinds=if($Kind -ceq 'all'){@('msi','nsis')}else{@($Kind)}
    # Run all MSI cells before any NSIS cell: NSIS writes a per-user remembered
    # path that Windows Installer AppSearch can observe from loaded user hives.
    $defaultTaskKinds=if($Kind -ceq 'all'){@('msi')}else{@($Kind)}
    if($Mode -cne 'custom'){
    foreach($installerKind in $defaultTaskKinds){
        $pattern=if($installerKind -eq 'nsis'){'*-setup.exe'}else{'*.msi'}
        $package=@(Get-ChildItem -LiteralPath (Join-Path $taskBundle $installerKind) -Filter $pattern -File)
        if($package.Count -ne 1){throw 'Exact default installer package missing'}
        if($installerKind -eq 'nsis'){[void](Invoke-Installer $package[0].FullName '/S')}
        else{
            $entryCleanup = Clear-ProductRememberedInstallDirectory
            [void](Invoke-Installer 'msiexec.exe' ('/i "'+$package[0].FullName+'" /qn /norestart /L*v "'+(Join-Path $taskOutput 'msi-default-install.log')+'"'))
            $taskResult.defaultMsiRegistryCleanup = @($entryCleanup)
        }
        $defaultRoot=Find-DefaultInstalledDirectory
        if ($AsInstallationOwner -and [IO.Path]::GetFullPath($defaultRoot).TrimEnd('/') -ine [IO.Path]::GetFullPath((Join-Path $env:LOCALAPPDATA 'Programs/OpenCodeX Desktop')).TrimEnd('/')) { throw '023 default installation escaped actual ordinary owner Programs directory' }
        try{
            $defaultExe=Join-Path $defaultRoot 'opencodex-desktop.exe'
            if ($AsInstallationOwner) {
                & (Join-Path $PSScriptRoot 'windows-startup.ps1') -Executable $defaultExe -Node $Node -OutputDirectory (Join-Path $taskOutput ($installerKind+'-default-startup')) -ExpectedCommit $taskSource -Mode ordinary
                if ($LASTEXITCODE -ne 0) { throw 'Default per-user package failed ordinary installation-owner startup' }
            } elseif($installerKind -eq 'nsis'){
                # Default NSIS is per-user under the installing account's AppData;
                # another temporary user's token correctly cannot read that tree.
                & (Join-Path $PSScriptRoot 'windows-startup.ps1') -Executable $defaultExe -Node $Node -OutputDirectory (Join-Path $taskOutput ($installerKind+'-default-startup')) -ExpectedCommit $taskSource -Mode any
                if($LASTEXITCODE -ne 0){throw 'Default per-user NSIS EXE failed owner-account startup smoke'}
            }else{
                & (Join-Path $PSScriptRoot 'windows-ordinary-startup.ps1') -Executable $defaultExe -Node $Node -OutputDirectory (Join-Path $taskOutput ($installerKind+'-default-startup')) -ExpectedCommit $taskSource
                if($LASTEXITCODE -ne 0){throw 'Default MSI EXE failed ordinary-account startup smoke'}
            }
            $taskResult.defaultInstallChecks+=@{kind=$installerKind;result='pass';installDirectory=$defaultRoot;pathOverride=$false;boundary=$(if($Kind -ceq 'all'){'MSI fresh first; NSIS may honor the existing product remembered directory'}else{'fresh isolated runner; no earlier installed product or directory override'})}
            foreach($resource in @('LICENSE','THIRD-PARTY-NOTICES.txt')){
                if(!(Test-Path -LiteralPath (Join-Path $defaultRoot ('resources/'+$resource)))){throw 'Installed local documentation resource missing'}
            }
        }finally{
            if($installerKind -eq 'nsis'){[void](Invoke-Installer (Join-Path $defaultRoot 'uninstall.exe') '/S')}
            else{[void](Invoke-Installer 'msiexec.exe' ('/x "'+$package[0].FullName+'" /qn /norestart'))}
        }
    }
    }
    # MSI reads the shared remembered InstallDir even after NSIS uninstall.
    # Test MSI on the fresh runner first; NSIS /D explicitly chooses its own root.
    if($Mode -cne 'default'){
    foreach ($installerKind in $taskKinds) {
        $pattern = if ($installerKind -eq 'nsis') { '*-setup.exe' } else { '*.msi' }
        $packages = @(Get-ChildItem -LiteralPath (Join-Path $taskBundle $installerKind) -Filter $pattern -File)
        if ($packages.Count -ne 1) { throw "Expected one $installerKind package, found $($packages.Count)" }
        $package = $packages[0]
        $installRoot = Join-Path $env:RUNNER_TEMP ('ocxd-candidate-' + $installerKind + '-' + [guid]::NewGuid().ToString('N'))
        $entry = [ordered]@{
            kind = $installerKind; name = $package.Name; bytes = $package.Length
            sha256 = (Get-FileHash -LiteralPath $package.FullName -Algorithm SHA256).Hash.ToLower()
            installDirectory = $installRoot; result = 'fail'
        }
        $retentionFile = $null
        $retentionHash = $null
        $taskResult.installers += $entry
        $installed = Join-Path $installRoot 'opencodex-desktop.exe'
        $installLog = Join-Path $taskOutput ($installerKind + '-install.log')
        if ($installerKind -eq 'nsis') {
            $entry.installExitCode = Invoke-Installer $package.FullName ('/S /D=' + $installRoot)
        } else {
            $entry.registryCleanup = @(Clear-ProductRememberedInstallDirectory)
            $entry.installExitCode = Invoke-Installer 'msiexec.exe' ('/i "' + $package.FullName + '" /qn /norestart INSTALLDIR="' + $installRoot + '" /L*v "' + $installLog + '"')
        }
        try {
            if (!(Test-Path -LiteralPath $installed)) { throw 'Installed main EXE missing' }
            $identityJson = & $Node (Join-Path $PSScriptRoot 'windows-bundle-identity.mjs') $taskBuilt $installed $installerKind
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
            if ($AsInstallationOwner) {
                & (Join-Path $PSScriptRoot 'windows-startup.ps1') -Executable $installed -Node $Node -OutputDirectory (Join-Path $taskOutput ($installerKind + '-startup')) -ExpectedCommit $taskSource -Mode ordinary
            } else {
                & (Join-Path $PSScriptRoot 'windows-ordinary-startup.ps1') -Executable $installed -Node $Node -OutputDirectory (Join-Path $taskOutput ($installerKind + '-startup')) -ExpectedCommit $taskSource
            }
            if($LASTEXITCODE -ne 0){throw 'Custom-path installed EXE failed ordinary-account startup smoke'}
            $entry.firstScreen = 'pass'
            $entry.explicitCleanupHelper = & (Join-Path $PSScriptRoot 'windows-uninstall-data.ps1') -InstallDirectory $installRoot
            # After confirmed cleanup, default uninstall must still retain data
            # written later. This fixture is never treated as an app binding.
            $retentionFile = Join-Path $installRoot 'data/default-uninstall-retain.sentinel'
            New-Item -ItemType Directory -Path (Split-Path -Parent $retentionFile) -Force | Out-Null
            [IO.File]::WriteAllText($retentionFile,'default uninstall retains data',[Text.UTF8Encoding]::new($false))
            $retentionHash = (Get-FileHash -LiteralPath $retentionFile).Hash.ToLowerInvariant()
        } finally {
            if ($installerKind -eq 'nsis') {
                $uninstaller = Join-Path $installRoot 'uninstall.exe'
                if (Test-Path -LiteralPath $uninstaller) { $entry.uninstallExitCode = Invoke-Installer $uninstaller '/S' }
            } else {
                $uninstallLog = Join-Path $taskOutput ($installerKind + '-uninstall.log')
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
    if($Mode -cne 'custom' -and $Kind -ceq 'all'){
        $installerKind='nsis'
        $package=@(Get-ChildItem -LiteralPath (Join-Path $taskBundle $installerKind) -Filter '*-setup.exe' -File)
        if($package.Count -ne 1){throw 'Exact default NSIS installer package missing'}
        [void](Invoke-Installer $package[0].FullName '/S')
        $defaultRoot=Find-DefaultInstalledDirectory
        if ($AsInstallationOwner -and [IO.Path]::GetFullPath($defaultRoot).TrimEnd('/') -ine [IO.Path]::GetFullPath((Join-Path $env:LOCALAPPDATA 'Programs/OpenCodeX Desktop')).TrimEnd('/')) { throw '023 NSIS default installation escaped actual ordinary owner Programs directory' }
        try{
            $defaultExe=Join-Path $defaultRoot 'opencodex-desktop.exe'
            $startupMode = if ($AsInstallationOwner) { 'ordinary' } else { 'any' }
            & (Join-Path $PSScriptRoot 'windows-startup.ps1') -Executable $defaultExe -Node $Node -OutputDirectory (Join-Path $taskOutput 'nsis-default-startup') -ExpectedCommit $taskSource -Mode $startupMode
            if($LASTEXITCODE -ne 0){throw 'Default per-user NSIS EXE failed owner-account startup smoke'}
            $taskResult.defaultInstallChecks+=@{kind='nsis';result='pass';installDirectory=$defaultRoot;pathOverride=$false;boundary='per-user NSIS is checked after MSI cells to avoid contaminating MSI AppSearch'}
            foreach($resource in @('LICENSE','THIRD-PARTY-NOTICES.txt')){
                if(!(Test-Path -LiteralPath (Join-Path $defaultRoot ('resources/'+$resource)))){throw 'Installed local documentation resource missing'}
            }
        }finally{[void](Invoke-Installer (Join-Path $defaultRoot 'uninstall.exe') '/S')}
    }
    Assert-WindowsInstallerMatrix -Kind $Kind -Mode $Mode -DefaultChecks @($taskResult.defaultInstallChecks) -CustomChecks @($taskResult.installers)
    $taskResult.result = 'pass'
} catch {
    $taskResult.error = $_.Exception.Message
    throw
} finally {
    $taskResult | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskOutput 'installer-verification.json') -Encoding utf8
}

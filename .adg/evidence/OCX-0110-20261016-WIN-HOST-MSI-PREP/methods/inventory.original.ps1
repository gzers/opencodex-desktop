param([string]$OutputDirectory)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $OutputDirectory){throw 'Preserve prior installer inventory'}
$null=New-Item -ItemType Directory -Path $OutputDirectory
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native process exists'}
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskPackages=@(
    @{kind='official019-msi';path=(Join-Path $taskCode 'test/software/OpenCodeX.Desktop_0.1.9_x64_en-US.msi');sha256='b33dd0c890511467e72a453daaceb4b60b6f5af64dcae5dba54f69ed46bbad6a'},
    @{kind='candidate823-msi';path=(Join-Path $taskCode 'test/software/candidate-823a8a-38841117656/installers/apps/desktop/tauri/target/release/bundle/msi/OpenCodeX Desktop_0.1.10_x64_en-US.msi');sha256='e64491715bc5e510d06eb8a18f723b6700be800e4555a44c4e2207059bf0318c'}
)
$taskInstaller=New-Object -ComObject WindowsInstaller.Installer
$taskResults=@()
try {
    foreach($taskPackage in $taskPackages){
        $taskPath=(Resolve-Path -LiteralPath $taskPackage.path).Path
        if(!$taskPath.StartsWith((Join-Path $taskCode 'test/software')+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Wrong package scope'}
        if((Get-FileHash -LiteralPath $taskPath).Hash.ToLowerInvariant() -cne $taskPackage.sha256){throw 'Package hash changed'}
        $taskDatabase=$taskInstaller.OpenDatabase($taskPath,0)
        try {
            $taskPropertyView=$taskDatabase.OpenView('SELECT `Property`,`Value` FROM `Property`')
            $taskProperties=[ordered]@{}
            try {
                $null=$taskPropertyView.Execute()
                while($taskRow=$taskPropertyView.Fetch()){
                    try {if($taskRow.StringData(1) -in @('ProductCode','ProductVersion','UpgradeCode','ALLUSERS','AUTOLAUNCHAPP','SecureCustomProperties')){$taskProperties[$taskRow.StringData(1)]=$taskRow.StringData(2)}}
                    finally {$null=[Runtime.InteropServices.Marshal]::ReleaseComObject($taskRow)}
                }
            }finally {$null=$taskPropertyView.Close();$null=[Runtime.InteropServices.Marshal]::ReleaseComObject($taskPropertyView)}
            $taskView=$taskDatabase.OpenView('SELECT `Action`,`Condition`,`Sequence` FROM `InstallExecuteSequence`')
            $taskLaunch=@()
            try {
                $null=$taskView.Execute()
                while($taskRow=$taskView.Fetch()){
                    try {if($taskRow.StringData(1) -ceq 'LaunchApplication'){$taskLaunch+=@{action=$taskRow.StringData(1);condition=$taskRow.StringData(2);sequence=$taskRow.StringData(3)}}}
                    finally {$null=[Runtime.InteropServices.Marshal]::ReleaseComObject($taskRow)}
                }
            }finally {$null=$taskView.Close();$null=[Runtime.InteropServices.Marshal]::ReleaseComObject($taskView)}
            $taskResults+=@{kind=$taskPackage.kind;path=$taskPath;sha256=$taskPackage.sha256;bytes=(Get-Item -LiteralPath $taskPath).Length;properties=$taskProperties;launchSequence=$taskLaunch;databaseMode='read-only'}
        }finally {$null=[Runtime.InteropServices.Marshal]::ReleaseComObject($taskDatabase)}
    }
    $taskInstalledCode='{7069DA97-F262-4585-98CA-AE416E6833A5}'
    $taskLocalPackage=$taskInstaller.ProductInfo($taskInstalledCode,'LocalPackage')
    $taskInfo=[ordered]@{productCode=$taskInstalledCode;version=$taskInstaller.ProductInfo($taskInstalledCode,'VersionString')
        installLocation=$taskInstaller.ProductInfo($taskInstalledCode,'InstallLocation')
        cachedPackage=$taskLocalPackage;cachedPackageSha256=(Get-FileHash -LiteralPath $taskLocalPackage).Hash.ToLowerInvariant()}
}finally {$null=[Runtime.InteropServices.Marshal]::ReleaseComObject($taskInstaller)}
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
$taskHarnessToken=Get-WindowsHarnessToken
$taskDaily='D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe'
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');packages=$taskResults;installedMsi=$taskInfo
    installedExe=@{path=$taskDaily;version=(Get-Item -LiteralPath $taskDaily).VersionInfo.ProductVersion;sha256=(Get-FileHash -LiteralPath $taskDaily).Hash.ToLowerInvariant()}
    actualObservationHarnessToken=$taskHarnessToken
    packageRegistryOnlyNotProductRootAcceptance=$true
    nativeCount=0;installerExecuted=$false;productLaunched=$false;registryWritten=$false
    currentShellAdministrator=$taskHarnessToken.elevated;hostInstallerGatePassed=$false} |
    ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $OutputDirectory 'msi-host-inventory.original.json') -Encoding utf8

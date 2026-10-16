# Inspect two verified databases in mode 0. No install or custom action execution.
$ErrorActionPreference='Stop'
if(!$IsWindows){throw 'Windows MSI database reader required'}
if(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'manifest.json')){throw 'Frozen review'}
if(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'msi-tables.json')){throw 'Keep the original read'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native product exists'}
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskOld=@(Get-ChildItem -LiteralPath (Join-Path $taskCode 'test/software/candidate-43142bd-38709701621/installers') -Recurse -File -Filter '*.msi')
if($taskOld.Count -ne 1){throw 'Old verified MSI is missing or ambiguous'}
$taskNew=Get-Content -LiteralPath (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261016-WIN-823-CANDIDATE/msi-readonly.original.json') -Raw | ConvertFrom-Json
$taskPackages=@(
    @{label='431';source='43142bdbcc7a0b9f1325103febae3f44dce407ab';path=$taskOld[0].FullName;sha256='2dc08092319fb91fc2866d010de850ca23bcc7716232ef2ac212b94c4d118dbe'},
    @{label='823';source='823a8a0615153068594daef530c2e80f1d95fc29';path=$taskNew.package;sha256=$taskNew.packageSha256}
)
$taskPlan=[ordered]@{
    Property=@('Property','Value')
    File=@('File','Component_','FileName','FileSize','Version','Language','Attributes','Sequence')
    Directory=@('Directory','Directory_Parent','DefaultDir')
    Component=@('Component','ComponentId','Directory_','Attributes','Condition','KeyPath')
    RemoveFile=@('FileKey','Component_','FileName','DirProperty','InstallMode')
    CustomAction=@('Action','Type','Source','Target')
    InstallExecuteSequence=@('Action','Condition','Sequence')
    Upgrade=@('UpgradeCode','VersionMin','VersionMax','Language','Attributes','Remove','ActionProperty')
    Registry=@('Registry','Root','Key','Name','Value','Component_')
    RemoveRegistry=@('RemoveRegistry','Root','Key','Name','Component_')
    CreateFolder=@('Directory_','Component_')
    ServiceInstall=@('ServiceInstall','Name','Component_')
    ServiceControl=@('ServiceControl','Name','Component_')
}
$taskInstaller=New-Object -ComObject WindowsInstaller.Installer
$taskDatabase=$null
function Read-Rows([string]$Sql,[string[]]$Columns){
    $taskView=$taskDatabase.OpenView($Sql)
    $taskCount=0
    try{
        [void]$taskView.Execute()
        while($taskRecord=$taskView.Fetch()){
            try{
                $taskCount++
                if($taskCount -gt 1024){throw 'Bounded MSI row count exceeded'}
                $taskRow=[ordered]@{}
                for($taskIndex=0;$taskIndex -lt $Columns.Count;$taskIndex++){$taskRow[$Columns[$taskIndex]]=$taskRecord.StringData($taskIndex+1)}
                [pscustomobject]$taskRow
            }finally{[void][Runtime.InteropServices.Marshal]::ReleaseComObject($taskRecord)}
        }
    }finally{[void]$taskView.Close();[void][Runtime.InteropServices.Marshal]::ReleaseComObject($taskView)}
}
$taskResults=@()
try{
    foreach($taskPackage in $taskPackages){
        $taskAllowed=Join-Path $taskCode 'test/software'
        $taskResolved=(Resolve-Path -LiteralPath $taskPackage.path).Path
        if(!$taskResolved.StartsWith($taskAllowed+'\',[StringComparison]::OrdinalIgnoreCase) -or
           (Get-Item -LiteralPath $taskResolved).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Package scope or alias refused'}
        $taskHash=(Get-FileHash -LiteralPath $taskResolved).Hash.ToLowerInvariant()
        if($taskHash -cne $taskPackage.sha256){throw 'Exact MSI bytes differ'}
        try{
            $taskDatabase=$taskInstaller.OpenDatabase($taskResolved,0)
            $taskTables=@(Read-Rows 'SELECT `Name` FROM `_Tables`' @('Name') | ForEach-Object {$_.Name})
            $taskData=[ordered]@{}
            foreach($taskTable in $taskPlan.Keys){
                if($taskTable -notin $taskTables){$taskData[$taskTable]=@();continue}
                $taskColumns=$taskPlan[$taskTable]
                $taskSelect=($taskColumns | ForEach-Object {'`'+$_+'`'}) -join ','
                $taskData[$taskTable]=@(Read-Rows ('SELECT '+$taskSelect+' FROM `'+$taskTable+'`') $taskColumns)
            }
            $taskResults+=@{label=$taskPackage.label;source=$taskPackage.source;package=$taskResolved
                packageSha256=$taskHash;databaseMode='read-only';tables=$taskData}
        }finally{if($taskDatabase){[void][Runtime.InteropServices.Marshal]::ReleaseComObject($taskDatabase);$taskDatabase=$null}}
    }
}finally{[void][Runtime.InteropServices.Marshal]::ReleaseComObject($taskInstaller)}
[ordered]@{collectedAtUtc=[datetimeoffset]::UtcNow.ToString('o');packages=$taskResults;installerExecuted=$false
    productStarted=$false;registryWritten=$false;hostInstallerGatePassed=$false} |
    ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'msi-tables.json') -Encoding utf8
foreach($taskResult in $taskResults){
    Write-Output ($taskResult.label+': '+$taskResult.packageSha256+'; RemoveFile='+$taskResult.tables.RemoveFile.Count+'; CustomAction='+$taskResult.tables.CustomAction.Count)
}

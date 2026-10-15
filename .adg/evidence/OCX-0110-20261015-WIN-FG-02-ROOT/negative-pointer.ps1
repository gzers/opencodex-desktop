$ErrorActionPreference='Stop'
$taskShell=[Diagnostics.Process]::GetCurrentProcess().MainModule.FileName
$taskRegistry='HKCU:\Software\com.gzers.opencodex.desktop'
$taskOriginal=(Get-ItemProperty -LiteralPath $taskRegistry).StartupDataRootV1
$taskRoot='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\foreground-02-current-20261015'
if(!$taskOriginal -or ($taskOriginal | ConvertFrom-Json).anchor -notlike '*foreground-02-current-20261015\install-a\data'){throw 'Unexpected test startup pointer'}
$taskResults=@()
try{
    foreach($taskCase in @('malformed','future')){
        if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native product exists before negative case'}
        $taskPhase='negative-'+$taskCase
        $taskValue=if($taskCase -eq 'malformed'){'{broken-test-pointer'}else{
            @{schema=2;anchor=(Join-Path $taskRoot 'install-a/data')} | ConvertTo-Json -Compress
        }
        Set-ItemProperty -LiteralPath $taskRegistry -Name StartupDataRootV1 -Value $taskValue
        Start-Process -FilePath $taskShell -WindowStyle Hidden -ArgumentList @(
            '-NoLogo','-NoProfile','-NonInteractive','-ExecutionPolicy','RemoteSigned',
            '-File',(Join-Path $PSScriptRoot 'launch.ps1'),'-Mode','ordinary','-Phase',$taskPhase,'-Location','install-b'
        ) -RedirectStandardOutput (Join-Path $PSScriptRoot ($taskPhase+'-controller.stdout.txt')) -RedirectStandardError (Join-Path $PSScriptRoot ($taskPhase+'-controller.stderr.txt'))
        Start-Sleep -Seconds 2
        & $taskShell -NoLogo -NoProfile -NonInteractive -ExecutionPolicy RemoteSigned -File (Join-Path $PSScriptRoot 'native.ps1') -Launch $taskPhase -Phase ($taskPhase+'-dialog') *> (Join-Path $PSScriptRoot ($taskPhase+'-inspection.log'))
        if($LASTEXITCODE -ne 0){throw 'Negative pointer dialog not observed'}
        $taskView=Get-Content -LiteralPath (Join-Path $PSScriptRoot ($taskPhase+'-dialog.json')) -Raw | ConvertFrom-Json
        if(!@($taskView.ax | Where-Object {$_.name -like '*未自动改用其他目录*'}).Count){throw 'Expected safe-stop explanation missing'}
        & $taskShell -NoLogo -NoProfile -NonInteractive -ExecutionPolicy RemoteSigned -File (Join-Path $PSScriptRoot 'native.ps1') -Launch $taskPhase -Phase ($taskPhase+'-close') -Action close *> (Join-Path $PSScriptRoot ($taskPhase+'-close.log'))
        Start-Sleep -Seconds 1
        $taskExit=Get-Content -LiteralPath (Join-Path $PSScriptRoot ($taskPhase+'-exit.json')) -Raw | ConvertFrom-Json
        if(!$taskExit.exitObserved -or $taskExit.exitCode -ne 1){throw 'Negative startup did not exit with the documented error'}
        $taskPointerUnchanged=(Get-ItemProperty -LiteralPath $taskRegistry).StartupDataRootV1 -ceq $taskValue
        $taskCount=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count
        $taskRoaming=Test-Path -LiteralPath (Join-Path $env:APPDATA 'com.gzers.opencodex.desktop')
        $taskNewData=Test-Path -LiteralPath (Join-Path $taskRoot 'install-b/data')
        if(!$taskPointerUnchanged -or $taskCount -ne 0 -or $taskRoaming -or $taskNewData){throw 'Negative startup silently changed root state'}
        $taskResults+=@{case=$taskCase;exitCode=$taskExit.exitCode;nativeCount=$taskCount;pointerUnchanged=$taskPointerUnchanged;roamingCreated=$taskRoaming;secondInstallDataCreated=$taskNewData}
    }
}finally{
    Set-ItemProperty -LiteralPath $taskRegistry -Name StartupDataRootV1 -Value $taskOriginal
    $taskRestored=(Get-ItemProperty -LiteralPath $taskRegistry).StartupDataRootV1 -ceq $taskOriginal
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');cases=$taskResults;originalTestPointerRestored=$taskRestored;newAccountCreated=$false;fullVersionGatePassed=$false} |
        ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'negative-pointer-results.json') -Encoding utf8
    if(!$taskRestored){throw 'Original test pointer restoration failed'}
}

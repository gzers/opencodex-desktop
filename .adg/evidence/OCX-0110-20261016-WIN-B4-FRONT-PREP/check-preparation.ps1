param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
$taskAllowed='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop\test\runtime'
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
if(!$taskOutput.StartsWith($taskAllowed+'\',[StringComparison]::OrdinalIgnoreCase) -or
   !(Test-Path -LiteralPath $taskOutput -PathType Container)){throw 'Existing owned guard-test directory required'}
if(Test-Path -LiteralPath (Join-Path $taskOutput 'human-window.json')){throw 'Guard test must not have any human execution receipt'}
if(Test-Path -LiteralPath (Join-Path $taskOutput 'guard-checks.json')){throw 'Do not overwrite guard evidence'}
$taskContract=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'contract.json') -Raw | ConvertFrom-Json
. $taskContract.directoryIdentityMethod
function Read-PreparationProtection {
    if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Require native0'}
    $taskPointer=(Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop' -Name StartupDataRootV1).StartupDataRootV1
    $taskPaths=@($taskContract.fixtureRoot,$taskContract.baselineFixtureRoot)
    [ordered]@{
        profiles=@($taskPaths | ForEach-Object {
            $taskId=Get-RecoveryDirectoryIdentity -Path $_
            [ordered]@{path=$_;fileId=$taskId.fileId;creationFileTime=$taskId.creationFileTime
                preferencesSha256=(Get-FileHash -LiteralPath (Join-Path $_ 'manager-state/preferences.json')).Hash.ToLowerInvariant()}
        })
        startupPointerSha256=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($taskPointer))).ToLowerInvariant()
        taskSha256=(Get-FileHash -LiteralPath 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\work\objects\task\TASK-OPENCODEX-DESKTOP-182.json').Hash.ToLowerInvariant()
        nativeCount=0
    }
}
$taskBefore=Read-PreparationProtection
$taskBefore | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $taskOutput 'before-protection.json') -Encoding utf8
$taskCases=@(
    @{name='prepare';method='prepare-fixture.ps1';arguments=@()},
    @{name='launch';method='launch.ps1';arguments=@('-Version','baseline019')},
    @{name='observe';method='observe.ps1';arguments=@('-Version','baseline019')},
    @{name='roles';method='capture-roles.ps1';arguments=@('-Version','baseline019','-Moment','settled')}
)
$taskResults=@()
foreach($taskCase in $taskCases){
    $taskLog=Join-Path $taskOutput ($taskCase.name+'-missing-human.original.log')
    if(Test-Path -LiteralPath $taskLog){throw 'Refusal attempt already exists'}
    $taskArguments=@('-NoProfile','-File',(Join-Path $PSScriptRoot $taskCase.method),'-RunDirectory',$taskOutput)+$taskCase.arguments
    & 'C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\native\powershell\pwsh.exe' @taskArguments *> $taskLog
    $taskExit=$LASTEXITCODE
    $taskText=Get-Content -LiteralPath $taskLog -Raw
    if($taskExit -eq 0 -or !$taskText.Contains('human-window.json')){throw 'Execution did not refuse specifically for the absent human receipt'}
    $taskResults+=@{entry=$taskCase.method;exitCode=$taskExit;missingHumanReceiptRefused=$true
        log=(Split-Path -Leaf $taskLog);logSha256=(Get-FileHash -LiteralPath $taskLog).Hash.ToLowerInvariant()}
    $null=Read-PreparationProtection
}
$taskAfter=Read-PreparationProtection
$taskAfter | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $taskOutput 'after-protection.json') -Encoding utf8
if(($taskBefore | ConvertTo-Json -Depth 7 -Compress) -cne ($taskAfter | ConvertTo-Json -Depth 7 -Compress)){throw 'Configuration or TASK changed during refusal checks'}
$taskUnexpected=@(Get-ChildItem -LiteralPath $taskOutput -Force |
    Where-Object {$_.Name -match 'fixture-ready|idle-.*launch|roles-attempt|human-window'})
if($taskUnexpected.Count){throw 'A refusal created an execution artifact'}
[ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');cases=$taskResults
    beforeAfterProtectionEqual=$true;humanReceiptCreated=$false;productStarted=$false
    configurationMoved=$false;registryOrAclChanged=$false;nativeCount=0
    fullNativeGatePassed=$false} | ConvertTo-Json -Depth 7 |
    Set-Content -LiteralPath (Join-Path $taskOutput 'guard-checks.json') -Encoding utf8
Get-Content -LiteralPath (Join-Path $taskOutput 'guard-checks.json')

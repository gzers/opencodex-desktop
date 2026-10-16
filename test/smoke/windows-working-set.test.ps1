param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'windows-working-set.ps1')
$taskChecks=0
function Require-PageCheck([bool]$Condition,[string]$Message){
    if(!$Condition){throw "Working-set metadata regression: $Message"}
    $script:taskChecks++
}
Initialize-WindowsWorkingSetReader
Require-PageCheck ([OpenCodexWorkingSetV1]::MbiSize -eq 48) 'MEMORY_BASIC_INFORMATION64 ABI'
Require-PageCheck ([OpenCodexWorkingSetV1]::CounterSize -eq 80) 'PROCESS_MEMORY_COUNTERS_EX ABI'
Require-PageCheck ([OpenCodexWorkingSetV1]::PageAddress([uint64]0x12345FFF) -eq [uint64]0x12345000) 'page address bits'
Require-PageCheck (![OpenCodexWorkingSetV1]::ExValid(0)) 'invalid bit'
Require-PageCheck ([OpenCodexWorkingSetV1]::ExValid(1)) 'valid bit'
Require-PageCheck (![OpenCodexWorkingSetV1]::ExShareable(1)) 'private flag'
Require-PageCheck ([OpenCodexWorkingSetV1]::ExShareable(32769)) 'shareable flag'
$taskRejected=$false
try{[void][OpenCodexWorkingSetV1]::ExShareable(32768)}catch{$taskRejected=$true}
Require-PageCheck $taskRejected 'invalid attributes cannot be classified as shared'

$taskOwn=@([OpenCodexWorkingSetV1]::TestOwnPages())
Require-PageCheck ($taskOwn.Count -eq 3) 'controlled own-page probes'
foreach($taskExpected in @(
    @{kind='controlled-private';type=0x20000},
    @{kind='controlled-mapped';type=0x40000},
    @{kind='controlled-image';type=0x1000000}
)){
    $taskRow=@($taskOwn | Where-Object Kind -CEQ $taskExpected.kind)
    Require-PageCheck ($taskRow.Count -eq 1) ('unique '+$taskExpected.kind)
    Require-PageCheck ($taskRow[0].RegionType -eq $taskExpected.type) ('region '+$taskExpected.kind)
    Require-PageCheck $taskRow[0].Valid ('resident '+$taskExpected.kind)
}
$taskSelf=[Diagnostics.Process]::GetCurrentProcess()
try{
    [void]$taskSelf.SafeHandle
    $taskLaunch=[pscustomobject]@{
        pid=$taskSelf.Id;creationTimeUtc=$taskSelf.StartTime.ToUniversalTime().ToString('o')
        executable=$taskSelf.MainModule.FileName
        exeSha256=(Get-FileHash -LiteralPath $taskSelf.MainModule.FileName).Hash.ToLowerInvariant()
    }
    $taskRead=Get-WindowsBoundWorkingSetSnapshot -Launch $taskLaunch -ControlledSelf
    Require-PageCheck $taskRead.controlledSelf 'self mode recorded'
    Require-PageCheck $taskRead.metadataOnly 'metadata boundary'
    Require-PageCheck $taskRead.snapshot.NonAtomic 'non-atomic boundary'
    Require-PageCheck (!$taskRead.snapshot.TargetMemoryContentsRead -and !$taskRead.snapshot.TargetMemoryChanged) 'target content untouched'
    Require-PageCheck (!$taskRead.snapshot.RawAddressesSaved -and !$taskRead.snapshot.SharedPagesDeduplicated) 'no raw addresses/dedup claim'
    Require-PageCheck ($taskRead.snapshot.ListedResidentPages -gt 0) 'actual resident enumeration'
    $taskBuckets=@($taskRead.snapshot.Buckets)
    Require-PageCheck ((($taskBuckets | Measure-Object Pages -Sum).Sum) -eq $taskRead.snapshot.ListedResidentPages) 'all pages accounted'
    Require-PageCheck (($taskRead.snapshot.ValidAtRecheckPages+$taskRead.snapshot.InvalidAtRecheckPages) -eq $taskRead.snapshot.ListedResidentPages) 'validity accounting'
    Require-PageCheck ([uint64]$taskRead.snapshot.ListedResidentBytes -eq $taskRead.snapshot.ListedResidentPages*4096) 'byte accounting'
    foreach($taskInvalid in @(
        @{created=([datetimeoffset]$taskLaunch.creationTimeUtc).UtcDateTime.ToFileTimeUtc()+1;image=$taskLaunch.executable},
        @{created=([datetimeoffset]$taskLaunch.creationTimeUtc).UtcDateTime.ToFileTimeUtc();image='C:\does-not-match.exe'}
    )){
        $taskRejected=$false
        try{[void][OpenCodexWorkingSetV1]::Read($taskSelf.Id,$taskInvalid.created,$taskInvalid.image,1000)}
        catch{$taskRejected=$true}
        Require-PageCheck $taskRejected 'wrong generation/image refused'
    }
    foreach($taskInvalid in @(
        [pscustomobject]@{pid=($PID+1);creationTimeUtc=$taskLaunch.creationTimeUtc;executable=$taskLaunch.executable;exeSha256=$taskLaunch.exeSha256},
        [pscustomobject]@{pid=$PID;creationTimeUtc=$taskLaunch.creationTimeUtc;executable=$taskLaunch.executable;exeSha256=('0'*64)}
    )){
        $taskRejected=$false
        try{Get-WindowsBoundWorkingSetSnapshot -Launch $taskInvalid -ControlledSelf | Out-Null}
        catch{$taskRejected=$true}
        Require-PageCheck $taskRejected 'controlled target/image guard'
    }
    $taskRejected=$false
    try{Get-WindowsBoundWorkingSetSnapshot -Launch $taskLaunch | Out-Null}
    catch{$taskRejected=$true}
    Require-PageCheck $taskRejected 'desktop mode requires root/GPU'
    Require-PageCheck (!$taskRead.newProductStarted -and !$taskRead.uiInputInjected -and !$taskRead.fullPerformanceGatePassed) 'no product/acceptance side effect'
    Require-PageCheck ($taskRead.sourceMethodSha256 -ceq (Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'windows-working-set.ps1')).Hash.ToLowerInvariant()) 'reader method source digest'
    if(Test-Path -LiteralPath $OutputDirectory){throw 'Use a fresh evidence directory'}
    New-Item -ItemType Directory -Path $OutputDirectory | Out-Null
    $taskRead | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $OutputDirectory 'controlled-self.original.json') -Encoding utf8
    $taskOwn | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $OutputDirectory 'controlled-pages.original.json') -Encoding utf8
    [ordered]@{checks=$taskChecks;passed=$true;ownControlledPagesOnly=$true
        nativeProductStarted=$false;externalTargetMemoryReadOrWritten=$false;uiInputInjected=$false
        controlledHarnessPagesAllocatedAndTouched=$true
        pageProbeOwnAllocationsReleased=$true;productSnapshotTaken=$false
        fullPerformanceGatePassed=$false} | ConvertTo-Json |
        Set-Content -LiteralPath (Join-Path $OutputDirectory 'result.json') -Encoding utf8
    Get-Content -LiteralPath (Join-Path $OutputDirectory 'result.json')
}finally{$taskSelf.Dispose()}

$ErrorActionPreference='Stop'
$taskTestCreated=[datetime]'2026-11-05T00:00:00Z'
function New-TestProcess([int]$Number,[datetime]$Created=$taskTestCreated){
    [pscustomobject]@{ProcessId=$Number;ParentProcessId=1;CreationDate=$Created;SessionId=2;ExecutablePath='D:\post021-fixture\ChatGPT.exe'}
}
function Get-CimInstance {
    param($ClassName,$Filter)
    if($Filter -eq ('ProcessId='+$PID)){return New-TestProcess $PID}
    if($Filter -eq 'ProcessId=7'){return $null}
    if($Filter -eq 'ProcessId=8'){return New-TestProcess 8}
    if($Filter -eq 'ProcessId=9'){return New-TestProcess 9 ($taskTestCreated.AddSeconds(1))}
    return @(7,8,9,10,11|ForEach-Object {New-TestProcess $_})
}
function Invoke-CimMethod {
    param($InputObject,$MethodName)
    if($InputObject.ProcessId -in 7,8,9){throw 'simulated CIM process vanished/owner unavailable'}
    $taskCurrent=[Security.Principal.WindowsIdentity]::GetCurrent().Name.Split('\')
    [pscustomobject]@{ReturnValue=0;Domain=$taskCurrent[0];User=if($InputObject.ProcessId -eq 10){$taskCurrent[-1]}else{'OtherTestOwner'}}
}
$taskMe=[Security.Principal.WindowsIdentity]::GetCurrent().Name.Split('\')[-1]
$taskResult=(& (Join-Path $PSScriptRoot 'formal-msix-snapshot.ps1') -ExpectedUser $taskMe)|ConvertFrom-Json
if($taskResult.enumerationRaces.Count -ne 2 -or $taskResult.unknownOwnerCount -ne 1 -or $taskResult.own.Count -ne 1 -or $taskResult.other.Count -ne 1){throw 'Snapshot failed to distinguish gone/reused from still-live unknown owner'}
if($taskResult.enumerationRaces[0].state -ne 'gone-before-owner-read' -or $taskResult.enumerationRaces[1].state -ne 'generation-changed'){throw 'Snapshot race states lost'}
Write-Output 'Snapshot race test passed: gone and changed generations recorded; still-live unknown remains refusal; own/other preserved'

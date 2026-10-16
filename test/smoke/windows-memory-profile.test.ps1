param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
if(!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7){throw 'Windows PowerShell 7 required'}
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $taskOutput){throw 'New output directory required'}
New-Item -ItemType Directory -Path $taskOutput | Out-Null
. (Join-Path $PSScriptRoot 'windows-memory-profile.ps1')
$taskWpr=Join-Path $env:SystemRoot 'System32/wpr.exe'
$taskChecks=0
function Assert-ProfileTest($Condition,[string]$Message){
    if(!$Condition){throw $Message}
    $script:taskChecks++
}
function Assert-ProfileRefusal([scriptblock]$Action,[string]$Message){
    $taskRefused=$false
    try{& $Action | Out-Null}catch{$taskRefused=$true}
    Assert-ProfileTest $taskRefused $Message
}
function Invoke-ReadOnlyWpr([string[]]$Arguments,[string]$Name){
    if($Arguments[0] -cnotin @('-exportprofile','-profiles','-profiledetails')){throw 'Read-only WPR preparation commands only'}
    $taskLines=@(& $taskWpr @Arguments 2>&1)
    $taskExit=$LASTEXITCODE
    [IO.File]::WriteAllText((Join-Path $taskOutput ($Name+'.decoded.log')),($taskLines -join "`r`n")+"`r`n",[Text.UTF8Encoding]::new($false))
    Assert-ProfileTest ($taskExit -eq 0) "WPR preparation command failed: $Name"
    return ($taskLines -join "`n")
}
$taskExport=Join-Path $taskOutput 'host-gpu-va-export.original.wprp'
[void](Invoke-ReadOnlyWpr @('-exportprofile','VirtualAllocation+GPU',$taskExport,'-filemode') 'export')
$taskRaw=[IO.File]::ReadAllText($taskExport)
$taskProfile=ConvertTo-WindowsGpuAllocationProfile -ExportedProfileXml $taskRaw
Assert-ProfileTest ($taskProfile.configuredBufferBytes -eq 33554432) 'Explicit 32MiB collector-buffer configuration required'
Assert-ProfileTest (!$taskProfile.executionAdmitted -and !$taskProfile.fileGrowthBoundEnforced -and !$taskProfile.eventLossVerified) 'Preparation cannot admit recording'
$taskPath=Join-Path $taskOutput 'ocx-gpu-allocation.wprp'
[IO.File]::WriteAllText($taskPath,$taskProfile.profileXml,[Text.UTF8Encoding]::new($false))
$taskInventory=Invoke-ReadOnlyWpr @('-profiles',$taskPath) 'profile-inventory'
Assert-ProfileTest ($taskInventory.Contains('OcxGpuAllocation')) 'WPR did not recognize custom profile'
$taskDetails=Invoke-ReadOnlyWpr @('-profiledetails',($taskPath+'!'+$taskProfile.profileName),'-filemode') 'profile-details'
Assert-ProfileTest ($taskDetails.Contains('VirtualAllocation') -and $taskDetails.Contains('VirtualAllocation.Verbose.File') -eq $false) 'Custom allocation stack profile required'
Assert-ProfileTest ($taskDetails.Contains('Microsoft-Windows-DxgKrnl') -and $taskDetails.Contains('Microsoft-Windows-Direct3D11') -and $taskDetails.Contains('Microsoft-Windows-DXGI')) 'WPR did not resolve all three graphics providers'
Assert-ProfileTest ([regex]::Matches($taskDetails,'Buffer Size \(KB\)\s*:\s*256').Count -eq 2) 'Actual WPR collector buffer sizes differ'
Assert-ProfileTest ([regex]::Matches($taskDetails,'Number of Buffers\s*:\s*64').Count -eq 2) 'Actual WPR collector buffer counts differ'
Assert-ProfileRefusal {ConvertTo-WindowsGpuAllocationProfile ($taskRaw.Replace('<Stack Value="VirtualAllocation" />',''))} 'Missing allocation stack must be rejected'
Assert-ProfileRefusal {ConvertTo-WindowsGpuAllocationProfile ($taskRaw.Replace('<Keyword Value="ProcessThread" />',''))} 'Missing process generation events must be rejected'
Assert-ProfileRefusal {ConvertTo-WindowsGpuAllocationProfile ($taskRaw.Replace('ca11c036-0102-4a2d-a6ad-f03cfed5d3c9','11111111-1111-1111-1111-111111111111'))} 'Missing D3D provider must be rejected'
Assert-ProfileRefusal {ConvertTo-WindowsGpuAllocationProfile ($taskRaw.Replace('0x0000000000000277','0x0000000000000000'))} 'Changed GPU event keywords must be rejected'
Assert-ProfileRefusal {ConvertTo-WindowsGpuAllocationProfile '<!DOCTYPE test [<!ENTITY x SYSTEM "file:///not-read">]><WindowsPerformanceRecorder Version="1.0">&x;</WindowsPerformanceRecorder>'} 'External XML entity must be rejected'
Assert-ProfileRefusal {ConvertTo-WindowsGpuAllocationProfile ('x'*262145)} 'Oversized export must be rejected'
$taskXml=[xml]$taskProfile.profileXml
Assert-ProfileTest ($taskXml.WindowsPerformanceRecorder.Profiles.Profile.Count -eq 1 -and $taskXml.WindowsPerformanceRecorder.Profiles.EventProvider.Count -eq 3) 'One profile and exactly three graphics providers required'
Assert-ProfileTest (!$taskProfile.traceStarted -and !$taskProfile.productStarted -and !$taskProfile.formalRssMetricReplaced) 'Preparation must not become product acceptance'
$taskResult=[ordered]@{checks=$taskChecks;passed=$true;readOnlyWprParserValidated=$true
    configuredBufferBytes=$taskProfile.configuredBufferBytes
    exportSha256=(Get-FileHash -LiteralPath $taskExport).Hash.ToLowerInvariant()
    generatedProfileSha256=(Get-FileHash -LiteralPath $taskPath).Hash.ToLowerInvariant()
    sourceMethodSha256=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'windows-memory-profile.ps1')).Hash.ToLowerInvariant()
    outputTextEncoding='UTF-8 transcripts decoded by PowerShell; not raw stdout bytes'
    recordingStarted=$false;eventLossVerified=$false;fileGrowthBoundEnforced=$false
    executionAdmitted=$false;desktopProductStarted=$false;queryReplayed=$false
    fullPerformanceGatePassed=$false}
$taskResult | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $taskOutput 'result.json') -Encoding utf8
$taskResult | ConvertTo-Json -Depth 5

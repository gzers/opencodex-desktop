[CmdletBinding(DefaultParameterSetName='Prepare')]
param(
    [Parameter(Mandatory)][string]$RootLaunchJson,
    [Parameter(Mandatory)][string]$GpuLaunchJson,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [ValidateSet('ordinary','administrator')][string]$Mode='ordinary',
    [ValidateRange(1,45)][int]$Seconds=30,
    [Parameter(Mandatory,ParameterSetName='Record')][string]$DecoderDirectory,
    [Parameter(Mandatory,ParameterSetName='Record')][string]$MethodAdmissionPath,
    [Parameter(Mandatory,ParameterSetName='Record')][switch]$RecordApproved
)
$ErrorActionPreference='Stop'
if(!$IsWindows -or [IntPtr]::Size -ne 8 -or $PSVersionTable.PSVersion.Major -ne 7){throw 'Windows x64 PowerShell 7 required'}
if($RecordApproved -and
   !([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){
    throw 'Bound GPU recording requires an elevated controller; no self-elevation performed'
}
. (Join-Path $PSScriptRoot 'windows-memory-target.ps1')
if($RecordApproved){
    [void](Assert-WindowsMemoryMethodAdmission -Path $MethodAdmissionPath)
    . (Join-Path $PSScriptRoot 'windows-memory-recorder-loader.ps1')
    Initialize-WindowsMemoryRecorder -DecoderDirectory $DecoderDirectory
}
$taskRootLaunch=Get-Content -LiteralPath $RootLaunchJson -Raw | ConvertFrom-Json
$taskGpuLaunch=Get-Content -LiteralPath $GpuLaunchJson -Raw | ConvertFrom-Json
$taskBinding=$null
try{
    $taskBinding=Open-WindowsMemoryTargetBinding -Launch $taskGpuLaunch -RootLaunch $taskRootLaunch -Mode $Mode
    Invoke-WindowsGpuMemoryRecording -LeaseId $taskBinding.leaseId -OutputDirectory $OutputDirectory `
        -Seconds $Seconds -MethodAdmissionPath $MethodAdmissionPath -RecordApproved:$RecordApproved |
        ConvertTo-Json -Depth 12
}finally{
    if($taskBinding){Close-WindowsMemoryTargetBinding -LeaseId $taskBinding.leaseId}
}

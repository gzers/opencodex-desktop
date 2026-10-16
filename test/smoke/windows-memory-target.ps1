# Retained metadata leases for a future allocation recorder. No process launch,
# ETW session, target-memory read/write, or configuration mutation here.
if(!$script:OcxMemoryTargetLeases){
    $script:OcxMemoryTargetLeases=[Collections.Generic.Dictionary[string,object]]::new()
}

function Initialize-WindowsMemoryTargetOwner {
    if(!$IsWindows -or [IntPtr]::Size -ne 8){throw 'Windows x64 required'}
    if('OpenCodexMemoryTokenOwnerV1' -as [type]){return}
    Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Security.Principal;
using System.Text;
using Microsoft.Win32.SafeHandles;
public static class OpenCodexMemoryTokenOwnerV1 {
    [DllImport("advapi32.dll",SetLastError=true)]
    static extern bool OpenProcessToken(SafeProcessHandle process,uint access,out SafeAccessTokenHandle token);
    [DllImport("advapi32.dll",SetLastError=true)]
    static extern bool GetTokenInformation(SafeAccessTokenHandle token,int kind,IntPtr data,int size,out int needed);
    public static string Read(Process process) {
        // Borrow the Process-owned handle; only the enclosing lease disposes it.
        var handle=process.SafeHandle;
        {
            SafeAccessTokenHandle token;
            if(!OpenProcessToken(handle,8,out token))throw new Win32Exception(Marshal.GetLastWin32Error());
            using(token) {
                int size;
                bool unexpected=GetTokenInformation(token,1,IntPtr.Zero,0,out size);
                if(unexpected||Marshal.GetLastWin32Error()!=122||size< IntPtr.Size||size>16384)
                    throw new InvalidOperationException("Bounded TokenUser metadata required");
                IntPtr buffer=Marshal.AllocHGlobal(size);
                try {
                    if(!GetTokenInformation(token,1,buffer,size,out size))throw new Win32Exception(Marshal.GetLastWin32Error());
                    var sid=new SecurityIdentifier(Marshal.ReadIntPtr(buffer));
                    return Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(sid.Value))).ToLowerInvariant();
                }finally{Marshal.FreeHGlobal(buffer);}
            }
        }
    }
}
'@
}

function Open-OcxMemoryProcess {
    param($Launch,[string]$ExecutableLeaf)
    if($null -eq $Launch -or [int]$Launch.pid -le 0 -or
       ![IO.Path]::IsPathFullyQualified([string]$Launch.executable) -or
       [IO.Path]::GetFileName([string]$Launch.executable) -ine $ExecutableLeaf -or
       $Launch.exeSha256 -cnotmatch '^[0-9a-f]{64}$' -or !$Launch.creationTimeUtc){
        throw 'Explicit process generation, image and digest required'
    }
    if((Get-FileHash -LiteralPath $Launch.executable).Hash.ToLowerInvariant() -cne $Launch.exeSha256){throw 'Process image digest differs'}
    $taskProcess=$null
    try{
        $taskProcess=[Diagnostics.Process]::GetProcessById([int]$Launch.pid)
        [void]$taskProcess.SafeHandle
        if($taskProcess.HasExited -or
           $taskProcess.StartTime.ToUniversalTime().Ticks -ne ([datetimeoffset]$Launch.creationTimeUtc).UtcTicks -or
           !$taskProcess.MainModule.FileName.Equals([string]$Launch.executable,[StringComparison]::OrdinalIgnoreCase)){
            throw 'Retained process generation or image differs'
        }
        [pscustomobject]@{process=$taskProcess;launch=$Launch;ownerSidSha256=[OpenCodexMemoryTokenOwnerV1]::Read($taskProcess)}
    }catch{if($taskProcess){$taskProcess.Dispose()};throw}
}

function Open-WindowsMemoryTargetBinding {
    [CmdletBinding(DefaultParameterSetName='Gpu')]
    param(
        [Parameter(Mandatory)]$Launch,
        [Parameter(Mandatory,ParameterSetName='Gpu')]$RootLaunch,
        [Parameter(ParameterSetName='Gpu')][ValidateSet('ordinary','administrator')][string]$Mode='ordinary',
        [Parameter(Mandatory,ParameterSetName='Self')][switch]$ControlledSelf
    )
    $ErrorActionPreference='Stop'
    Initialize-WindowsMemoryTargetOwner
    . (Join-Path $PSScriptRoot 'windows-process-token.ps1')
    $taskItems=[Collections.Generic.List[object]]::new()
    try{
        $taskNative=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'")
        if($ControlledSelf){
            if([int]$Launch.pid -ne $PID -or $taskNative.Count){throw 'Controlled binding reads this harness only with native0'}
            $taskTarget=Open-OcxMemoryProcess $Launch ([IO.Path]::GetFileName((Get-Process -Id $PID).Path))
            $taskItems.Add($taskTarget)
            $taskRoot=$taskTarget
            $taskToken=Get-WindowsProcessToken -Process $taskTarget.process -ExpectedExecutable $Launch.executable
            $taskGpuRole=$false
        }else{
            if($taskNative.Count -ne 1 -or $taskNative[0].ProcessId -ne [int]$RootLaunch.pid){throw 'One exact native desktop required'}
            if([int]$Launch.pid -eq [int]$RootLaunch.pid){throw 'GPU and native root must be different generations'}
            $taskRoot=Open-OcxMemoryProcess $RootLaunch 'opencodex-desktop.exe'
            $taskItems.Add($taskRoot)
            $taskTarget=Open-OcxMemoryProcess $Launch 'msedgewebview2.exe'
            $taskItems.Add($taskTarget)
            if($RootLaunch.applicationToken.sidSha256 -cnotmatch '^[0-9a-f]{64}$' -or
               $taskRoot.ownerSidSha256 -cne $RootLaunch.applicationToken.sidSha256 -or
               $taskTarget.ownerSidSha256 -cne $taskRoot.ownerSidSha256){throw 'Actual TokenUser does not match the launch witness'}
            $taskToken=Get-WindowsProcessToken -Process $taskRoot.process -ExpectedExecutable $RootLaunch.executable
            Assert-WindowsProcessMode -Observation $taskToken -Mode $Mode
            $taskRows=@(Get-CimInstance -Query 'SELECT ProcessId,ParentProcessId,CreationDate,Name FROM Win32_Process')
            $taskCurrent=$taskTarget
            $taskSeen=[Collections.Generic.HashSet[int]]::new()
            while($taskCurrent.process.Id -ne $taskRoot.process.Id){
                if(!$taskSeen.Add($taskCurrent.process.Id) -or $taskSeen.Count -gt 16){throw 'Bound ancestry is cyclic or too deep'}
                $taskRow=@($taskRows | Where-Object ProcessId -EQ $taskCurrent.process.Id)
                if($taskRow.Count -ne 1 -or [math]::Abs($taskCurrent.process.StartTime.ToUniversalTime().Ticks-
                    ([datetime]$taskRow[0].CreationDate).ToUniversalTime().Ticks) -gt 9){throw 'Ancestry generation changed'}
                $taskParentId=[int]$taskRow[0].ParentProcessId
                if($taskParentId -eq $taskRoot.process.Id){$taskParent=$taskRoot}
                else{
                    $taskParentProcess=[Diagnostics.Process]::GetProcessById($taskParentId)
                    $taskParent=[pscustomobject]@{process=$taskParentProcess;launch=$null;ownerSidSha256=$null}
                    $taskItems.Add($taskParent)
                    [void]$taskParentProcess.SafeHandle
                    $taskParent.launch=[pscustomobject]@{
                        pid=$taskParentId;creationTimeUtc=$taskParentProcess.StartTime.ToUniversalTime().ToString('o')
                        executable=$taskParentProcess.MainModule.FileName
                        exeSha256=(Get-FileHash -LiteralPath $taskParentProcess.MainModule.FileName).Hash.ToLowerInvariant()
                    }
                    $taskParent.ownerSidSha256=[OpenCodexMemoryTokenOwnerV1]::Read($taskParentProcess)
                    if(!$taskParentProcess.MainModule.FileName.Equals([string]$Launch.executable,[StringComparison]::OrdinalIgnoreCase) -or
                       $taskParent.ownerSidSha256 -cne $taskRoot.ownerSidSha256){throw 'Ancestry leaves the bound WebView image or owner'}
                }
                if($taskParent.process.StartTime.ToUniversalTime() -gt $taskCurrent.process.StartTime.ToUniversalTime()){throw 'Parent generation is newer than child'}
                $taskCurrent=$taskParent
            }
            . (Join-Path $PSScriptRoot 'windows-gpu-resource.ps1')
            $taskCommand=Get-CimInstance -Query "SELECT CommandLine FROM Win32_Process WHERE ProcessId=$([int]$Launch.pid)"
            if((Get-WindowsWebViewRole -CommandLine $taskCommand.CommandLine) -cne 'gpu'){throw 'Bound WebView is not the actual GPU role'}
            $taskGpuRole=$true
        }
        $taskId=[guid]::NewGuid().ToString('N')
        $taskEntry=[pscustomobject]@{items=$taskItems;root=$taskRoot;target=$taskTarget;mode=$Mode
            controlledSelf=[bool]$ControlledSelf;gpuRoleVerified=$taskGpuRole;rootToken=$taskToken}
        $script:OcxMemoryTargetLeases.Add($taskId,$taskEntry)
        try{Assert-WindowsMemoryTargetBinding -LeaseId $taskId}
        catch{Close-WindowsMemoryTargetBinding -LeaseId $taskId;throw}
    }catch{
        foreach($taskItem in $taskItems){$taskItem.process.Dispose()}
        throw
    }
}

function Assert-WindowsMemoryTargetBinding {
    param([Parameter(Mandatory)][string]$LeaseId)
    $ErrorActionPreference='Stop'
    if(!$script:OcxMemoryTargetLeases.ContainsKey($LeaseId)){throw 'Unknown or closed memory-target lease'}
    $taskEntry=$script:OcxMemoryTargetLeases[$LeaseId]
    foreach($taskItem in $taskEntry.items){
        if($taskItem.process.HasExited -or
           $taskItem.process.StartTime.ToUniversalTime().Ticks -ne ([datetimeoffset]$taskItem.launch.creationTimeUtc).UtcTicks -or
           !$taskItem.process.MainModule.FileName.Equals([string]$taskItem.launch.executable,[StringComparison]::OrdinalIgnoreCase) -or
           (Get-FileHash -LiteralPath $taskItem.launch.executable).Hash.ToLowerInvariant() -cne $taskItem.launch.exeSha256 -or
           [OpenCodexMemoryTokenOwnerV1]::Read($taskItem.process) -cne $taskItem.ownerSidSha256){
            throw 'Retained memory-target identity changed'
        }
    }
    $taskNative=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'")
    if($taskEntry.controlledSelf){
        if($taskNative.Count -or $taskEntry.target.process.Id -ne $PID){throw 'Controlled native set changed'}
    }else{
        if($taskNative.Count -ne 1 -or $taskNative[0].ProcessId -ne $taskEntry.root.process.Id){throw 'Bound native set changed'}
        $taskEntry.rootToken=Get-WindowsProcessToken -Process $taskEntry.root.process -ExpectedExecutable $taskEntry.root.launch.executable
        Assert-WindowsProcessMode -Observation $taskEntry.rootToken -Mode $taskEntry.mode
        $taskCommand=Get-CimInstance -Query "SELECT CommandLine FROM Win32_Process WHERE ProcessId=$($taskEntry.target.process.Id)"
        if((Get-WindowsWebViewRole -CommandLine $taskCommand.CommandLine) -cne 'gpu'){throw 'GPU role changed'}
    }
    [pscustomobject][ordered]@{
        leaseId=$LeaseId;atUtc=[datetimeoffset]::UtcNow.ToString('o')
        rootPid=$taskEntry.root.process.Id;targetPid=$taskEntry.target.process.Id
        targetCreationUtc=$taskEntry.target.launch.creationTimeUtc;targetExeSha256=$taskEntry.target.launch.exeSha256
        ownerSidSha256=$taskEntry.target.ownerSidSha256;rootToken=$taskEntry.rootToken
        controlledSelf=$taskEntry.controlledSelf;gpuRoleVerified=$taskEntry.gpuRoleVerified
        retainedProcessCount=$taskEntry.items.Count;rawSidSaved=$false;rawCommandLineSaved=$false
        tracingStarted=$false;productStarted=$false;recordingExecutionAdmitted=$false
    }
}

function Close-WindowsMemoryTargetBinding {
    param([Parameter(Mandatory)][string]$LeaseId)
    if(!$script:OcxMemoryTargetLeases.ContainsKey($LeaseId)){throw 'Unknown or closed memory-target lease'}
    $taskEntry=$script:OcxMemoryTargetLeases[$LeaseId]
    [void]$script:OcxMemoryTargetLeases.Remove($LeaseId)
    foreach($taskItem in $taskEntry.items){$taskItem.process.Dispose()}
}

# Supported read-only working-set metadata. No target memory contents or
# addresses are serialized; use outside formal performance counter spans.
function Initialize-WindowsWorkingSetReader {
    if (!$IsWindows -or [IntPtr]::Size -ne 8 -or $PSVersionTable.PSVersion.Major -ne 7) {
        throw 'Windows x64 PowerShell 7 required'
    }
    if ('OpenCodexWorkingSetV1' -as [type]) { return }
    Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Diagnostics;
using System.IO;
using System.IO.MemoryMappedFiles;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;

public sealed class OcxPageBucket {
    public string RegionType {get;set;}
    public bool ValidAtRecheck {get;set;}
    public bool? ShareableAtRecheck {get;set;}
    public long Pages {get;set;}
    public string Bytes {get;set;}
}
public sealed class OcxWorkingSetSnapshot {
    public int ProcessId {get;set;}
    public string CreationFileTime {get;set;}
    public string StartedAtUtc {get;set;}
    public string FinishedAtUtc {get;set;}
    public int PageSize {get;set;}
    public long ListedResidentPages {get;set;}
    public string ListedResidentBytes {get;set;}
    public long ValidAtRecheckPages {get;set;}
    public long InvalidAtRecheckPages {get;set;}
    public int RegionQueries {get;set;}
    public string ProcessWorkingSetBeforeBytes {get;set;}
    public string ProcessWorkingSetAfterBytes {get;set;}
    public string ProcessPrivateCommitBeforeBytes {get;set;}
    public string ProcessPrivateCommitAfterBytes {get;set;}
    public OcxPageBucket[] Buckets {get;set;}
    public bool NonAtomic {get {return true;}}
    public bool RawAddressesSaved {get {return false;}}
    public bool TargetMemoryContentsRead {get {return false;}}
    public bool TargetMemoryChanged {get {return false;}}
    public bool SharedPagesDeduplicated {get {return false;}}
}
public sealed class OcxOwnPageProbe {
    public string Kind {get;set;}
    public uint RegionType {get;set;}
    public bool Valid {get;set;}
    public bool Shareable {get;set;}
}
public static class OpenCodexWorkingSetV1 {
    const int MaximumPages=524288;
    const int BatchPages=4096;
    const uint ReadRights=0x00100000|0x0400|0x0010;
    [StructLayout(LayoutKind.Sequential)]
    struct Mbi {
        public ulong BaseAddress,AllocationBase;
        public uint AllocationProtect,Alignment1;
        public ulong RegionSize;
        public uint State,Protect,Type,Alignment2;
    }
    [StructLayout(LayoutKind.Sequential)]
    struct Pmc {
        public uint Size,PageFaultCount;
        public UIntPtr PeakWorkingSet,WorkingSet,PeakPagedPool,PagedPool,PeakNonPagedPool,NonPagedPool;
        public UIntPtr PagefileUsage,PeakPagefileUsage,PrivateUsage;
    }
    [DllImport("kernel32.dll",SetLastError=true)]
    static extern SafeProcessHandle OpenProcess(uint access,bool inherit,int id);
    [DllImport("kernel32.dll",SetLastError=true)]
    static extern uint WaitForSingleObject(SafeProcessHandle process,uint milliseconds);
    [DllImport("kernel32.dll",SetLastError=true)]
    static extern bool GetProcessTimes(SafeProcessHandle process,out long created,out long exited,out long kernel,out long user);
    [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)]
    static extern bool QueryFullProcessImageNameW(SafeProcessHandle process,uint flags,StringBuilder text,ref uint size);
    [DllImport("psapi.dll",SetLastError=true)]
    static extern bool QueryWorkingSet(SafeProcessHandle process,IntPtr buffer,int size);
    [DllImport("psapi.dll",SetLastError=true)]
    static extern bool QueryWorkingSetEx(SafeProcessHandle process,IntPtr buffer,int size);
    [DllImport("psapi.dll",SetLastError=true)]
    static extern bool GetProcessMemoryInfo(SafeProcessHandle process,ref Pmc counters,int size);
    [DllImport("kernel32.dll",SetLastError=true)]
    static extern UIntPtr VirtualQueryEx(SafeProcessHandle process,IntPtr address,out Mbi info,UIntPtr length);
    [DllImport("kernel32.dll",SetLastError=true)]
    static extern IntPtr VirtualAlloc(IntPtr address,UIntPtr size,uint allocation,uint protection);
    [DllImport("kernel32.dll",SetLastError=true)]
    static extern bool VirtualFree(IntPtr address,UIntPtr size,uint freeType);

    public static int MbiSize {get {return Marshal.SizeOf<Mbi>();}}
    public static int CounterSize {get {return Marshal.SizeOf<Pmc>();}}
    public static bool ExValid(ulong flags) {return (flags&1)!=0;}
    public static bool ExShareable(ulong flags) {
        if(!ExValid(flags))throw new InvalidOperationException("Invalid page attributes have no accepted shared classification");
        return (flags&(1UL<<15))!=0;
    }
    public static ulong PageAddress(ulong block) {return block&~4095UL;}
    static long Creation(SafeProcessHandle handle) {
        long created,exited,kernel,user;
        if(!GetProcessTimes(handle,out created,out exited,out kernel,out user))throw new Win32Exception(Marshal.GetLastWin32Error());
        if(WaitForSingleObject(handle,0)!=258)throw new InvalidOperationException("Bound process is not alive");
        return created;
    }
    static Pmc Memory(SafeProcessHandle handle) {
        var result=new Pmc();result.Size=(uint)Marshal.SizeOf<Pmc>();
        if(!GetProcessMemoryInfo(handle,ref result,(int)result.Size))throw new Win32Exception(Marshal.GetLastWin32Error());
        return result;
    }
    static string Image(SafeProcessHandle handle) {
        var result=new StringBuilder(32768);uint size=(uint)result.Capacity;
        if(!QueryFullProcessImageNameW(handle,0,result,ref size))throw new Win32Exception(Marshal.GetLastWin32Error());
        return result.ToString();
    }
    static string RegionKind(Mbi region) {
        if(region.State!=0x1000)return "not-committed-at-region-query";
        switch(region.Type) {
            case 0x20000:return "private-region";
            case 0x40000:return "mapped-region";
            case 0x1000000:return "image-region";
            default:return "unrecognized-region";
        }
    }
    static Mbi Region(SafeProcessHandle handle,ulong address) {
        Mbi region;
        if(VirtualQueryEx(handle,new IntPtr(unchecked((long)address)),out region,(UIntPtr)MbiSize).ToUInt64()!=(ulong)MbiSize)
            throw new Win32Exception(Marshal.GetLastWin32Error());
        if(region.RegionSize==0)throw new InvalidOperationException("Empty memory region");
        return region;
    }
    static ulong[] ListResident(SafeProcessHandle handle) {
        int length=4096;
        for(int attempt=0;attempt<5;attempt++) {
            IntPtr buffer=Marshal.AllocHGlobal(length);
            try {
                Marshal.WriteInt64(buffer,0);
                bool ok=QueryWorkingSet(handle,buffer,length);
                int error=Marshal.GetLastWin32Error();
                ulong count=unchecked((ulong)Marshal.ReadInt64(buffer));
                if(count>MaximumPages)throw new InvalidOperationException("Bounded resident-page budget exceeded");
                if(ok) {
                    if(count*8+8>(ulong)length)throw new InvalidOperationException("Working-set buffer length inconsistent");
                    var result=new ulong[(int)count];
                    for(int i=0;i<result.Length;i++)result[i]=PageAddress(unchecked((ulong)Marshal.ReadInt64(buffer,8+i*8)));
                    Array.Sort(result);
                    for(int i=1;i<result.Length;i++)if(result[i]==result[i-1])throw new InvalidOperationException("Duplicate resident page");
                    return result;
                }
                if(error!=24&&error!=122)throw new Win32Exception(error);
                length=checked((int)Math.Max((long)length*2,((long)count+1024)*8+8));
                if(length>MaximumPages*8+8)throw new InvalidOperationException("Working-set buffer budget exceeded");
            } finally {Marshal.FreeHGlobal(buffer);}
        }
        throw new InvalidOperationException("Working-set enumeration did not stabilize within the bounded attempts");
    }
    public static OcxWorkingSetSnapshot Read(int processId,long expectedCreation,string expectedImage,int maximumMilliseconds) {
        if(IntPtr.Size!=8||Environment.SystemPageSize!=4096||MbiSize!=48||CounterSize!=80)
            throw new PlatformNotSupportedException("Verified Windows x64 4KiB-page ABI required");
        if(processId<=0||expectedCreation<=0||!Path.IsPathFullyQualified(expectedImage)||maximumMilliseconds<1||maximumMilliseconds>20000)
            throw new ArgumentException("Explicit bound process, image, generation and bounded read required");
        using(var handle=OpenProcess(ReadRights,false,processId)) {
            if(handle.IsInvalid)throw new Win32Exception(Marshal.GetLastWin32Error());
            if(Creation(handle)!=expectedCreation||!string.Equals(Image(handle),expectedImage,StringComparison.OrdinalIgnoreCase))
                throw new InvalidOperationException("Bound generation or image differs");
            var started=DateTimeOffset.UtcNow;var timer=Stopwatch.StartNew();
            var before=Memory(handle);var pages=ListResident(handle);
            var buckets=new Dictionary<string,OcxPageBucket>();
            long valid=0,invalid=0;int queries=0;
            Mbi cached=new Mbi();ulong cachedEnd=0;
            IntPtr batch=Marshal.AllocHGlobal(BatchPages*16);
            try {
                for(int offset=0;offset<pages.Length;offset+=BatchPages) {
                    if(timer.ElapsedMilliseconds>maximumMilliseconds)throw new TimeoutException("Bounded metadata read expired");
                    int count=Math.Min(BatchPages,pages.Length-offset);
                    for(int i=0;i<count;i++) {
                        Marshal.WriteInt64(batch,i*16,unchecked((long)pages[offset+i]));
                        Marshal.WriteInt64(batch,i*16+8,0);
                    }
                    if(!QueryWorkingSetEx(handle,batch,count*16))throw new Win32Exception(Marshal.GetLastWin32Error());
                    for(int i=0;i<count;i++) {
                        if(i%128==0&&timer.ElapsedMilliseconds>maximumMilliseconds)throw new TimeoutException("Bounded metadata read expired");
                        ulong address=pages[offset+i],flags=unchecked((ulong)Marshal.ReadInt64(batch,i*16+8));
                        bool isValid=ExValid(flags);bool? shareable=isValid?(bool?)ExShareable(flags):null;
                        if(isValid)valid++;else invalid++;
                        if(address<cached.BaseAddress||address>=cachedEnd) {
                            cached=Region(handle,address);cachedEnd=checked(cached.BaseAddress+cached.RegionSize);queries++;
                        }
                        string kind=RegionKind(cached),key=kind+":"+isValid+":"+shareable;
                        OcxPageBucket bucket;
                        if(!buckets.TryGetValue(key,out bucket)) {
                            bucket=new OcxPageBucket {RegionType=kind,ValidAtRecheck=isValid,ShareableAtRecheck=shareable};
                            buckets.Add(key,bucket);
                        }
                        bucket.Pages++;
                    }
                }
            } finally {Marshal.FreeHGlobal(batch);}
            var after=Memory(handle);
            if(Creation(handle)!=expectedCreation)throw new InvalidOperationException("Bound generation changed during read");
            if(timer.ElapsedMilliseconds>maximumMilliseconds)throw new TimeoutException("Bounded metadata read expired");
            foreach(var bucket in buckets.Values)bucket.Bytes=(bucket.Pages*4096).ToString();
            var result=new List<OcxPageBucket>(buckets.Values);
            result.Sort((a,b)=>string.CompareOrdinal(a.RegionType+":"+a.ValidAtRecheck+":"+a.ShareableAtRecheck,
                                                   b.RegionType+":"+b.ValidAtRecheck+":"+b.ShareableAtRecheck));
            return new OcxWorkingSetSnapshot {
                ProcessId=processId,CreationFileTime=expectedCreation.ToString(),
                StartedAtUtc=started.ToString("o"),FinishedAtUtc=DateTimeOffset.UtcNow.ToString("o"),PageSize=4096,
                ListedResidentPages=pages.Length,ListedResidentBytes=((long)pages.Length*4096).ToString(),
                ValidAtRecheckPages=valid,InvalidAtRecheckPages=invalid,RegionQueries=queries,
                ProcessWorkingSetBeforeBytes=before.WorkingSet.ToUInt64().ToString(),
                ProcessWorkingSetAfterBytes=after.WorkingSet.ToUInt64().ToString(),
                ProcessPrivateCommitBeforeBytes=before.PrivateUsage.ToUInt64().ToString(),
                ProcessPrivateCommitAfterBytes=after.PrivateUsage.ToUInt64().ToString(),Buckets=result.ToArray()
            };
        }
    }
    static OcxOwnPageProbe OwnPage(SafeProcessHandle handle,IntPtr address,string kind) {
        Mbi region=Region(handle,unchecked((ulong)address.ToInt64()));
        IntPtr info=Marshal.AllocHGlobal(16);
        try {
            Marshal.WriteInt64(info,address.ToInt64());Marshal.WriteInt64(info,8,0);
            if(!QueryWorkingSetEx(handle,info,16))throw new Win32Exception(Marshal.GetLastWin32Error());
            ulong flags=unchecked((ulong)Marshal.ReadInt64(info,8));
            return new OcxOwnPageProbe {Kind=kind,RegionType=region.Type,Valid=ExValid(flags),
                                       Shareable=ExValid(flags)&&ExShareable(flags)};
        }finally{Marshal.FreeHGlobal(info);}
    }
    // Own-process allocations only, for ABI/behavior tests. Never used on a target.
    public static OcxOwnPageProbe[] TestOwnPages() {
        using(var self=Process.GetCurrentProcess())
        using(var handle=OpenProcess(ReadRights,false,self.Id)) {
            if(handle.IsInvalid)throw new Win32Exception(Marshal.GetLastWin32Error());
            var result=new List<OcxOwnPageProbe>();
            IntPtr page=VirtualAlloc(IntPtr.Zero,(UIntPtr)4096,0x3000,4);
            if(page==IntPtr.Zero)throw new Win32Exception(Marshal.GetLastWin32Error());
            try {Marshal.WriteByte(page,1);result.Add(OwnPage(handle,page,"controlled-private"));}
            finally {if(!VirtualFree(page,UIntPtr.Zero,0x8000))throw new Win32Exception(Marshal.GetLastWin32Error());}
            using(var mapped=MemoryMappedFile.CreateNew(null,4096))
            using(var view=mapped.CreateViewAccessor()) {
                view.Write(0,(byte)1);
                result.Add(OwnPage(handle,view.SafeMemoryMappedViewHandle.DangerousGetHandle(),"controlled-mapped"));
            }
            var image=self.MainModule.BaseAddress;
            Marshal.ReadByte(image);
            result.Add(OwnPage(handle,image,"controlled-image"));
            return result.ToArray();
        }
    }
}
'@
}

function Get-WindowsBoundWorkingSetSnapshot {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Launch,
        $RootLaunch,
        [switch]$ControlledSelf
    )
    $ErrorActionPreference='Stop'
    Initialize-WindowsWorkingSetReader
    if($Launch.exeSha256 -notmatch '^[0-9a-f]{64}$' -or
       (Get-FileHash -LiteralPath $Launch.executable).Hash.ToLowerInvariant() -cne $Launch.exeSha256){
        throw 'Exact bound target image required'
    }
    $taskRoot=$null
    try {
        if($ControlledSelf){
            if([int]$Launch.pid -ne $PID -or $null -ne $RootLaunch){throw 'Controlled mode reads this harness only'}
        }else{
            if($null -eq $RootLaunch -or [IO.Path]::GetFileName([string]$RootLaunch.executable) -cne 'opencodex-desktop.exe' -or
               [IO.Path]::GetFileName([string]$Launch.executable) -cne 'msedgewebview2.exe'){
                throw 'Bound desktop root and GPU child required'
            }
            if((Get-FileHash -LiteralPath $RootLaunch.executable).Hash.ToLowerInvariant() -cne $RootLaunch.exeSha256){throw 'Root image changed'}
            $taskRoot=[Diagnostics.Process]::GetProcessById([int]$RootLaunch.pid)
            [void]$taskRoot.SafeHandle
            if($taskRoot.StartTime.ToUniversalTime().Ticks -ne ([datetimeoffset]$RootLaunch.creationTimeUtc).UtcTicks -or
               $taskRoot.MainModule.FileName -cne $RootLaunch.executable){throw 'Root generation differs'}
            . (Join-Path $PSScriptRoot 'windows-process-token.ps1')
            Assert-WindowsProcessMode -Observation (Get-WindowsProcessToken -Process $taskRoot -ExpectedExecutable $RootLaunch.executable) -Mode ordinary
            $taskRows=@(Get-CimInstance -Query 'SELECT ProcessId,ParentProcessId,CreationDate,Name FROM Win32_Process')
            $taskNative=@($taskRows | Where-Object Name -in @('opencodex-desktop.exe','ocxd.exe'))
            if($taskNative.Count -ne 1 -or $taskNative[0].ProcessId -ne $taskRoot.Id){throw 'One bound desktop required'}
            $taskTarget=@($taskRows | Where-Object ProcessId -eq ([int]$Launch.pid))
            if($taskTarget.Count -ne 1){throw 'Target missing'}
            $taskCurrent=$taskTarget[0]
            $taskSeen=[Collections.Generic.HashSet[int]]::new()
            while([int]$taskCurrent.ProcessId -ne $taskRoot.Id){
                if(!$taskSeen.Add([int]$taskCurrent.ProcessId)){throw 'Ancestry cycle'}
                $taskParents=@($taskRows | Where-Object ProcessId -eq ([int]$taskCurrent.ParentProcessId))
                if($taskParents.Count -ne 1 -or $taskCurrent.CreationDate -lt $taskParents[0].CreationDate){throw 'Bound descendant ancestry unavailable'}
                $taskCurrent=$taskParents[0]
            }
            . (Join-Path $PSScriptRoot 'windows-gpu-resource.ps1')
            $taskCommand=Get-CimInstance -Query "SELECT CommandLine FROM Win32_Process WHERE ProcessId=$([int]$Launch.pid)"
            if((Get-WindowsWebViewRole -CommandLine $taskCommand.CommandLine) -cne 'gpu'){throw 'Actual child is not the GPU role'}
        }
        $taskSnapshot=[OpenCodexWorkingSetV1]::Read([int]$Launch.pid,
            ([datetimeoffset]$Launch.creationTimeUtc).UtcDateTime.ToFileTimeUtc(),[string]$Launch.executable,20000)
        if($taskRoot -and $taskRoot.HasExited){throw 'Bound root exited during metadata read'}
        if($taskRoot){
            $taskAfterNative=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'")
            if($taskAfterNative.Count -ne 1 -or $taskAfterNative[0].ProcessId -ne $taskRoot.Id){throw 'Native set changed during metadata read'}
        }
        if((Get-FileHash -LiteralPath $Launch.executable).Hash.ToLowerInvariant() -cne $Launch.exeSha256){throw 'Target disk image changed'}
        [pscustomobject][ordered]@{
            sourceMethodSha256=(Get-FileHash -LiteralPath $PSCommandPath).Hash.ToLowerInvariant()
            targetPid=$Launch.pid;targetCreationUtc=$Launch.creationTimeUtc;targetExeSha256=$Launch.exeSha256
            rootPid=if($RootLaunch){$RootLaunch.pid}else{$null};controlledSelf=[bool]$ControlledSelf
            snapshot=$taskSnapshot;metadataOnly=$true;rawCommandLinesSaved=$false
            unrelatedProcessesSerialized=$false;newProductStarted=$false;uiInputInjected=$false
            formalRssMetricReplaced=$false;allocationStackAttributed=$false;fullPerformanceGatePassed=$false
        }
    }finally{if($taskRoot){$taskRoot.Dispose()}}
}

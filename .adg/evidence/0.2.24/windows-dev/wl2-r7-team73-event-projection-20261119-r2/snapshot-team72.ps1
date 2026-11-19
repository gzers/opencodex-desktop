$ErrorActionPreference = 'Stop'
$utf8 = [Text.UTF8Encoding]::new($false, $true)
$profileRoot = 'C:\Users\marenben'
$sid = 'S-1-5-21-587408064-856152790-2583328096-1003'
$scene = Join-Path $profileRoot 'ocx024-r7-isolated-owner-range-r1'
$log = Join-Path $scene 'D\before-resume-private-control-v2-events.jsonl'
$receipts = 'C:\Users\Public\ocx-r7-isolated-owner-range-r1\out01'
$publicResult = Join-Path $receipts 'TEAM72-readonly-event-snapshot-r2.json'
$priorReceipt = Join-Path $receipts 'TEAM72-readonly-event-snapshot-r1.json'
$priorReceiptSha = 'ba74770fa579a673d2a402383552c9f842825a55aff52efde051d5e5fa5dc3e4'
$priorLength = 5622
$priorPrefixSha = '4faebbaf1807aa1aeb9ef7524892d681645974172ab3d81ff129bf733aaf7819'
$priorFileKey = '0ebd4e59:00010000001a4e61'
$priorCreation = '134395302700540299'
$priorDaclSha = '2e65b8119cde90e80709f5ccff35de45a725be8e167ed03d241eee1b96044951'
$hostFiletime = '134395302283231479'
$explicitFiletime = '134395359826678405'
$guidPattern = '^[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}$'

$native = @'
using System;
using System.ComponentModel;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;

public static class Team72FileEvidence {
    [StructLayout(LayoutKind.Sequential)]
    struct Info {
        public uint attributes, creationLow, creationHigh, accessLow, accessHigh, writeLow, writeHigh;
        public uint volume, sizeHigh, sizeLow, links, indexHigh, indexLow;
    }
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    static extern SafeFileHandle CreateFile(string path, uint access, uint share, IntPtr sa, uint disposition, uint flags, IntPtr template);
    [DllImport("kernel32.dll", SetLastError=true)]
    static extern bool GetFileInformationByHandle(SafeFileHandle handle, out Info info);
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    static extern uint GetFinalPathNameByHandle(SafeFileHandle handle, StringBuilder path, uint size, uint flags);
    [DllImport("user32.dll", SetLastError=true)] static extern IntPtr GetProcessWindowStation();
    [DllImport("user32.dll", SetLastError=true)] static extern IntPtr GetThreadDesktop(uint thread);
    [DllImport("kernel32.dll")] static extern uint GetCurrentThreadId();
    [DllImport("user32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    static extern bool GetUserObjectInformation(IntPtr handle, int index, StringBuilder value, int capacity, out int needed);
    [DllImport("advapi32.dll", SetLastError=true)]
    static extern bool GetTokenInformation(IntPtr token, int kind, out int value, int capacity, out int needed);
    static Win32Exception LastError() { return new Win32Exception(Marshal.GetLastWin32Error()); }
    static ulong Combine(uint high, uint low) { return ((ulong)high << 32) | low; }
    static string ObjectName(IntPtr handle) {
        if (handle == IntPtr.Zero) throw LastError();
        StringBuilder value = new StringBuilder(256);
        int needed;
        if (!GetUserObjectInformation(handle, 2, value, value.Capacity * 2, out needed)) throw LastError();
        return value.ToString();
    }
    public static string Desktop() {
        return ObjectName(GetProcessWindowStation()) + "/" + ObjectName(GetThreadDesktop(GetCurrentThreadId()));
    }
    public static int TokenValue(IntPtr token, int kind) {
        int value, needed;
        if (!GetTokenInformation(token, kind, out value, 4, out needed)) throw LastError();
        return value;
    }
    public sealed class FileFacts {
        public string key, creationFiletime, lastWriteFiletime;
        public long length;
        public uint attributes, links;
    }
    public static FileFacts Observe(SafeFileHandle handle, string expected, bool directory) {
        Info i;
        if (!GetFileInformationByHandle(handle, out i)) throw LastError();
        if ((i.attributes & 0x400) != 0 || ((i.attributes & 0x10) != 0) != directory || (!directory && i.links != 1))
            throw new InvalidDataException("Unexpected reparse point, file kind, or link count.");
        StringBuilder path = new StringBuilder(32768);
        uint length = GetFinalPathNameByHandle(handle, path, (uint)path.Capacity, 0);
        if (length == 0) throw LastError();
        if (length >= path.Capacity || !String.Equals(path.ToString(), @"\\?\" + expected, StringComparison.OrdinalIgnoreCase))
            throw new InvalidDataException("Unexpected native final path.");
        return new FileFacts { key = i.volume.ToString("x8") + ":" + Combine(i.indexHigh, i.indexLow).ToString("x16"),
            creationFiletime = Combine(i.creationHigh, i.creationLow).ToString(), lastWriteFiletime = Combine(i.writeHigh, i.writeLow).ToString(),
            length = (long)Combine(i.sizeHigh, i.sizeLow), attributes = i.attributes, links = i.links };
    }
    public static SafeFileHandle OpenDirectory(string path) { return Open(path, 0x80u, 0x02200000u); }
    public static FileStream OpenRead(string path) {
        SafeFileHandle handle = Open(path, 0x80000000u, 0x00200000u);
        try { return new FileStream(handle, FileAccess.Read); }
        catch { handle.Dispose(); throw; }
    }
    static SafeFileHandle Open(string path, uint access, uint flags) {
        SafeFileHandle handle = CreateFile(path, access, 3, IntPtr.Zero, 3, flags, IntPtr.Zero);
        if (!handle.IsInvalid) return handle;
        Win32Exception error = LastError();
        handle.Dispose();
        throw error;
    }
}
'@

function Get-Digest([byte[]]$Bytes) {
    $hash = [Security.Cryptography.SHA256]::Create()
    try { [BitConverter]::ToString($hash.ComputeHash($Bytes)).Replace('-', '').ToLowerInvariant() }
    finally { $hash.Dispose() }
}

function Assert-FixedFile([string]$Path, [string]$Digest) {
    if ((Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash -ine $Digest) { throw 'Fixed input digest differs.' }
}

function Get-FileProtection($Security) {
    $owner = $Security.GetOwner([Security.Principal.SecurityIdentifier]).Value
    $sections = [Security.AccessControl.AccessControlSections]'Owner, Access'
    $sddl = $Security.GetSecurityDescriptorSddlForm($sections)
    $descriptor = [Security.AccessControl.RawSecurityDescriptor]::new($sddl)
    $trusted = @($sid, 'S-1-5-18')
    $principals = @()
    if ($owner -cne $sid -or $null -eq $descriptor.DiscretionaryAcl) { throw 'Private owner or DACL differs.' }
    foreach ($ace in $descriptor.DiscretionaryAcl) {
        if ($ace.AceType -ne 'AccessAllowed' -or $ace.SecurityIdentifier.Value -cnotin $trusted) { throw 'Unexpected private DACL principal or ACE.' }
        if ($ace.AceFlags -ne 'None' -or $ace.AccessMask -ne 0x1f01ff) { throw 'Unexpected original private ACE flags or mask.' }
        $principals += $ace.SecurityIdentifier.Value
    }
    if (!$Security.AreAccessRulesProtected -or $principals.Count -ne 2 -or $sid -cnotin $principals -or 'S-1-5-18' -cnotin $principals) {
        throw 'Expected private file protection missing.'
    }
    [ordered]@{ protectedDacl = $Security.AreAccessRulesProtected; ownerIsOriginalSid = $true; grantCount = $principals.Count; ownerAndDaclSha256 = Get-Digest ($utf8.GetBytes($sddl)) }
}

function Read-Range([IO.FileStream]$Stream, [int]$Length) {
    $Stream.Position = 0
    $reader = [IO.BinaryReader]::new($Stream, $utf8, $true)
    try { $bytes = $reader.ReadBytes($Length) } finally { $reader.Dispose() }
    if ($bytes.Length -ne $Length) { throw 'Snapshot prefix was truncated.' }
    return ,$bytes
}

function Write-NewBytes([string]$Path, [byte[]]$Bytes) {
    $output = [IO.File]::Open($Path, 'CreateNew', 'Write', 'Read')
    try { $output.Write($Bytes, 0, $Bytes.Length); $output.Flush($true) } finally { $output.Dispose() }
}

function Observe-Host {
    $facts = [R6ArchiveProcessFacts]::Observe(25144)
    if ($facts.exited -or $facts.pid -ne 25144 -or $facts.filetime -cne $hostFiletime -or $facts.session -ne 7 -or $facts.image -ine (Join-Path $scene 'I\ocxd.exe')) {
        throw 'Original live host generation differs.'
    }
    [ordered]@{ pid = $facts.pid; filetime = $facts.filetime; session = $facts.session; exited = $facts.exited }
}

function Summarize-Events([byte[]]$Bytes) {
    $lastLineEnd = [Array]::LastIndexOf([byte[]]$Bytes, [byte]10)
    $text = $utf8.GetString($Bytes, 0, $lastLineEnd + 1)
    $lines = $text.Split([char]10)
    $events = [Collections.Generic.List[object]]::new()
    for ($i = 0; $i -lt $lines.Length - 1; $i++) {
        $event = $lines[$i] | ConvertFrom-Json -ErrorAction Stop
        $events.Add($event)
    }
    $intent = @($events | Where-Object {
        $_.stage -ceq 'v2-original-request-intent' -and
        $_.facts.bothNoOuterJobObservedAtAdmission -is [bool] -and $_.facts.bothNoOuterJobObservedAtAdmission -and
        $_.facts.nativePeerAndLocatorRechecked -is [bool] -and $_.facts.nativePeerAndLocatorRechecked -and $_.facts.peer.pid -eq 123212 -and $_.facts.peer.creationFiletime -ceq $explicitFiletime -and
        $_.facts.peer.sid -ceq $sid -and $_.facts.peer.sessionId -eq 7 -and $_.facts.peer.elevated -is [bool] -and !$_.facts.peer.elevated -and $_.facts.peer.elevationType -eq 1 -and
        $_.facts.broker.pid -eq 25144 -and $_.facts.broker.creationFiletime -ceq $hostFiletime -and $_.facts.broker.sid -ceq $sid -and $_.facts.broker.sessionId -eq 7 -and
        $_.facts.broker.elevated -is [bool] -and !$_.facts.broker.elevated -and $_.facts.broker.elevationType -eq 1
    })
    $summary = [ordered]@{
        completeLines = $events.Count; incompleteTailUtf8Bytes = ($Bytes.Length - $lastLineEnd - 1); matchingIntents = $intent.Count
        latestBoundStage = 'Unknown'; requestId = $null; transportActionId = $null; resultCount = $null; resultPurpose = $null
        resultErrorKind = $null; resultRaw = $null; explicitRecoveryActionId = $null
        v2Durability = 'Unknown'; scanStage = 'Unknown'; replyDelivery = 'Unknown'
    }
    if ($intent.Count -ne 1 -or $intent[0].facts.requestId -cnotmatch $guidPattern -or $intent[0].facts.transportActionId -cnotmatch $guidPattern) { return $summary }
    $summary.requestId = $intent[0].facts.requestId
    $summary.transportActionId = $intent[0].facts.transportActionId
    $summary.latestBoundStage = 'v2-original-request-intent'
    $intentIndex = $events.IndexOf($intent[0])
    $results = @($events | Where-Object { $_.stage -ceq 'v2-original-request-result' -and $_.facts.requestId -ceq $summary.requestId })
    $summary.resultCount = $results.Count
    if ($results.Count -eq 1 -and $events.IndexOf($results[0]) -gt $intentIndex) {
        $reply = $results[0].facts
        $validRefusal = $reply.purpose -ceq 'refused' -and $null -ne $reply.error
        $validSuccess = $reply.purpose -ceq 'canonical-abort-result' -and $null -eq $reply.error -and
            $reply.facts.transportActionId -ceq $summary.transportActionId
        if ($reply.schema -ceq 'windows-before-resume-private-control-v2' -and ($validRefusal -or $validSuccess)) {
            $summary.latestBoundStage = 'v2-original-request-result'
            $summary.resultPurpose = $reply.purpose
            if ($reply.error.kind -cmatch '^(TimedOut|WouldBlock|PermissionDenied|InvalidData|BrokenPipe|Other|NotFound|ConnectionAborted)$') { $summary.resultErrorKind = $reply.error.kind }
            if ($reply.error.raw -is [int]) { $summary.resultRaw = $reply.error.raw }
        }
    }
    $summary
}

$result = [ordered]@{
    schema = 'team72-event-snapshot-r2'; passed = $false; startedUtc = [DateTime]::UtcNow.ToString('o'); firstError = $null
    explicitReceiptSha256 = 'f06070f4f474b94289323872db63f73657f7a39b1eccc571d07d80326967648b'
    acceptedWorker = 'ThreadId(51)'; acceptedWorkerEvidenceCommit = 'd88b1839e456751268ae2bf741cca64e665c548e'
    fileName = 'before-resume-private-control-v2-events.jsonl'; pathResolution = 'fixed source and TEAM72 filename, normalized r7 path'
}
$step = 'context'
$allowExport = $false
try {
    Add-Type -TypeDefinition $native
    $user = [Security.Principal.WindowsIdentity]::GetCurrent()
    $desktop = [Team72FileEvidence]::Desktop()
    $session = [Team72FileEvidence]::TokenValue($user.Token, 12)
    $elevation = [Team72FileEvidence]::TokenValue($user.Token, 20)
    $elevationType = [Team72FileEvidence]::TokenValue($user.Token, 18)
    $isAdmin = ([Security.Principal.WindowsPrincipal]::new($user)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if ($user.User.Value -cne $sid -or $session -ne 7 -or $desktop -cne 'WinSta0/Default' -or $elevation -ne 0 -or $elevationType -ne 1 -or $isAdmin -or
        [Environment]::GetFolderPath('UserProfile') -ine $profileRoot) { throw 'Use the original ordinary marenben desktop session 7.' }
    $allowExport = $true
    $result.context = [ordered]@{ sid = $sid; session = $session; desktop = $desktop; elevated = $elevation; elevationType = $elevationType }
    $step = 'fixed-receipts'
    if ([IO.File]::Exists($publicResult)) { throw 'Existing snapshot output. Preserve it.' }
    Assert-FixedFile $priorReceipt $priorReceiptSha
    $result.priorReceiptSha256 = $priorReceiptSha
    Assert-FixedFile (Join-Path $receipts 'Explicit-result.json') $result.explicitReceiptSha256
    Assert-FixedFile (Join-Path $receipts 'DiagnosticBefore-result.json') '677d8e9f2d1ceec235837c69a1ddffd5d8023c89e5f464ae5f79843d24dd6977'
    $processSource = 'C:\Users\Public\ocx-r7-range-observers-20261118-r1\archive-r6-process-facts.cs'
    [byte[]]$processSourceBytes = [IO.File]::ReadAllBytes($processSource)
    if ((Get-Digest $processSourceBytes) -cne '1959b81e02779b9a88bb64f7321c1cc1975c68c506f4746340178b81e7d05603') { throw 'Live observer source digest differs.' }
    Add-Type -TypeDefinition ($utf8.GetString($processSourceBytes))
    $step = 'host-before'
    $result.hostBefore = Observe-Host
    $pins = [Collections.Generic.List[object]]::new()
    try {
        $step = 'pin-log-parents'
        foreach ($parent in @('C:\Users', $profileRoot, $scene, (Join-Path $scene 'D'))) {
            $handle = [Team72FileEvidence]::OpenDirectory($parent)
            $pins.Add(@{ handle = $handle; path = $parent; before = $null })
            $pins[$pins.Count - 1].before = [Team72FileEvidence]::Observe($handle, $parent, $true)
        }
        $step = 'open-log'
        $stream = [Team72FileEvidence]::OpenRead($log)
        try {
            $step = 'original-file-before'
            $result.handleBefore = [Team72FileEvidence]::Observe($stream.SafeFileHandle, $log, $false)
            $result.protectionBefore = Get-FileProtection ($stream.GetAccessControl())
            $pathBefore = [Team72FileEvidence]::OpenRead($log)
            try { $result.pathBefore = [Team72FileEvidence]::Observe($pathBefore.SafeFileHandle, $log, $false) } finally { $pathBefore.Dispose() }
            $step = 'prior-original-identity'
            if ($result.handleBefore.key -cne $priorFileKey -or $result.handleBefore.creationFiletime -cne $priorCreation -or
                $result.protectionBefore.ownerAndDaclSha256 -cne $priorDaclSha) { throw 'Original file generation or protection differs from v1.' }
            $step = 'read-stable-prefix'
            $length = $result.handleBefore.length
            if ($length -lt 0 -or $length -gt 16MB) { throw 'Original event log exceeds the snapshot limit.' }
            [byte[]]$bytes = Read-Range $stream $length
            $digest = Get-Digest $bytes
            $pathCheck = [Team72FileEvidence]::OpenRead($log)
            try {
                $result.repeatHandle = [Team72FileEvidence]::Observe($pathCheck.SafeFileHandle, $log, $false)
                if ($result.repeatHandle.key -cne $result.handleBefore.key) { throw 'Independent repeat opened a different file.' }
                if ((Get-Digest (Read-Range $pathCheck $length)) -cne $digest) { throw 'Original event prefix changed during independent repeat read.' }
            } finally { $pathCheck.Dispose() }
            $step = 'original-file-after'
            $result.handleAfter = [Team72FileEvidence]::Observe($stream.SafeFileHandle, $log, $false)
            $result.protectionAfter = Get-FileProtection ($stream.GetAccessControl())
            $pathCheck = [Team72FileEvidence]::OpenRead($log)
            try { $result.pathAfter = [Team72FileEvidence]::Observe($pathCheck.SafeFileHandle, $log, $false) } finally { $pathCheck.Dispose() }
            if ($result.handleBefore.key -cne $result.handleAfter.key -or $result.handleBefore.key -cne $result.pathBefore.key -or
                $result.pathBefore.key -cne $result.pathAfter.key -or
                $result.handleBefore.creationFiletime -cne $result.pathAfter.creationFiletime -or $result.pathAfter.length -lt $length -or
                $result.handleAfter.length -lt $length -or $result.protectionBefore.ownerAndDaclSha256 -cne $result.protectionAfter.ownerAndDaclSha256) {
                throw 'Original file identity, length, or protection changed.'
            }
            foreach ($pin in $pins) {
                $after = [Team72FileEvidence]::Observe($pin.handle, $pin.path, $true)
                if ($after.key -cne $pin.before.key) { throw 'Original log ancestor changed.' }
            }
            $result.range = [ordered]@{ start = 0; endExclusive = $length; sha256 = $digest; completeFileAtLastObservation = ($length -eq $result.pathAfter.length -and $length -eq $result.handleAfter.length) }
            $step = 'prior-prefix-binding'
            if ($length -lt $priorLength) { throw 'Previously observed range is truncated.' }
            $priorBytes = [byte[]]::new($priorLength)
            [Array]::Copy($bytes, $priorBytes, $priorLength)
            try {
                if ((Get-Digest $priorBytes) -cne $priorPrefixSha) { throw 'Previously observed prefix digest differs.' }
                $result.priorRange = [ordered]@{ start = 0; endExclusive = $priorLength; sha256 = $priorPrefixSha }
            } finally { [Array]::Clear($priorBytes, 0, $priorBytes.Length) }
        } finally { $stream.Dispose() }
        $step = 'host-after'
        $result.hostAfter = Observe-Host
        $result.snapshotMode = 'in-memory-projection-only'
        $step = 'summarize-events'
        $result.events = Summarize-Events $bytes
        $result.passed = $true
    } finally { foreach ($pin in $pins) { $pin.handle.Dispose() } }
} catch {
    $cause = $_.Exception.GetBaseException()
    $raw = $null
    if ($cause -is [ComponentModel.Win32Exception]) { $raw = $cause.NativeErrorCode }
    $result.firstError = [ordered]@{ stage = $step; type = $cause.GetType().Name; hresult = $cause.HResult; raw = $raw }
    if (!$allowExport) { Write-Warning $cause.Message }
}
if ($null -ne $bytes) { [Array]::Clear($bytes, 0, $bytes.Length) }
$result.finishedUtc = [DateTime]::UtcNow.ToString('o')
$json = $result | ConvertTo-Json -Depth 10
if ($allowExport -and ![IO.File]::Exists($publicResult)) { Write-NewBytes $publicResult ($utf8.GetBytes($json + [Environment]::NewLine)) }
Write-Output $json
if (!$result.passed) { exit 1 }

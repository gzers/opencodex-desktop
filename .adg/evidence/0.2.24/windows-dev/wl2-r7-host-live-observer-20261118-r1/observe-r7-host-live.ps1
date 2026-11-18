param(
    [Parameter(Mandatory)]
    [ValidatePattern('^[0-9a-f]{64}$')]
    [string]$ExpectedHostResultSha256
)

$ErrorActionPreference = 'Stop'

function ReadBoundedBytes([string]$Path, [int]$Limit) {
    if ((Get-Item -LiteralPath $Path -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) {
        throw 'Evidence path is a reparse point.'
    }
    $stream = [IO.File]::Open($Path, 'Open', 'Read', 'Read')
    try {
        if ($stream.Length -gt $Limit) { throw 'Evidence length exceeds its limit.' }
        $length = [int]$stream.Length
        $reader = [IO.BinaryReader]::new($stream, [Text.Encoding]::UTF8, $true)
        try { $bytes = $reader.ReadBytes($length) } finally { $reader.Dispose() }
        if ($bytes.Length -ne $length) { throw 'Evidence read was incomplete.' }
        return ,$bytes
    } finally { $stream.Dispose() }
}

function GetDigest([byte[]]$Bytes) {
    $hasher = [Security.Cryptography.SHA256]::Create()
    try { [BitConverter]::ToString($hasher.ComputeHash($Bytes)).Replace('-', '').ToLowerInvariant() }
    finally { $hasher.Dispose() }
}

function ObserveLiveGeneration([uint32]$ProcessId, [string]$Filetime, [uint32]$Session, [string]$Image) {
    $receipt = [ordered]@{ liveProcessVerified = $false; raw = $null; firstError = $null; facts = $null }
    $receipt.queryStartedUtc = [DateTime]::UtcNow.ToString('o')
    try {
        $facts = [R6ArchiveProcessFacts]::Observe($ProcessId)
        $receipt.facts = $facts
        $receipt.raw = 0
        if ($facts.exited -or $facts.pid -ne $ProcessId -or $facts.filetime -cne $Filetime -or
            $facts.session -ne $Session -or $facts.image -ine $Image) {
            throw 'Live PID, creation FILETIME, session or image differs from the verified Host receipt.'
        }
        $receipt.liveProcessVerified = $true
    } catch {
        $cause = $_.Exception.GetBaseException()
        $receipt.firstError = $cause.Message
        if ($cause -is [ComponentModel.Win32Exception]) { $receipt.raw = $cause.NativeErrorCode }
    } finally { $receipt.queryFinishedUtc = [DateTime]::UtcNow.ToString('o') }
    $receipt
}

$profile = 'C:\Users\marenben'
$image = Join-Path $profile 'ocx024-r7-isolated-owner-range-r1\I\ocxd.exe'
$public = 'C:\Users\Public\ocx-r7-isolated-owner-range-r1'
$out = Join-Path $public 'out01'
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$session = [Diagnostics.Process]::GetCurrentProcess().SessionId
$admin = [Security.Principal.WindowsPrincipal]::new($identity).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if ($identity.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or $session -ne 7 -or
    $admin -or $env:USERPROFILE -cne $profile) { throw 'Use the existing marenben ordinary session 7.' }
foreach ($parent in @('C:\Users', 'C:\Users\Public', $public, $out)) {
    $item = Get-Item -LiteralPath $parent -Force
    if (!$item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw 'Fixed receipt directory differs.'
    }
}
if (Test-Path -LiteralPath (Join-Path $out 'DiagnosticBefore-intent.json')) {
    throw 'DiagnosticBefore already has an attempt. Preserve its receipts.'
}
$utf8 = [Text.UTF8Encoding]::new($false, $true)
$hostBytes = ReadBoundedBytes (Join-Path $out 'Host-result.json') 1048576
$hostSha = GetDigest $hostBytes
if ($hostSha -cne $ExpectedHostResultSha256) { throw 'Use the independently verified Host result bytes.' }
$record = $utf8.GetString($hostBytes) | ConvertFrom-Json
if ($record.mode -cne 'Host' -or $record.passed -isnot [bool] -or !$record.passed -or
    $record.pid -isnot [int] -or $record.pid -le 0 -or $record.creationFiletime -cnotmatch '^[1-9][0-9]{15,18}$') {
    throw 'Host result does not identify a passed native process generation.'
}
$native = ReadBoundedBytes (Join-Path $PSScriptRoot 'archive-r6-process-facts.cs') 65536
$nativeSha = GetDigest $native
if ($nativeSha -cne '1959b81e02779b9a88bb64f7321c1cc1975c68c506f4746340178b81e7d05603') {
    throw 'The fixed query-only native source differs.'
}
Add-Type -TypeDefinition $utf8.GetString($native)
$receiptPath = Join-Path $out 'Host-live-independent.json'
$file = [IO.File]::Open($receiptPath, 'CreateNew', 'Write', 'Read')
try {
    $receipt = ObserveLiveGeneration $record.pid $record.creationFiletime $session $image
    $receipt.hostResultSha256 = $hostSha
    $receipt.nativeSourceSha256 = $nativeSha
    $receipt.observerSid = $identity.User.Value
    $receipt.observerSession = $session
    $receipt.originalRangeContinuityVerified = $false
    $json = $receipt | ConvertTo-Json -Depth 8
    $bytes = $utf8.GetBytes($json)
    $file.Write($bytes, 0, $bytes.Length)
    $file.Flush($true)
} finally { $file.Dispose() }
Write-Output $json
if ($receipt.liveProcessVerified) { exit 0 }
exit 1

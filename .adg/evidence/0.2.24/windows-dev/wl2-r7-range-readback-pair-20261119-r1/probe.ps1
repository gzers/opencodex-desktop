param(
    [Parameter(Mandatory)][string]$Launcher,
    [Parameter(Mandatory)][string]$Archive
)

$ErrorActionPreference = 'Stop'
$expectedHash = 'b7207c94ceb391230639fb6230e4fe228faed874e39ece68e75c27f2d3487b4f'
$launcherHash = (Get-FileHash -LiteralPath $Launcher -Algorithm SHA256).Hash.ToLowerInvariant()
if ($launcherHash -cne $expectedHash) { throw 'Fixed range-r1 launcher bytes differ.' }

$tokens = $null
$errors = $null
$parser = [Management.Automation.Language.Parser]
$ast = $parser::ParseFile($Launcher, [ref]$tokens, [ref]$errors)
if ($errors.Count -ne 0) { throw 'Launcher parse failed.' }
$names = @('Check', 'RequirePlain', 'WriteBytesOnce', 'ExportExact', 'SameData', 'CheckAttachment')
$functions = @($ast.FindAll({
    param($node)
    $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -cin $names
}, $false))
if ($functions.Count -ne $names.Count) { throw 'Expected exactly six launcher helpers.' }
foreach ($function in $functions) { . ([scriptblock]::Create($function.Extent.Text)) }
$exact = ($functions | Where-Object Name -CEQ 'ExportExact').Extent.Text
$oldOpen = "[IO.File]::Open(`$Path, 'Open', 'Read', 'Read')"
$newOpen = "[IO.File]::Open(`$Path, 'Open', 'Read', 'ReadWrite')"
if ([regex]::Matches($exact, [regex]::Escape($oldOpen)).Count -ne 1) { throw 'Expected one original open.' }
$compatible = $exact.Replace('function ExportExact(', 'function ExportExactCompatible(').Replace($oldOpen, $newOpen)
. ([scriptblock]::Create($compatible))

Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.IO;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

public static class RangeAttachmentHandle
{
    [DllImport("kernel32.dll", ExactSpelling = true, CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern SafeFileHandle CreateFileW(string path, uint access, uint share,
        IntPtr security, uint disposition, uint flags, IntPtr template);

    public static FileStream Create(string path)
    {
        // Match create_private_journal_file access, share, disposition and flags.
        SafeFileHandle file = CreateFileW(path, 0xc0020000, 1, IntPtr.Zero, 1, 0x200000, IntPtr.Zero);
        if (file.IsInvalid) throw new Win32Exception(Marshal.GetLastWin32Error());
        return new FileStream(file, FileAccess.ReadWrite);
    }
}
'@

function GetFailure($Record) {
    $cause = $Record.Exception.GetBaseException()
    [ordered]@{ raw = $cause.HResult -band 65535; hresult = $cause.HResult; message = $cause.Message }
}

$attachment = ($functions | Where-Object Name -CEQ 'CheckAttachment').Extent.Text
$oldCall = 'ExportExact $path'
Check ([regex]::Matches($attachment, [regex]::Escape($oldCall)).Count -eq 1) 'Expected one original attachment reader call.'
$candidate = $attachment.Replace('function CheckAttachment(', 'function CheckAttachmentCompatible(').Replace($oldCall, 'ExportExactCompatible $path')
. ([scriptblock]::Create($candidate))

$utf8 = [Text.UTF8Encoding]::new($false)
$fileNames = @('v2-record-000001.json', 'v2-record-000002.json')
$archivePaths = @($fileNames | ForEach-Object { Join-Path $Archive $_ })
$record = [IO.File]::ReadAllText($archivePaths[0]) | ConvertFrom-Json
$digests = @($archivePaths | ForEach-Object { (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash.ToLowerInvariant() })
Check ($digests[0] -ceq '736fc8b702c8decf098d5f45863d2395857982962b6ef5a22ab24f30aa496260') 'Original archived Intent bytes differ.'
$snapshot = [pscustomobject]@{
    actionId = $record.recoveryActionId
    preparationPending = [pscustomobject]@{ intentDigest = $digests[0]; pendingDigest = $digests[1] }
}
$injection = [pscustomobject]@{
    captureSessionId = $record.sessionId
    failedActionId = $record.failedActionId
    hostInstanceId = $record.reference.ownerInstanceId
    sourceBindingDigest = $record.reference.source.bindingDigest
}
$imageProof = [pscustomobject]@{ current = [pscustomobject]@{ imageSha256 = $record.reference.source.imageDigest } }
$base = Join-Path ([IO.Path]::GetTempPath()) ('ocx-r7-readback-pair-' + [guid]::NewGuid().ToString('N'))
$outRoot = Join-Path $base 'out'
$actionDir = Join-Path $base ('D\before-resume-pending-' + $snapshot.actionId)
$extra = Join-Path $actionDir 'unexpected.json'
$sourcePaths = @($fileNames | ForEach-Object { Join-Path $actionDir $_ })
$outputNames = @($fileNames | ForEach-Object { 'DiagnosticAfter-' + $_ })
$cases = @('fixed', 'candidate', 'wrong-action', 'extra-member', 'changed-bytes')
$vs = [ordered]@{
    launcherSha256 = $launcherHash
    powerShell = $PSVersionTable.PSVersion.ToString()
    scope = 'archived synthetic records; no live worker or real deadline proof'
    scratch = $base
    nativeAccess = '0xc0020000'
    nativeShare = 1
    nativeDisposition = 1
    nativeFlags = '0x200000'
    archiveSha256 = $digests
    results = [ordered]@{}
    cleanupComplete = $false
}

function ExpectFailure([scriptblock]$Call, [string]$Message, [int]$Raw) {
    $failure = $null
    try { $null = & $Call } catch { $failure = GetFailure $_ }
    Check ($null -ne $failure) 'The negative control was unexpectedly accepted.'
    if ($Raw -ne 0) {
        Check ($failure.raw -eq $Raw) ('Unexpected raw code: ' + ($failure | ConvertTo-Json -Compress))
    } else {
        Check ($failure.message -ceq $Message) ('Unexpected first error: ' + $failure.message)
    }
    return $failure
}

$writers = @()
New-Item -ItemType Directory -Path $base | Out-Null
try {
    [void][IO.Directory]::CreateDirectory($actionDir)
    [void][IO.Directory]::CreateDirectory($outRoot)
    foreach ($case in $cases) { [void][IO.Directory]::CreateDirectory((Join-Path $outRoot $case)) }
    for ($i = 0; $i -lt $fileNames.Count; $i++) {
        $writer = [RangeAttachmentHandle]::Create($sourcePaths[$i])
        $writers += $writer
        $bytes = [IO.File]::ReadAllBytes($archivePaths[$i])
        $writer.Write($bytes, 0, $bytes.Length)
        $writer.Flush($true)
    }

    $out = Join-Path $outRoot 'fixed'
    $vs.results.fixed = ExpectFailure { CheckAttachment $snapshot $injection $imageProof } '' 32
    Check (@(Get-ChildItem -LiteralPath $out -Force).Count -eq 0) 'The fixed reader unexpectedly exported data.'

    $out = Join-Path $outRoot 'candidate'
    $rows = CheckAttachmentCompatible $snapshot $injection $imageProof
    Check ($rows.Count -eq 2) 'Expected two complete exports.'
    for ($i = 0; $i -lt $rows.Count; $i++) {
        $export = [IO.File]::ReadAllBytes((Join-Path $out $rows[$i].name))
        $originalBytes = [IO.File]::ReadAllBytes($archivePaths[$i])
        Check ($rows[$i].sha256 -ceq $digests[$i] -and
            [Convert]::ToBase64String($export) -ceq [Convert]::ToBase64String($originalBytes)) 'Exported attachment bytes differ.'
    }
    $vs.results.candidate = @($rows | Select-Object name, bytes, sha256)
    $vs.results.repeatedExport = ExpectFailure { CheckAttachmentCompatible $snapshot $injection $imageProof } '' 80
    for ($i = 0; $i -lt $rows.Count; $i++) {
        $exportHash = (Get-FileHash -LiteralPath (Join-Path $out $rows[$i].name) -Algorithm SHA256).Hash.ToLowerInvariant()
        Check ($exportHash -ceq $digests[$i]) 'Create-only negative control changed an export.'
    }
    $vs.results.secondWriter = @($sourcePaths | ForEach-Object {
        $path = $_
        ExpectFailure {
            $unexpectedWriter = [IO.File]::Open($path, 'Open', 'Write', 'ReadWrite')
            $unexpectedWriter.Dispose()
        } '' 32
    })

    $out = Join-Path $outRoot 'wrong-action'
    $otherInjection = $injection.PSObject.Copy()
    $otherInjection.failedActionId = '44444444-4444-4444-8444-444444444444'
    $vs.results.wrongFailedAction = ExpectFailure { CheckAttachmentCompatible $snapshot $otherInjection $imageProof } 'Pending record schema or original Owner and action binding differs.' 0

    $out = Join-Path $outRoot 'extra-member'
    [IO.File]::WriteAllBytes($extra, [byte[]]@())
    $vs.results.extraMember = ExpectFailure { CheckAttachmentCompatible $snapshot $injection $imageProof } 'Pending directory has unexpected membership.' 0
    Check (@(Get-ChildItem -LiteralPath $out -Force).Count -eq 0) 'Extra membership should be rejected before export.'
    [IO.File]::Delete($extra)

    $out = Join-Path $outRoot 'changed-bytes'
    # Appending JSON whitespace leaves semantics intact and breaks the exact byte binding.
    $writers[1].WriteByte(32)
    $writers[1].Flush($true)
    $vs.results.changedPendingBytes = ExpectFailure { CheckAttachmentCompatible $snapshot $injection $imageProof } 'Pending record event or exact file digest chain differs.' 0
} finally {
    foreach ($writer in $writers) { $writer.Dispose() }
    foreach ($path in ($sourcePaths + @($extra))) { [IO.File]::Delete($path) }
    foreach ($case in $cases) {
        $caseDir = Join-Path $outRoot $case
        if ([IO.Directory]::Exists($caseDir)) {
            foreach ($name in $outputNames) { [IO.File]::Delete((Join-Path $caseDir $name)) }
            [IO.Directory]::Delete($caseDir, $false)
        }
    }
    foreach ($path in @($actionDir, (Join-Path $base 'D'), $outRoot, $base)) {
        if ([IO.Directory]::Exists($path)) { [IO.Directory]::Delete($path, $false) }
    }
    $vs.cleanupComplete = !(Test-Path -LiteralPath $base)
    $vs | ConvertTo-Json -Depth 8
}

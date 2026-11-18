param(
    [Parameter(Mandatory)][string]$Launcher
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
$names = @('Check', 'RequirePlain', 'WriteBytesOnce', 'ExportExact')
$functions = @($ast.FindAll({
    param($node)
    $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -cin $names
}, $false))
if ($functions.Count -ne $names.Count) { throw 'Expected four exact launcher helpers.' }
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

$utf8 = [Text.UTF8Encoding]::new($false, $true)
$out = Join-Path ([IO.Path]::GetTempPath()) ('ocx-r7-readback-share-' + [guid]::NewGuid().ToString('N'))
$source = Join-Path $out 'synthetic-v2-record.json'
$compatibleName = 'compatible-export.json'
$fixedName = 'fixed-export.json'
$vs = [ordered]@{
    launcherSha256 = $launcherHash
    powerShell = $PSVersionTable.PSVersion.ToString()
    scratch = $out
    nativeAccess = '0xc0020000'
    nativeShare = 1
    nativeDisposition = 1
    nativeFlags = '0x200000'
    fixedWhileHeld = $null
    compatibleWhileHeld = $null
    secondWriterWhileHeld = $null
    fixedAfterRelease = $null
    cleanupComplete = $false
}

New-Item -ItemType Directory -Path $out | Out-Null
try {
    $writer = [RangeAttachmentHandle]::Create($source)
    try {
        $bytes = $utf8.GetBytes('{"synthetic":"flush and readback"}')
        $writer.Write($bytes, 0, $bytes.Length)
        $writer.Flush($true)
        try { $vs.fixedWhileHeld = ExportExact $source $fixedName 65536 }
        catch { $vs.fixedWhileHeld = GetFailure $_ }
        Check ($vs.fixedWhileHeld.raw -eq 32) 'Fixed read was expected to return ERROR_SHARING_VIOLATION.'
        Check (!(Test-Path -LiteralPath (Join-Path $out $fixedName))) 'Failed read unexpectedly exported data.'

        $vs.compatibleWhileHeld = ExportExactCompatible $source $compatibleName 65536
        $encodedExport = [Convert]::ToBase64String([IO.File]::ReadAllBytes((Join-Path $out $compatibleName)))
        Check ($encodedExport -ceq [Convert]::ToBase64String($bytes)) 'Compatible readback changed bytes.'
        $other = $null
        try { $other = [IO.File]::Open($source, 'Open', 'Write', 'ReadWrite') }
        catch { $vs.secondWriterWhileHeld = GetFailure $_ }
        finally { if ($null -ne $other) { $other.Dispose() } }
        Check ($vs.secondWriterWhileHeld.raw -eq 32) 'The retained handle still needs to deny a second writer.'
    } finally { $writer.Dispose() }

    $vs.fixedAfterRelease = ExportExact $source $fixedName 65536
    Check ($vs.fixedAfterRelease.sha256 -ceq $vs.compatibleWhileHeld.sha256) 'Readback digests differ.'
} finally {
    foreach ($name in @('synthetic-v2-record.json', $compatibleName, $fixedName)) {
        $path = Join-Path $out $name
        if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path }
    }
    [IO.Directory]::Delete($out, $false)
    $vs.cleanupComplete = !(Test-Path -LiteralPath $out)
    $vs | ConvertTo-Json -Depth 8
}

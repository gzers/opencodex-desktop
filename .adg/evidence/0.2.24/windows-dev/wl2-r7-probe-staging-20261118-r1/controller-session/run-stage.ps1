param([string]$CodeRoot,[string]$BuildEvidence)
$ErrorActionPreference='Stop'
$root=$PSScriptRoot
$started=[DateTime]::UtcNow
$pi=[Diagnostics.ProcessStartInfo]::new()
$pi.FileName=(Get-Process -Id $PID).Path
$pi.UseShellExecute=$false
$pi.CreateNoWindow=$true
$pi.RedirectStandardOutput=$true
$pi.RedirectStandardError=$true
foreach($arg in @('-NoProfile','-NonInteractive','-File',(Join-Path $root 'stage.ps1'),'-CodeRoot',$CodeRoot,'-EvidenceRoot',$root,'-BuildEvidence',$BuildEvidence)){[void]$pi.ArgumentList.Add($arg)}
$stdout=[IO.File]::Open((Join-Path $root 'staging.stdout.log'),'CreateNew','Write','Read')
$stderr=[IO.File]::Open((Join-Path $root 'staging.stderr.log'),'CreateNew','Write','Read')
try {
    $proc=[Diagnostics.Process]::Start($pi)
    $o=$proc.StandardOutput.BaseStream.CopyToAsync($stdout)
    $e=$proc.StandardError.BaseStream.CopyToAsync($stderr)
    $proc.WaitForExit()
    [void]$o.GetAwaiter().GetResult()
    [void]$e.GetAwaiter().GetResult()
    $raw=$proc.ExitCode
    $proc.Dispose()
} finally { $stdout.Dispose(); $stderr.Dispose() }
@{rawExit=$raw;startedUtc=$started.ToString('o');finishedUtc=[DateTime]::UtcNow.ToString('o');controller=$pi.FileName} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $root 'staging.result.json') -Encoding utf8NoBOM
Get-Content -LiteralPath (Join-Path $root 'staging.result.json')
if ($raw -ne 0) { Get-Content -LiteralPath (Join-Path $root 'staging.stderr.log'); exit $raw }
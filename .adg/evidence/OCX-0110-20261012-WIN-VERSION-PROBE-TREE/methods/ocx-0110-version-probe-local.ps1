param(
    [Parameter(Mandatory=$true)][ValidateSet('probe-red','probe-fixed','probe-atomic-red','probe-atomic-fixed')][string]$Label,
    [ValidateSet('ordinary','administrator')][string]$Mode = 'ordinary'
)
$ErrorActionPreference = 'Stop'
$codeRoot = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$evidence = 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-VERSION-PROBE-TREE'
$expectedSources = @{
    'probe-red' = '3d449ac0e9c1c02afa5fb2a1422db06fc3778531'
    'probe-fixed' = '65b910f2a0e5dfa31d18caa3b169d6ee20790539'
    'probe-atomic-red' = '277f76251b2db5401c152b3517bf4e539224a75c'
    'probe-atomic-fixed' = 'eae85322ebca3b755c034db97cbebf2ea5bf71fc'
}
$expectedSource = $expectedSources[$Label]
$receipt = Get-Content -LiteralPath (Join-Path $evidence "$Label-test-executable-verification.json") -Raw |
    ConvertFrom-Json -DateKind String
$testExe = [IO.Path]::GetFullPath($receipt.executable)
if (!$testExe.StartsWith((Join-Path $codeRoot 'test/software/query-job-source') + '\',
    [StringComparison]::OrdinalIgnoreCase) -or $receipt.sourceCommit -cne $expectedSource -or
    (Get-FileHash -LiteralPath $testExe -Algorithm SHA256).Hash.ToLowerInvariant() -cne $receipt.sha256 -or
    !$receipt.ciTestExecutableReceiptMatched) {
    throw 'Source-bound verified CI test image unavailable'
}
. (Join-Path $codeRoot 'test/smoke/windows-process-token.ps1')
$helperToken = Get-WindowsHarnessToken
Assert-WindowsProcessMode -Observation $helperToken -Mode $Mode
$node = 'C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe'
$runtime = Join-Path $codeRoot "test/runtime/version-probe-$Label-$Mode"
if (Test-Path -LiteralPath $runtime) { throw 'Do not silently reuse a previous probe sample' }
New-Item -ItemType Directory -Path $runtime -Force | Out-Null
$before = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'")
if ($before.Count -ne 1 -or $before[0].ProcessId -ne 38924) { throw 'Original desktop lifecycle changed' }
$records = @()
foreach ($spec in @(
    @{name='version';filter='infrastructure::official_version_source::windows_tests::'},
    @{name='metadata';filter='modules::about::remote::windows_tests::'}
)) {
    $queryRoot = Join-Path $runtime $spec.name
    $queryHome = Join-Path $queryRoot 'home'
    $queryTemp = Join-Path $queryRoot 'temp'
    New-Item -ItemType Directory -Path $queryHome, $queryTemp -Force | Out-Null
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $testExe
    $info.WorkingDirectory = $queryRoot
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    foreach ($argument in @($spec.filter, '--test-threads=1', '--nocapture')) { [void]$info.ArgumentList.Add($argument) }
    $info.Environment.Clear()
    $info.Environment['SystemRoot'] = 'C:\Windows'
    $info.Environment['WINDIR'] = 'C:\Windows'
    $info.Environment['COMSPEC'] = 'C:\Windows\System32\cmd.exe'
    $info.Environment['PATH'] = (Split-Path -Parent $node) + ';C:\Windows\System32;C:\Windows'
    $info.Environment['HOME'] = $queryHome
    $info.Environment['USERPROFILE'] = $queryHome
    $info.Environment['APPDATA'] = Join-Path $queryHome 'AppData/Roaming'
    $info.Environment['LOCALAPPDATA'] = Join-Path $queryHome 'AppData/Local'
    $info.Environment['TEMP'] = $queryTemp
    $info.Environment['TMP'] = $queryTemp
    $info.Environment['OPENCODEX_TEST_NODE'] = $node
    $info.Environment['OPENCODEX_SANDBOX'] = '1'
    $info.Environment['OPENCODEX_SANDBOX_ROOT'] = $queryRoot
    $process = $null
    $stdout = $null
    $stderr = $null
    $record = [ordered]@{
        sourceCommit=$expectedSource;ciTestExecutableSha256=$receipt.sha256
        label=$Label;requestedMode=$Mode;testGroup=$spec.name;filter=$spec.filter
        helperToken=$helperToken;testProcessToken=$null;runtimeRoot=$queryRoot
        startedAtUtc=[DateTime]::UtcNow.ToString('o');status='preparing'
        credentialsInherited=$false;desktopApplicationLaunched=$false
        productionVersionDeadlineSeconds=2
        scope='actual source-bound Rust functions and process fixtures; current desktop not replaced'
    }
    try {
        $clock = [Diagnostics.Stopwatch]::StartNew()
        $process = [Diagnostics.Process]::Start($info)
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        $record.testProcessToken = Get-WindowsProcessToken -Process $process -ExpectedExecutable $testExe
        Assert-WindowsProcessMode -Observation $record.testProcessToken -Mode $Mode
        if (!$process.WaitForExit(45000)) { throw 'Probe fixture exceeded bounded test observation' }
        $record.elapsedMs = $clock.Elapsed.TotalMilliseconds
        $record.exitCode = $process.ExitCode
        $record.status = if ($process.ExitCode -eq 0) { 'recorded-tests-passed' } else { 'recorded-tests-failed' }
    } catch {
        $record.status = 'harness-failed'
        $record.error = $_.Exception.Message
        throw
    } finally {
        if ($process) {
            if (!$process.HasExited) { $process.Kill($true); [void]$process.WaitForExit(5000) }
            if ($stdout -and $stdout.Wait(5000)) {
                $stdout.Result | Set-Content -LiteralPath (Join-Path $evidence "$Label-$Mode-$($spec.name)-stdout.log") -Encoding utf8
            }
            if ($stderr -and $stderr.Wait(5000)) {
                $stderr.Result | Set-Content -LiteralPath (Join-Path $evidence "$Label-$Mode-$($spec.name)-stderr.log") -Encoding utf8
            }
            $process.Dispose()
        }
        $after = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe'")
        $record.finishedAtUtc = [DateTime]::UtcNow.ToString('o')
        $record.desktopCountAfter = $after.Count
        $record.sameOriginalDesktopLifecycle = $after.Count -eq 1 -and
            $after[0].ProcessId -eq 38924 -and $after[0].CreationDate -eq $before[0].CreationDate
        $record | ConvertTo-Json -Depth 8 |
            Set-Content -LiteralPath (Join-Path $evidence "$Label-$Mode-$($spec.name)-result.json") -Encoding utf8
        $records += $record
    }
}
$records | ConvertTo-Json -Depth 8

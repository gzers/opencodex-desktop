param([Parameter(Mandatory)][string]$Launcher)
$ErrorActionPreference = 'Stop'
$tokens = $null
$errors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile((Resolve-Path -LiteralPath $Launcher), [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw ($errors | Out-String) }
foreach ($node in $ast.EndBlock.Statements) {
    if ($node -is [Management.Automation.Language.FunctionDefinitionAst]) {
        . ([scriptblock]::Create($node.Extent.Text))
    }
}
$utf8 = [Text.UTF8Encoding]::new($false)
$fixture = Join-Path $PSScriptRoot 'scan-gate-fixture'
RequireAbsent $fixture
[void][IO.Directory]::CreateDirectory($fixture)
$log = Join-Path $fixture 'Host.stdout.log'
$writer = [IO.File]::Open($log, 'CreateNew', 'Write', 'Read')
$rows = [Collections.Generic.List[string]]::new()

function SetLog([string]$Text) {
    $writer.SetLength(0)
    $writer.Position = 0
    $bytes = $utf8.GetBytes($Text)
    $writer.Write($bytes, 0, $bytes.Length)
    $writer.Flush($true)
}

function Scan([int]$Ordinal, [string]$Operation, [int]$Before) {
    return [ordered]@{ordinal=$Ordinal; operation=$Operation; elapsedUs=2000000; remainingBeforeUs=$Before;
        remainingAfterUs=$Before-2000000; completed=$true; firstError=$null}
}

function Encode($Report) {
    return 'r7-initial-image-scans=' + ($Report | ConvertTo-Json -Compress -Depth 8) + [char]10
}

function Reject([string]$Name, [scriptblock]$Block, [string]$Expected) {
    $message = $null
    try { & $Block } catch { $message = $_.Exception.Message }
    Check ($message -ceq $Expected) "$Name expected '$Expected'; received '$message'."
    $rows.Add($Name)
    Write-Output "PASS $Name"
}

try {
    $entry = [ordered]@{phase='start-entry'; scans=@((Scan 1 'launch-plan' 29990000))}
    $worker = [ordered]@{phase='start-worker'; scans=@((Scan 1 'source-capture' 27000000),
        (Scan 2 'source-readback' 24000000), (Scan 3 'source-readback' 21000000),
        (Scan 4 'source-readback' 18000000), (Scan 5 'source-readback' 15000000), (Scan 6 'source-readback' 12000000))}
    $header = 'test infrastructure::windows_owner_host024::partial_capture_tests::isolated_r7_partial_capture_host ... '
    $entryLine = Encode $entry
    $workerLine = Encode $worker
    SetLog ($header + $workerLine + $entryLine)
    CheckInitialScans (ReadInitialScans $log)
    $rows.Add('live-writer-and-libtest-prefix'); Write-Output 'PASS live-writer-and-libtest-prefix'
    SetLog ($workerLine + $entryLine.TrimEnd([char]10))
    Check ($null -eq (ReadInitialScans $log)) 'Incomplete final line was accepted.'
    $rows.Add('partial-line-waits'); Write-Output 'PASS partial-line-waits'
    SetLog ($entryLine + $workerLine)
    Reject 'reversed-phases' { ReadInitialScans $log } 'Initial Start scan report count or phase order differs.'
    SetLog ($workerLine + $entryLine + $entryLine)
    Reject 'duplicate-phase' { ReadInitialScans $log } 'Initial Start scan report count or phase order differs.'
    $full = $worker.scans
    $worker.scans = $full[0..3]
    SetLog ((Encode $worker) + $entryLine)
    Reject 'missing-scan' { CheckInitialScans (ReadInitialScans $log) } 'Initial Start full scan count differs.'
    $worker.scans = $full
    $worker.scans[0].ordinal = 2
    SetLog ((Encode $worker) + $entryLine)
    Reject 'wrong-ordinal' { CheckInitialScans (ReadInitialScans $log) } 'Initial Start full scan did not complete in the expected order.'
    $worker.scans[0].ordinal = 1
    $worker.scans[3].completed = $false
    $worker.scans[3].firstError = 'original scan failed'
    SetLog ((Encode $worker) + $entryLine)
    Reject 'native-error' { CheckInitialScans (ReadInitialScans $log) } 'Initial Start full scan did not complete in the expected order.'
    $worker.scans[3].completed = $true
    $worker.scans[3].firstError = $null
    $worker.scans[0].remainingBeforeUs = 29900000
    $worker.scans[0].remainingAfterUs = 27900000
    SetLog ((Encode $worker) + $entryLine)
    Reject 'renewed-clock' { CheckInitialScans (ReadInitialScans $log) } 'Initial Start scans did not share the original 30-second deadline.'
    $worker.scans[0] = Scan 1 'source-capture' 27000000
    $worker.scans[5] = Scan 6 'source-readback' 2000000
    SetLog ((Encode $worker) + $entryLine)
    Reject 'expired-scan' { CheckInitialScans (ReadInitialScans $log) } 'Initial Start scans did not share the original 30-second deadline.'
    SetLog ('x' * 1048577)
    Reject 'bounded-stdout' { ReadInitialScans $log } 'Resident host stdout exceeds 1 MiB.'

    $failedAction = '22222222-2222-4333-8333-222222222222'
    $imageDigest = 'a' * 64
    $prep = @(for ($i = 1; $i -le 3; $i++) {
        [ordered]@{ordinal=$i;phase='preparation-full-image-verify';failedAction=$failedAction;sourceDigest=$imageDigest;
            startedUs=$i*2000000;finishedUs=$i*2000000+1000000;deadlineUs=9999990;
            remainingBeforeUs=9999990-$i*2000000;remainingAfterUs=8999990-$i*2000000;completed=$true;errorCode=$null}
    })
    $prepPrefix = 'r7-preparation-image-scans='
    $prepLine = $prepPrefix + (ConvertTo-Json -InputObject $prep -Compress) + [char]13 + [char]10
    SetLog ($header + $workerLine + $entryLine + $prepLine)
    $scans = ReadPreparationScans $log
    CheckPreparationScans $scans $failedAction $imageDigest
    $rows.Add('preparation-three-scans'); Write-Output 'PASS preparation-three-scans'
    Reject 'preparation-wrong-image' { CheckPreparationScans $scans $failedAction ('b' * 64) } 'Recovery preparation scan failed or its action, image or order differs.'
    $scans[2].deadlineUs++
    Reject 'preparation-renewed-clock' { CheckPreparationScans $scans $failedAction $imageDigest } 'Recovery preparation scans did not share the original 10-second deadline.'
    SetLog ($prepLine + $prepLine)
    Reject 'preparation-duplicate-report' { ReadPreparationScans $log } 'Expected one recovery preparation scan report.'
    SetLog ($prepLine.TrimEnd([char]13, [char]10))
    Reject 'preparation-partial-line' { ReadPreparationScans $log } 'Expected one recovery preparation scan report.'
    $passed = $true
} finally {
    $writer.Dispose()
    if ($passed) {
        [IO.File]::Delete($log)
        [IO.Directory]::Delete($fixture)
        RequireAbsent $fixture
        @{passed=$rows.Count;cases=$rows.ToArray();powerShell=$PSVersionTable.PSVersion.ToString();fixtureAbsent=$true} | ConvertTo-Json -Compress
    }
}

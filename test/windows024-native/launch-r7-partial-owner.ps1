param(
    [Parameter(Mandatory)][ValidateSet('Host','DiagnosticBefore','Explicit','DiagnosticAfter')][string]$Mode
)
$ErrorActionPreference = 'Stop'
$public = 'C:\Users\Public\ocx-r7-isolated-owner-nine-r1'
$profile = 'C:\Users\marenben'
$base = Join-Path $profile 'ocx024-r7-isolated-owner-nine-r1'
$out = Join-Path $public 'out01'
$image = Join-Path $base 'I\ocxd.exe'
$utf8 = [Text.UTF8Encoding]::new($false)

function WriteBytesOnce([string]$Name, [byte[]]$Bytes) {
    $path = Join-Path $out $Name
    $file = [IO.File]::Open($path, 'CreateNew', 'Write', 'Read')
    try { $file.Write($Bytes, 0, $Bytes.Length); $file.Flush($true) } finally { $file.Dispose() }
}

function WriteOnce([string]$Name, $Object) {
    WriteBytesOnce $Name ($utf8.GetBytes(($Object | ConvertTo-Json -Depth 32)))
}

function ReadJson([string]$Path) {
    $file = [IO.File]::Open($Path, 'Open', 'Read', ([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
    try {
        if ($file.Length -gt 1048576) { throw 'fixture DATA exceeds 1 MiB' }
        $reader = [IO.StreamReader]::new($file, $utf8)
        try { return ($reader.ReadToEnd() | ConvertFrom-Json) } finally { $reader.Dispose() }
    } finally { $file.Dispose() }
}

function Check([bool]$Condition, [string]$Message) {
    if (!$Condition) { throw $Message }
}

function ReadScanLines([string]$Path, [string]$Prefix) {
    # stdout is still open for writing by the resident host launcher.
    $file = [IO.File]::Open($Path, 'Open', 'Read', 'ReadWrite')
    try {
        $length = $file.Length
        Check ($length -le 1048576) 'Resident host stdout exceeds 1 MiB.'
        $reader = [IO.BinaryReader]::new($file, $utf8, $true)
        try { $bytes = $reader.ReadBytes([int]$length) } finally { $reader.Dispose() }
        Check ($bytes.Length -eq $length) 'Short read of resident host stdout.'
    } finally { $file.Dispose() }
    $lines = $utf8.GetString($bytes).Split([char]10)
    $testPrefix = 'test infrastructure::windows_owner_host024::partial_capture_tests::isolated_r7_partial_capture_host ... '
    for ($i = 0; $i -lt $lines.Length - 1; $i++) {
        $line = $lines[$i]
        if ($line.StartsWith($testPrefix, [StringComparison]::Ordinal)) {
            $line = $line.Substring($testPrefix.Length)
        }
        if ($line.StartsWith($Prefix, [StringComparison]::Ordinal)) {
            $line.Substring($Prefix.Length)
        }
    }
}

function ReadInitialScans([string]$Path) {
    $reports = @(foreach ($line in (ReadScanLines $Path 'r7-initial-image-scans=')) {
        $line | ConvertFrom-Json -ErrorAction Stop
    })
    if ($reports.Count -lt 2) { return }
    Check ($reports.Count -eq 2 -and $reports[0].phase -ceq 'start-worker' -and
        $reports[1].phase -ceq 'start-entry') 'Initial Start scan report count or phase order differs.'
    return ,$reports
}

function CheckInitialScans($Reports) {
    $entry = @($Reports[1].scans)
    $worker = @($Reports[0].scans)
    $expectedEntry = @('launch-plan-prepare', 'launch-plan', 'windows-read-complete')
    # Source capture includes a readback. Content capture and journal context
    # then both bracket their work with Source rechecks.
    $expectedWorker = @('source-capture') + (@('source-readback') * 5)
    Check ($entry.Count -eq $expectedEntry.Count -and $worker.Count -eq $expectedWorker.Count) 'Initial Start full scan count differs.'
    $scans = $entry + $worker
    $operations = $expectedEntry + $expectedWorker
    $deadline = $Reports[1].deadlineNs
    Check ($deadline -gt 0 -and $Reports[0].deadlineNs -eq $deadline) 'Initial Start scan phases have different absolute deadlines.'
    $remaining = 30000000
    $lastFinish = 0
    for ($i = 0; $i -lt $scans.Count; $i++) {
        $scan = $scans[$i]
        $ordinal = if ($i -lt $entry.Count) { $i + 1 } else { $i - $entry.Count + 1 }
        $before = $scan.remainingBeforeUs
        $after = $scan.remainingAfterUs
        $elapsed = $scan.elapsedUs
        Check ($scan.ordinal -eq $ordinal -and $scan.operation -ceq $operations[$i] -and
            $scan.completed -eq $true -and $null -eq $scan.firstError) 'Initial Start full scan did not complete in the expected order.'
        Check ($null -ne $scan.startedNs -and $scan.startedNs -ge $lastFinish -and
            $scan.finishedNs -gt $scan.startedNs -and $scan.finishedNs -lt $deadline -and
            $before -le $remaining -and $after -gt 0 -and $before -ge $after -and $elapsed -gt 0 -and
            [Math]::Abs((($deadline - $scan.startedNs) / 1000) - $before) -lt 1 -and
            [Math]::Abs((($deadline - $scan.finishedNs) / 1000) - $after) -lt 1 -and
            [Math]::Abs((($scan.finishedNs - $scan.startedNs) / 1000) - $elapsed) -lt 1) 'Initial Start scans did not share the original 30-second deadline.'
        $remaining = $after
        $lastFinish = $scan.finishedNs
    }
}

function ReadPreparationScans([string]$Path) {
    $lines = @(ReadScanLines $Path 'r7-preparation-image-scans=')
    Check ($lines.Count -eq 1) 'Expected one recovery preparation scan report.'
    Check ($lines[0].StartsWith('[')) 'Recovery preparation scan report is not an array.'
    $scans = $lines[0] | ConvertFrom-Json -ErrorAction Stop
    return ,$scans
}

function CheckPreparationScans($Scans, [string]$FailedAction, [string]$SourceDigest) {
    Check ($Scans.Count -eq 3) 'Recovery preparation full scan count differs.'
    $deadline = $Scans[0].deadlineUs
    Check ($deadline -gt 0 -and $deadline -le 10000000) 'Recovery preparation deadline exceeds the original 10-second window.'
    $lastFinish = 0
    $remaining = $deadline
    for ($i = 0; $i -lt $Scans.Count; $i++) {
        $scan = $Scans[$i]
        Check ($scan.ordinal -eq ($i + 1) -and $scan.phase -ceq 'preparation-full-image-verify' -and
            $scan.failedAction -ceq $FailedAction -and $scan.sourceDigest -ceq $SourceDigest -and
            $scan.completed -eq $true -and $null -eq $scan.errorCode) 'Recovery preparation scan failed or its action, image or order differs.'
        Check ($scan.deadlineUs -eq $deadline -and $scan.startedUs -ge $lastFinish -and
            $scan.finishedUs -gt $scan.startedUs -and $scan.finishedUs -lt $deadline -and
            $scan.remainingBeforeUs -le $remaining -and $scan.remainingAfterUs -gt 0 -and
            [Math]::Abs($deadline - $scan.startedUs - $scan.remainingBeforeUs) -le 1 -and
            [Math]::Abs($deadline - $scan.finishedUs - $scan.remainingAfterUs) -le 1) 'Recovery preparation scans did not share the original 10-second deadline.'
        $lastFinish = $scan.finishedUs
        $remaining = $scan.remainingAfterUs
    }
}

function SameData($Left, $Right) {
    return ($Left | ConvertTo-Json -Compress -Depth 32) -ceq ($Right | ConvertTo-Json -Compress -Depth 32)
}

function ExportExact([string]$Path, [string]$Name, [int]$Limit) {
    RequirePlain $Path $false
    $file = [IO.File]::Open($Path, 'Open', 'Read', 'Read')
    try {
        Check ($file.Length -le $Limit) 'Private fixture evidence exceeds its byte limit.'
        $reader = [IO.BinaryReader]::new($file, $utf8, $true)
        try { $bytes = $reader.ReadBytes([int]$file.Length) } finally { $reader.Dispose() }
        Check ($bytes.Length -eq $file.Length) 'Short read of private fixture evidence.'
    } finally { $file.Dispose() }
    $hasher = [Security.Cryptography.SHA256]::Create()
    try { $sha = [BitConverter]::ToString($hasher.ComputeHash($bytes)).Replace('-', '').ToLowerInvariant() }
    finally { $hasher.Dispose() }
    WriteBytesOnce $Name $bytes
    return [pscustomobject]@{ name=$Name; bytes=$bytes.Length; sha256=$sha; record=($utf8.GetString($bytes) | ConvertFrom-Json) }
}

function CheckOriginalWorker($Worker, $Injection) {
    Check ($Worker.thread -ceq $Injection.workerThread -and $Worker.actionId -ceq $Injection.failedActionId -and
        $Worker.sessionId -ceq $Injection.ownerSessionId -and $Worker.scopeDigest -ceq $Injection.scopeDigest -and
        $Worker.rawError -eq 31 -and $Worker.creatorResumed -eq $false -and $Worker.phase -ceq 'FailedRetained') 'Original Start worker binding or first error differs.'
}

function CheckPendingWorker($Snapshot, $Injection) {
    CheckOriginalWorker $Snapshot.startWorker $Injection
    Check ($Snapshot.workerThread -ceq $Injection.workerThread -and $Snapshot.sessionId -ceq $Injection.ownerSessionId -and
        $Snapshot.scopeDigest -ceq $Injection.scopeDigest -and $Snapshot.actionId -cne $Injection.failedActionId -and
        $Snapshot.actionId -cmatch '^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$' -and $Snapshot.recoveryRequired -eq $true -and
        $Snapshot.normalStop -eq $false -and $Snapshot.closed -eq $false -and $Snapshot.runtimeAuthority -eq $false) 'Independent recovery was not retained on the original thread and scope.'
    $pending = $Snapshot.preparationPending
    Check ($pending.attachmentAttempted -eq $true -and $pending.durable -eq $true -and $pending.startRawError -eq 31 -and
        $null -eq $pending.attachmentRawError -and $null -eq $pending.attachmentErrorKind -and
        (SameData $pending.originalRanges $Injection.originalClaimAndNlkVerified)) 'Durable Pending or original claim/NLK continuity is unproven.'
}

function CheckAttachment($Snapshot, $Injection, $ImageProof) {
    RequirePlain $base $true
    RequirePlain (Join-Path $base 'D') $true
    $actionDir = Join-Path $base ('D\before-resume-pending-' + $Snapshot.actionId)
    RequirePlain $actionDir $true
    $names = @('v2-record-000001.json', 'v2-record-000002.json')
    $members = @(Get-ChildItem -LiteralPath $actionDir -Force | Sort-Object -Property Name | Select-Object -ExpandProperty Name)
    Check (SameData $members $names) 'Pending directory has unexpected membership.'
    $rows = foreach ($name in $names) {
        $path = Join-Path $actionDir $name
        ExportExact $path ("DiagnosticAfter-" + $name) 65536
    }
    $intent = $rows[0].record
    $pending = $rows[1].record
    $expectedFields = @('deadlineQpc','event','failedActionId','previousRecordDigest','qpcFrequency','recoveryActionId','reference','schema','securityPolicyDigest','sequence','sessionId','startedQpc')
    foreach ($row in $rows) {
        $record = $row.record
        $fields = @($record.PSObject.Properties.Name | Sort-Object -CaseSensitive)
        Check (SameData $fields $expectedFields) 'Pending record field set differs.'
        Check ($record.schema -ceq 'windows-before-resume-recovery-v2' -and $record.sessionId -ceq $Injection.captureSessionId -and
            $record.failedActionId -ceq $Injection.failedActionId -and $record.recoveryActionId -ceq $Snapshot.actionId -and
            $record.securityPolicyDigest -cmatch '^[0-9a-f]{64}$') 'Pending record schema or original Owner and action binding differs.'
    }
    $ref = $intent.reference
    $expectedReference = @('components','intentPrefixDigest','kind','lastStage','nativeStagePrefixDigest','ownerInstanceId','source')
    Check (SameData @($ref.PSObject.Properties.Name | Sort-Object -CaseSensitive) $expectedReference) 'Preparation reference fields differ.'
    Check ($ref.kind -ceq 'preparation' -and $ref.ownerInstanceId -ceq $Injection.hostInstanceId -and $ref.lastStage -ceq 'handleRetained' -and
        $ref.source.state -ceq 'captured' -and $ref.source.bindingDigest -ceq $Injection.sourceBindingDigest -and
        $ref.source.imageDigest -ceq $ImageProof.current.imageSha256 -and $ref.intentPrefixDigest -cmatch '^[0-9a-f]{64}$' -and
        $ref.nativeStagePrefixDigest -cmatch '^[0-9a-f]{64}$' -and
        (SameData @($ref.components | ForEach-Object { $_.role }) @('source','custody','codex','grok','journal')) -and
        (SameData $pending.reference $ref)) 'Preparation reference or image binding differs.'
    $duration = [decimal]$intent.deadlineQpc - [decimal]$intent.startedQpc
    Check ($duration -gt 0 -and $duration -le (10 * [decimal]$intent.qpcFrequency) -and
        $intent.startedQpc -ceq $pending.startedQpc -and $intent.deadlineQpc -ceq $pending.deadlineQpc -and
        $intent.qpcFrequency -ceq $pending.qpcFrequency -and $intent.securityPolicyDigest -ceq $pending.securityPolicyDigest) 'Pending clock or policy differs.'
    Check ($intent.sequence -eq 1 -and $null -eq $intent.previousRecordDigest -and $intent.event.kind -ceq 'AbortRecoveryIntent' -and
        $pending.sequence -eq 2 -and $pending.previousRecordDigest -ceq $rows[0].sha256 -and
        $pending.event.kind -ceq 'AbortRecoveryPending' -and $pending.event.stage -ceq $ref.lastStage -and $pending.event.rawError -eq 31 -and
        $rows[0].sha256 -ceq $Snapshot.preparationPending.intentDigest -and $rows[1].sha256 -ceq $Snapshot.preparationPending.pendingDigest) 'Pending record event or exact file digest chain differs.'
    return ,$rows
}

function RequirePlain([string]$Path, [bool]$Directory) {
    $item = Get-Item -LiteralPath $Path -Force
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $item.PSIsContainer -ne $Directory) {
        throw "Fixture locator is not the expected plain object: $Path"
    }
}

function NewFixtureProcess([string]$Image, [string]$Directory, [string]$Test) {
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $Image
    $info.Arguments = "--ignored --exact infrastructure::windows_owner_host024::partial_capture_tests::$Test --nocapture --test-threads=1"
    $info.WorkingDirectory = $Directory
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardInput = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    return $info
}

function SendBootstrap($Process, [byte[]]$Payload) {
    $pipe = $Process.StandardInput.BaseStream
    $length = [BitConverter]::GetBytes([uint32]$Payload.Length)
    $pipe.Write($length, 0, $length.Length)
    $pipe.Write($Payload, 0, $Payload.Length)
    $pipe.Flush()
}

if ($PSScriptRoot -cne $public) { throw 'Use the fixed staged fixture root.' }
foreach ($folder in @('C:\Users', 'C:\Users\Public', $public, $out, (Join-Path $public 'input'), $profile)) {
    RequirePlain $folder $true
}
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$session = [Diagnostics.Process]::GetCurrentProcess().SessionId
$admin = [Security.Principal.WindowsPrincipal]::new($identity).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if ($identity.User.Value -cne 'S-1-5-21-587408064-856152790-2583328096-1003' -or $admin -or $session -le 0 -or $env:USERPROFILE -cne $profile) {
    throw 'Use the existing marenben ordinary interactive desktop.'
}
$input = Join-Path $public 'input'
$manifestFile = Join-Path $input 'manifest.json'
$source = Join-Path $input 'fixture-test.exe'
$bundle = Join-Path $input 'runtime-bundle-0.2.24-win32-x64.tar.gz'
foreach ($path in @($manifestFile, $source, $bundle)) { RequirePlain $path $false }
$manifest = ReadJson $manifestFile
if ($manifest.schema -ne 1 -or $manifest.sid -cne $identity.User.Value -or $manifest.session -ne $session -or
    $manifest.testSha256 -cnotmatch '^[0-9a-f]{64}$' -or
    $manifest.bundleSha256 -cne '14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7') {
    throw 'Immutable source manifest or actual fixture identity differs.'
}
if ((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -cne $manifest.testSha256) {
    throw 'Fixed test image digest differs.'
}
$config = @{ schema=1; scope='isolated-r7-partial-capture-original-worker'; expectedSid=$identity.User.Value; expectedSession=$session; profile=$profile;
    installation=(Join-Path $base 'I'); data=(Join-Path $base 'D'); home=(Join-Path $base 'H'); brokerSha256=$manifest.testSha256; managerSha256=$manifest.testSha256 }
$payload = $utf8.GetBytes(($config | ConvertTo-Json -Compress))

$result = @{ mode=$Mode; passed=$false; publicStartOpen=$false; normalStop=$false; closed=$false }
$process = $null
$outFile = $null
$errFile = $null
$stage = $null
$stageReady = $false
try {
    $stageTest = if ($Mode -ceq 'Host') { 'isolated_r7_partial_capture_stage_host' } else { 'isolated_r7_partial_capture_stage_manager' }
    $stage = [Diagnostics.Process]::Start((NewFixtureProcess $source $input $stageTest))
    $stageError = $stage.StandardError.ReadToEndAsync()
    SendBootstrap $stage $payload
    $stageClock = [Diagnostics.Stopwatch]::StartNew()
    do {
        $remaining = [Math]::Max(0, 210000 - $stageClock.ElapsedMilliseconds)
        $read = $stage.StandardOutput.ReadLineAsync()
        if (!$read.Wait([int]$remaining)) { throw 'Native fixture staging timed out.' }
        $line = $read.Result
        if ($null -eq $line) {
            $detail = if ($stageError.Wait(1000)) { $stageError.Result } else { 'stderr pending' }
            throw "Native fixture staging refused. $detail"
        }
        $stageReady = $line.EndsWith('r7-stage-ready', [StringComparison]::Ordinal)
    } until ($stageReady)
    WriteOnce "$Mode-intent.json" @{ mode=$Mode; sid=$identity.User.Value; session=$session; testSha256=$manifest.testSha256; publicStartOpen=$false; nativeStageReady=$true }
    if ($Mode -ceq 'Host') {
        $test = 'isolated_r7_partial_capture_host'
    } else {
        $ready = ReadJson (Join-Path $out 'Host-ready.json')
        if ($ready.reachedOriginalInjection -ne $true -or $ready.privateEvidenceComplete -ne $true -or
            $ready.initialScansVerified -ne $true) { throw 'Original worker injection, full scans and private evidence must be proven first.' }
        $resident = [Diagnostics.Process]::GetProcessById([int]$ready.pid)
        try {
            if ($resident.HasExited -or $resident.SessionId -ne $session -or
                $resident.StartTime.ToUniversalTime().ToFileTimeUtc().ToString() -cne $ready.creationFiletime -or
                $resident.MainModule.FileName -ine $image) { throw 'Exact original fixture process changed.' }
        } finally { $resident.Dispose() }
        if ($Mode -ceq 'Explicit') {
            if (!(ReadJson (Join-Path $out 'DiagnosticBefore-result.json')).passed) { throw 'Verify the original worker with DiagnosticBefore first.' }
            $test = 'isolated_r7_partial_capture_explicit'
        } else {
            if ($Mode -ceq 'DiagnosticAfter') { [void](ReadJson (Join-Path $out 'Explicit-intent.json')) }
            $test = 'isolated_r7_partial_capture_diagnostic'
        }
    }
    RequirePlain $image $false
    if ((Get-FileHash -LiteralPath $image -Algorithm SHA256).Hash.ToLowerInvariant() -cne $manifest.testSha256) {
        throw 'Private same-image fixture digest differs.'
    }
    $info = NewFixtureProcess $image $base $test
    $stdoutPath = Join-Path $out "$Mode.stdout.log"
    $outFile = [IO.FileStream]::new($stdoutPath, 'CreateNew', 'Write', 'Read', 1)
    $errFile = [IO.FileStream]::new((Join-Path $out "$Mode.stderr.log"), 'CreateNew', 'Write', 'Read', 1)
    $process = [Diagnostics.Process]::Start($info)
    $result.pid = $process.Id
    $result.creationFiletime = $process.StartTime.ToUniversalTime().ToFileTimeUtc().ToString()
    WriteOnce "$Mode-creation.json" @{ pid=$result.pid; creationFiletime=$result.creationFiletime; image=$image; arguments=$info.Arguments }
    $copyOut = $process.StandardOutput.BaseStream.CopyToAsync($outFile)
    $copyErr = $process.StandardError.BaseStream.CopyToAsync($errFile)
    SendBootstrap $process $payload
    $process.StandardInput.Close()
    if ($Mode -ceq 'Host') {
        $nativeResult = Join-Path $base 'D\fixture-result.json'
        $observation = [Diagnostics.Stopwatch]::StartNew()
        $record = $null
        while (!$process.HasExited -and $null -eq $record -and $observation.Elapsed.TotalSeconds -lt 210) {
            try { $record = ReadJson $nativeResult }
            catch { $result.lastObservationError = $_.Exception.Message; Start-Sleep -Milliseconds 200 }
        }
        $result.observationMs = $observation.ElapsedMilliseconds
        if ($process.HasExited -and $null -eq $record) {
            try { $record = ReadJson $nativeResult }
            catch { $result.lastObservationError = $_.Exception.Message }
        }
        if ($null -ne $record) { $result.Remove('lastObservationError') }
        $result.reachedOriginalInjection = !$process.HasExited -and $record.injection.injected -eq $true -and $record.injection.raw -eq 31 -and
            $record.sameSampleStatus.payload.startWorker.rawError -eq 31 -and $record.sameSampleStatus.recovery -ceq 'Required' -and
            $record.sameSampleStatus.payload.startWorker.creatorResumed -eq $false -and $record.elapsedMs -le 30000 -and
            $record.statusElapsedMs -le 5000 -and $record.sameSampleStatus.sameSample -eq $true -and
            $record.injection.manifestIntentAndFileAbsent -eq $true -and $record.injection.currentIntentAndFileAbsent -eq $true -and
            $record.injection.originalCreatorNeverResumed -eq $true
        if ($null -eq $record) {
            if ($process.HasExited) {
                $result.firstError = @{ message='Fixture host exited before publishing its first native result.'; raw=$process.ExitCode }
            } else {
                $result.firstError = @{ message='Initial fixture observation timed out. Original action budgets remain unchanged.' }
            }
        } elseif (!$result.reachedOriginalInjection) {
            $result.firstError = @{ message=$record.firstError; status=$record.statusError }
        }
        $result.privateEvidence = @(foreach ($relative in @('D\fixture-preflight.json', 'D\fixture-result.json', 'preparation\image-created-verified.json')) {
            try {
                RequirePlain $base $true
                RequirePlain (Join-Path $base (Split-Path $relative)) $true
                $export = ExportExact (Join-Path $base $relative) ('Host-' + (Split-Path $relative -Leaf)) 1048576
                if ($relative -ceq 'D\fixture-result.json') {
                    Check (SameData $export.record $record) 'Original fixture result changed between observation and export.'
                }
                $export | Select-Object name, bytes, sha256
            } catch {
                [pscustomobject]@{ path=$relative; error=$_.Exception.Message; hresult=$_.Exception.HResult }
            }
        })
        $result.privateEvidenceComplete = @($result.privateEvidence | Where-Object { $_.sha256 -cmatch '^[0-9a-f]{64}$' }).Count -eq 3
        if (!$result.privateEvidenceComplete -and $result.reachedOriginalInjection) {
            $result.firstError = @{ message='Private fixture exports were incomplete.' }
        }
        $result.initialScansVerified = $false
        if ($result.reachedOriginalInjection) {
            try {
                do {
                    $result.initialScans = ReadInitialScans $stdoutPath
                    if ($null -ne $result.initialScans) { break }
                    Start-Sleep -Milliseconds 50
                } while (!$process.HasExited -and $observation.Elapsed.TotalSeconds -lt 210)
                Check ($null -ne $result.initialScans) 'Initial Start full scan reports are missing.'
                CheckInitialScans $result.initialScans
                $result.initialScansVerified = $true
            } catch {
                $result.initialScanError = $_.Exception.Message
                if (!$result.firstError) { $result.firstError = @{ message=$result.initialScanError } }
            }
        }
        WriteOnce 'Host-ready.json' @{ pid=$result.pid; creationFiletime=$result.creationFiletime; reachedOriginalInjection=$result.reachedOriginalInjection; privateEvidenceComplete=$result.privateEvidenceComplete; initialScansVerified=$result.initialScansVerified; nativeResult=$record }
        $result.passed = $result.reachedOriginalInjection -and $result.privateEvidenceComplete -and $result.initialScansVerified
        if ($result.passed) { Write-Host 'Original worker retained. Leave this window open for the separate diagnostic.' }
        else { Write-Host 'Initial Start prerequisite failed. Preserve this window and do not rerun.' }
        WriteOnce 'Host-result.json' $result
        $process.WaitForExit()
        [Threading.Tasks.Task]::WaitAll(@($copyOut, $copyErr))
        WriteOnce 'Host-exit.json' @{ pid=$result.pid; raw=$process.ExitCode; expected=$false }
        exit 1
    }
    if (!$process.WaitForExit(15000)) { throw 'Manager observation timed out; original action deadline remains unchanged.' }
    [Threading.Tasks.Task]::WaitAll(@($copyOut, $copyErr))
    $outFile.Flush($true)
    $errFile.Flush($true)
    $result.raw = $process.ExitCode
    $line = Get-Content -LiteralPath (Join-Path $out "$Mode.stdout.log") -Encoding UTF8 | Where-Object { $_ -like '*r7-partial-manager=*' }
    if (@($line).Count -ne 1) { throw 'Exact manager reply row missing.' }
    $result.manager = ($line -split 'r7-partial-manager=', 2)[1] | ConvertFrom-Json
    $reply = $result.manager.result.response
    $facts = $reply.facts
    $injection = $ready.nativeResult.injection
    Check ($result.raw -eq 0 -and $null -ne $reply -and $reply.publicStartOpen -eq $false -and
        $reply.normalStop -eq $false -and $reply.closed -eq $false -and $reply.runtimeAuthority -ne $true) 'Manager transport or refusal authority differs.'
    if ($Mode -ceq 'Explicit') {
        $prefix = 'original recovery retained/unknown; worker DATA='
        Check ($result.manager.elapsedMs -le 10000 -and $result.manager.result.refused -eq $true -and
            $reply.purpose -ceq 'refused' -and $reply.error.detail.StartsWith($prefix)) 'Explicit request did not preserve the actual canonical-abort refusal.'
        $snapshot = $reply.error.detail.Substring($prefix.Length) | ConvertFrom-Json
        CheckPendingWorker $snapshot $injection
    } else {
        Check ($result.manager.elapsedMs -le 5000 -and $result.manager.result.refused -eq $false -and
            $facts.sameSampleStatus.sameSample -eq $true -and $facts.sameSampleStatus.recovery -ceq 'Required') 'Diagnostic did not observe Required within the original Status budget.'
        CheckOriginalWorker $facts.sameSampleStatus.payload.startWorker $injection
        if ($Mode -ceq 'DiagnosticBefore') {
            CheckOriginalWorker $facts.snapshot $injection
            Check ($null -eq $facts.sameSampleStatus.payload.recoveryWorker) 'Recovery was already present at the first diagnostic.'
        } else {
            CheckPendingWorker $facts.snapshot $injection
            $sample = $facts.sameSampleStatus.payload.recoveryWorker
            Check ($sample.actionId -ceq $facts.snapshot.actionId -and $sample.workerThread -ceq $facts.snapshot.workerThread -and
                (SameData $sample.preparationPending $facts.snapshot.preparationPending)) 'Same-sample Status does not confirm this Pending record.'
            $explicit = ReadJson (Join-Path $out 'Explicit-result.json')
            Check ($explicit.passed -eq $true -and $explicit.recoveryActionId -ceq $facts.snapshot.actionId) 'Explicit result and subsequent diagnostic action differ.'
            $imageProof = ReadJson (Join-Path $out 'Host-image-created-verified.json')
            $result.attachment = CheckAttachment $facts.snapshot $injection $imageProof
            $result.preparationScans = ReadPreparationScans (Join-Path $out 'Host.stdout.log')
            CheckPreparationScans $result.preparationScans $injection.failedActionId $imageProof.current.imageSha256
        }
    }
    if ($Mode -ceq 'Explicit') { $result.recoveryActionId = $snapshot.actionId }
    $result.passed = $true
} catch {
    $result.firstError = @{ message=$_.Exception.Message; hresult=$_.Exception.HResult; line=$_.InvocationInfo.ScriptLineNumber }
    if ($null -ne $process -and $process.HasExited) { $result.raw = $process.ExitCode }
    if (!$stageReady) { throw }
} finally {
    if ($null -ne $stage) {
        $stage.StandardInput.Close()
        [void]$stage.WaitForExit(1000)
        $stage.Dispose()
    }
    if ($null -ne $outFile) { $outFile.Dispose() }
    if ($null -ne $errFile) { $errFile.Dispose() }
    if ($null -ne $process) { $process.Dispose() }
}
if ($Mode -cne 'Host' -or !(Test-Path -LiteralPath (Join-Path $out 'Host-result.json'))) { WriteOnce "$Mode-result.json" $result }
if ($result.passed) { exit 0 }
exit 1

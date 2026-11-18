param(
    [Parameter(Mandatory)][ValidateSet('Host','DiagnosticBefore','Explicit','DiagnosticAfter')][string]$Mode
)
$ErrorActionPreference = 'Stop'
$public = 'C:\Users\Public\ocx-r7-partial-owner-r1'
$profile = 'C:\Users\ocxr7owner'
$base = Join-Path $profile 'ocx024-r7-partial-owner-r1'
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

function RequireAbsent([string]$Path) {
    try { [void][IO.File]::GetAttributes($Path) }
    catch [IO.FileNotFoundException] { return }
    catch [IO.DirectoryNotFoundException] { return }
    throw "Fixture path already present: $Path"
}

function RequirePlain([string]$Path, [bool]$Directory) {
    $item = Get-Item -LiteralPath $Path -Force
    if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $item.PSIsContainer -ne $Directory) {
        throw "Fixture locator is not the expected plain object: $Path"
    }
}

function CopyExact([string]$Source, [string]$Target) {
    $from = [IO.File]::Open($Source, 'Open', 'Read', 'Read')
    try {
        $to = [IO.File]::Open($Target, 'CreateNew', 'Write', 'Read')
        try { $from.CopyTo($to); $to.Flush($true) } finally { $to.Dispose() }
    } finally { $from.Dispose() }
}

if ($PSScriptRoot -cne $public) { throw 'Use the fixed staged fixture root.' }
foreach ($folder in @('C:\Users', 'C:\Users\Public', $public, $out, (Join-Path $public 'input'), $profile)) {
    RequirePlain $folder $true
}
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$session = [Diagnostics.Process]::GetCurrentProcess().SessionId
$admin = [Security.Principal.WindowsPrincipal]::new($identity).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
$local = Get-LocalUser -Name 'ocxr7owner'
if (!$local.Enabled -or $local.SID.Value -cne $identity.User.Value -or $admin -or $session -le 0 -or $env:USERPROFILE -cne $profile) {
    throw 'Use the dedicated ocxr7owner ordinary interactive desktop.'
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
WriteOnce "$Mode-intent.json" @{ mode=$Mode; sid=$identity.User.Value; session=$session; testSha256=$manifest.testSha256; publicStartOpen=$false }

$result = @{ mode=$Mode; passed=$false; publicStartOpen=$false; normalStop=$false; closed=$false }
$process = $null
$outFile = $null
$errFile = $null
try {
    if ($Mode -ceq 'Host') {
        $targets = @($base, (Join-Path $profile '.codex'), (Join-Path $profile '.grok'), (Join-Path $profile 'AppData\Local\OpenCodex'))
        $facts = foreach ($target in $targets) {
            try { RequireAbsent $target; @{ path=$target; absent=$true } }
            catch { @{ path=$target; absent=$false; error=$_.Exception.Message; hresult=$_.Exception.HResult } }
        }
        WriteOnce 'Host-absence.json' @{ paths=@($facts) }
        if ($facts | Where-Object { !$_.absent }) { throw 'Fixture targets must be absent before preparation.' }
        if ((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -cne $manifest.testSha256 -or
            (Get-FileHash -LiteralPath $bundle -Algorithm SHA256).Hash.ToLowerInvariant() -cne $manifest.bundleSha256) {
            throw 'Fixed input digest differs.'
        }
        foreach ($path in @($base, (Join-Path $base 'I'), (Join-Path $base 'D'), (Join-Path $base 'H'))) {
            RequireAbsent $path
            $security = [Security.AccessControl.DirectorySecurity]::new()
            $security.SetSecurityDescriptorSddlForm(('O:{0}D:P(A;OICI;FA;;;{0})(A;OICI;FA;;;SY)' -f $identity.User.Value))
            [void][IO.Directory]::CreateDirectory($path, $security)
            RequirePlain $path $true
        }
        CopyExact $source $image
        $test = 'isolated_r7_partial_capture_host'
    } else {
        $ready = ReadJson (Join-Path $out 'Host-ready.json')
        if (!$ready.reachedOriginalInjection -or !$ready.privateEvidenceComplete) { throw 'Original worker injection and private evidence must be proven first.' }
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
    $config = @{ schema=1; scope='isolated-r7-partial-capture-original-worker'; expectedSid=$identity.User.Value; expectedSession=$session; profile=$profile;
        installation=(Join-Path $base 'I'); data=(Join-Path $base 'D'); home=(Join-Path $base 'H'); brokerSha256=$manifest.testSha256; managerSha256=$manifest.testSha256 }
    $payload = $utf8.GetBytes(($config | ConvertTo-Json -Compress))
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $image
    $info.Arguments = "--ignored --exact infrastructure::windows_owner_host024::partial_capture_tests::$test --nocapture --test-threads=1"
    $info.WorkingDirectory = $base
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardInput = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $outFile = [IO.FileStream]::new((Join-Path $out "$Mode.stdout.log"), 'CreateNew', 'Write', 'Read', 1)
    $errFile = [IO.FileStream]::new((Join-Path $out "$Mode.stderr.log"), 'CreateNew', 'Write', 'Read', 1)
    $process = [Diagnostics.Process]::Start($info)
    $result.pid = $process.Id
    $result.creationFiletime = $process.StartTime.ToUniversalTime().ToFileTimeUtc().ToString()
    WriteOnce "$Mode-creation.json" @{ pid=$result.pid; creationFiletime=$result.creationFiletime; image=$image; arguments=$info.Arguments }
    $copyOut = $process.StandardOutput.BaseStream.CopyToAsync($outFile)
    $copyErr = $process.StandardError.BaseStream.CopyToAsync($errFile)
    $pipe = $process.StandardInput.BaseStream
    $length = [BitConverter]::GetBytes([uint32]$payload.Length)
    $pipe.Write($length, 0, $length.Length)
    $pipe.Write($payload, 0, $payload.Length)
    $pipe.Flush()
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
        WriteOnce 'Host-ready.json' @{ pid=$result.pid; creationFiletime=$result.creationFiletime; reachedOriginalInjection=$result.reachedOriginalInjection; privateEvidenceComplete=$result.privateEvidenceComplete; nativeResult=$record }
        $result.passed = $result.reachedOriginalInjection -and $result.privateEvidenceComplete
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
        }
    }
    if ($Mode -ceq 'Explicit') { $result.recoveryActionId = $snapshot.actionId }
    $result.passed = $true
} catch {
    $result.firstError = @{ message=$_.Exception.Message; hresult=$_.Exception.HResult; line=$_.InvocationInfo.ScriptLineNumber }
    if ($null -ne $process -and $process.HasExited) { $result.raw = $process.ExitCode }
} finally {
    if ($null -ne $outFile) { $outFile.Dispose() }
    if ($null -ne $errFile) { $errFile.Dispose() }
    if ($null -ne $process) { $process.Dispose() }
}
if ($Mode -cne 'Host' -or !(Test-Path -LiteralPath (Join-Path $out 'Host-result.json'))) { WriteOnce "$Mode-result.json" $result }
if ($result.passed) { exit 0 }
exit 1

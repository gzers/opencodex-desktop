param([Parameter(Mandatory)][string]$Script)
$ErrorActionPreference = 'Stop'
$tokens = $null
$errors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile((Resolve-Path $Script), [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw ($errors | Out-String) }
foreach ($node in $ast.EndBlock.Statements) {
    if ($node -is [Management.Automation.Language.FunctionDefinitionAst]) {
        . ([scriptblock]::Create($node.Extent.Text))
    }
}
$utf8 = [Text.UTF8Encoding]::new($false)
$base = Join-Path $PSScriptRoot 'fixture'
$out = Join-Path $base 'out'
$cases = 0
function Reject([scriptblock]$Block, [string]$Expected) {
    $message = $null
    try { & $Block } catch { $message = $_.Exception.Message }
    Check ($message -like $Expected) "Expected '$Expected'; received '$message'."
    $script:cases++
}
try {
    RequireAbsent $base
    $action = '33333333-3333-4333-8333-333333333333'
    $failed = '22222222-2222-4222-8222-222222222222'
    $capture = '11111111-1111-4111-8111-111111111111'
    $dir = Join-Path $base "D\before-resume-pending-$action"
    [void][IO.Directory]::CreateDirectory($dir)
    [void][IO.Directory]::CreateDirectory($out)
    $injection = [pscustomobject]@{ workerThread='ThreadId(8)'; failedActionId=$failed; ownerSessionId='session'
        hostInstanceId='host'; scopeDigest=('f'*64); captureSessionId=$capture; sourceBindingDigest=('a'*64)
        originalClaimAndNlkVerified=[pscustomobject]@{epoch='original';claimKey=('5'*64);objectsDigest=('6'*64)} }
    $worker = [pscustomobject]@{ thread=$injection.workerThread; actionId=$failed; sessionId=$injection.ownerSessionId
        scopeDigest=$injection.scopeDigest; rawError=31; creatorResumed=$false; phase='FailedRetained' }
    CheckOriginalWorker $worker $injection
    $cases++
    $worker.rawError = 5
    Reject { CheckOriginalWorker $worker $injection } 'Original Start worker binding or first error differs.'
    $worker.rawError = 31
    $snapshot = [pscustomobject]@{startWorker=$worker;workerThread=$worker.thread;sessionId=$worker.sessionId
        scopeDigest=$worker.scopeDigest;actionId=$action;recoveryRequired=$true;normalStop=$false;closed=$false;runtimeAuthority=$false
        preparationPending=[pscustomobject]@{attachmentAttempted=$true;durable=$true;startRawError=31;attachmentRawError=$null
            attachmentErrorKind=$null;originalRanges=$injection.originalClaimAndNlkVerified;intentDigest=$null;pendingDigest=$null} }
    CheckPendingWorker $snapshot $injection
    $cases++
    $snapshot.workerThread = 'ThreadId(10)'
    Reject { CheckPendingWorker $snapshot $injection } 'Independent recovery was not retained on the original thread and scope.'
    $snapshot.workerThread = $worker.thread
    $snapshot.preparationPending.durable = $false
    Reject { CheckPendingWorker $snapshot $injection } 'Durable Pending or original claim/NLK continuity is unproven.'
    $snapshot.preparationPending.durable = $true
    $image = [pscustomobject]@{ current=[pscustomobject]@{imageSha256=('b'*64)} }
    $reference = [ordered]@{kind='preparation';ownerInstanceId='host';source=[ordered]@{state='captured';bindingDigest=('a'*64);imageDigest=('b'*64)}
        intentPrefixDigest=('c'*64);nativeStagePrefixDigest=('d'*64);lastStage='handleRetained'
        components=@('source','custody','codex','grok','journal') | ForEach-Object { [ordered]@{role=$_;state='retained';prefixDigest=$null;objects=@()} } }
    $record = [ordered]@{schema='windows-before-resume-recovery-v2';sessionId=$capture;failedActionId=$failed;recoveryActionId=$action
        securityPolicyDigest=('e'*64);reference=$reference;sequence=1;previousRecordDigest=$null;startedQpc='1000';deadlineQpc='11000'
        qpcFrequency='1000';event=[ordered]@{kind='AbortRecoveryIntent'} }
    $one = Join-Path $dir 'v2-record-000001.json'
    $two = Join-Path $dir 'v2-record-000002.json'
    [IO.File]::WriteAllText($one, ($record | ConvertTo-Json -Depth 15), $utf8)
    $snapshot.preparationPending.intentDigest = (Get-FileHash -LiteralPath $one).Hash.ToLowerInvariant()
    $record.sequence = 2
    $record.previousRecordDigest = $snapshot.preparationPending.intentDigest
    $record.event = [ordered]@{kind='AbortRecoveryPending';stage='handleRetained';rawError=31}
    [IO.File]::WriteAllText($two, ($record | ConvertTo-Json -Depth 15), $utf8)
    $snapshot.preparationPending.pendingDigest = (Get-FileHash -LiteralPath $two).Hash.ToLowerInvariant()
    $rows = CheckAttachment $snapshot $injection $image
    Check ($rows.Count -eq 2 -and $rows[0].sha256 -ceq $snapshot.preparationPending.intentDigest -and
        (Get-FileHash -LiteralPath (Join-Path $out $rows[1].name)).Hash.ToLowerInvariant() -ceq $snapshot.preparationPending.pendingDigest) 'Original byte export differs.'
    $cases++
    Reject { ExportExact $two 'exceeds-limit.json' 1 } 'Private fixture evidence exceeds its byte limit.'
    $native = $null
    try { ExportExact $two $rows[1].name 65536 } catch { $native = $_.Exception.GetBaseException() }
    Check ($native -is [IO.IOException] -and ($native.HResult -band 65535) -eq 80) 'Expected ERROR_FILE_EXISTS from CreateNew.'
    $cases++
    $extra = Join-Path $dir 'extra.json'
    [IO.File]::WriteAllText($extra, '{}', $utf8)
    Reject { CheckAttachment $snapshot $injection $image } 'Pending directory has unexpected membership.'
    [IO.File]::Delete($extra)
    $out = Join-Path $base 'tampered'
    [void][IO.Directory]::CreateDirectory($out)
    $record.previousRecordDigest = '9'*64
    [IO.File]::WriteAllText($two, ($record | ConvertTo-Json -Depth 15), $utf8)
    Reject { CheckAttachment $snapshot $injection $image } 'Pending record event or exact file digest chain differs.'
    $out = Join-Path $base 'expired'
    [void][IO.Directory]::CreateDirectory($out)
    $record.deadlineQpc = '11001'
    [IO.File]::WriteAllText($two, ($record | ConvertTo-Json -Depth 15), $utf8)
    Reject { CheckAttachment $snapshot $injection $image } 'Pending clock or policy differs.'
    $passed = $true
} finally {
    if ($passed) {
        $resolved = (Resolve-Path -LiteralPath $base).ProviderPath
        if ($resolved -cne (Join-Path $PSScriptRoot 'fixture')) { throw 'Fixture cleanup path differs.' }
        Remove-Item -LiteralPath $resolved -Recurse
        RequireAbsent $resolved
        @{passed=$cases;powerShell=$PSVersionTable.PSVersion.ToString();fixture=$resolved;fixtureAbsent=$true} | ConvertTo-Json -Compress
    }
}

$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskSource='493da80675e6508e46e0fd83d86ee7216abf96fe'
$taskDestination=Join-Path $taskCode 'test\software\candidate-493da80-38673288645'
foreach ($taskOutputName in @('msi-extraction.stdout.original.json','msi-extraction.stderr.original.txt',
    'msi-readonly.original.json','installer-verification.raw.original.json',
    'candidate-environment.raw.original.json','candidate-runner-evidence.original.zip',
    'local-payload-readonly-check.json','candidate-local-postcheck.json')) {
    if (Test-Path -LiteralPath (Join-Path $PSScriptRoot $taskOutputName)) {
        throw 'Prior finalization exists; inspect original files, never overwrite them'
    }
}
$taskDownload=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'candidate-download-result.json') -Raw | ConvertFrom-Json
$taskTerminal=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'candidate-terminal.json') -Raw | ConvertFrom-Json
if ($taskDownload.source -cne $taskSource -or $taskDownload.runId -ne 38673288645 -or
    $taskTerminal.run.head_sha -cne $taskSource -or $taskTerminal.run.conclusion -cne 'success' -or
    $taskDownload.destination -cne $taskDestination) { throw 'Exact completed candidate identity required' }
if ((& git -C $taskCode rev-parse HEAD).Trim() -cne $taskSource -or
    @(& git -C $taskCode status --porcelain).Count) { throw 'Code source must remain exact and clean' }
if (@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count) {
    throw 'Read-only finalization requires zero native test products'
}
$taskReceipts=@(Get-ChildItem -LiteralPath (Join-Path $taskDestination 'installers') -Filter installer-verification.json -Recurse -File)
$taskEnvironments=@(Get-ChildItem -LiteralPath (Join-Path $taskDestination 'evidence') -Filter environment.json -Recurse -File)
if ($taskReceipts.Count -ne 1 -or $taskEnvironments.Count -ne 1) { throw 'Expected exactly one runner receipt/environment' }
$taskReceipt=Get-Content -LiteralPath $taskReceipts[0].FullName -Raw | ConvertFrom-Json
if ($taskReceipt.sourceCommit -cne $taskSource -or $taskReceipt.result -cne 'pass') { throw 'Receipt identity mismatch' }
$taskMsiRows=@($taskReceipt.installers | Where-Object kind -CEQ msi)
if ($taskMsiRows.Count -ne 1) { throw 'Expected exactly one MSI receipt' }
$taskMsiFiles=@(Get-ChildItem -LiteralPath (Join-Path $taskDestination 'installers') -Recurse -File |
    Where-Object Name -CEQ $taskMsiRows[0].name)
if ($taskMsiFiles.Count -ne 1) { throw 'Expected exactly one verified MSI' }
$taskExtract=Join-Path $taskDestination 'msi-readonly'
if (Test-Path -LiteralPath $taskExtract) { throw 'Prior extraction exists; inspect it without overwriting' }
& (Join-Path $PSScriptRoot 'msi-readonly.ps1') -MsiPath $taskMsiFiles[0].FullName `
    -OutputDirectory $taskExtract -AllowedOutputRoot $taskDestination -ExpectedSha256 $taskMsiRows[0].sha256 `
    -ExtractExecutable > (Join-Path $PSScriptRoot 'msi-extraction.stdout.original.json') `
    2> (Join-Path $PSScriptRoot 'msi-extraction.stderr.original.txt')
$taskCopies=@(
    @((Join-Path $taskExtract 'msi-readonly.json'),'msi-readonly.original.json'),
    @($taskReceipts[0].FullName,'installer-verification.raw.original.json'),
    @($taskEnvironments[0].FullName,'candidate-environment.raw.original.json')
)
foreach ($taskCopy in $taskCopies) {
    $taskTarget=Join-Path $PSScriptRoot $taskCopy[1]
    if (Test-Path -LiteralPath $taskTarget) { throw 'Do not overwrite archived original bytes' }
    [IO.File]::WriteAllBytes($taskTarget,[IO.File]::ReadAllBytes($taskCopy[0]))
}
$taskRunnerArtifact=@($taskDownload.artifacts | Where-Object name -CEQ 'windows-candidate-evidence')
if ($taskRunnerArtifact.Count -ne 1) { throw 'Evidence artifact identity missing' }
$taskEvidenceZip=Join-Path $taskDestination ($taskRunnerArtifact[0].id.ToString()+'.zip')
if ((Get-FileHash -LiteralPath $taskEvidenceZip).Hash.ToLowerInvariant() -cne $taskRunnerArtifact[0].sha256) {
    throw 'Evidence ZIP no longer matches API digest'
}
$taskZipTarget=Join-Path $PSScriptRoot 'candidate-runner-evidence.original.zip'
if (Test-Path -LiteralPath $taskZipTarget) { throw 'Do not overwrite the archived runner ZIP' }
[IO.File]::WriteAllBytes($taskZipTarget,[IO.File]::ReadAllBytes($taskEvidenceZip))
$taskExe=Join-Path $taskExtract 'opencodex-desktop.exe'
$taskHash=(Get-FileHash -LiteralPath $taskExe).Hash.ToLowerInvariant()
if ($taskHash -cne $taskMsiRows[0].installedExecutableSha256) { throw 'Extracted EXE differs from exact runner MSI payload' }
$taskBytes=[IO.File]::ReadAllBytes($taskExe)
if (![Text.Encoding]::UTF8.GetString($taskBytes).Contains($taskSource)) { throw 'Build source literal missing' }
$taskIdentity=[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskSource
    executable=$taskExe;applicationSha256=$taskHash;applicationBytes=$taskBytes.Length
    applicationVersion=(Get-Item -LiteralPath $taskExe).VersionInfo.ProductVersion
    applicationSourceLiteralFound=$true;applicationExecuted=$false;installerExecuted=$false
}
$taskIdentity | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'local-payload-readonly-check.json') -Encoding utf8
$taskProtectedExpected=[ordered]@{
    'preferences.json'='8498f7cf8984c74a555202027800027fa7fd1366e0e5ce51c455aaf3ba1fdb7a'
    'data-root.json'='8ae75370e9f9966182caddd12ef094f16449c88119e66012a8a4a99a0c1f0e25'
    'backup-cleanup-policy.json'='85884ab649b82b6b4ab880fe24492c6c400a266254277e29e78c8080b20d2e11'
    'update-schedule.json'='045991fb0a7a71604632e5c9255c3fe008e8e5f21c7e28feabe0d4178bc4ffd2'
}
$taskProtectedAfter=[ordered]@{}
foreach ($taskName in $taskProtectedExpected.Keys) {
    $taskProtectedPath=Join-Path $taskCode ('test\runtime\windows0110\data\manager-state\'+$taskName)
    $taskProtectedAfter[$taskName]=(Get-FileHash -LiteralPath $taskProtectedPath).Hash.ToLowerInvariant()
    if ($taskProtectedAfter[$taskName] -cne $taskProtectedExpected[$taskName]) { throw 'Protected sandbox changed' }
}
$taskDailyHash=(Get-FileHash -LiteralPath 'D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe').Hash.ToLowerInvariant()
if ($taskDailyHash -cne '43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67') { throw 'Daily EXE changed' }
$taskDocs='D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop'
$taskTaskHash=(Get-FileHash -LiteralPath (Join-Path $taskDocs '.adg\work\objects\task\TASK-OPENCODEX-DESKTOP-182.json')).Hash.ToLowerInvariant()
if ($taskTaskHash -cne 'bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93') { throw 'Original TASK changed' }
$taskNative=@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'" |
    Select-Object ProcessId,Name,CreationDate,ExecutablePath)
if ($taskNative.Count) { throw 'Native product appeared during finalization; inspect before accepting checkpoint' }
$taskPost=[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskSource
    nativeCount=$taskNative.Count;nativeProcesses=$taskNative
    codeWorktreeClean=(@(& git -C $taskCode status --porcelain).Count -eq 0)
    dailyExeUnchanged=$true;dailyExeSha256=$taskDailyHash
    protectedSandboxUnchanged=$true;protectedSandboxHashes=$taskProtectedAfter;originalTaskSha256=$taskTaskHash
    localApplicationStarted=$false;localInstallerExecuted=$false;foregroundInputIssued=$false
    historicalFailuresCleared=$false;fullVersionGatePassed=$false
}
$taskPost | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'candidate-local-postcheck.json') -Encoding utf8
$taskIdentity | ConvertTo-Json -Depth 5
$taskPost | ConvertTo-Json -Depth 5

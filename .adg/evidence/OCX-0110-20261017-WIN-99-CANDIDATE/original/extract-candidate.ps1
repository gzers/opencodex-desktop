param()
$ErrorActionPreference='Stop'
$taskSource='99a62e4ad35e9404cb08922571ad8b77f89214dd'
$taskRunId=39007300430
$taskRun=$PSScriptRoot
$taskDownload=Get-Content -LiteralPath (Join-Path $taskRun 'candidate-download-result.json') -Raw | ConvertFrom-Json
$taskReceipt=Get-Content -LiteralPath (Join-Path $taskRun 'candidate-installer-verification.original.json') -Raw | ConvertFrom-Json
$taskPackageLog=Get-Content -LiteralPath (Join-Path $taskRun 'candidate-package-log-verification.json') -Raw | ConvertFrom-Json
$taskCi=Get-Content -LiteralPath (Join-Path $taskRun 'ci-complete-admission.json') -Raw | ConvertFrom-Json
if($taskDownload.source -cne $taskSource -or $taskDownload.runId -ne $taskRunId -or
   $taskReceipt.sourceCommit -cne $taskSource -or $taskReceipt.result -cne 'pass' -or
   $taskPackageLog.source -cne $taskSource -or $taskPackageLog.actualCheckout -cne $taskSource -or
   $taskPackageLog.passed -ne 654 -or $taskPackageLog.failed -ne 0 -or
   $taskCi.source -cne $taskSource -or !$taskCi.wholeToolCiAdmitted){throw 'Exact source/CI/package admission differs'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Require native0'}
$taskDescriptor=Join-Path $taskRun 'candidate-descriptor.json'
if(Test-Path -LiteralPath $taskDescriptor){throw 'Descriptor exists; inspect instead of extracting again'}
$taskAllowed='C:\Users\15119\.codex\worktrees\windows-mesh-initial-size\opencodex-desktop\test\software'
$taskDestination=[IO.Path]::GetFullPath([string]$taskDownload.destination)
if($taskDestination -cne (Join-Path $taskAllowed 'candidate-99a62e-39007300430')){throw 'Exact owned candidate destination required'}
$taskCursor=Get-Item -LiteralPath $taskDestination
while($taskCursor){
    if($taskCursor.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Candidate path alias refused'}
    $taskCursor=$taskCursor.Parent
}
$taskMsiRow=@($taskReceipt.installers | Where-Object kind -CEQ 'msi')
$taskNssRow=@($taskReceipt.installers | Where-Object kind -CEQ 'nsis')
if($taskMsiRow.Count -ne 1 -or $taskNssRow.Count -ne 1){throw 'Exactly one MSI and NSIS receipt required'}
$taskMsi=@(Get-ChildItem -LiteralPath (Join-Path $taskDestination 'installers') -Recurse -File -Filter $taskMsiRow[0].name)
if($taskMsi.Count -ne 1){throw 'Exact MSI missing or ambiguous'}
if((Get-FileHash -LiteralPath $taskMsi[0].FullName).Hash.ToLowerInvariant() -cne $taskMsiRow[0].sha256){throw 'MSI digest differs'}
$taskMethod='D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261015-WIN-02-CANDIDATE\msi-readonly.ps1'
$taskOutput=Join-Path $taskDestination 'msi-readonly'
if(Test-Path -LiteralPath $taskOutput){throw 'Extraction directory exists; preserve it for inspection'}
& $taskMethod -MsiPath $taskMsi[0].FullName -OutputDirectory $taskOutput -AllowedOutputRoot $taskDestination -ExpectedSha256 $taskMsiRow[0].sha256 -ExtractExecutable | Out-Null
$taskReadback=Join-Path $taskOutput 'msi-readonly.json'
Copy-Item -LiteralPath $taskReadback -Destination (Join-Path $taskRun 'candidate-msi-readonly.original.json')
$taskExtract=Get-Content -LiteralPath $taskReadback -Raw | ConvertFrom-Json
$taskExe=@($taskExtract.extractedExecutables | Where-Object name -CEQ 'opencodex-desktop.exe')
if($taskExtract.result -cne 'pass' -or $taskExtract.installerExecuted -or $taskExe.Count -ne 1 -or
   $taskExe[0].productVersion -cne '0.1.10' -or
   $taskExe[0].sha256 -cne $taskMsiRow[0].installedExecutableSha256 -or
   $taskExe[0].sha256 -cne $taskMsiRow[0].expectedInstalledExecutableSha256){throw 'Readonly payload differs from runner installed bytes'}
$taskBytes=[IO.File]::ReadAllBytes($taskExe[0].path)
$taskPrefix=[Text.Encoding]::ASCII.GetBytes('__TAURI_BUNDLE_TYPE_VAR_')
$taskOffset=[int]$taskMsiRow[0].bundleMarkerOffset
if($taskOffset -ne [int]$taskNssRow[0].bundleMarkerOffset -or
   $taskOffset -lt 0 -or $taskOffset+$taskPrefix.Length+3 -gt $taskBytes.Length){throw 'Bundle marker offset differs'}
$taskMsiMarker=[Text.Encoding]::ASCII.GetString($taskBytes,$taskOffset,$taskPrefix.Length+3)
if($taskMsiMarker -cne '__TAURI_BUNDLE_TYPE_VAR_MSI'){throw 'MSI marker at admitted offset differs'}
$taskVariant=[byte[]]$taskBytes.Clone()
[Text.Encoding]::ASCII.GetBytes('NSS').CopyTo($taskVariant,$taskOffset+$taskPrefix.Length)
$taskNssHash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($taskVariant)).ToLowerInvariant()
if($taskNssHash -cne $taskNssRow[0].installedExecutableSha256 -or
   $taskNssHash -cne $taskNssRow[0].expectedInstalledExecutableSha256){throw 'Exact in-memory NSIS suffix variant differs'}
[Text.Encoding]::ASCII.GetBytes('UNK').CopyTo($taskVariant,$taskOffset+$taskPrefix.Length)
$taskBuiltHash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($taskVariant)).ToLowerInvariant()
if($taskBuiltHash -cne $taskReceipt.builtExecutableSha256){throw 'Exact in-memory unpatched bundle variant differs'}
if(![Text.Encoding]::UTF8.GetString($taskBytes).Contains($taskSource)){throw 'Expected source provenance literal missing'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native appeared during readonly extraction'}
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskSource;runId=$taskRunId
    executable=$taskExe[0].path;sha256=$taskExe[0].sha256;exeSha256=$taskExe[0].sha256;bytes=$taskExe[0].bytes
    productVersion=$taskExe[0].productVersion;msiPackageSha256=$taskMsiRow[0].sha256
    markerStartOffset=$taskOffset;inMemoryNssVariantSha256=$taskNssHash;inMemoryBuiltVariantSha256=$taskBuiltHash
    exactThreeByteBundleSuffixVariantsPassed=$true;sourceLiteralAloneIsNotProvenance=$true
    msiReadonlyMethodSha256=(Get-FileHash -LiteralPath $taskMethod).Hash.ToLowerInvariant()
    sameSourceCiAndArtifactVerified=$true;runnerMsiPayloadHashMatched=$true
    installerExecuted=$false;productStarted=$false;nativeCount=0;originalExeWritten=$false
    memoryBudgetPassed=$false;fullVersionGatePassed=$false;releaseRequested=$false
} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $taskDescriptor -Encoding utf8
Get-Content -LiteralPath $taskDescriptor

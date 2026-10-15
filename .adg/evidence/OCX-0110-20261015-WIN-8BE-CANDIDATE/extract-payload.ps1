$ErrorActionPreference='Stop'
$taskSource='8be87e2d54a895d4fe02e31300100a1ffed90121'
$taskDownload=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'candidate-download-result.json') -Raw | ConvertFrom-Json
$taskReceipt=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'installer-verification.raw.original.json') -Raw | ConvertFrom-Json
if($taskDownload.source -cne $taskSource -or $taskReceipt.sourceCommit -cne $taskSource){throw 'New candidate source mismatch'}
$taskDescriptor=Join-Path $PSScriptRoot 'candidate-descriptor.json'
if(Test-Path -LiteralPath $taskDescriptor){throw 'New payload already extracted; inspect instead of repeating'}
$taskMsiRows=@($taskReceipt.installers | Where-Object kind -CEQ 'msi')
if($taskMsiRows.Count -ne 1){throw 'Expected one precise MSI receipt'}
$taskMsi=@(Get-ChildItem -LiteralPath (Join-Path $taskDownload.destination 'installers') -Recurse -File -Filter $taskMsiRows[0].name)
if($taskMsi.Count -ne 1){throw 'Verified MSI missing or ambiguous'}
$taskMethod=Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-02-CANDIDATE/msi-readonly.ps1'
$taskOutput=Join-Path $taskDownload.destination 'msi-readonly'
& $taskMethod -MsiPath $taskMsi[0].FullName -OutputDirectory $taskOutput -AllowedOutputRoot $taskDownload.destination -ExpectedSha256 $taskMsiRows[0].sha256 -ExtractExecutable | Out-Null
Copy-Item -LiteralPath (Join-Path $taskOutput 'msi-readonly.json') -Destination (Join-Path $PSScriptRoot 'msi-readonly.original.json')
$taskExtract=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'msi-readonly.original.json') -Raw | ConvertFrom-Json
$taskExe=@($taskExtract.extractedExecutables | Where-Object name -CEQ 'opencodex-desktop.exe')
if($taskExtract.result -cne 'pass' -or $taskExtract.installerExecuted -or $taskExe.Count -ne 1 -or
    $taskExe[0].productVersion -cne '0.1.10' -or $taskExe[0].sha256 -cne $taskMsiRows[0].installedExecutableSha256){
    throw 'Readonly payload does not match the exact runner-installed MSI'
}
$taskData=[IO.File]::ReadAllBytes($taskExe[0].path)
if(![Text.Encoding]::UTF8.GetString($taskData).Contains($taskSource)){throw 'Application provenance literal missing'}
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskSource;runId=$taskDownload.runId
    executable=$taskExe[0].path;sha256=$taskExe[0].sha256;bytes=$taskExe[0].bytes
    sameShaCiAndArtifactVerified=$true;runnerMsiPayloadHashMatched=$true
    sourceLiteralAloneIsNotProvenance=$true;msiReadonlyMethodSha256=(Get-FileHash -LiteralPath $taskMethod).Hash.ToLowerInvariant()
    installerExecuted=$false;productStarted=$false;memoryBudgetPassed=$false;releaseRequested=$false
} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $taskDescriptor -Encoding utf8
Get-Content -LiteralPath $taskDescriptor

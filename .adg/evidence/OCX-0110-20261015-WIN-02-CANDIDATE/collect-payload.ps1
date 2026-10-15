$ErrorActionPreference = 'Stop'
$taskSource = '02a0d4a8cbae8ef1450908d86b2c75fcad65f04f'
$taskDownload = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'candidate-download-result.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskReceipt = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'installer-verification.raw.original.json') -Raw -Encoding utf8 | ConvertFrom-Json
if ($taskDownload.source -cne $taskSource -or $taskReceipt.sourceCommit -cne $taskSource -or $taskDownload.runId -ne 38737628201) {
    throw 'Expected the precise 02 source candidate'
}
$taskIdentityPath = Join-Path $PSScriptRoot 'candidate-payload-identity.json'
$taskLiteralPath = Join-Path $PSScriptRoot 'candidate-source-literal.json'
if ((Test-Path -LiteralPath $taskIdentityPath) -or (Test-Path -LiteralPath $taskLiteralPath)) {
    throw 'Keep existing payload evidence; independently inspect it instead of repeating extraction'
}
$taskMsiRows = @($taskReceipt.installers | Where-Object kind -ceq 'msi')
$taskNsisRows = @($taskReceipt.installers | Where-Object kind -ceq 'nsis')
if ($taskMsiRows.Count -ne 1 -or $taskNsisRows.Count -ne 1) { throw 'Expected one MSI and one NSIS receipt' }
$taskMsiFiles = @(Get-ChildItem -LiteralPath (Join-Path $taskDownload.destination 'installers') -Filter $taskMsiRows[0].name -File -Recurse)
if ($taskMsiFiles.Count -ne 1) { throw 'Verified MSI is ambiguous or missing' }
$taskOutput = Join-Path $taskDownload.destination 'msi-readonly'
if (Test-Path -LiteralPath $taskOutput) { throw 'Inspect an existing extraction; do not overwrite it' }
& (Join-Path $PSScriptRoot 'msi-readonly.ps1') -MsiPath $taskMsiFiles[0].FullName `
    -OutputDirectory $taskOutput -AllowedOutputRoot $taskDownload.destination `
    -ExpectedSha256 $taskMsiRows[0].sha256 -ExtractExecutable | Out-Null
Copy-Item -LiteralPath (Join-Path $taskOutput 'msi-readonly.json') -Destination (Join-Path $PSScriptRoot 'msi-readonly.original.json') -ErrorAction Stop
$taskRead = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'msi-readonly.original.json') -Raw -Encoding utf8 | ConvertFrom-Json
$taskExe = @($taskRead.extractedExecutables | Where-Object name -ceq 'opencodex-desktop.exe')
if ($taskRead.result -cne 'pass' -or $taskRead.installerExecuted -ne $false -or $taskExe.Count -ne 1 -or
    $taskExe[0].sha256 -cne $taskMsiRows[0].installedExecutableSha256 -or $taskExe[0].productVersion -cne '0.1.10') {
    throw 'Extracted MSI payload does not match the runner-installed candidate'
}
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');source=$taskSource;runId=38737628201
    executable=$taskExe[0];msiSha256=$taskMsiRows[0].sha256
    nsisInstalledExecutableSha256=$taskNsisRows[0].installedExecutableSha256
    packageApiHashesAndRunnerPayloadMatch=$true;readonlyMsiExtraction=$true
    hostInstallerExecuted=$false;hostProductStarted=$false;nativeRootRecoveryPassed=$false
    official019HostPairedPerformancePassed=$false;fullVersionGatePassed=$false;releaseRequested=$false
} | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath $taskIdentityPath -Encoding utf8
$taskData = [IO.File]::ReadAllBytes($taskExe[0].path)
if (![Text.Encoding]::UTF8.GetString($taskData).Contains($taskSource) -or
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($taskData)).ToLowerInvariant() -cne $taskExe[0].sha256) {
    throw 'Static source literal or extracted bytes mismatch'
}
[ordered]@{
    source=$taskSource;runId=38737628201;applicationSourceLiteralFound=$true
    sha256=$taskExe[0].sha256;bytes=$taskExe[0].bytes
    applicationExecuted=$false;sourceLiteralAloneIsNotBuildProvenance=$true
} | ConvertTo-Json | Set-Content -LiteralPath $taskLiteralPath -Encoding utf8
[ordered]@{source=$taskSource;runId=38737628201;exeSha256=$taskExe[0].sha256;exeBytes=$taskExe[0].bytes;
    hostInstallerExecuted=$false;hostProductStarted=$false} | ConvertTo-Json

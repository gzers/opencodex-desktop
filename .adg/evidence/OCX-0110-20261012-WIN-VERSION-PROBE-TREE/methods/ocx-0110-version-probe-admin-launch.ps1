$ErrorActionPreference = 'Stop'
$evidence = 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261012-WIN-VERSION-PROBE-TREE'
$script = 'C:\Users\15119\.codex\tmp\ocx-0110-version-probe-local.ps1'
$pwsh = 'C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\native\powershell\pwsh.exe'
$receipt = Get-Content -LiteralPath (Join-Path $evidence 'probe-atomic-fixed-test-executable-verification.json') -Raw |
    ConvertFrom-Json -DateKind String
if ($receipt.sourceCommit -cne 'eae85322ebca3b755c034db97cbebf2ea5bf71fc' -or
    !$receipt.ciTestExecutableReceiptMatched -or
    (Get-FileHash -LiteralPath $receipt.executable -Algorithm SHA256).Hash.ToLowerInvariant() -cne $receipt.sha256) {
    throw 'Final fixed probe test artifact not independently verified'
}
foreach ($name in @('version','metadata')) {
    if (Test-Path -LiteralPath (Join-Path $evidence "probe-atomic-fixed-administrator-$name-result.json")) {
        throw 'Administrator result already exists; do not overwrite it'
    }
}
$parseTokens = $null
$parseErrors = $null
[void][Management.Automation.Language.Parser]::ParseFile($script, [ref]$parseTokens, [ref]$parseErrors)
if ($parseErrors.Count) { throw 'Local test runner syntax is invalid' }
$launch = [ordered]@{
    requestedAtUtc = [DateTime]::UtcNow.ToString('o')
    sourceCommit = $receipt.sourceCommit
    ciTestExecutableSha256 = $receipt.sha256
    helperScriptSha256 = (Get-FileHash -LiteralPath $script -Algorithm SHA256).Hash.ToLowerInvariant()
    mode = 'administrator'
    status = 'requesting-standard-runas'
    desktopApplicationLaunched = $false
    authenticationAutomated = $false
    securitySettingsChanged = $false
    scope = 'standard RunAs for source-bound version and metadata Rust function fixtures only'
}
function Save-Launch {
    $launch | ConvertTo-Json -Depth 7 |
        Set-Content -LiteralPath (Join-Path $evidence 'final-admin-launch.json') -Encoding utf8
}
Save-Launch
try {
    $process = Start-Process -FilePath $pwsh -ArgumentList @(
        '-NoProfile','-NonInteractive','-File',$script,'-Label','probe-atomic-fixed','-Mode','administrator'
    ) -Verb RunAs -WindowStyle Hidden -PassThru
    try {
        $launch.status = 'helper-started-test-token-not-yet-proved'
        $launch.helperPid = $process.Id
        $launch.helperCreationUtc = $process.StartTime.ToUniversalTime().ToString('o')
        $launch.returnedAtUtc = [DateTime]::UtcNow.ToString('o')
        Save-Launch
    } finally {
        $process.Dispose()
    }
} catch {
    $launch.status = 'standard-runas-request-not-completed'
    $launch.errorType = $_.Exception.GetType().FullName
    $launch.errorCode = $_.Exception.HResult
    Save-Launch
    throw
}
$launch | ConvertTo-Json -Depth 7

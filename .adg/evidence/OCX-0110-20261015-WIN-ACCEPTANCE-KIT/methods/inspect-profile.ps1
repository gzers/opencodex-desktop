param(
    [Parameter(Mandatory=$true)][string]$ExpectedProfile,
    [Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedKitManifestSha256
)
$ErrorActionPreference='Stop'
if (!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7) { throw 'Windows PowerShell 7 required' }
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskManifestPath = Join-Path $taskRoot 'kit-manifest.json'
if ((Get-FileHash -LiteralPath $taskManifestPath).Hash.ToLowerInvariant() -cne $ExpectedKitManifestSha256) {
    throw 'Kit manifest does not match the separately recorded digest'
}
$taskManifest = Get-Content -LiteralPath $taskManifestPath -Raw -Encoding utf8 | ConvertFrom-Json
if ($taskManifest.schemaVersion -ne 1 -or $taskManifest.source -cne '02a0d4a8cbae8ef1450908d86b2c75fcad65f04f') {
    throw 'Unexpected kit schema or product source'
}
foreach ($taskEntry in $taskManifest.files) {
    if ($taskEntry.path -notmatch '^[A-Za-z0-9_. /-]+$' -or $taskEntry.path -match '(^|/)\.\.?(/|$)' -or
        $taskEntry.path.StartsWith('/') -or $taskEntry.sha256 -cnotmatch '^[0-9a-f]{64}$') {
        throw 'Invalid kit file inventory entry'
    }
    $taskPath = [IO.Path]::GetFullPath((Join-Path $taskRoot $taskEntry.path))
    if (!$taskPath.StartsWith($taskRoot + [IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) {
        throw 'Kit file escaped its root'
    }
    $taskItem = Get-Item -LiteralPath $taskPath -Force -ErrorAction Stop
    $taskAncestor = $taskItem
    while ($null -ne $taskAncestor) {
        if ($taskAncestor.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Kit aliases are refused' }
        if ($taskAncestor -is [IO.FileInfo]) { $taskAncestor=$taskAncestor.Directory }
        else { $taskAncestor=$taskAncestor.Parent }
    }
    if ($taskItem.PSIsContainer -or $taskItem.Length -ne $taskEntry.bytes -or
        (Get-FileHash -LiteralPath $taskPath).Hash.ToLowerInvariant() -cne $taskEntry.sha256) {
        throw 'Kit file bytes do not match the manifest'
    }
}
$taskIdentity = [Security.Principal.WindowsIdentity]::GetCurrent()
try {
    $taskSid = $taskIdentity.User.Value
    $taskSidHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData(
        [Text.Encoding]::UTF8.GetBytes($taskSid))).ToLowerInvariant()
} finally { $taskIdentity.Dispose() }
if ($taskSidHash -ceq $taskManifest.hostDailySidSha256) {
    [ordered]@{
        schemaVersion=1;status='rejected';reason='daily-account';kitBytesVerified=$true
        source=$taskManifest.source;sidSha256=$taskSidHash;rawSidPublished=$false
        accountOrProfileCreated=$false;registryOrAclChanged=$false
        productStarted=$false;launchPermissionGranted=$false;fullVersionGatePassed=$false
    } | ConvertTo-Json -Depth 6
    return
}
. (Join-Path $PSScriptRoot 'windows-acceptance-profile.ps1')
$taskHashSid = {
    param([string]$Value)
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($Value))).ToLowerInvariant()
}
# Resolve the independently pinned daily identity only to call the original
# admission method. No raw identity or path is included in the returned receipt.
$taskLocal = @(Get-CimInstance Win32_UserAccount -Filter "LocalAccount=True")
$taskDaily = @($taskLocal | Where-Object { (& $taskHashSid $_.SID) -ceq $taskManifest.hostDailySidSha256 })
$taskExpected = @($taskLocal | Where-Object Name -ceq 'ocx0110_accept')
if ($taskDaily.Count -ne 1 -or $taskExpected.Count -ne 1) {
    throw 'Approved daily/test account identities are not independently resolvable'
}
$taskProfile = Get-WindowsAcceptanceProfile -DailySid $taskDaily[0].SID -ExpectedSid $taskExpected[0].SID -ExpectedProfile $ExpectedProfile
$taskProfile['kitBytesVerified']=$true
$taskProfile['kitManifestSha256']=$ExpectedKitManifestSha256
$taskProfile['launchPermissionGranted']=$false
$taskProfile | ConvertTo-Json -Depth 8

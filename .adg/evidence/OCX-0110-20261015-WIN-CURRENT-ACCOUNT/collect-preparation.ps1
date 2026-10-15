$ErrorActionPreference = 'Stop'
$taskOutput = Join-Path $PSScriptRoot 'preparation.original.json'
if (Test-Path -LiteralPath $taskOutput) { throw 'Do not overwrite this observation' }
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskSource = '02a0d4a8cbae8ef1450908d86b2c75fcad65f04f'
$taskKit = Join-Path $taskCode 'test/software/acceptance-kit-02a0d4-20261015'
$taskManifestPath = Join-Path $taskKit 'kit-manifest.json'
$taskManifestHash = '96a756942733c34cc3c09b9c4886a8ee2ba9f9c2379fb8b95688642d4cb6692a'
if ((& git -C $taskCode rev-parse HEAD).Trim() -cne $taskSource -or
    @(& git -C $taskCode status --porcelain).Count) { throw 'Source changed or worktree is not clean' }
if ((Get-FileHash -LiteralPath $taskManifestPath).Hash.ToLowerInvariant() -cne $taskManifestHash) {
    throw 'Prepared kit manifest changed'
}
$taskManifest = Get-Content -LiteralPath $taskManifestPath -Raw -Encoding utf8 | ConvertFrom-Json
$taskVerifiedBytes = [long]0
foreach ($taskEntry in $taskManifest.files) {
    if ($taskEntry.path -notmatch '^[A-Za-z0-9_. /-]+$' -or
        $taskEntry.path -match '(^|/)\.\.?(/|$)' -or $taskEntry.path.StartsWith('/')) {
        throw 'Invalid prepared inventory path'
    }
    $taskPath = [IO.Path]::GetFullPath((Join-Path $taskKit $taskEntry.path))
    if (!$taskPath.StartsWith($taskKit + '\', [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Prepared path escaped the kit'
    }
    $taskItem = Get-Item -LiteralPath $taskPath -Force
    $taskAncestor = $taskItem
    while ($null -ne $taskAncestor) {
        if ($taskAncestor.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Kit alias refused' }
        if ($taskAncestor -is [IO.FileInfo]) { $taskAncestor = $taskAncestor.Directory }
        else { $taskAncestor = $taskAncestor.Parent }
    }
    if ($taskItem.PSIsContainer -or $taskItem.Length -ne $taskEntry.bytes -or
        (Get-FileHash -LiteralPath $taskPath).Hash.ToLowerInvariant() -cne $taskEntry.sha256) {
        throw 'Prepared payload changed'
    }
    $taskVerifiedBytes += $taskItem.Length
}
. (Join-Path $taskKit 'methods/windows-process-token.ps1')
$taskToken = Get-WindowsHarnessToken
Assert-WindowsProcessMode -Observation $taskToken -Mode ordinary
$taskIdentity = [Security.Principal.WindowsIdentity]::GetCurrent()
try {
    $taskSidHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData(
        [Text.Encoding]::UTF8.GetBytes($taskIdentity.User.Value))).ToLowerInvariant()
    if ($taskSidHash -cne $taskManifest.hostDailySidSha256) { throw 'Unexpected current identity' }
    $taskAdminGroupPresent = @($taskIdentity.Groups | Where-Object Value -eq 'S-1-5-32-544').Count -gt 0
} finally { $taskIdentity.Dispose() }
$taskNativeCount = @(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count
if ($taskNativeCount -ne 0) { throw 'Product instance exists; stop preparation' }
$taskPointerKey = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\com.gzers.opencodex.desktop', $false)
try {
    $taskPointerPresent = $null -ne $taskPointerKey -and
        @($taskPointerKey.GetValueNames()) -contains 'StartupDataRootV1'
} finally { if ($taskPointerKey) { $taskPointerKey.Dispose() } }
$taskTaskPath = 'D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\work\objects\task\TASK-OPENCODEX-DESKTOP-182.json'
$taskTaskHash = (Get-FileHash -LiteralPath $taskTaskPath).Hash.ToLowerInvariant()
if ($taskTaskHash -cne 'bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93') {
    throw 'Original task changed'
}
[ordered]@{
    schemaVersion = 1; atUtc = [datetimeoffset]::UtcNow.ToString('o')
    source = $taskSource; codeWorktreeClean = $true
    scope = 'read-only-current-account-preparation'
    authorization = 'User permits resetting OpenCodex Desktop configuration; no new account is needed for configuration isolation.'
    sidSha256 = $taskSidHash; processId = $taskToken.processId
    creationTimeUtc = $taskToken.creationTimeUtc; elevated = $taskToken.elevated
    elevationType = $taskToken.elevationType; administratorsSidPresent = $taskAdminGroupPresent
    identityMeaning = 'Current controller only; not an observed product token or a dedicated nonadministrator account.'
    nativeCount = $taskNativeCount; startupPointerPresent = $taskPointerPresent
    kitManifestSha256 = $taskManifestHash
    kitFilesVerified = $taskManifest.files.Count; kitBytesVerified = $taskVerifiedBytes
    oldDedicatedAccountInspectorUnmodified = $true; oldInspectorUsedForCurrentAdmission = $false
    originalTaskSha256 = $taskTaskHash
    originalTaskBytes = (Get-Item -LiteralPath $taskTaskPath).Length
    productStarted = $false; foregroundInputPerformed = $false
    accountCreated = $false; configurationDeleted = $false; registryWritten = $false
    installerExecuted = $false; performanceSamplesCollected = 0
    sameSidDualProductTokensProved = $false; fullVersionGatePassed = $false
    taskClosed = $false; releaseRequested = $false
} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $taskOutput -Encoding utf8

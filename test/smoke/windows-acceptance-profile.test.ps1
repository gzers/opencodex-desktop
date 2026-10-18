param([Parameter(Mandatory=$true)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
if (!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7) { throw 'Windows PowerShell 7 required' }
. (Join-Path $PSScriptRoot 'windows-acceptance-profile.ps1')
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Path $taskOutput -Force | Out-Null
$taskDaily='S-1-5-21-100-200-300-1001'
$taskExpected='S-1-5-21-100-200-300-1002'
$taskProfile='C:\Users\ocx-acceptance-fixture'
$taskGood=[ordered]@{
    sid=$taskExpected; elevated=$false; elevationType=1; administratorsSidPresent=$false
    localAccountConfirmed=$true; threadImpersonating=$false
    profile=$taskProfile; roaming="$taskProfile\AppData\Roaming"; local="$taskProfile\AppData\Local"
    existingDirectoriesVerified=$true; reparseFree=$true; desktopCount=[int]0
}
$taskChecks=[Collections.Generic.List[object]]::new()
function Test-ProfileCase {
    param([string]$Name,[hashtable]$Changes,[bool]$Admitted,[string]$Reason='')
    $taskInput=[ordered]@{}
    foreach ($key in $taskGood.Keys) { $taskInput[$key]=$taskGood[$key] }
    foreach ($key in $Changes.Keys) { $taskInput[$key]=$Changes[$key] }
    $taskDecision=Get-AcceptanceProfileDecision $taskInput $taskDaily $taskExpected $taskProfile
    if ($taskDecision.admitted -ne $Admitted -or
        ($Reason -and $Reason -notin $taskDecision.reasons)) { throw "Profile contract failed: $Name" }
    $taskChecks.Add([ordered]@{name=$Name; admitted=$taskDecision.admitted; reasons=$taskDecision.reasons; result='pass'})
}
Test-ProfileCase 'ordinary-isolated-profile' @{} $true
Test-ProfileCase 'daily-account' @{sid=$taskDaily} $false 'daily-account'
Test-ProfileCase 'unexpected-account' @{sid='S-1-5-21-100-200-300-1003'} $false 'unexpected-account'
Test-ProfileCase 'filtered-administrator' @{elevationType=3;administratorsSidPresent=$true} $false 'ordinary-nonadministrator-account-required'
Test-ProfileCase 'elevated-administrator' @{elevated=$true;elevationType=2} $false 'ordinary-nonadministrator-account-required'
Test-ProfileCase 'string-false-is-not-evidence' @{elevated='false'} $false 'ordinary-nonadministrator-account-required'
Test-ProfileCase 'string-type-is-not-evidence' @{elevationType='1'} $false 'ordinary-nonadministrator-account-required'
Test-ProfileCase 'unconfirmed-account' @{localAccountConfirmed=$false} $false 'local-account-unconfirmed'
Test-ProfileCase 'string-account-confirmation' @{localAccountConfirmed='True'} $false 'local-account-unconfirmed'
Test-ProfileCase 'impersonating-thread' @{threadImpersonating=$true} $false 'thread-impersonation'
Test-ProfileCase 'string-impersonation' @{threadImpersonating='False'} $false 'thread-impersonation'
Test-ProfileCase 'missing-directory' @{existingDirectoriesVerified=$false} $false 'profile-directories-unverified'
Test-ProfileCase 'string-directory-confirmation' @{existingDirectoriesVerified='True'} $false 'profile-directories-unverified'
Test-ProfileCase 'reparse-path' @{reparseFree=$false} $false 'reparse-boundary-unverified'
Test-ProfileCase 'string-reparse-confirmation' @{reparseFree='True'} $false 'reparse-boundary-unverified'
Test-ProfileCase 'live-desktop' @{desktopCount=[int]1} $false 'native-instances-not-zero'
Test-ProfileCase 'unknown-desktop-count' @{desktopCount=$null} $false 'native-instances-not-zero'
Test-ProfileCase 'known-profile-still-daily' @{profile='C:\Users\daily'} $false 'known-profile-outside-fixture'
Test-ProfileCase 'same-prefix-neighbor' @{roaming="${taskProfile}-other\AppData\Roaming"} $false 'known-roaming-outside-fixture'
Test-ProfileCase 'local-data-outside' @{local='D:\Shared\AppData'} $false 'known-local-outside-fixture'
Test-ProfileCase 'relative-known-folder' @{profile='relative'} $false 'invalid-known-folder-path'
Test-ProfileCase 'dot-escape' @{local="$taskProfile\..\daily"} $false 'invalid-known-folder-path'
Test-ProfileCase 'case-insensitive-and-forward-separators' @{
    profile='c:/users/OCX-ACCEPTANCE-FIXTURE/'
    local='c:/users/OCX-ACCEPTANCE-FIXTURE/AppData/Local'
} $true
$taskInvalidPaths=@('C:\','relative','\\server\share\user','\\?\C:\Users\test',
    'C:relative','C:\Users\test:stream','C:\Users\.\test','C:\Users\test\..\daily')
foreach ($path in $taskInvalidPaths) {
    $rejected=$false
    try { ConvertTo-AcceptanceProfilePath $path | Out-Null } catch { $rejected=$true }
    if (!$rejected) { throw 'Invalid profile path accepted' }
    $taskChecks.Add([ordered]@{name='reject-invalid-path';result='pass'})
}
$taskResult=[ordered]@{
    result='pass'; scope='Offline profile admission predicates only; native test-account acceptance unproven'
    checks=$taskChecks.ToArray(); checksPassed=$taskChecks.Count
    methodSha256=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'windows-acceptance-profile.ps1')).Hash.ToLowerInvariant()
    nativeProfileProbeExecuted=$false; syntheticAccountCreated=$false; productStarted=$false
    installerIsolationProved=$false; fullVersionGatePassed=$false
}
$taskResult | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskOutput 'profile-contract.json') -Encoding utf8
$taskResult | ConvertTo-Json -Depth 8

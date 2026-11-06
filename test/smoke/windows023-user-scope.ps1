param(
    [Parameter(Mandatory=$true)][string]$ProbeExecutable,
    [Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-fA-F]{64}$')][string]$ExpectedProbeSha256,
    [Parameter(Mandatory=$true)][ValidatePattern('^S-1-5-')][string]$ExpectedSid,
    [Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-f]{40}$')][string]$ExpectedSourceCommit,
    [Parameter(Mandatory=$true)][string]$OutputDirectory,
    [string]$PeerPrivateFile,
    [switch]$KeepFixtureForPeer
)
$ErrorActionPreference='Stop'
$sourceRoot=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$actualCommit=(& git -C $sourceRoot rev-parse HEAD).Trim()
if($LASTEXITCODE -ne 0 -or $actualCommit -cne $ExpectedSourceCommit){throw 'Source commit differs; use the exact contributed probe/helper source'}
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if($identity.User.Value -cne $ExpectedSid){throw 'Actual process SID differs from assigned account'}
$exe=(Resolve-Path -LiteralPath $ProbeExecutable).ProviderPath
if((Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash -ine $ExpectedProbeSha256){throw 'Probe executable digest differs'}
if(![IO.Path]::IsPathRooted($OutputDirectory)){throw 'Absolute new output directory required'}
$out=[IO.Path]::GetFullPath($OutputDirectory)
if(Test-Path -LiteralPath $out){throw 'Use a new evidence directory; never overwrite previous results'}
$null=New-Item -ItemType Directory -Path $out
$fixture=Join-Path $out ('windows023-fixture-'+[Guid]::NewGuid().ToString('N'))
$null=New-Item -ItemType Directory -Path $fixture
$principal=[Security.Principal.WindowsPrincipal]::new($identity)
$result=[ordered]@{kind='native-helper-only';sourceCommit=$actualCommit;probeSha256=$ExpectedProbeSha256.ToLowerInvariant();sid=$identity.User.Value;sessionId=(Get-Process -Id $PID).SessionId;elevated=$principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator);result='fail';fullProductVerified=$false;cleanup='pending';fixtureRetained=$false}
$result.helperSha256=(Get-FileHash -LiteralPath (Join-Path $sourceRoot 'apps/desktop/tauri/src/infrastructure/windows_user_scope.rs') -Algorithm SHA256).Hash.ToLowerInvariant()
$result.workingTreeStatus=@(& git -C $sourceRoot status --porcelain=v1 -- apps/desktop/tauri/src/infrastructure/windows_user_scope.rs test/smoke/windows023-user-scope-probe.rs test/smoke/windows023-user-scope.ps1)
try {
    $text=& $exe --fixture $fixture 2> (Join-Path $out 'probe-stderr.log')
    if($LASTEXITCODE -ne 0){throw 'Native helper failed; see retained stderr'}
    $text | Set-Content -LiteralPath (Join-Path $out 'probe.json') -Encoding UTF8
    $probe=$text | ConvertFrom-Json
    if($probe.sid -cne $ExpectedSid -or !$probe.ownReadback){throw 'Native helper did not prove own SID/readback'}
    if($probe.sourceCommit -cne $ExpectedSourceCommit -or $probe.helperSha256 -cne $result.helperSha256){throw 'Probe build metadata does not match the current helper/commit'}
    $result.native=$probe
    $acl=Get-Acl -LiteralPath $probe.privateFile
    $allowed=@($acl.Access | Where-Object {$_.AccessControlType -eq 'Allow'} | ForEach-Object {$_.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value})
    if(!$acl.AreAccessRulesProtected -or @($allowed | Where-Object {$_ -cne $ExpectedSid -and $_ -cne 'S-1-5-18'}).Count -gt 0 -or $allowed -cnotcontains $ExpectedSid){throw 'Private file DACL contains unexpected grants or still inherits'}
    $result.privateAcl=[ordered]@{owner=$acl.Owner;protected=$acl.AreAccessRulesProtected;allowedSids=$allowed;sddl=$acl.Sddl}
    if($PeerPrivateFile){
        if([IO.Path]::GetFileName($PeerPrivateFile) -cne 'windows023-synthetic-secret.txt' -or [IO.Path]::GetFileName([IO.Path]::GetDirectoryName($PeerPrivateFile)) -notlike 'windows023-fixture-*'){throw 'Peer path must identify a retained synthetic fixture, never daily data'}
        try{$handle=[IO.File]::Open($PeerPrivateFile,'Open','Read','ReadWrite');$handle.Dispose();throw 'Other account synthetic fixture was readable'}catch [UnauthorizedAccessException]{$result.peerRead='access-denied'}
    } else {$result.peerRead='not-tested'}
    $result.result='pass'
} catch {
    $result.failure=$_.Exception.Message
} finally {
    if($KeepFixtureForPeer){$result.fixtureRetained=$true;$result.cleanup='retained-explicitly-for-peer-check'}else{
        $resolved=[IO.Path]::GetFullPath($fixture)
        if(![string]::Equals([IO.Path]::GetDirectoryName($resolved),$out,[StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetFileName($resolved) -notlike 'windows023-fixture-*'){throw 'Refusing cleanup outside this attempt'}
        Remove-Item -LiteralPath $resolved -Recurse -Force
        $result.cleanup=if(Test-Path -LiteralPath $resolved){'failed'}else{'removed-own-fixture'}
    }
    $result | ConvertTo-Json -Depth 9 | Set-Content -LiteralPath (Join-Path $out 'receipt.json') -Encoding UTF8
}
if($result.result -cne 'pass' -or $result.cleanup -ceq 'failed'){exit 1}
$result | ConvertTo-Json -Depth 9

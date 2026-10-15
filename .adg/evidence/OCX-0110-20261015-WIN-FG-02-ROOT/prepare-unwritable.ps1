$ErrorActionPreference='Stop'
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskRuntime=Join-Path $taskCode 'test/runtime/foreground-02-current-20261015'
$taskDirectory=[IO.Path]::GetFullPath((Join-Path $taskRuntime 'install-c'))
$taskRecord=Join-Path $PSScriptRoot 'unwritable-fixture.json'
if(Test-Path -LiteralPath $taskRecord){throw 'Fixture already prepared'}
if(@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count){throw 'Native product exists'}
if(Test-Path -LiteralPath $taskDirectory){throw 'Fixture directory already exists'}
if($taskDirectory -cne ($taskRuntime+'\install-c')){throw 'Unexpected fixture endpoint'}
New-Item -ItemType Directory -Path $taskDirectory | Out-Null
foreach($taskName in @('opencodex-desktop.exe','ocxd.exe')){
    Copy-Item -LiteralPath (Join-Path $taskRuntime ('install-a/'+$taskName)) -Destination (Join-Path $taskDirectory $taskName)
}
$taskAcl=Get-Acl -LiteralPath $taskDirectory
# Original ACL stays in the ignored local runtime for exact restoration.
$taskAcl.Sddl | Set-Content -LiteralPath (Join-Path $taskRuntime 'unwritable-fixture.original.sddl') -Encoding utf8
$taskIdentity=[Security.Principal.WindowsIdentity]::GetCurrent()
try{
    $taskRule=[Security.AccessControl.FileSystemAccessRule]::new(
        $taskIdentity.User,
        [Security.AccessControl.FileSystemRights]'CreateDirectories,CreateFiles',
        [Security.AccessControl.InheritanceFlags]::None,
        [Security.AccessControl.PropagationFlags]::None,
        [Security.AccessControl.AccessControlType]::Deny)
    $taskAcl.AddAccessRule($taskRule)
    Set-Acl -LiteralPath $taskDirectory -AclObject $taskAcl
}finally{$taskIdentity.Dispose()}
$taskPointer=(Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop').StartupDataRootV1
if(!$taskPointer){throw 'Original test pointer missing'}
if(($taskPointer | ConvertFrom-Json).anchor -notlike '*foreground-02-current-20261015\install-a\data'){throw 'Pointer escaped the test scope'}
$taskPointer | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'startup-pointer-before-reset.original.txt') -Encoding utf8
Remove-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop' -Name StartupDataRootV1
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o')
    directory=$taskDirectory;deniedRights='CreateDirectories,CreateFiles';inheritance='None'
    affected='Owned test install-c directory only';originalAclSavedLocally=$true
    defaultDataBeforeExists=(Test-Path -LiteralPath (Join-Path $taskDirectory 'data'))
    roamingBeforeExists=(Test-Path -LiteralPath (Join-Path $env:APPDATA 'com.gzers.opencodex.desktop'))
    pointerResetOnly='StartupDataRootV1';oldRootDeleted=$false;permissionsBroadened=$false
    launchMode='ordinary';requiresRealWriteDenialObservation=$true;fullVersionGatePassed=$false
} | ConvertTo-Json | Set-Content -LiteralPath $taskRecord -Encoding utf8

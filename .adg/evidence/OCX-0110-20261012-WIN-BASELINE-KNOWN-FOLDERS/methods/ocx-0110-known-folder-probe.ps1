param(
    [Parameter(Mandatory=$true)][string]$FixtureRoot,
    [ValidateSet('control','environment-override')][string]$ProbeMode = 'control'
)
$ErrorActionPreference = 'Stop'
$normalizedFixture = [IO.Path]::GetFullPath($FixtureRoot).TrimEnd('\')
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class OcxKnownFolderReadOnly {
    [DllImport("shell32.dll", ExactSpelling=true)]
    private static extern int SHGetKnownFolderPath(ref Guid folderId, uint flags, IntPtr token, out IntPtr path);
    [DllImport("ole32.dll", ExactSpelling=true)]
    private static extern void CoTaskMemFree(IntPtr memory);
    public static string Read(string id) {
        Guid folder = Guid.Parse(id);
        IntPtr pointer = IntPtr.Zero;
        try {
            // flags=0: never request creation, redirection or a persisted change.
            int status = SHGetKnownFolderPath(ref folder, 0, IntPtr.Zero, out pointer);
            if (status != 0 || pointer == IntPtr.Zero)
                throw new InvalidOperationException("Known Folder read failed, HRESULT=" + status);
            return Marshal.PtrToStringUni(pointer);
        } finally {
            if (pointer != IntPtr.Zero) CoTaskMemFree(pointer);
        }
    }
}
'@
function Normalize-ObservedPath([string]$Value) {
    [IO.Path]::GetFullPath($Value).TrimEnd('\').ToLowerInvariant()
}
function Path-Hash([string]$Value) {
    $bytes = [Text.Encoding]::UTF8.GetBytes((Normalize-ObservedPath $Value))
    [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant()
}
$folders = @(
    @{name='profile';id='5e6c858f-0e22-4760-9afe-ea3317b67173';environment='USERPROFILE'},
    @{name='roaming';id='3eb685db-65f9-4cf6-a03a-e3ef65729f3d';environment='APPDATA'},
    @{name='local';id='f1b32785-6fba-4fcf-9d55-7b8e7f157091';environment='LOCALAPPDATA'}
)
$rows = @()
foreach ($folder in $folders) {
    $observed = [OcxKnownFolderReadOnly]::Read($folder.id)
    $environmentValue = [Environment]::GetEnvironmentVariable($folder.environment)
    $nativeInside = (Normalize-ObservedPath $observed).StartsWith(
        (Normalize-ObservedPath $normalizedFixture) + '\', [StringComparison]::Ordinal)
    $environmentInside = $environmentValue -and (Normalize-ObservedPath $environmentValue).StartsWith(
        (Normalize-ObservedPath $normalizedFixture) + '\', [StringComparison]::Ordinal)
    $rows += [ordered]@{
        folder=$folder.name;folderId=$folder.id
        nativePathSha256=(Path-Hash $observed)
        nativePathInsideFixture=$nativeInside
        environmentName=$folder.environment
        environmentPathInsideFixture=[bool]$environmentInside
        nativePathEqualsEnvironment=($environmentValue -and
            (Normalize-ObservedPath $observed) -ceq (Normalize-ObservedPath $environmentValue))
        rawUserPathIncluded=$false
    }
}
[ordered]@{
    collectedAtUtc=[DateTime]::UtcNow.ToString('o')
    probeMode=$ProbeMode
    processId=$PID
    fixtureRoot=$normalizedFixture
    knownFolderApi='SHGetKnownFolderPath(flags=0, token=NULL)'
    userProfileEnvironmentOverridden=($ProbeMode -eq 'environment-override')
    windowsSecuritySettingsChanged=$false
    registryOrKnownFolderBindingChanged=$false
    baselineApplicationLaunched=$false
    rows=$rows
    scope='actual read-only Known Folder API behavior for current account token; no baseline EXE execution or claim of original dependency resolution'
} | ConvertTo-Json -Depth 7

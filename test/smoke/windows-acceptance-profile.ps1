# Read-only admission for an already provisioned ordinary test account.
# This never creates an account, redirects Known Folders, or launches a product.
. (Join-Path $PSScriptRoot 'windows-process-token.ps1')

function ConvertTo-AcceptanceProfilePath {
    param([Parameter(Mandatory=$true)][string]$Path)
    if ($Path -notmatch '^[a-zA-Z]:[\\/]' -or $Path -match '[\x00-\x1f]' -or
        $Path -match '(^|[\\/])\.\.?([\\/]|$)' -or $Path.Substring(2).Contains(':')) {
        throw 'Expected a local absolute path without device, stream, or dot components'
    }
    $result = [IO.Path]::GetFullPath($Path).TrimEnd('\')
    if ($result.Length -le 3) { throw 'A drive root cannot be an acceptance Profile' }
    $result
}

function Test-AcceptanceProfileBoundary {
    param([string]$ParentPath, [string]$ChildPath, [switch]$AllowEqual)
    $parent = ConvertTo-AcceptanceProfilePath $ParentPath
    $child = ConvertTo-AcceptanceProfilePath $ChildPath
    ($AllowEqual -and [StringComparer]::OrdinalIgnoreCase.Equals($parent, $child)) -or
        $child.StartsWith($parent + '\', [StringComparison]::OrdinalIgnoreCase)
}

function Get-AcceptanceProfileDecision {
    # Pure predicate for contract tests, not a reusable authorization certificate.
    param(
        [Parameter(Mandatory=$true)]$Observation,
        [Parameter(Mandatory=$true)][string]$DailySid,
        [Parameter(Mandatory=$true)][string]$ExpectedSid,
        [Parameter(Mandatory=$true)][string]$ExpectedProfile
    )
    $reasons = [Collections.Generic.List[string]]::new()
    foreach ($sid in @($DailySid, $ExpectedSid)) {
        if ($sid -notmatch '^S-1-5-21-\d+-\d+-\d+-\d+$') { throw 'A full account SID is required' }
    }
    $expected = ConvertTo-AcceptanceProfilePath $ExpectedProfile
    if ($DailySid -eq $ExpectedSid -or $Observation.sid -eq $DailySid) { $reasons.Add('daily-account') }
    if ($Observation.sid -cne $ExpectedSid) { $reasons.Add('unexpected-account') }
    if ($Observation.elevated -isnot [bool] -or $Observation.elevated -ne $false -or
        $Observation.elevationType -isnot [int] -or $Observation.elevationType -ne 1 -or
        $Observation.administratorsSidPresent -isnot [bool] -or $Observation.administratorsSidPresent) {
        $reasons.Add('ordinary-nonadministrator-account-required')
    }
    if ($Observation.localAccountConfirmed -isnot [bool] -or !$Observation.localAccountConfirmed) {
        $reasons.Add('local-account-unconfirmed')
    }
    if ($Observation.threadImpersonating -isnot [bool] -or $Observation.threadImpersonating) {
        $reasons.Add('thread-impersonation')
    }
    if ($Observation.existingDirectoriesVerified -isnot [bool] -or !$Observation.existingDirectoriesVerified) {
        $reasons.Add('profile-directories-unverified')
    }
    if ($Observation.reparseFree -isnot [bool] -or !$Observation.reparseFree) {
        $reasons.Add('reparse-boundary-unverified')
    }
    if ($Observation.desktopCount -isnot [int] -or $Observation.desktopCount -ne 0) {
        $reasons.Add('native-instances-not-zero')
    }
    try {
        if (![StringComparer]::OrdinalIgnoreCase.Equals(
                (ConvertTo-AcceptanceProfilePath $Observation.profile), $expected)) {
            $reasons.Add('known-profile-outside-fixture')
        }
        foreach ($name in @('roaming', 'local')) {
            if (!(Test-AcceptanceProfileBoundary -ParentPath $expected -ChildPath $Observation[$name])) {
                $reasons.Add("known-$name-outside-fixture")
            }
        }
    } catch { $reasons.Add('invalid-known-folder-path') }
    [ordered]@{ admitted=($reasons.Count -eq 0); reasons=$reasons.ToArray() }
}

function Get-WindowsAcceptanceProfile {
    param(
        [Parameter(Mandatory=$true)][string]$DailySid,
        [Parameter(Mandatory=$true)][string]$ExpectedSid,
        [Parameter(Mandatory=$true)][string]$ExpectedProfile
    )
    if (!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7) { throw 'Windows PowerShell 7 required' }
    if (!('OpenCodexAcceptanceKnownFolders' -as [type])) {
        Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class OpenCodexAcceptanceKnownFolders {
    [DllImport("shell32.dll", ExactSpelling=true)]
    private static extern int SHGetKnownFolderPath(ref Guid folder, uint flags, IntPtr token, out IntPtr path);
    public static string Read(string id) {
        Guid folder = new Guid(id);
        IntPtr path = IntPtr.Zero;
        try {
            // flags=0: current location, verify existence, never KF_FLAG_CREATE.
            int result = SHGetKnownFolderPath(ref folder, 0, IntPtr.Zero, out path);
            Marshal.ThrowExceptionForHR(result);
            if (path == IntPtr.Zero) throw new InvalidOperationException("Known Folder path missing");
            return Marshal.PtrToStringUni(path);
        } finally {
            if (path != IntPtr.Zero) Marshal.FreeCoTaskMem(path);
        }
    }
}
'@
    }
    $token = Get-WindowsHarnessToken
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $threadIdentity = [Security.Principal.WindowsIdentity]::GetCurrent($true)
    try {
        $observation = [ordered]@{
            sid=$identity.User.Value; elevated=$token.elevated; elevationType=$token.elevationType
            administratorsSidPresent=(@($identity.Groups | Where-Object Value -eq 'S-1-5-32-544').Count -gt 0)
            localAccountConfirmed=$false; threadImpersonating=($null -ne $threadIdentity)
            profile=[OpenCodexAcceptanceKnownFolders]::Read('5e6c858f-0e22-4760-9afe-ea3317b67173')
            roaming=[OpenCodexAcceptanceKnownFolders]::Read('3eb685db-65f9-4cf6-a03a-e3ef65729f3d')
            local=[OpenCodexAcceptanceKnownFolders]::Read('f1b32785-6fba-4fcf-9d55-7b8e7f157091')
            existingDirectoriesVerified=$true; reparseFree=$true
            desktopCount=[int]@(Get-CimInstance Win32_Process -Filter "Name='opencodex-desktop.exe' OR Name='ocxd.exe'").Count
        }
        try {
            $localUser = Get-LocalUser -SID $identity.User -ErrorAction Stop
            $observation.localAccountConfirmed = $localUser.Enabled -and ($localUser.SID.Value -eq $observation.sid)
        } catch { $observation.localAccountConfirmed = $false }
        # Check the proposed directory and actual locations, including all ancestors.
        # Reading attributes does not create a Profile or change ACLs.
        foreach ($path in @($ExpectedProfile, $observation.profile, $observation.roaming, $observation.local)) {
            try {
                $current = [IO.DirectoryInfo]::new((ConvertTo-AcceptanceProfilePath $path))
                while ($null -ne $current) {
                    $current.Refresh()
                    if (!$current.Exists) { $observation.existingDirectoriesVerified=$false; break }
                    if (($current.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                        $observation.reparseFree=$false
                    }
                    $current=$current.Parent
                }
            } catch { $observation.existingDirectoriesVerified=$false; $observation.reparseFree=$false }
        }
        $decision = Get-AcceptanceProfileDecision -Observation $observation -DailySid $DailySid `
            -ExpectedSid $ExpectedSid -ExpectedProfile $ExpectedProfile
        $hashText = {
            param([string]$Value)
            [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($Value))).ToLowerInvariant()
        }
        $paths = @('profile', 'roaming', 'local') | ForEach-Object {
            [ordered]@{folder=$_; pathSha256=(& $hashText $observation[$_])}
        }
        [ordered]@{
            schemaVersion=1; collectedAtUtc=[datetimeoffset]::UtcNow.ToString('o')
            status=$(if ($decision.admitted) {'admitted-profile-only'} else {'rejected'})
            processId=$token.processId; creationTimeUtc=$token.creationTimeUtc
            elevated=$token.elevated; elevationType=$token.elevationType
            sidSha256=(& $hashText $observation.sid); expectedSidMatched=($observation.sid -ceq $ExpectedSid)
            dailySidMatched=($observation.sid -ceq $DailySid)
            administratorsSidPresent=$observation.administratorsSidPresent
            localAccountConfirmed=$observation.localAccountConfirmed
            threadImpersonating=$observation.threadImpersonating
            existingDirectoriesVerified=$observation.existingDirectoriesVerified
            reparseFree=$observation.reparseFree; desktopCount=$observation.desktopCount
            knownFolderApi='SHGetKnownFolderPath(flags=0, token=NULL)'; knownFolders=@($paths)
            expectedProfileSha256=(& $hashText (ConvertTo-AcceptanceProfilePath $ExpectedProfile))
            admitted=$decision.admitted; reasons=$decision.reasons
            methodSha256=(Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant()
            rawUserPathsOrSidsPublished=$false; accountOrProfileCreated=$false
            knownFolderBindingsChanged=$false; environmentChanged=$false; permissionsChanged=$false
            productStarted=$false; inputInjected=$false; installerIsolationProved=$false
            profileAdmissionIsLaunchPermission=$false; fullVersionGatePassed=$false
        }
    } finally {
        if ($threadIdentity) { $threadIdentity.Dispose() }
        $identity.Dispose()
    }
}

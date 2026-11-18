$ErrorActionPreference = 'Stop'
$code = 'C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code'
$evidence = (Resolve-Path -LiteralPath '.adg/evidence/0.2.24/windows-dev/wl2-r7-host-live-observer-20261118-r1').Path
$timing = (Resolve-Path -LiteralPath '.adg/evidence/0.2.24/windows-dev/wl2-r7-readback-timing-observer-20261118-r1/observe-r7-readback-timing.ps1').Path
$public = (Resolve-Path -LiteralPath 'C:/Users/Public').Path
$fixed = Join-Path $public 'ocx-r7-isolated-owner-range-r1'
$target = Join-Path $public 'ocx-r7-range-observers-20261118-r1'
if (Test-Path -LiteralPath $target) { throw 'A fresh observer folder is required.' }
foreach ($parent in @('C:/Users', $public, $fixed)) {
    if ((Get-Item -LiteralPath $parent -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'A staging directory is a reparse point.' }
}
$acl = Get-Acl -LiteralPath $fixed
$owner = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$ordinary = 'S-1-5-21-587408064-856152790-2583328096-1003'
$system = 'S-1-5-18'
$bySid = @{}
foreach ($rule in $acl.GetAccessRules($true, $true, [Security.Principal.SecurityIdentifier])) { $bySid[$rule.IdentityReference.Value] = $rule }
if (!$acl.AreAccessRulesProtected -or $acl.Access.Count -ne 3 -or $bySid.Count -ne 3 -or
    $bySid[$owner].FileSystemRights -ne [Security.AccessControl.FileSystemRights]::FullControl -or
    $bySid[$system].FileSystemRights -ne [Security.AccessControl.FileSystemRights]::FullControl -or
    $bySid[$ordinary].FileSystemRights -ne ([Security.AccessControl.FileSystemRights]::ReadAndExecute -bor [Security.AccessControl.FileSystemRights]::Synchronize)) {
    throw 'The accepted staging ACL differs.'
}
$files = [ordered]@{
    'observe-r7-host-live.ps1' = (Join-Path $evidence 'observe-r7-host-live.ps1')
    'archive-r6-process-facts.cs' = (Join-Path $code 'test/windows024-native/archive-r6-process-facts.cs')
    'observe-r7-readback-timing.ps1' = $timing
}
$digests = @{}
foreach ($name in $files.Keys) { $digests[$name] = (Get-FileHash -LiteralPath $files[$name]).Hash.ToLowerInvariant() }
if ($digests['archive-r6-process-facts.cs'] -cne '1959b81e02779b9a88bb64f7321c1cc1975c68c506f4746340178b81e7d05603' -or
    $digests['observe-r7-readback-timing.ps1'] -cne '5567494d2174270d77bac6f862ab789c0d43eb3aaab7689e4e5a20e771a2f426') { throw 'Previously verified input bytes differ.' }
[void][IO.Directory]::CreateDirectory($target)
Set-Acl -LiteralPath $target -AclObject $acl
if ((Get-Acl -LiteralPath $target).Sddl -cne $acl.Sddl) { throw 'Observer folder ACL readback differs.' }
$rows = foreach ($name in $files.Keys) {
    $path = Join-Path $target $name
    [IO.File]::Copy($files[$name], $path, $false)
    $output = Get-Item -LiteralPath $path -Force
    if (($output.Attributes -band [IO.FileAttributes]::ReparsePoint) -or (Get-FileHash -LiteralPath $path).Hash -ine $digests[$name]) { throw 'Staged file differs.' }
    $fileAcl = Get-Acl -LiteralPath $path
    $access = @($fileAcl.GetAccessRules($true, $true, [Security.Principal.SecurityIdentifier]))
    if ($access.Count -ne 3 -or @($access | Where-Object { !$_.IsInherited -or $_.AccessControlType -ne 'Allow' }).Count) { throw 'Staged file has unexpected ACL.' }
    foreach ($entry in $access) {
        if ($entry.FileSystemRights -ne $bySid[$entry.IdentityReference.Value].FileSystemRights) { throw 'Staged file grants unexpected rights.' }
    }
    [ordered]@{ name = $name; length = $output.Length; sha256 = $digests[$name]; sddl = $fileAcl.Sddl }
}
if (@(Get-ChildItem -LiteralPath $target -Force).Count -ne $files.Count) { throw 'Staged directory membership differs.' }
[ordered]@{ root = $target; sddl = $acl.Sddl; files = @($rows); observedUtc = [DateTime]::UtcNow.ToString('o') } | ConvertTo-Json -Depth 8

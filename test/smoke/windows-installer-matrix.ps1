# This verifier is independent of Windows so the report contract can be checked
# with PowerShell on every host, without installing packages or editing registry.
function Get-WindowsInstallerSource([string]$ExpectedCommit) {
    if ($ExpectedCommit) {
        if ($ExpectedCommit -cnotmatch '^[0-9a-f]{40}$') { throw 'Source commit missing' }
        return $ExpectedCommit
    }
    $observed = & git rev-parse HEAD
    $gitExit = $LASTEXITCODE
    if ($gitExit -ne 0 -or !$observed -or $observed.Trim() -cnotmatch '^[0-9a-f]{40}$') { throw 'Source commit missing' }
    return $observed.Trim()
}

function Assert-WindowsInstallerMatrix {
    param(
        [ValidateSet('all','msi','nsis')][string]$Kind,
        [ValidateSet('all','default','custom')][string]$Mode,
        [AllowEmptyCollection()][object[]]$DefaultChecks,
        [AllowEmptyCollection()][object[]]$CustomChecks
    )
    $expectedKinds=if($Kind -ceq 'all'){@('msi','nsis')}else{@($Kind)}
    $expectedModes=if($Mode -ceq 'all'){@('default','custom')}else{@($Mode)}
    $expected=@(foreach($expectedKind in $expectedKinds){
        foreach($expectedMode in $expectedModes){"$expectedKind/$expectedMode"}
    })
    $observed=@(
        foreach($item in $DefaultChecks){
            if($item.result -cne 'pass'){throw 'Installer matrix contains a failed default cell'}
            "$($item.kind)/default"
        }
        foreach($item in $CustomChecks){
            if($item.result -cne 'pass'){throw 'Installer matrix contains a failed custom cell'}
            "$($item.kind)/custom"
        }
    )
    if($observed.Count -ne $expected.Count -or
       @($observed | Select-Object -Unique).Count -ne $observed.Count -or
       @($observed | Where-Object {$_ -cnotin $expected}).Count -ne 0){
        throw "Installer matrix incomplete: expected [$($expected -join ', ')]; observed [$($observed -join ', ')]"
    }
}

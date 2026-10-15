function Assert-BackupForegroundAuthorization {
    param([Parameter(Mandatory=$true)]$Authorization,
          [datetimeoffset]$AtUtc=[datetimeoffset]::UtcNow)
    if ($Authorization.origin -cne 'human' -or $Authorization.foregroundStartGranted -cne $true) {
        throw 'A new actual human foreground-start signal is required; goal continuation is insufficient.'
    }
    if ([string]::IsNullOrWhiteSpace([string]$Authorization.userText)) { throw 'Original human signal text is required.' }
    if ($Authorization.source -cne 'ca26e771b0808b8b1b58c7ffadf5d1bad46189ff' -or
        $Authorization.scope -cne 'isolated-backup-tree') { throw 'Foreground signal is not bound to this source and scope.' }
    $taskMinutes=0L
    if (![long]::TryParse([string]$Authorization.windowMinutes,[ref]$taskMinutes) -or
        $taskMinutes -lt 1 -or $taskMinutes -gt 30) { throw 'A bounded 1-30 minute foreground window is required.' }
    $taskStarted=[datetimeoffset]::Parse([string]$Authorization.atUtc,[Globalization.CultureInfo]::InvariantCulture)
    if ($taskStarted -gt $AtUtc.AddSeconds(5)) { throw 'Future-dated authorization is not accepted.' }
    $taskExpires=$taskStarted.AddMinutes($taskMinutes)
    if ($AtUtc -ge $taskExpires) { throw 'The earlier foreground window has expired.' }
    [pscustomobject]@{
        origin='human'; userText=[string]$Authorization.userText
        source=[string]$Authorization.source; scope=[string]$Authorization.scope
        startedAtUtc=$taskStarted.ToUniversalTime().ToString('o')
        expiresAtUtc=$taskExpires.ToUniversalTime().ToString('o')
        windowMinutes=$taskMinutes; remainingSeconds=($taskExpires-$AtUtc).TotalSeconds
        syntheticAuthorizationCreated=$false
    }
}

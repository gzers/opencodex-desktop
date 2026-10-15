$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'foreground-contract-live.ps1')
$taskActual = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'foreground-authorization-20261015T0943.json') -Raw | ConvertFrom-Json
$taskStart = [datetimeoffset]'2026-10-15T01:43:50.7232350Z'
$taskAt = $taskStart.AddMinutes(2)
$taskCases = [System.Collections.Generic.List[object]]::new()
function Test-TimeInput {
    param([string]$Name, $Value, [bool]$ShouldPass, [datetimeoffset]$At = $taskAt)
    $taskCopy = [pscustomobject]@{
        origin=$taskActual.origin; foregroundStartGranted=$taskActual.foregroundStartGranted
        userText=$taskActual.userText; source=$taskActual.source; scope=$taskActual.scope
        atUtc=$Value; windowMinutes=$taskActual.windowMinutes
    }
    $taskAccepted = $false
    $taskError = $null
    $taskResult = $null
    try {
        $taskResult = Assert-BackupForegroundAuthorization -Authorization $taskCopy -AtUtc $At
        $taskAccepted = $true
    } catch {
        $taskError = $_.Exception.Message
    }
    if ($taskAccepted -ne $ShouldPass) { throw "Unexpected authorization type result: $Name" }
    if ($taskAccepted) {
        if ([datetimeoffset]$taskResult.startedAtUtc -ne $taskStart -or
            [datetimeoffset]$taskResult.expiresAtUtc -ne $taskStart.AddMinutes(30)) {
            throw "UTC instant changed: $Name"
        }
    }
    $taskCases.Add([pscustomobject]@{
        name=$Name; inputType=$Value.GetType().FullName
        expectedAccepted=$ShouldPass; accepted=$taskAccepted
        atUtc=$At.ToString('o'); result=$taskResult; error=$taskError
    })
}
Test-TimeInput 'actual-json-native-date' $taskActual.atUtc $true
Test-TimeInput 'explicit-z-string' '2026-10-15T01:43:50.7232350Z' $true
Test-TimeInput 'explicit-plus-eight-string' '2026-10-15T09:43:50.7232350+08:00' $true
Test-TimeInput 'date-time-offset' $taskStart.ToOffset([timespan]::FromHours(8)) $true
Test-TimeInput 'utc-date-time' $taskStart.UtcDateTime $true
Test-TimeInput 'local-date-time' $taskStart.LocalDateTime $true
Test-TimeInput 'expired-exact-boundary' $taskActual.atUtc $false $taskStart.AddMinutes(30)
Test-TimeInput 'unspecified-date-time' ([datetime]::SpecifyKind($taskStart.UtcDateTime,[DateTimeKind]::Unspecified)) $false
Test-TimeInput 'timezone-free-string' '2026-10-15T01:43:50.7232350' $false
Test-TimeInput 'future-six-seconds' $taskActual.atUtc $false $taskStart.AddSeconds(-6)
$taskReport = [pscustomobject]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o')
    scope='Offline regression of authorization timestamp representation only'
    actualJsonDateType=$taskActual.atUtc.GetType().FullName
    cases=$taskCases.ToArray(); positiveChecks=6; negativeChecks=4
    originalAuthorizationChanged=$false; applicationStarted=$false
    inputInjected=$false; syntheticAuthorizationFileWritten=$false
    foregroundWindowExtended=$false; fullVersionGatePassed=$false
}
$taskOutput = Join-Path $PSScriptRoot 'authorization-time-type-regression.json'
if (Test-Path -LiteralPath $taskOutput) { throw 'Refusing to overwrite the original regression result.' }
$taskJson = $taskReport | ConvertTo-Json -Depth 10
[IO.File]::WriteAllText($taskOutput, $taskJson + "`n", [Text.UTF8Encoding]::new($false))
$taskJson

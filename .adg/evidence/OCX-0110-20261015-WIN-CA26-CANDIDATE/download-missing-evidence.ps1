$ErrorActionPreference = 'Stop'
$taskSource = 'ca26e771b0808b8b1b58c7ffadf5d1bad46189ff'
$taskCode = 'C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
$taskDestination = Join-Path $taskCode 'test/software/candidate-ca26e77-38663871103'
$taskArchive = Join-Path $taskDestination '11940483370.zip'
$taskPending = Join-Path $taskDestination '11940483370.net-download.zip'
$taskResultPath = Join-Path $PSScriptRoot 'missing-evidence-net-download.json'
if (Test-Path -LiteralPath $taskResultPath) { throw 'Inspect the existing read result; no automatic retry.' }
if ((Test-Path -LiteralPath $taskArchive) -or (Test-Path -LiteralPath $taskPending)) { throw 'Do not overwrite retained download bytes.' }
$taskTerminal = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'candidate-terminal.json') -Raw | ConvertFrom-Json
if ($taskTerminal.run.id -ne 38663871103 -or $taskTerminal.run.head_sha -cne $taskSource -or
    $taskTerminal.run.conclusion -ne 'success') { throw 'Exact-source terminal candidate required.' }
$taskArtifact = @($taskTerminal.artifacts.artifacts | Where-Object {$_.id -eq 11940483370})
if ($taskArtifact.Count -ne 1 -or $taskArtifact[0].name -ne 'windows-candidate-evidence') { throw 'Expected evidence artifact missing.' }
$taskArtifact = $taskArtifact[0]
$taskRecord = [ordered]@{
    atUtc=[DateTime]::UtcNow.ToString('o'); artifactId=11940483370; runId=38663871103
    transport='HttpClient; normal certificate validation; redirects followed with a new unauthenticated request'
    status='reading'; automaticRetry=$false; tlsValidationDisabled=$false
    authorizationForwardedAcrossHosts=$false; candidateRerunRequested=$false
    installerExecuted=$false; applicationStarted=$false
}
$taskHandler = [Net.Http.HttpClientHandler]::new()
$taskHandler.AllowAutoRedirect = $false
$taskClient = [Net.Http.HttpClient]::new($taskHandler)
$taskClient.Timeout = [TimeSpan]::FromSeconds(40)
$taskFile = $null
$taskInput = $null
$taskResponse = $null
try {
    $taskCredentialText = "protocol=https`nhost=github.com`n`n" | & git -C $taskCode -c credential.interactive=never credential fill
    if ($LASTEXITCODE -ne 0) { throw 'GitHub credential read unavailable.' }
    $taskToken = ($taskCredentialText | Where-Object {$_ -like 'password=*'} | Select-Object -First 1).Substring(9)
    $taskRequest = [Net.Http.HttpRequestMessage]::new([Net.Http.HttpMethod]::Get,
        'https://api.github.com/repos/gzers/opencodex-desktop/actions/artifacts/11940483370/zip')
    $taskRequest.Headers.Authorization = [Net.Http.Headers.AuthenticationHeaderValue]::new('Bearer',$taskToken)
    $taskRequest.Headers.UserAgent.ParseAdd('OpenCodex-candidate-readonly')
    $taskRequest.Headers.Accept.ParseAdd('application/vnd.github+json')
    $taskResponse = $taskClient.SendAsync($taskRequest,[Net.Http.HttpCompletionOption]::ResponseHeadersRead).GetAwaiter().GetResult()
    if ([int]$taskResponse.StatusCode -in @(301,302,303,307,308)) {
        $taskLocation = $taskResponse.Headers.Location
        if (!$taskLocation.IsAbsoluteUri -or $taskLocation.Scheme -ne 'https' -or
            !($taskLocation.Host.EndsWith('.blob.core.windows.net') -or
              $taskLocation.Host.EndsWith('.actions.githubusercontent.com'))) { throw 'Unexpected evidence download redirect host.' }
        $taskResponse.Dispose()
        $taskRequest.Dispose()
        # The fresh request deliberately carries no Authorization header.
        $taskRequest = [Net.Http.HttpRequestMessage]::new([Net.Http.HttpMethod]::Get,$taskLocation)
        $taskResponse = $taskClient.SendAsync($taskRequest,[Net.Http.HttpCompletionOption]::ResponseHeadersRead).GetAwaiter().GetResult()
    }
    [void]$taskResponse.EnsureSuccessStatusCode()
    $taskInput = $taskResponse.Content.ReadAsStream()
    $taskFile = [IO.File]::Open($taskPending,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
    $taskBytes = [byte[]]::new(65536)
    $taskWritten = 0L
    while (($taskRead = $taskInput.Read($taskBytes,0,$taskBytes.Length)) -gt 0) {
        $taskWritten += $taskRead
        if ($taskWritten -gt 32MB) { throw 'Evidence archive download limit exceeded.' }
        $taskFile.Write($taskBytes,0,$taskRead)
    }
    $taskFile.Dispose(); $taskFile = $null
    $taskDigest = (Get-FileHash -LiteralPath $taskPending).Hash.ToLowerInvariant()
    if ($taskWritten -ne $taskArtifact.size_in_bytes -or
        ('sha256:'+$taskDigest) -cne $taskArtifact.digest) { throw 'Downloaded evidence does not match original API digest/size.' }
    Move-Item -LiteralPath $taskPending -Destination $taskArchive
    $taskRecord.status='downloaded-and-api-digest-verified'
    $taskRecord.bytes=$taskWritten; $taskRecord.sha256=$taskDigest
} catch {
    $taskRecord.status='read-failed-no-automatic-retry'
    $taskRecord.errorType=$_.Exception.GetType().Name
    # Do not persist exceptions that might contain a signed redirect URL.
    $taskRecord.partialBytesRetained=Test-Path -LiteralPath $taskPending
} finally {
    if ($taskFile) { $taskFile.Dispose() }
    if ($taskInput) { $taskInput.Dispose() }
    if ($taskResponse) { $taskResponse.Dispose() }
    if ($taskRequest) { $taskRequest.Dispose() }
    $taskClient.Dispose(); $taskHandler.Dispose()
    $taskToken = $null; $taskCredentialText = $null; $taskLocation = $null
    $taskRecord | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $taskResultPath -Encoding utf8
}
$taskRecord | ConvertTo-Json -Depth 5

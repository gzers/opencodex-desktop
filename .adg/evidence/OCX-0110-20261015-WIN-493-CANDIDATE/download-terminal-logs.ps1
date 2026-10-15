$ErrorActionPreference = 'Stop'
$taskDestination = Join-Path $PSScriptRoot 'candidate-original-job-logs.zip'
$taskResultPath = Join-Path $PSScriptRoot 'candidate-net-log-download.json'
if ((Test-Path -LiteralPath $taskDestination) -or (Test-Path -LiteralPath $taskResultPath)) {
    throw 'Preserve earlier bytes and read result; do not overwrite.'
}
$taskTerminal = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'candidate-terminal.json') -Raw | ConvertFrom-Json
if ($taskTerminal.run.id -ne 38673288645 -or $taskTerminal.run.head_sha -cne '493da80675e6508e46e0fd83d86ee7216abf96fe' -or
    $taskTerminal.run.status -ne 'completed') { throw 'Exact terminal candidate required.' }
$taskHandler = [Net.Http.HttpClientHandler]::new()
$taskHandler.AllowAutoRedirect = $false
$taskClient = [Net.Http.HttpClient]::new($taskHandler)
$taskClient.Timeout = [TimeSpan]::FromSeconds(40)
$taskRecord = [ordered]@{
    atUtc=[DateTime]::UtcNow.ToString('o'); runId=38673288645
    status='reading'; tlsValidationDisabled=$false; crossHostAuthorizationForwarded=$false
    ciRerunRequested=$false; applicationStarted=$false; installerExecuted=$false
}
$taskResponse=$null; $taskRequest=$null; $taskStream=$null; $taskFile=$null
try {
    $taskCredential = "protocol=https`nhost=github.com`n`n" | & git -c credential.interactive=never credential fill
    if ($LASTEXITCODE -ne 0) { throw 'Credential read unavailable.' }
    $taskToken = ($taskCredential | Where-Object {$_ -like 'password=*'} | Select-Object -First 1).Substring(9)
    $taskRequest = [Net.Http.HttpRequestMessage]::new([Net.Http.HttpMethod]::Get,
        'https://api.github.com/repos/gzers/opencodex-desktop/actions/runs/38673288645/logs')
    $taskRequest.Headers.UserAgent.ParseAdd('OpenCodex-candidate-log-readonly')
    $taskRequest.Headers.Accept.ParseAdd('application/vnd.github+json')
    $taskRequest.Headers.Authorization = [Net.Http.Headers.AuthenticationHeaderValue]::new('Bearer',$taskToken)
    $taskResponse = $taskClient.SendAsync($taskRequest,[Net.Http.HttpCompletionOption]::ResponseHeadersRead).GetAwaiter().GetResult()
    if ([int]$taskResponse.StatusCode -in @(301,302,303,307,308)) {
        $taskLocation = $taskResponse.Headers.Location
        if (!$taskLocation.IsAbsoluteUri -or $taskLocation.Scheme -ne 'https' -or
            !($taskLocation.Host.EndsWith('.blob.core.windows.net') -or $taskLocation.Host.EndsWith('.actions.githubusercontent.com'))) {
            throw 'Unexpected log redirect.'
        }
        $taskResponse.Dispose();$taskRequest.Dispose()
        $taskRequest = [Net.Http.HttpRequestMessage]::new([Net.Http.HttpMethod]::Get,$taskLocation)
        $taskResponse = $taskClient.SendAsync($taskRequest,[Net.Http.HttpCompletionOption]::ResponseHeadersRead).GetAwaiter().GetResult()
    }
    [void]$taskResponse.EnsureSuccessStatusCode()
    $taskStream = $taskResponse.Content.ReadAsStream()
    $taskFile = [IO.File]::Open($taskDestination,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
    $taskBuffer = [byte[]]::new(65536)
    $taskBytes = 0L
    while (($taskRead=$taskStream.Read($taskBuffer,0,$taskBuffer.Length)) -gt 0) {
        $taskBytes += $taskRead
        if ($taskBytes -gt 16MB) { throw 'Log ZIP size exceeded.' }
        $taskFile.Write($taskBuffer,0,$taskRead)
    }
    $taskFile.Dispose();$taskFile=$null
    $taskRecord.status='downloaded-crc-and-contents-pending'
    $taskRecord.bytes=$taskBytes
    $taskRecord.sha256=(Get-FileHash -LiteralPath $taskDestination).Hash.ToLowerInvariant()
} catch {
    $taskRecord.status='read-failed-no-automatic-retry'
    $taskRecord.errorType=$_.Exception.GetType().Name
    $taskRecord.partialZipRetained=Test-Path -LiteralPath $taskDestination
} finally {
    if ($taskFile) {$taskFile.Dispose()}
    if ($taskStream) {$taskStream.Dispose()}
    if ($taskResponse) {$taskResponse.Dispose()}
    if ($taskRequest) {$taskRequest.Dispose()}
    $taskClient.Dispose();$taskHandler.Dispose()
    $taskToken=$null;$taskCredential=$null;$taskLocation=$null
    $taskRecord | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $taskResultPath -Encoding utf8
}
$taskRecord | ConvertTo-Json -Depth 4

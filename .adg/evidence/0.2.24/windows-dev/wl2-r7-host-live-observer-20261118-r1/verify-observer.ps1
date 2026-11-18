param([Parameter(Mandatory)][string]$CodeRoot)
$ErrorActionPreference = 'Stop'
$observer = Join-Path $PSScriptRoot 'observe-r7-host-live.ps1'
$tokens = $null
$errors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile($observer, [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw ($errors | Out-String) }
foreach ($definition in $ast.EndBlock.Statements) {
    if ($definition -is [Management.Automation.Language.FunctionDefinitionAst]) {
        . ([ScriptBlock]::Create($definition.Extent.Text))
    }
}
$nativePath = Join-Path $CodeRoot 'test/windows024-native/archive-r6-process-facts.cs'
$native = ReadBoundedBytes $nativePath 65536
$nativeSha = GetDigest $native
if ($nativeSha -cne '1959b81e02779b9a88bb64f7321c1cc1975c68c506f4746340178b81e7d05603') {
    throw 'Fixed native query source differs.'
}
$utf8 = [Text.UTF8Encoding]::new($false, $true)
Add-Type -TypeDefinition $utf8.GetString($native)
$self = [Diagnostics.Process]::GetCurrentProcess()
try {
    $time = $self.StartTime.ToUniversalTime().ToFileTimeUtc().ToString()
    $image = $self.MainModule.FileName
    $session = $self.SessionId
    $typedPid = ('{"pid":' + $PID + '}' | ConvertFrom-Json).pid
    if ($typedPid -isnot [int]) { throw 'PowerShell JSON numeric conversion differs.' }
    $match = ObserveLiveGeneration $PID $time $session $image
    $wrongTime = ObserveLiveGeneration $PID '1' $session $image
    $wrongSession = ObserveLiveGeneration $PID $time ($session + 1) $image
    $wrongImage = ObserveLiveGeneration $PID $time $session ($image + '.wrong')
    $unknown = ObserveLiveGeneration 0 $time $session $image
    if (!$match.liveProcessVerified -or $match.raw -ne 0) { throw 'Current same-user process was not proved live.' }
    foreach ($rejected in @($wrongTime, $wrongSession, $wrongImage)) {
        if ($rejected.liveProcessVerified -or $rejected.raw -ne 0 -or $rejected.firstError -notlike 'Live PID*differs*') {
            throw 'A process identity mismatch failed to refuse.'
        }
    }
    if ($unknown.liveProcessVerified -or $unknown.raw -ne 87 -or !$unknown.firstError) { throw 'Unknown target did not retain raw87.' }
    [ordered]@{
        powerShellVersion = $PSVersionTable.PSVersion.ToString()
        nativeSha256 = $nativeSha
        cases = [ordered]@{ match = $match; wrongTime = $wrongTime; wrongSession = $wrongSession; wrongImage = $wrongImage; unknown = $unknown }
    } | ConvertTo-Json -Depth 8
} finally { $self.Dispose() }

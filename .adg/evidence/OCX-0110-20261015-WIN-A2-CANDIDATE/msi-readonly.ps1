param(
    [Parameter(Mandatory=$true)][string]$MsiPath,
    [Parameter(Mandatory=$true)][string]$OutputDirectory,
    [Parameter(Mandatory=$true)][string]$AllowedOutputRoot,
    [string]$ExpectedSha256,
    [switch]$ExtractExecutable
)
$ErrorActionPreference = 'Stop'
$taskMsi = (Resolve-Path -LiteralPath $MsiPath).Path
$taskAllowed = (Resolve-Path -LiteralPath $AllowedOutputRoot).Path.TrimEnd('\')
$taskOutput = [IO.Path]::GetFullPath($OutputDirectory)
if (!$taskOutput.StartsWith($taskAllowed + '\', [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Output must be a child of the explicitly allowed isolated root'
}
if (Test-Path -LiteralPath $taskOutput) { throw 'Refusing to reuse an existing extraction directory' }
$taskHash = (Get-FileHash -LiteralPath $taskMsi -Algorithm SHA256).Hash.ToLower()
if ($ExtractExecutable -and $ExpectedSha256 -notmatch '^[0-9a-f]{64}$') {
    throw 'Verified package SHA-256 is required before extracting an executable'
}
if ($ExpectedSha256 -and $ExpectedSha256 -cne $taskHash) { throw 'MSI SHA-256 mismatch' }
New-Item -ItemType Directory -Path $taskOutput | Out-Null
$taskInstaller = New-Object -ComObject WindowsInstaller.Installer
$taskDatabase = $null
$taskResult = [ordered]@{
    schemaVersion=1; collectedAtUtc=[datetime]::UtcNow.ToString('o')
    package=$taskMsi; packageSha256=$taskHash; databaseMode='read-only'
    installerExecuted=$false; properties=@{}; media=@(); files=@(); upgrades=@()
    extractedExecutables=@()
}
function Read-MsiRows([string]$Sql, [string[]]$Columns) {
    $taskView = $taskDatabase.OpenView($Sql)
    try {
        [void]$taskView.Execute()
        while ($taskRecord = $taskView.Fetch()) {
            try {
                $taskRow = [ordered]@{}
                for ($taskIndex=0; $taskIndex -lt $Columns.Count; $taskIndex++) {
                    $taskRow[$Columns[$taskIndex]] = $taskRecord.StringData($taskIndex+1)
                }
                [pscustomobject]$taskRow
            } finally { [void][Runtime.InteropServices.Marshal]::ReleaseComObject($taskRecord) }
        }
    } finally {
        [void]$taskView.Close()
        [void][Runtime.InteropServices.Marshal]::ReleaseComObject($taskView)
    }
}
try {
    $taskDatabase = $taskInstaller.OpenDatabase($taskMsi, 0)
    $taskSql = @'
SELECT `Property`, `Value` FROM `Property`
'@
    foreach ($taskRow in @(Read-MsiRows $taskSql @('property','value'))) {
        if ($taskRow.property -in @('ProductCode','UpgradeCode','ProductVersion','ProductName','ALLUSERS','MSIINSTALLPERUSER')) {
            $taskResult.properties[$taskRow.property] = $taskRow.value
        }
    }
    $taskSql = @'
SELECT `DiskId`, `Cabinet`, `LastSequence` FROM `Media`
'@
    $taskResult.media = @(Read-MsiRows $taskSql @('diskId','cabinet','lastSequence'))
    $taskSql = @'
SELECT `File`, `FileName`, `FileSize`, `Sequence` FROM `File`
'@
    $taskResult.files = @(Read-MsiRows $taskSql @('id','fileName','bytes','sequence'))
    $taskSql = @'
SELECT `UpgradeCode`, `VersionMin`, `VersionMax`, `Attributes`, `ActionProperty` FROM `Upgrade`
'@
    $taskResult.upgrades = @(Read-MsiRows $taskSql @('upgradeCode','versionMin','versionMax','attributes','actionProperty'))
    if ($ExtractExecutable) {
        foreach ($taskMedia in $taskResult.media) {
            if ($taskMedia.cabinet -notmatch '^#([A-Za-z0-9_.-]+)$') {
                throw 'Only an embedded cabinet with a bounded stream name is supported'
            }
            $taskCabinet = $Matches[1]
            $taskSql = 'SELECT `Data` FROM `_Streams` WHERE `Name` = ' + "'" + $taskCabinet + "'"
            $taskView = $taskDatabase.OpenView($taskSql)
            $taskRecord = $null
            try {
                [void]$taskView.Execute()
                $taskRecord = $taskView.Fetch()
                if (!$taskRecord) { throw 'Embedded cabinet stream missing' }
                $taskCabPath = Join-Path $taskOutput $taskCabinet
                $taskFile = [IO.File]::Open($taskCabPath, [IO.FileMode]::CreateNew)
                try {
                    while ($true) {
                        # msiReadStreamBytes returns one byte per BSTR character.
                        $taskChunk = [string]$taskRecord.ReadStream(1, 65536, 1)
                        if (!$taskChunk.Length) { break }
                        $taskBytes = [byte[]]::new($taskChunk.Length)
                        for ($taskIndex=0; $taskIndex -lt $taskChunk.Length; $taskIndex++) {
                            $taskBytes[$taskIndex] = [byte][int][char]$taskChunk[$taskIndex]
                        }
                        $taskFile.Write($taskBytes, 0, $taskBytes.Length)
                    }
                } finally { $taskFile.Dispose() }
                $taskExpanded = Join-Path $taskOutput ('cab-' + $taskMedia.diskId)
                New-Item -ItemType Directory -Path $taskExpanded | Out-Null
                & "$env:SystemRoot\System32\expand.exe" '-F:*' $taskCabPath $taskExpanded *> (Join-Path $taskOutput ('expand-' + $taskMedia.diskId + '.log'))
                if ($LASTEXITCODE -ne 0) { throw 'Cabinet extraction failed' }
                foreach ($taskEntry in $taskResult.files) {
                    $taskName = ($taskEntry.fileName -split '\|')[-1]
                    if ($taskName -notin @('opencodex-desktop.exe','ocxd.exe')) { continue }
                    if ($taskEntry.id -notmatch '^[A-Za-z0-9_.-]+$') { throw 'Unexpected cabinet file identifier' }
                    $taskSource = Join-Path $taskExpanded $taskEntry.id
                    if (!(Test-Path -LiteralPath $taskSource -PathType Leaf)) { continue }
                    $taskDestination = Join-Path $taskOutput $taskName
                    Copy-Item -LiteralPath $taskSource -Destination $taskDestination
                    $taskExe = Get-Item -LiteralPath $taskDestination
                    if ($taskExe.Length -ne [long]$taskEntry.bytes) { throw 'Extracted file size mismatch' }
                    $taskResult.extractedExecutables += [ordered]@{
                        path=$taskDestination; name=$taskName; bytes=$taskExe.Length
                        sha256=(Get-FileHash -LiteralPath $taskDestination).Hash.ToLower()
                        productVersion=$taskExe.VersionInfo.ProductVersion
                    }
                }
            } finally {
                if ($taskRecord) { [void][Runtime.InteropServices.Marshal]::ReleaseComObject($taskRecord) }
                [void]$taskView.Close()
                [void][Runtime.InteropServices.Marshal]::ReleaseComObject($taskView)
            }
        }
        if (@($taskResult.extractedExecutables | Where-Object {$_.name -eq 'opencodex-desktop.exe'}).Count -ne 1) {
            throw 'Expected exactly one extracted application executable'
        }
    }
    $taskResult.result='pass'
} catch {
    $taskResult.result='fail'; $taskResult.error=$_.Exception.Message
    throw
} finally {
    $taskResult | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $taskOutput 'msi-readonly.json') -Encoding utf8
    if ($taskDatabase) { [void][Runtime.InteropServices.Marshal]::ReleaseComObject($taskDatabase) }
    [void][Runtime.InteropServices.Marshal]::ReleaseComObject($taskInstaller)
}
$taskResult | ConvertTo-Json -Depth 8

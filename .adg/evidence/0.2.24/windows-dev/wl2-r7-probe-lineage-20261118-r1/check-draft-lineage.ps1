param(
    [Parameter(Mandatory)][string]$CodeRoot,
    [Parameter(Mandatory)][string]$BuildEvidence
)
$ErrorActionPreference = 'Stop'
$baseline = 'f3564fbb34713beee97ed221fb4797b422b11d57'
$verifiedHead = '673d1166334c4fb78d5094bc09ef130e578e0991'
$verifier = 'test/windows024-native/verify-r7-host-receipts.py'
$expectedImage = '1b482ce541885a141dbd672497643ed65d16fee28644b81d36f0d5478fd427b8'
function ReadGit([string[]]$GitArguments) {
    $answer = @(git -C $CodeRoot @GitArguments)
    if ($LASTEXITCODE -ne 0) { throw "Git returned $LASTEXITCODE for $GitArguments" }
    $answer
}
function Sha([string]$Path) {
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}
$head = ReadGit @('rev-parse','HEAD')
if ($head -cne $verifiedHead -or (ReadGit @('rev-parse','HEAD^')) -cne $baseline) {
    throw 'Current draft parent or its direct predecessor differs'
}
$committed = @(ReadGit @('diff','--name-only','--no-renames',$baseline,$head))
if ($committed.Count -ne 1 -or $committed[0] -cne $verifier) {
    throw 'Draft parent changed outside the independent receipt verifier'
}
if (ReadGit @('diff','--cached','--name-only')) { throw 'Unexpected staged inputs' }
$sources = Get-Content -Raw -LiteralPath (Join-Path $BuildEvidence 'sources.json') | ConvertFrom-Json
$paths = @($sources.sources | ForEach-Object path)
$dirty = @(ReadGit @('-c','core.safecrlf=false','diff','--name-only','--no-renames','HEAD'))
if ($sources.baseline -cne $baseline -or $paths.Count -ne 7 -or $dirty.Count -ne $paths.Count -or
    @($paths | Sort-Object -Unique).Count -ne $paths.Count -or (Compare-Object $paths $dirty -CaseSensitive)) {
    throw 'Draft source membership differs from the build evidence'
}
foreach ($entry in $sources.sources) {
    $path = Join-Path $CodeRoot $entry.path
    if ((Get-Item -LiteralPath $path).Length -ne $entry.bytes -or (Sha $path) -cne $entry.sha256) {
        throw "Raw source drift $($entry.path)"
    }
}
$image = Get-Content -Raw -LiteralPath (Join-Path $BuildEvidence 'image.json') | ConvertFrom-Json
if ($image.sha256 -cne $expectedImage -or (Get-Item -LiteralPath $image.path).Length -ne $image.bytes -or
    (Sha $image.path) -cne $expectedImage) { throw 'Draft image differs from the build evidence' }
[ordered]@{
    auditedAt=(Get-Date -Format o)
    buildEvidenceCommit='644f8bc861bd7a1009aed9c5789699ff569f0895'
    buildBaseline=$baseline
    draftParent=$head
    interveningPaths=$committed
    sources=$sources.sources
    imageSha256=$expectedImage
    imageBytes=$image.bytes
    sourceCommitFixed=$false
    stageExecuted=$false
} | ConvertTo-Json -Depth 5

param([Parameter(Mandatory)][string]$Code,[Parameter(Mandatory)][string]$Evidence)
$ErrorActionPreference='Stop'
$scriptPath=Join-Path $Code 'test/windows024-native/archive-h024-fixed-success.ps1'
$tokens=$null;$errors=$null
$ast=[Management.Automation.Language.Parser]::ParseFile($scriptPath,[ref]$tokens,[ref]$errors)
if(@($errors).Count -ne 0){throw 'cleanup script parse failed'}
foreach($function in $ast.FindAll({param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst]},$false)){
 # Load only actual pure archive helpers. Never execute the desktop entry,
 # receipt, permission changes or Remove-Item during controller verification.
 Invoke-Expression $function.Extent.Text
}
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$base=Join-Path $Code ('test/windows024-native/target/gui-archive-check-'+[guid]::NewGuid().ToString())
[void][IO.Directory]::CreateDirectory($base)
$target=Join-Path $base 'source';[void][IO.Directory]::CreateDirectory($target)
[void][IO.Directory]::CreateDirectory((Join-Path $target 'empty'))
[IO.File]::WriteAllText((Join-Path $target 'task.txt'),'isolated archive positive',[Text.UTF8Encoding]::new($false))
[IO.File]::WriteAllBytes((Join-Path $target 'raw.bin'),[byte[]](0,1,2,255,0))
$rows=@(Inventory $target)
$zipPath=Join-Path $base 'positive.zip'
$positive=Archive-Tree $target $zipPath $rows
if(!$positive.verified -or $positive.files -ne 2 -or $positive.directories -ne 1){throw 'positive full ZIP byte/readback verification failed'}
$archive=[IO.Compression.ZipFile]::OpenRead($zipPath)
try{if(!$archive.GetEntry('empty/')){throw 'empty directory was lost'};$entry=$archive.GetEntry('raw.bin');$stream=$entry.Open();try{if((Digest-Stream $stream) -cne (Digest-File (Join-Path $target 'raw.bin'))){throw 'independent ZIP read differs'}}finally{$stream.Dispose()}}finally{$archive.Dispose()}
[IO.File]::WriteAllText((Join-Path $target 'task.txt'),'isolated archive negative',[Text.UTF8Encoding]::new($false))
$changedRefused=$false
try{[void](Archive-Tree $target (Join-Path $base 'changed-source.zip') $rows)}catch{$changedRefused=$true;$changedError=$_.Exception.Message}
if(!$changedRefused){throw 'changed source bytes were accepted'}
$current=@(Inventory $target)
[void][IO.Directory]::CreateDirectory((Join-Path $target 'unexpected-directory'))
$setRefused=$false
try{Same-Inventory $current (Inventory $target)}catch{$setRefused=$true;$setError=$_.Exception.Message}
if(!$setRefused){throw 'additional empty directory was accepted'}
if(!(Test-Path -LiteralPath (Join-Path $target 'task.txt')) -or !(Test-Path -LiteralPath (Join-Path $target 'raw.bin'))){throw 'primitive checks deleted original fixture'}
@{rawExit=0;parseErrors=0;positiveZipVerified=$true;sourceChangeRefused=$changedRefused;sourceChangeError=$changedError;extraDirectoryRefused=$setRefused;extraDirectoryError=$setError;fixture=$base;fixtureRetained=$true;deletionAttempted=$false;desktopStarted=$false;officialStarted=$false;productionAcceptance=$false;sourceSha256=(Get-FileHash -LiteralPath $scriptPath -Algorithm SHA256).Hash.ToLowerInvariant()}|ConvertTo-Json -Depth 7|Set-Content -LiteralPath (Join-Path $Evidence 'archive-primitives-result.json') -Encoding utf8
Write-Output 'Archive bytes and complete entry sets verified; changed content/extra empty directory refused; no deletion or desktop launch.'

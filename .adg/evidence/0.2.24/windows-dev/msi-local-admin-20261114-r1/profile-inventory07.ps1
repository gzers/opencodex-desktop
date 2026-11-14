$ErrorActionPreference='Stop'
$root='C:/Users/Public/ocx-msi-admin-20261114-r1'
$source='C:/Users/ocxw1d1d3a825b8'
$destination=Join-Path $root 'inventory-placeholder'
if(Test-Path -LiteralPath $destination){throw 'read-only inventory placeholder must not exist'}
& robocopy.exe $source $destination /L /E /B /BYTES /R:0 /W:0 /XJ /XJD /XJF /NP /NFL /NDL ('/UNILOG:'+(Join-Path $root 'profile-inventory07.log'))|Out-Null
@{exitCode=$LASTEXITCODE;scope='read-only exact test Profile backup-semantics inventory; /L no copy or deletion';source=$source;destinationPresent=Test-Path -LiteralPath $destination}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $root 'profile-inventory07.json') -Encoding utf8

param([Parameter(Mandatory)][string]$Evidence)
$ErrorActionPreference='Stop'
$evidencePath=[IO.Path]::GetFullPath($Evidence)
if(Test-Path -LiteralPath $evidencePath){throw 'fresh immutable probe output required'}
[void][IO.Directory]::CreateDirectory($evidencePath)
$source=Join-Path $PSScriptRoot 'cleanup-r6-authorized-processes.cs'
$parameters=New-Object CodeDom.Compiler.CompilerParameters
$parameters.CompilerOptions='/define:R6_CLEANUP_PROBE'
[void]$parameters.ReferencedAssemblies.Add('System.dll')
[void]$parameters.ReferencedAssemblies.Add('System.Core.dll')
$result=$null
try {
 Add-Type -TypeDefinition ([IO.File]::ReadAllText($source)) -CompilerParameters $parameters
 $fixture=Join-Path $PSScriptRoot ('target/r6-authorized-cleanup-probe-'+[guid]::NewGuid().ToString('N'))
 $result=[R6AuthorizedFailureCleanup]::Probe($fixture)
 $result|Add-Member NoteProperty fixture $fixture
 $result|Add-Member NoteProperty sourceSha256 (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant()
 $result|ConvertTo-Json -Depth 12|Set-Content (Join-Path $evidencePath 'result.json') -Encoding UTF8
 if(!$result.passed){throw 'same-helper actual suspended process probe failed'}
 Write-Host 'Own never-resumed pair: rejection cases passed; original handles confirmed forced242/242. No ordinary acceptance.'
}catch {
 $_|Out-String|Set-Content (Join-Path $evidencePath 'error.log') -Encoding UTF8
 if($null -eq $result){@{passed=$false;firstError=$_.Exception.ToString();ordinaryAcceptance=$false}|ConvertTo-Json -Depth 4|Set-Content (Join-Path $evidencePath 'result.json') -Encoding UTF8}
 exit 1
}

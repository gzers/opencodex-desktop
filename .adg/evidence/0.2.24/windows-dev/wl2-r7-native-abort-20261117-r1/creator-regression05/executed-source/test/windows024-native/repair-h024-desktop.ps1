$ErrorActionPreference='Stop'
$root=$PSScriptRoot
if([IO.Path]::GetFullPath($root) -cne [IO.Path]::GetFullPath('C:/Users/Public/ocx-h024-repaired-20261115-r1')){throw 'exact prepared repaired task root required'}
$output=Join-Path $root 'out01'
if(Test-Path -LiteralPath (Join-Path $output 'repair-sequence-result.json')){throw 'immutable repaired round already completed; no repeat'}
$result=[ordered]@{scope='one explicit old empty-host recovery then new H component round; no official runtime';accepted=$false}
try{
 & (Join-Path $root 'recover-h024-empty-original.ps1') -Evidence $output
 $result.originalEmptyRecoveryPassed=$true
 & (Join-Path $root 'launch-h024-desktop.ps1') -Root $root
 $result.repairedHLaunchFinished=$true
}catch{$result.error=@{message=$_.Exception.ToString();line=$_.InvocationInfo.ScriptLineNumber}}
finally{
 try{& (Join-Path $root 'export-h024-task.ps1') -Root $root;$result.finalExportAttempted=$true}catch{$result.exportError=$_.Exception.ToString()}
 $result|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $output 'repair-sequence-result.json') -Encoding utf8
}

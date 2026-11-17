param([Parameter(Mandatory)][string]$Evidence)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $Evidence){throw 'fresh immutable DATA comparison evidence required'}
[void][IO.Directory]::CreateDirectory($Evidence)
$path='C:/Users/Public/ocx-r7-r2-readonly-20261117-r1/out01/archive-result.json'
$raw=[IO.File]::ReadAllBytes($path)
$receipt=Get-Content -LiteralPath $path -Raw -Encoding UTF8|ConvertFrom-Json
if(!$receipt.archiveVerified -or !$receipt.nativeBeforePassed -or !$receipt.nativeAfterPassed -or $receipt.forced -or $receipt.restored){throw 'exact unchanged read-only receipt required'}
Add-Type -Path (Join-Path $PSScriptRoot 'r7-readonly-scene.cs')
function Facts {
 $items=@($receipt.processBefore|ForEach-Object {$f=New-Object 'R7ReadOnlyScene+ProcessFacts';$f.pid=$_.pid;$f.image=$_.image;$f.exited=$_.exited;$f})
 return ,([R7ReadOnlyScene+ProcessFacts[]]$items)
}
$result=[ordered]@{scope='comparison of existing actual native/WMI observations DATA; no new process access or native authority';rawReceiptSha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant();passed=$false;rawOriginalFirstError=$receipt.errors;comparisons=@();refused=@();originalSceneTouched=$false;nativeAuthority=$false;forced=$false;restored=$false;closed=$false}
try{
 foreach($row in $receipt.processLineage){
  $fact=[R7ReadOnlyScene]::CompareLineageData((Facts),[uint32]$row.ProcessId,[uint32]$row.ParentProcessId,$row.ExecutablePath,$row.CommandLine)
  if(!$fact.consistent -or $fact.nativeAuthority){throw 'DATA comparison result invalid'}
  $result.comparisons+=@($fact)
 }
 $node=@($receipt.processLineage|Where-Object {$_.ProcessId -eq 21500})[0]
 foreach($case in @('native-image','contradictory-wmi','parent','command-image','command-script','port','missing-native-pair','old-target','duplicate-native')){
  $facts=Facts;$pidValue=[uint32]21500;$parent=[uint32]75744;$image=$node.ExecutablePath;$command=$node.CommandLine
  switch($case){
   'native-image' {$facts[0].image='C:/foreign.exe'}
   'contradictory-wmi' {$image='C:/foreign.exe'}
   'parent' {$parent=40588}
   'command-image' {$command='"C:/foreign.exe" "script" start --port 0'}
   'command-script' {$command='"'+$facts[0].image+'" "C:/foreign.mjs" start --port 0'}
   'port' {$command=$command.Replace('"0"','"1"')}
   'missing-native-pair' {$facts=[R7ReadOnlyScene+ProcessFacts[]]@()}
   'old-target' {$pidValue=50580}
   'duplicate-native' {$facts[1].pid=21500}
  }
  $refused=$false
  try{[void][R7ReadOnlyScene]::CompareLineageData($facts,$pidValue,$parent,$image,$command)}
  catch{$refused=$true;$result.refused+=@{case=$case;error=$_.Exception.ToString()}}
  if(!$refused){throw ('wrong lineage accepted: '+$case)}
 }
 $result.passed=$true
}catch{$result.firstError=$_.Exception.ToString()}
finally{$result|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $Evidence 'result.json') -Encoding utf8}
if(!$result.passed){exit 1}

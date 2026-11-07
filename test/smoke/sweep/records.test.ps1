param([Parameter(Mandatory=$true)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'records.ps1')
if(Test-Path -LiteralPath $OutputDirectory){throw 'Fresh behavior test directory required'}
$failed=Invoke-SweepCheck $OutputDirectory '01' 'first' {throw ('actual first error'+[char]10+'second line')}
$passed=Invoke-SweepCheck $OutputDirectory '02' 'later' {@{actual='executed after failure'}}
if($failed.status -cne 'fail' -or $failed.firstError -cne 'actual first error' -or $passed.status -cne 'pass' -or $passed.accepted -ne $false){throw 'Continuation/result contract failed'}
$before=(Get-FileHash (Join-Path $OutputDirectory '01-first.json')).Hash
$rejected=$false
try{Write-SweepRecord $OutputDirectory '01' 'first' pass|Out-Null}catch{$rejected=$true}
if(!$rejected -or (Get-FileHash (Join-Path $OutputDirectory '01-first.json')).Hash -cne $before){throw 'Failure evidence was overwritten'}
. (Join-Path $PSScriptRoot '../windows023-two-user-native-helper.ps1')
$c=@{diagnosticSweep=$true};$out=Join-Path $OutputDirectory 'user1';New-Item -ItemType Directory $out|Out-Null
Invoke-TwoUserSweepCheck 'supply' {throw 'unresolved actual source'}|Out-Null
Invoke-TwoUserSweepCheck 'ipc' {@{ran=$true}}|Out-Null
if((Get-Content (Join-Path $out 'sweep-supply.json') -Raw|ConvertFrom-Json).status -cne 'fail'){throw 'Unready supply was accepted'}
if((Get-Content (Join-Path $out 'sweep-ipc.json') -Raw|ConvertFrom-Json).status -cne 'pass'){throw 'Independent IPC action was suppressed'}
$c.diagnosticSweep=$false;$rejected=$false
try{Invoke-TwoUserSweepCheck 'legacy' {throw 'original assertion'}|Out-Null}catch{$rejected=$true}
if(!$rejected){throw 'Normal acceptance assertion was swallowed'}
Write-Output 'PASS: wrappers continue, refuse overwrite, and preserve normal strict assertions'

$busy=Get-TwoUserDiagnosticStartupState @(@{sid='A';failed=$true;ready=$false},@{sid='B';failed=$false;ready=$false})
if($busy.ready -or $busy.allSettled){throw 'One failed SID prematurely stopped the independently running second SID'}
$settled=Get-TwoUserDiagnosticStartupState @(@{sid='A';failed=$true;ready=$false},@{sid='B';failed=$false;ready=$true})
if($settled.ready -or !$settled.allSettled -or $settled.failedSids.Count -ne 1){throw 'Failed SID became ready or settled independent result was lost'}
$both=Get-TwoUserDiagnosticStartupState @(@{sid='A';failed=$false;ready=$true},@{sid='B';failed=$false;ready=$true})
if(!$both.ready -or !$both.allSettled -or $both.failedSids.Count){throw 'Two real ready coordination receipts were not handled'}
Write-Output 'PASS: real controller state function waits for the independent SID and never converts failure to ready'

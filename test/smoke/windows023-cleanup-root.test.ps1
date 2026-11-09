$ErrorActionPreference='Stop'
. ([IO.Path]::Combine($PSScriptRoot,'windows023-two-user-native-helper.ps1'))
$script:observations=0;$script:exited=$false
function Test-TwoUserOriginalExited($Fact){$script:observations++;return $script:exited}
function Assert-TwoUserLive($Fact){if($script:race){$script:exited=$true;throw 'original exited in observation'};if($script:unknown){throw 'unknown live owner'};return $Fact}
$fact=@{pid=42;sid='synthetic';creationTicks='1'}
if((Assert-TwoUserRootForDescendantObservation $fact)-cne 'original-live'){throw 'Live root not admitted'}
$script:race=$true
if((Assert-TwoUserRootForDescendantObservation $fact)-cne 'original-exited'){throw 'Same retained handle exit not recognized'}
$script:race=$false;$script:exited=$true
if((Assert-TwoUserRootForDescendantObservation $fact)-cne 'original-exited'){throw 'Positive native exit refused'}
$script:exited=$false;$script:unknown=$true;$rejected=$false
try{$null=Assert-TwoUserRootForDescendantObservation $fact}catch{$rejected=$true}
if(!$rejected){throw 'Unknown live root was skipped'}
[Console]::WriteLine('Exact extracted root admission: live, positive original exit, exit race, unreadable-live refusal pass; native process implementation not mocked into acceptance')

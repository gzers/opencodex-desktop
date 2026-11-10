function Read-ProofBinding([string]$Path) {
 try { return @{readable=$true;value=([IO.File]::ReadAllText($Path)|ConvertFrom-Json -AsHashtable)} }
 catch { return @{readable=$false;value=$null;failureType=$_.Exception.GetType().FullName} }
}
function Get-ProofDecision($Expected,$Readback,$Observed,[string]$HomeIdentity) {
 if(!$Readback.readable -or $null -eq $Readback.value -or $null -eq $Observed -or !$HomeIdentity){return 'unknown'}
 foreach($key in @('schema','pid','filetime','sid','home','homeIdentity')) {
  if(!$Readback.value.ContainsKey($key) -or $null -eq $Readback.value[$key]){return 'unknown'}
  if([string]$Readback.value[$key] -cne [string]$Expected[$key]){return 'refused'}
 }
 if($Observed.pid -ne $Expected.pid -or $Observed.filetime -cne $Expected.filetime -or $Observed.sid -cne $Expected.sid -or $Observed.elevated -or $HomeIdentity -cne $Expected.homeIdentity){return 'refused'}
 return 'admitted'
}

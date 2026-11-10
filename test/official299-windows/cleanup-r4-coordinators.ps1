param([Parameter(Mandatory)][string]$EvidenceRoot)
$ErrorActionPreference='Stop';Set-StrictMode -Version Latest
Add-Type -Path (Join-Path $PSScriptRoot Native.cs),(Join-Path $PSScriptRoot Recovery.cs)
$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value;$runtimeBase=Join-Path $env:LOCALAPPDATA ('OpenCodex/Runtime/v1/'+$sid);$receipts=@();$pass=$false
try{
 foreach($n in 1..4){
  $key='prototype-coordinator-'+$n.ToString('00');$dir=Join-Path $env:TEMP ('ocxd299-r4-coordinator-'+$n.ToString('00'));$raw=Get-Content -LiteralPath (Join-Path $EvidenceRoot ($key+'/coordinator-result.json')) -Raw|ConvertFrom-Json
  $expectedHome=Join-Path $dir codex;if($raw.home -ine $expectedHome){throw 'test coordinator HOME mismatch'}
  $normalized=[IO.Path]::GetFullPath($raw.home);$sha=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($normalized))).ToLowerInvariant()
  foreach($pair in @(@{kind='native-write-locks';path=$raw.namespace.npath},@{kind='catalog-write-locks';path=$raw.namespace.kpath})){
   $path=[IO.Path]::GetFullPath($pair.path);$expected=Join-Path $runtimeBase ($pair.kind+'/'+$sha+'.sqlite')
   if($path -ine $expected){throw 'refuse coordinator path not derived from precise new HOME/SID'}
   if(!(Test-Path -LiteralPath $path)){$receipts+=@{prototype=$key;path=$path;before='absent';after='absent'};continue}
   $g=[Official299Proof.Guard]::new($path,$false);try{$id=$g.Identity}finally{$g.Dispose()}
   $acl=Get-Acl $path;if($acl.GetOwner([Security.Principal.SecurityIdentifier]).Value -cne $sid){throw 'coordinator owner differs'}
   $digest=[Official299Proof.Recovery]::RemoveExactFixtureFile($path,$id)
   $receipts+=@{prototype=$key;path=$path;fileIdentity=$id;sha256Before=$digest;owner=$sid;sddl=$acl.Sddl;exclusiveNativeDeleteOnSameHandle=$true;existsAfter=(Test-Path -LiteralPath $path)}
   if(Test-Path -LiteralPath $path){throw 'test coordinator object remains'}
  }
 }
 $pass=$true
}catch{$receipts+=@{error=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
[IO.File]::WriteAllText((Join-Path $EvidenceRoot prototype-coordinator-cleanup.json),(@{pass=$pass;receipts=$receipts;stableHomeLocksUnlinked=$false;scope='only exact N/K namespace objects of four new small-test HOMEs; stable claim/owner test fixture files preserved with their evidence'}|ConvertTo-Json -Depth 12))
if(!$pass){exit 1}

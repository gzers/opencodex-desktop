param([Parameter(Mandatory)][string]$Evidence,[Parameter(Mandatory)][string]$PrivateParent)
$ErrorActionPreference='Stop'
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
$principal=[Security.Principal.WindowsPrincipal]::new($identity)
if($principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw 'ordinary existing token required'}
$parent=(Resolve-Path -LiteralPath $PrivateParent).Path
$acl=Get-Acl -LiteralPath $parent
if(-not $acl.AreAccessRulesProtected){throw 'new protected task-private parent required'}
foreach($rule in $acl.GetAccessRules($true,$true,[Security.Principal.SecurityIdentifier])){
 if($rule.IdentityReference.Value -ne $identity.User.Value){throw 'foreign private-parent rule refused'}
}
$base=Join-Path $parent 'populated-probe01'
if(Test-Path -LiteralPath $base){throw 'probe scene already exists; no retry'}
[void][IO.Directory]::CreateDirectory($base)
Add-Type -Path "$PSScriptRoot/populated-security-probe.cs"
$sid=$identity.User.Value
$prefix='O:'+ $sid +'D:P(A;;FA;;;'+$sid+')'
$trials=@(
 @{name='conditional-dacl';mask=4;ace=9;suffix='(XA;;FR;;;WD;(@Resource.Project == "Windows"))'},
 @{name='resource-attribute';mask=32;ace=18;suffix='S:(RA;;;;;WD;("Project",TS,0x0,"Windows"))'},
 @{name='scoped-policy';mask=64;ace=19;suffix='S:(SP;;;;;S-1-17-1-2-3-4-5-6-7-8)'},
 @{name='process-trust';mask=128;ace=20;suffix='S:(TL;;0x00120089;;;S-1-19-512-4096)'},
 @{name='access-filter';mask=256;ace=21;suffix='S:(FL;;FR;;;WD;(@Resource.Project == "Windows"))'}
)
$results=@()
foreach($trial in $trials){
 $results += [PopulatedSecurityProbe024]::Trial($trial.name,(Join-Path $base ($trial.name+'.synthetic')),$prefix+$trial.suffix,[uint32]$trial.mask,[byte]$trial.ace)
}
@{time=[DateTimeOffset]::Now.ToString('o');sid=$sid;session=[Diagnostics.Process]::GetCurrentProcess().SessionId;elevated=$false;os=[Environment]::OSVersion.Version.ToString();base=$base;trials=$results;accepted=$false;productionExecuted=$false;scope='new-fixture-creation-and-query-feasibility-only';privilegesOrPolicyChanged=$false;fullSaclRequested=$false} | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $Evidence 'populated-probe01.json') -Encoding utf8
foreach($result in $results){[PSCustomObject]@{Name=$result.Name;Created=$result.Created;Stage=$result.Stage;Raw=$result.RawStatus;TargetMask=$result.TargetMask}}

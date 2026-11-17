param([Parameter(Mandatory)][string]$CodeRoot,[Parameter(Mandatory)][string]$Output)
$ErrorActionPreference='Stop'
$rows=@();$launcher=$null
foreach($name in @('launch-r6-official-existing-r2.ps1','export-r6-official-existing-r2.ps1')){
 $tokens=$null;$errors=$null;$path=Join-Path $CodeRoot ('test/windows024-native/'+$name)
 $ast=[Management.Automation.Language.Parser]::ParseFile($path,[ref]$tokens,[ref]$errors)
 if(@($errors).Count){throw ($errors|Out-String)}
 $rows+=@{path=$path;sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant();parseErrors=0}
 if($name -like 'launch-*'){$launcher=$ast}
}
$id=[Security.Principal.WindowsIdentity]::GetCurrent()
$base=Join-Path $CodeRoot ('test/windows024-native/target/r6-fresh-dir-probe-'+[guid]::NewGuid().ToString('N'))
$function=$launcher.Find({param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq 'NewPrivateTaskDirectory'},$true)
if($null -eq $function){throw 'exact source private-directory function absent'}
. ([ScriptBlock]::Create($function.Extent.Text))
NewPrivateTaskDirectory $base
$directories=@($base)
foreach($name in @('I','D','H')){$path=Join-Path $base $name;NewPrivateTaskDirectory $path;$directories+=$path}
$securityRows=@()
foreach($path in $directories){
 $acl=Get-Acl -LiteralPath $path
 $owner=$acl.GetOwner([Security.Principal.SecurityIdentifier]).Value
 $sids=@($acl.Access|ForEach-Object {$_.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value}|Sort-Object)
 if($owner -cne $id.User.Value -or !$acl.AreAccessRulesProtected -or $sids.Count -ne 2 -or $sids[0] -cne 'S-1-5-18' -or $sids[1] -cne $id.User.Value){throw 'same source private directory owner/DACL differs'}
 $securityRows+=@{path=$path;owner=$owner;protected=$acl.AreAccessRulesProtected;sddl=$acl.Sddl}
}
$existingRefused=$false;try{NewPrivateTaskDirectory $base}catch{$existingRefused=$_.Exception.Message -like '*already exists*'}
$foreignRefused=$false;try{NewPrivateTaskDirectory (Join-Path $base 'foreign')}catch{$foreignRefused=$_.Exception.Message -like '*exact fresh task directory only*'}
if(!$existingRefused -or !$foreignRefused){throw 'existing/foreign fixture directory accepted'}
@{scripts=$rows;privateDirectoryProbe=$securityRows;existingRefused=$existingRefused;foreignRefused=$foreignRefused;fixtureRetained=$true;ordinaryAcceptance=$false;officialStarted=$false}|ConvertTo-Json -Depth 8|Set-Content -LiteralPath $Output -Encoding UTF8

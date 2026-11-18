$ErrorActionPreference='Stop'
$launcher='C:\Users\15119\.codex\worktrees\r6-windows-native-resume\code\test\windows024-native\launch-r7-private-native-empty.ps1'
$tokens=$null;$errors=$null
$ast=[Management.Automation.Language.Parser]::ParseFile($launcher,[ref]$tokens,[ref]$errors)
if($errors.Count){$errors|Format-List|Out-String|Write-Output;throw 'PS5 launcher parse failed'}
$scriptSha=(Get-FileHash -LiteralPath $launcher -Algorithm SHA256).Hash.ToLowerInvariant()
foreach($name in @('WriteOnce','NewPrivate','SnapshotBytes')){
 $function=@($ast.FindAll({param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst]},$true)|Where-Object {$_.Name -ceq $name})
 if($function.Count -ne 1){throw 'exact actual helper AST required'}
 . ([scriptblock]::Create($function[0].Extent.Text))
}
$utf8=New-Object Text.UTF8Encoding($false)
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
$privateRoot=Join-Path ([IO.Path]::GetTempPath()) ('ocx024-r7-native-script-helper-'+[Guid]::NewGuid().ToString())
$out=Join-Path $privateRoot 'D'
$checks=[ordered]@{scriptSha256=$scriptSha;actualPowerShellVersion=$PSVersionTable.PSVersion.ToString();parsed=$true;productionScriptExecuted=$false;desktopOrNoJobAcceptance=$false;helperRoot=$privateRoot;currentR2Touched=$false;checks=@()}
try{
 NewPrivate $privateRoot;foreach($name in @('I','D','H')){NewPrivate (Join-Path $privateRoot $name)}
 $acl=Get-Acl -LiteralPath $privateRoot
 if(!$acl.AreAccessRulesProtected -or $acl.GetOwner([Security.Principal.SecurityIdentifier]).Value -cne $identity.User.Value){throw 'actual helper private directory identity/protection wrong'}
 $checks.checks+=@{item='actual protected private directories under current own new temporary root';passed=$true}
 $rejected=$false;try{NewPrivate $privateRoot}catch{$rejected=$true};if(!$rejected){throw 'existing private root must refuse'}
 $rejected=$false;try{NewPrivate (Join-Path $privateRoot 'foreign')}catch{$rejected=$true};if(!$rejected){throw 'outside finite set must refuse'}
 $checks.checks+=@{item='existing and out-of-set directories refused';passed=$true}
 WriteOnce 'sample.json' @{test='ordinary helper only';normalStop=$false;closed=$false}
 $before=[IO.File]::ReadAllBytes((Join-Path $out 'sample.json'))
 $rejected=$false;try{WriteOnce 'sample.json' @{replacement=$true}}catch{$rejected=$true};if(!$rejected){throw 'CreateNew collision must refuse'}
 if([Convert]::ToBase64String($before) -cne [Convert]::ToBase64String([IO.File]::ReadAllBytes((Join-Path $out 'sample.json')))){throw 'collision changed source'}
 $checks.checks+=@{item='actual immutable JSON write bytes and collision retained';passed=$true}
 $log=Join-Path $out 'shared.log';$bytes=$utf8.GetBytes('held original bytes')
 $writer=[IO.File]::Open($log,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try{$writer.Write($bytes,0,$bytes.Length);$writer.Flush($true);$snapshot=SnapshotBytes $log;if([Convert]::ToBase64String($snapshot) -cne [Convert]::ToBase64String($bytes)){throw 'original open writer snapshot differs'}}finally{$writer.Dispose()}
 $checks.checks+=@{item='actual readable snapshot compatible with retained writer share';passed=$true}
 $big=Join-Path $out 'oversize.log';$file=[IO.File]::Open($big,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
 try{$file.SetLength(4194305)}finally{$file.Dispose()}
 $rejected=$false;try{SnapshotBytes $big|Out-Null}catch{$rejected=$true};if(!$rejected){throw 'oversize raw snapshot must refuse'}
 $checks.checks+=@{item='actual 4MiB+1 file snapshot refuses';passed=$true}
 # Resolve exact newly created own root before recursive fixture cleanup.
 $resolved=[IO.Path]::GetFullPath($privateRoot)
 $temp=[IO.Path]::GetFullPath([IO.Path]::GetTempPath())
 if(!$resolved.StartsWith($temp,[StringComparison]::OrdinalIgnoreCase) -or !(Split-Path $resolved -Leaf).StartsWith('ocx024-r7-native-script-helper-')){throw 'own cleanup root scope mismatch'}
 Remove-Item -LiteralPath $resolved -Recurse -Force
 if(Test-Path -LiteralPath $resolved){throw 'own helper cleanup not absent'}
 $checks.helperRootAbsent=$true;$checks.raw=0
}catch{$checks.firstError=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber};$checks.raw=1;$checks.helperRootRetained=Test-Path -LiteralPath $privateRoot}
$result=Join-Path $PSScriptRoot 'launcher-ps51-check.json'
if(Test-Path -LiteralPath $result){throw 'immutable helper evidence already exists'}
[IO.File]::WriteAllText($result,($checks|ConvertTo-Json -Depth 12),$utf8)
$checks|ConvertTo-Json -Depth 12
exit $checks.raw

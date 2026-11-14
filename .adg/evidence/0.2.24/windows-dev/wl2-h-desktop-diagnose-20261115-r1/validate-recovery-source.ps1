param([string]$Source)
$ErrorActionPreference='Stop'
$t=$null;$errors=$null;$ast=[Management.Automation.Language.Parser]::ParseFile($Source,[ref]$t,[ref]$errors)
if($errors.Count){throw 'source parse failure'}
$nodes=@($ast.FindAll({param($n) $n -is [Management.Automation.Language.StringConstantExpressionAst] -and $n.Value.Contains('public sealed class H024ReadOnlyWitness')},$true))
if($nodes.Count -ne 1){throw 'exact native observation class not found'}
Add-Type -TypeDefinition $nodes[0].Value
$constructors=@([IO.FileStream].GetConstructors()|Where-Object {$_.ToString().Contains('FileSystemRights') -and $_.ToString().Contains('FileSecurity')})
$directories=@([IO.Directory].GetMethods()|Where-Object {$_.Name -eq 'CreateDirectory' -and $_.ToString().Contains('DirectorySecurity')})
if(!$constructors.Count -or !$directories.Count){throw 'Framework ACL-at-creation overload unavailable'}
@{scope='compile and reflection only; no process witness constructed, no account or permission writes';witnessCompiled=$true;fileSecurityConstructor=$constructors[0].ToString();directorySecurityOverload=$directories[0].ToString();framework=[Environment]::Version.ToString()}|ConvertTo-Json
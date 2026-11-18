$ErrorActionPreference = 'Stop'
$stage = Join-Path $PSScriptRoot 'stage-r7-desktop.ps1'
$tokens = $null
$errors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile($stage, [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw ($errors | Out-String) }
foreach ($function in $ast.EndBlock.Statements | Where-Object { $_ -is [Management.Automation.Language.FunctionDefinitionAst] }) {
    . ([scriptblock]::Create($function.Extent.Text))
}
$controller = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$surrogate = 'S-1-5-32-545'
$common = ('O:{0}D:P(A;OICI;FA;;;{0})(A;OICI;FA;;;SY)' -f $controller)
$readonly = $common + ('(A;OICI;0x1200a9;;;{0})' -f $surrogate)
$write = $common + ('(A;OICI;0x1201bf;;;{0})' -f $surrogate)
$root = Join-Path $PSScriptRoot ('stage-acl-probe-' + [Guid]::NewGuid().ToString('N'))
$input = Join-Path $root 'input'
$out = Join-Path $root 'out01'
$file = Join-Path $input 'copy.ps1'
if (!(IsAbsent $root)) { throw 'ACL probe root exists.' }
try {
    MakeDirectory $root $readonly
    MakeDirectory $input $readonly
    MakeDirectory $out $write
    $source = RequirePlain $stage $false
    $expected = Sha $stage
    CopyExact @{from=$stage; to=$file; bytes=$source.Length; sha256=$expected}
    $rules = @( (Get-Acl -LiteralPath $file).GetAccessRules($true, $true, [Security.Principal.SecurityIdentifier]) |
        Where-Object { $_.IdentityReference.Value -ceq $surrogate } )
    if ($rules.Count -ne 1 -or !$rules[0].IsInherited -or [int]$rules[0].FileSystemRights -ne 0x1200a9) { throw 'Copied input failed the inherited read-only check.' }
    $rejected = $false
    try { CopyExact @{from=$stage; to=$file; bytes=$source.Length; sha256=$expected} }
    catch [Management.Automation.MethodInvocationException] {
        if ($_.Exception.InnerException -isnot [IO.IOException]) { throw }
        $rejected = $true
    }
    if (!$rejected -or (Sha $file) -cne $expected) { throw 'An existing input was overwritten.' }
} finally {
    if (!(IsAbsent $file)) { [IO.File]::Delete($file) }
    foreach ($path in @($out,$input,$root)) { if (!(IsAbsent $path)) { [IO.Directory]::Delete($path, $false) } }
}
if (!(IsAbsent $root)) { throw 'ACL probe root still exists.' }
[ordered]@{passed=$true; syntheticSid=$surrogate; protectedDirectoryAclExact=$true; copiedInputReadOnly=$true;
    createNewCollisionRefused=$true; fixtureAbsent=$true} | ConvertTo-Json -Compress
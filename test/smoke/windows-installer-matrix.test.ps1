param([Parameter(Mandatory=$true)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'windows-installer-matrix.ps1')
$checks=[Collections.Generic.List[string]]::new()
$sourceFixture='a'*40
$priorGit=Get-Item Function:git -ErrorAction SilentlyContinue
try {
    function git { throw 'Explicit source must not call git' }
    $global:LASTEXITCODE=7
    if ((Get-WindowsInstallerSource $sourceFixture) -cne $sourceFixture) { throw 'Explicit source lost' }
    foreach($bad in @('HEAD',('A'*40),('a'*39))) {
        $rejected=$false
        try { $null=Get-WindowsInstallerSource $bad } catch { if ($_.Exception.Message -cne 'Source commit missing') { throw };$rejected=$true }
        if (!$rejected) { throw 'Malformed explicit source accepted' }
    }
    function git { $global:LASTEXITCODE=1;return $sourceFixture }
    $rejected=$false
    try { $null=Get-WindowsInstallerSource '' } catch { if ($_.Exception.Message -cne 'Source commit missing') { throw };$rejected=$true }
    if (!$rejected) { throw 'Failed git source accepted' }
    function git { $global:LASTEXITCODE=0;return $sourceFixture }
    if ((Get-WindowsInstallerSource '') -cne $sourceFixture) { throw 'Successful git source lost' }
    $checks.Add('explicit-source-independent-of-stale-exit-and-git-fallback-verified')
} finally {
    if ($priorGit) { Set-Item Function:git $priorGit.ScriptBlock } else { Remove-Item Function:git }
    $global:LASTEXITCODE=0
}
function Assert-MatrixRejects([scriptblock]$Action){
    $rejected=$false
    try{& $Action}catch{
        if($_.Exception.Message -notmatch '^Installer matrix'){throw}
        $rejected=$true
    }
    if(!$rejected){throw 'An incomplete/duplicate/failed installer matrix was accepted'}
}
# The report from Release 41589977450 was green but omitted NSIS/default.
$fullDefault=@(@{kind='msi';result='pass'},@{kind='nsis';result='pass'})
$fullCustom=@(@{kind='msi';result='pass'},@{kind='nsis';result='pass'})
Assert-MatrixRejects { Assert-WindowsInstallerMatrix -Kind all -Mode all -DefaultChecks @($fullDefault[0]) -CustomChecks $fullCustom }
$checks.Add('rejects-real-three-cell-green-report')
foreach($requestedKind in @('all','msi','nsis')){
    foreach($requestedMode in @('all','default','custom')){
        $validDefault=@($fullDefault | Where-Object {$requestedMode -cne 'custom' -and ($requestedKind -ceq 'all' -or $_.kind -ceq $requestedKind)})
        $validCustom=@($fullCustom | Where-Object {$requestedMode -cne 'default' -and ($requestedKind -ceq 'all' -or $_.kind -ceq $requestedKind)})
        Assert-WindowsInstallerMatrix -Kind $requestedKind -Mode $requestedMode -DefaultChecks $validDefault -CustomChecks $validCustom
        Assert-MatrixRejects { Assert-WindowsInstallerMatrix -Kind $requestedKind -Mode $requestedMode -DefaultChecks @() -CustomChecks @() }
        $checks.Add("validates-$requestedKind-$requestedMode")
    }
}
Assert-MatrixRejects { Assert-WindowsInstallerMatrix -Kind all -Mode all -DefaultChecks @($fullDefault[0],$fullDefault[0]) -CustomChecks $fullCustom }
Assert-MatrixRejects { Assert-WindowsInstallerMatrix -Kind all -Mode all -DefaultChecks @($fullDefault[0],@{kind='nsis';result='fail'}) -CustomChecks $fullCustom }
Assert-MatrixRejects { Assert-WindowsInstallerMatrix -Kind msi -Mode default -DefaultChecks $fullDefault -CustomChecks @() }
$checks.Add('rejects-duplicate-failed-and-unrequested-cells')

# Inspect the actual production AST using PowerShell's case-insensitive rules.
# A foreach variable named $kind overwrites the parameter $Kind and skipped the
# last default NSIS case. Reject that collision at the real orchestration site.
$tokens=$null;$errors=$null
$source=Join-Path $PSScriptRoot 'windows-installer.ps1'
$ast=[Management.Automation.Language.Parser]::ParseFile($source,[ref]$tokens,[ref]$errors)
if($errors.Count){throw ($errors | Out-String)}
$parameters=@($ast.ParamBlock.Parameters | ForEach-Object {$_.Name.VariablePath.UserPath})
foreach($loop in $ast.FindAll({param($node) $node -is [Management.Automation.Language.ForEachStatementAst]},$true)){
    if($loop.Variable.VariablePath.UserPath -in $parameters){throw 'Installer loop overwrites a requested matrix parameter'}
}
$calls=@($ast.FindAll({param($node) $node -is [Management.Automation.Language.CommandAst] -and $node.GetCommandName() -ceq 'Assert-WindowsInstallerMatrix'},$true))
if($calls.Count -ne 1){throw 'Production installer must call the matrix gate exactly once before success'}
$checks.Add('production-loops-preserve-requested-kind-and-mode')
$output=[IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Force $output | Out-Null
[ordered]@{result='pass';powershell=$PSVersionTable.PSVersion.ToString();checks=@($checks)} |
    ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $output 'matrix-contract.json') -Encoding utf8
Write-Output "PASS: $($checks.Count) installer matrix contract checks"

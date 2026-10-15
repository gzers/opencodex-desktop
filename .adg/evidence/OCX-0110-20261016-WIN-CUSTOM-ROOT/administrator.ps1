$ErrorActionPreference='Stop'
try {
    & (Join-Path $PSScriptRoot 'launch.ps1') -Mode administrator -Phase administrator-relocated -Location install-b -ScriptedRootChecks *>> (Join-Path $PSScriptRoot 'administrator-controller.original.log')
} catch {
    $_ | Out-String | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'administrator-controller.error.txt') -Encoding utf8
    throw
}

param([Parameter(Mandatory)][string]$CodeRoot,[Parameter(Mandatory)][string]$Output)
$ErrorActionPreference='Stop'
$rows=@()
foreach($name in @('cleanup-r6-authorized-processes.ps1','probe-r6-authorized-cleanup.ps1')){
 $tokens=$null;$errors=$null;$path=Join-Path $CodeRoot ('test/windows024-native/'+$name)
 [void][Management.Automation.Language.Parser]::ParseFile($path,[ref]$tokens,[ref]$errors)
 if(@($errors).Count){throw ($errors|Out-String)}
 $rows+=@{path=$path;sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant();parseErrors=0}
}
$rows|ConvertTo-Json -Depth 5|Set-Content -LiteralPath $Output -Encoding UTF8

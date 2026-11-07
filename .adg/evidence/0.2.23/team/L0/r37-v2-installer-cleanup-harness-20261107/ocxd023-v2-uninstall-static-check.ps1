$ErrorActionPreference='Stop'
$checks=[Collections.Generic.List[object]]::new()
foreach($file in Get-ChildItem -LiteralPath test/smoke -Filter '*.ps1'){
 $tokens=$null;$errors=$null;$ast=[Management.Automation.Language.Parser]::ParseFile($file.FullName,[ref]$tokens,[ref]$errors)
 if($errors.Count){throw ('Smoke parser failed: '+$file.Name)}
 $checks.Add(@{name=('parser-'+$file.Name);result='pass'})
}
$script=Join-Path $PWD 'test/smoke/windows-uninstall-data.ps1'
$before=@(Get-ChildItem -LiteralPath /tmp -Filter 'ocxd023-uninstall-refused-*').Count
$refused=$false
try{& $script -InstallDirectory '/tmp/ocxd023-uninstall-refused-static-only'}catch{
 if(!$_.Exception.Message.Contains('disposable repository hosted runner')){throw};$refused=$true
}
if(!$refused -or (Test-Path -LiteralPath /tmp/ocxd023-uninstall-refused-static-only)){throw 'Outside-runner guard did not refuse before mutation'}
$checks.Add(@{name='actual-outside-runner-refusal-before-fixture-creation';result='pass'})
$x=@{result='pass-static-only';checks=$checks;checksPassed=$checks.Count;nativeCleanupExecuted=$false;profileChanged=$false}
$x|ConvertTo-Json -Depth 5|Set-Content -LiteralPath /tmp/ocxd023-v2-uninstall-static-result.json -Encoding utf8
$x|Select-Object result,checksPassed,nativeCleanupExecuted|ConvertTo-Json

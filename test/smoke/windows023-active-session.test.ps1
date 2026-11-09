param([Parameter(Mandatory=$true)][string]$OutputDirectory,[Parameter(Mandatory=$true)][string]$Node)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $OutputDirectory){throw 'Fresh evidence directory required'}
New-Item -ItemType Directory -Path $OutputDirectory | Out-Null
. (Join-Path $PSScriptRoot 'windows023-two-user-native-helper.ps1')
. (Join-Path $PSScriptRoot 'windows-process-token.ps1')
$checks=@();$child=$null;$fact=$null
function Check([bool]$Value,[string]$Name){if(!$Value){throw $Name};$script:checks+=@{name=$Name;passed=$true}}
try{
 $path=Join-Path $PSScriptRoot 'windows023-active-session.ps1'
 $tokens=$null;$errors=$null;$ast=[Management.Automation.Language.Parser]::ParseFile($path,[ref]$tokens,[ref]$errors)
 Check ($errors.Count -eq 0) 'session runner parses'
 $assignment=$ast.Find({param($n) $n -is [Management.Automation.Language.AssignmentStatementAst] -and $n.Left -is [Management.Automation.Language.VariableExpressionAst] -and $n.Left.VariablePath.UserPath -eq 'taskReplacements'},$true)
 $literals=@($assignment.Right.FindAll({param($n) $n -is [Management.Automation.Language.StringConstantExpressionAst]},$true) | ForEach-Object Value)
 Check ($literals.Count -eq 6) 'exactly three literal binding pairs; no fixture expressions executed'
 $replacements=@();for($index=0;$index -lt $literals.Count;$index+=2){$replacements+=,@($literals[$index],$literals[$index+1])}
 $driver=Get-TwoUserDriverText;$original=$driver
 foreach($replacement in $replacements){Check ($driver.Contains($replacement[0])) 'reviewed binding exists';$driver=$driver.Replace($replacement[0],$replacement[1])}
 for($index=$replacements.Count-1;$index -ge 0;$index--){$driver=$driver.Replace($replacements[$index][1],$replacements[$index][0])}
 Check ($driver -ceq $original) 'only the three Sandbox binding substitutions; entire original driver retained'
 [IO.File]::WriteAllText((Join-Path $OutputDirectory 'original-driver.mjs'),$original)
 Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'windows-cdp.mjs') -Destination $OutputDirectory
 & $Node (Join-Path $OutputDirectory 'original-driver.mjs') --runtime-self-test *> (Join-Path $OutputDirectory 'pure-driver.log')
 Check ($LASTEXITCODE -eq 0) 'original attestation/config/negative-conflict assertions execute'
 $sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value
 $self=[Diagnostics.Process]::GetCurrentProcess()
 $selfFact=Get-TwoUserProcessFact $self $sid
 Check ($selfFact.pid -eq $PID -and $selfFact.sessionId -eq $self.SessionId) 'own actual retained PID SID session'
 $info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=$Node;$info.UseShellExecute=$false;$info.CreateNoWindow=$true
 $info.ArgumentList.Add('-e');$info.ArgumentList.Add('setTimeout(()=>{},60000)')
 $child=[Diagnostics.Process]::Start($info);$fact=Get-TwoUserProcessFact $child $sid
 $observation=Get-TwoUserProcessObservation $child
 Assert-TwoUserNativeIdentity $observation $fact.pid $fact.path $fact.creationTicks
 $rejected=$false;try{Assert-TwoUserNativeIdentity $observation $fact.pid $fact.path ([string]([long]$fact.creationTicks+1))}catch{$rejected=$true}
 Check $rejected 'one-tick changed generation rejected'
 $badRoot=Join-Path $OutputDirectory 'must-never-exist'
 foreach($case in @(@{sid='S-1-5-21-1-2-3-4';session=$self.SessionId},@{sid=$sid;session=2147483647})){
  $rejected=$false
  try{& $path -RunDirectory $badRoot -ProgramDirectory $badRoot -ProvisionTool $badRoot -Role rdp -ExpectedSid $case.sid -ExpectedSessionId $case.session}catch{$rejected=$_.Exception.Message -eq 'Actual SID/session differs from sealed launch request'}
  Check ($rejected -and !(Test-Path -LiteralPath $badRoot)) 'wrong actual SID/session rejected before fixture mutation'
 }
}finally{
 if($child){if(!$fact -and !$child.HasExited){$fact=Get-TwoUserProcessFact $child ([Security.Principal.WindowsIdentity]::GetCurrent().User.Value)};if($fact){Stop-TwoUserTracked $fact};Check ($child.WaitForExit(10000)) 'own exact test process exits';$child.Dispose()}
 Close-TwoUserProcessHandles
 Write-TwoUserJson (Join-Path $OutputDirectory 'active-session-preflight.json') @{accepted=$false;scope='fixture guards and own native process only; no app/service started';checks=$checks}
}
@{passed=$checks.Count;accepted=$false} | ConvertTo-Json -Compress

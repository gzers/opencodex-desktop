$ErrorActionPreference='Stop'
. 'C:\Users\15119\.codex\worktrees\ocxd-023-windows-r1\code\test\smoke\windows023-two-user-native-helper.ps1'
$dir=$PSScriptRoot;$helper=Join-Path $dir 'logon-reference-query.ps1'
if(Test-Path -LiteralPath (Join-Path $dir 'query-finished.json')){throw 'Never replace earlier query'}
$hypothesis=@{writtenUtc=[datetime]::UtcNow.ToString('o');id='H-TAIL-2';observed='Elevated WTS snapshot has no matching SID1018 among478 processes, only PID0/System owner unknown; Profile remains Loaded';hypothesis='A bounded read-only LSA enumeration of matching synthetic SID can reveal retained logon-session references without new login, profile load or product actions';queryTimeoutMilliseconds=30000;noProductActions=$true;noForcedHiveUnload=$true;doNotInferAbsentFromUnknown=$true;primaryReferences=@('https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsenumerateprocessesexw','https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/ns-wtsapi32-wts_process_info_exw','https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createprocesswithlogonw')}
Write-TwoUserJson (Join-Path $dir 'hypothesis.json') $hypothesis
$errors=$null;$tokens=$null
[void][Management.Automation.Language.Parser]::ParseFile($helper,[ref]$tokens,[ref]$errors)
if($errors.Count){throw 'Query helper syntax failed'}
$info=[Diagnostics.ProcessStartInfo]::new();$info.FileName='D:\ocxu1-20261109-191803\powershell\pwsh.exe';$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
foreach($arg in @('-NoLogo','-NoProfile','-NonInteractive','-File',$helper,'-Directory',$dir)){$info.ArgumentList.Add($arg)}
$process=[Diagnostics.Process]::Start($info);$out=$process.StandardOutput.ReadToEndAsync();$err=$process.StandardError.ReadToEndAsync()
$report=[ordered]@{scope='Only own exact read-only helper; no new user, product, service, profile load or termination';startedUtc=[datetime]::UtcNow.ToString('o');helperSha256=(Get-FileHash -LiteralPath $helper).Hash.ToLowerInvariant();timedOut=$false}
try{
 $sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value;$fact=Get-TwoUserProcessFact $process $sid;$report.identity=$fact
 if(!$process.WaitForExit(30000)){$report.timedOut=$true;Stop-TwoUserTracked $fact}
 if(!$process.WaitForExit(5000)){throw 'Original query did not exit after exact own cancellation'}
 $report.exit=Get-TwoUserOriginalExit $fact
 [IO.File]::WriteAllText((Join-Path $dir 'query-stdout.log'),$out.GetAwaiter().GetResult())
 [IO.File]::WriteAllText((Join-Path $dir 'query-stderr.log'),$err.GetAwaiter().GetResult())
 $report.resultAvailable=Test-Path -LiteralPath (Join-Path $dir 'query-result.json')
}catch{$report.failure=@{message=$_.Exception.Message;line=$_.InvocationInfo.ScriptLineNumber}}
finally{$process.Dispose();Close-TwoUserProcessHandles;$report.finishedUtc=[datetime]::UtcNow.ToString('o');Write-TwoUserJson (Join-Path $dir 'query-finished.json') $report}
$report|ConvertTo-Json -Depth 7

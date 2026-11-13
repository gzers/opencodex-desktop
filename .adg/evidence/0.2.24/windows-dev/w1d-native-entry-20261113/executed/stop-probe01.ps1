$ErrorActionPreference='Stop'
$f=Get-CimInstance Win32_Process -Filter 'ProcessId=115876'
if(!$f -or $f.CommandLine -notlike '*C:\Users\Public\ocxw1d-98b5378c87\input\w1d.msi*' -or $f.ParentProcessId -ne 40632){throw 'exact owned MSI probe differs'}
$p=[Diagnostics.Process]::GetProcessById(115876);$null=$p.Handle
if([Math]::Abs(($p.StartTime-$f.CreationDate).TotalSeconds) -gt 1){throw 'generation differs'}
$report=@{pid=$p.Id;creation=$p.StartTime.ToString('o');title=$p.MainWindowTitle;command=$f.CommandLine;logExists=(Test-Path 'C:/Users/Public/ocxw1d-98b5378c87/out01/native/msi-install.msi.log');reason='test MSI invocation remained at Windows Installer help; fix property quoting, no install receipt'}
$p.Kill();[void]$p.WaitForExit(5000);$report.exited=$p.HasExited;$report.exitCode=$p.ExitCode
$report|ConvertTo-Json|Set-Content 'C:/Users/Public/ocxw1d-98b5378c87/out01/msi-command-timeout.json'
$p.Dispose()

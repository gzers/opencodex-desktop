$ErrorActionPreference='Stop'
$taskBase='D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/.toolchain/post021-materials/baseline'
$taskInfo=[Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName="$taskBase/opencodex-desktop.exe"
$taskInfo.UseShellExecute=$false
$taskInfo.WorkingDirectory=$taskBase
$taskInfo.RedirectStandardOutput=$true
$taskInfo.RedirectStandardError=$true
foreach($taskPair in @{OPENCODEX_SANDBOX='1';OPENCODEX_SANDBOX_ROOT="$taskBase/sandbox";OPENCODEX_SANDBOX_BOUNDARY=$taskBase;OPENCODEX_WINDOWS_SMOKE_CDP_PORT='9421';HOME="$taskBase/home";USERPROFILE="$taskBase/home";APPDATA="$taskBase/home/AppData/Roaming";LOCALAPPDATA="$taskBase/home/AppData/Local";CODEX_HOME="$taskBase/home/.codex";OPENCODEX_HOME="$taskBase/sandbox/opencodex-home";TEMP="$taskBase/temp";TMP="$taskBase/temp"}.GetEnumerator()){$taskInfo.Environment[$taskPair.Key]=$taskPair.Value}
$taskProcess=[Diagnostics.Process]::Start($taskInfo)
Write-Output "baseline pid=$($taskProcess.Id)"
$taskOut=$taskProcess.StandardOutput.ReadToEndAsync()
$taskErr=$taskProcess.StandardError.ReadToEndAsync()
$taskProcess.WaitForExit()
$taskOut.Result | Set-Content -LiteralPath "$taskBase/stdout.log"
$taskErr.Result | Set-Content -LiteralPath "$taskBase/stderr.log"
Write-Output "exit=$($taskProcess.ExitCode)"
Write-Output $taskErr.Result

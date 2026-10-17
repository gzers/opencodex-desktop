param(
    [Parameter(Mandatory)][ValidateSet('baseline019','candidate0110')][string]$Version,
    [Parameter(Mandatory)][string]$RunDirectory
)
$ErrorActionPreference='Stop'
$taskExpected='C:\Users\15119\.codex\worktrees\windows-etw-query-threads\opencodex-desktop\test\runtime\foreground-ce-pair-20261017T113045'
if([IO.Path]::GetFullPath($RunDirectory) -cne $taskExpected){throw 'Exact owned run required'}
$taskScript='D:\Users\15119\WorkSpace\Code\tools\opencodex-desktop\.adg\evidence\OCX-0110-20261017-WIN-CE-FRONT-PREP\observe.ps1'
$taskInvocation=Join-Path $RunDirectory ($Version+'-collector-invocation.json')
if(Test-Path -LiteralPath $taskInvocation){throw 'One declared collector invocation only'}
$taskInfo=[Diagnostics.ProcessStartInfo]::new()
$taskInfo.FileName='C:\Users\15119\.cache\codex-runtimes\codex-primary-runtime\dependencies\native\powershell\pwsh.exe'
$taskInfo.UseShellExecute=$false
$taskInfo.CreateNoWindow=$true
$taskInfo.RedirectStandardOutput=$true
$taskInfo.RedirectStandardError=$true
foreach($taskArgument in @('-NoProfile','-File',$taskScript,'-Version',$Version,'-RunDirectory',$RunDirectory)){
    $taskInfo.ArgumentList.Add($taskArgument)
}
# Apply physical-coordinate awareness to the read-only observer only, as in
# the admitted 019/823 pair. Do not alter the product or Windows display.
$taskInfo.Environment['__COMPAT_LAYER']='HIGHDPIAWARE'
$taskProcess=[Diagnostics.Process]::Start($taskInfo)
try {
    [void]$taskProcess.SafeHandle
    $taskStdout=$taskProcess.StandardOutput.ReadToEndAsync()
    $taskStderr=$taskProcess.StandardError.ReadToEndAsync()
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');version=$Version;observerPid=$taskProcess.Id
        observerCreationUtc=$taskProcess.StartTime.ToUniversalTime().ToString('o')
        observerScript=$taskScript;observerScriptSha256=(Get-FileHash -LiteralPath $taskScript).Hash.ToLowerInvariant()
        observerOnlyEnvironment=@{__COMPAT_LAYER='HIGHDPIAWARE'};productEnvironmentChanged=$false
        systemDisplayChanged=$false;uiInputInjected=$false;automaticRetry=$false} |
        ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $taskInvocation -Encoding utf8
    $taskTimer=[Diagnostics.Stopwatch]::StartNew()
    while(!$taskProcess.WaitForExit(1000)){
        if($taskTimer.Elapsed.TotalSeconds -gt 700){throw 'Observer controller time limit; do not kill or retry'}
    }
    [IO.File]::WriteAllText((Join-Path $RunDirectory ($Version+'-collector.stdout.original.log')),$taskStdout.GetAwaiter().GetResult(),[Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText((Join-Path $RunDirectory ($Version+'-collector.stderr.original.log')),$taskStderr.GetAwaiter().GetResult(),[Text.UTF8Encoding]::new($false))
    [ordered]@{atUtc=[datetimeoffset]::UtcNow.ToString('o');observerPid=$taskProcess.Id
        observerExitCode=$taskProcess.ExitCode;forceTerminated=$false;uiInputInjected=$false} |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $RunDirectory ($Version+'-collector-exit.json')) -Encoding utf8
    if($taskProcess.ExitCode -ne 0){throw 'Bound observer failed; preserve attempt'}
}finally{$taskProcess.Dispose()}

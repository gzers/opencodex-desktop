$ErrorActionPreference='Stop'
$taskRepo='C:\Users\15119\.codex\worktrees\windows-working-set\opencodex-desktop'
$taskRoot=$PSScriptRoot
$taskPwsh=(Get-Process -Id $PID).Path
$taskEntry=Join-Path $taskRepo 'test/smoke/windows-bound-gpu-memory.ps1'
$taskChecks=0
function Assert-Entry($Condition,[string]$Message){if(!$Condition){throw $Message};$script:taskChecks++}
$taskTokens=$null;$taskErrors=$null
[void][Management.Automation.Language.Parser]::ParseFile($taskEntry,[ref]$taskTokens,[ref]$taskErrors)
Assert-Entry ($taskErrors.Count -eq 0) 'Bound GPU entry syntax failed'
$taskOrdinary=[Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
Assert-Entry (!$taskOrdinary.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) 'Local ordinary refusal requires the actual ordinary token'
$taskOutput=Join-Path $taskRoot 'ordinary-no-recording'
$taskArgs=@('-NoProfile','-NonInteractive','-File',$taskEntry,'-RootLaunchJson',(Join-Path $taskRoot 'not-opened-root.json'),'-GpuLaunchJson',(Join-Path $taskRoot 'not-opened-gpu.json'),'-OutputDirectory',$taskOutput,'-DecoderDirectory',(Join-Path $taskRoot 'not-opened-decoder'),'-MethodAdmissionPath',(Join-Path $taskRoot 'not-opened-admission.json'),'-RecordApproved')
$taskLines=@(& $taskPwsh @taskArgs 2>&1);$taskCode=$LASTEXITCODE
[IO.File]::WriteAllText((Join-Path $taskRoot 'ordinary-entry-refusal.decoded.log'),($taskLines -join "`n")+"`n",[Text.UTF8Encoding]::new($false))
Assert-Entry ($taskCode -ne 0 -and ($taskLines -join "`n").Contains('requires an elevated controller')) 'Ordinary entry did not refuse before inputs or tracing'
Assert-Entry (!(Test-Path -LiteralPath $taskOutput)) 'Ordinary entry created a recording output'
$taskSelf=Get-Process -Id $PID
$taskLaunch=@{pid=$PID;creationTimeUtc=$taskSelf.StartTime.ToUniversalTime().ToString('o');executable=$taskSelf.Path;exeSha256=(Get-FileHash -LiteralPath $taskSelf.Path).Hash.ToLowerInvariant()}
$taskLaunchPath=Join-Path $taskRoot 'controlled-self-launch.json'
$taskLaunch | ConvertTo-Json | Set-Content -LiteralPath $taskLaunchPath -Encoding utf8
$taskOutput=Join-Path $taskRoot 'native0-no-recording'
$taskArgs=@('-NoProfile','-NonInteractive','-File',$taskEntry,'-RootLaunchJson',$taskLaunchPath,'-GpuLaunchJson',$taskLaunchPath,'-OutputDirectory',$taskOutput)
$taskLines=@(& $taskPwsh @taskArgs 2>&1);$taskCode=$LASTEXITCODE
[IO.File]::WriteAllText((Join-Path $taskRoot 'native0-entry-refusal.decoded.log'),($taskLines -join "`n")+"`n",[Text.UTF8Encoding]::new($false))
Assert-Entry ($taskCode -ne 0 -and ($taskLines -join "`n").Contains('One exact native desktop required')) 'Native0 prepare must not substitute a harness for the desktop'
Assert-Entry (!(Test-Path -LiteralPath $taskOutput)) 'Native0 refusal created capture output'
. (Join-Path $taskRepo 'test/smoke/windows-memory-target.ps1')
$taskFailure=$null
try{Assert-WindowsMemoryMethodAdmission -Path (Join-Path $taskRepo 'test/runtime/controlled-memory-validation-20261016T2330/ci-terminal-admission.json') | Out-Null}catch{$taskFailure=$_.Exception.Message}
Assert-Entry ($taskFailure -and $taskFailure.Contains('Same committed tool source required')) 'Old full CI must not admit a new tool source'
$taskFailure=$null
try{Assert-WindowsMemoryMethodAdmission -Path (Join-Path $taskRepo 'test/runtime/memory-target-binding-20261017T0010/method-admission.json') | Out-Null}catch{$taskFailure=$_.Exception.Message}
Assert-Entry ($taskFailure -and $taskFailure.Contains('first complete tool CI admission')) 'A method-only job must not admit full recording'
[ordered]@{passed=$true;checks=$taskChecks;ordinaryControllerRefused=$true;native0PrepareRefused=$true;oldFullCiRefused=$true;partialCiRefused=$true;tracingStarted=$false;productStarted=$false;recordingExecutionAdmitted=$false} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskRoot 'entry-refusal-result.json') -Encoding utf8
Get-Content -LiteralPath (Join-Path $taskRoot 'entry-refusal-result.json')
$taskSelf.Dispose()
